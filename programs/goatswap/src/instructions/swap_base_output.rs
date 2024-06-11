use super::swap_base_input::Swap;
use crate::curve::fees;
use crate::curve::{calculator::CurveCalculator, TradeDirection};
use crate::error::ErrorCode;
use crate::states::*;
use crate::utils::token::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;

pub fn swap_base_output(
    ctx: Context<Swap>,
    max_amount_in: u64,
    amount_out_less_fee: u64,
) -> Result<()> {
    let block_timestamp = solana_program::clock::Clock::get()?.unix_timestamp as u64;
    let pool_id = ctx.accounts.pool_state.key();
    let pool_state = &mut ctx.accounts.pool_state.load_mut()?;
    if !pool_state.get_status_by_bit(PoolStatusBitIndex::Swap)
        || block_timestamp < pool_state.open_time
    {
        return err!(ErrorCode::NotApproved);
    }

    // check in|out tax
    let has_out_tax = !pool_state.tax_disabled
        && pool_state.out_tax_rate > 0
        && pool_state.tax_mint == ctx.accounts.output_token_mint.key();
    let has_in_tax = !pool_state.tax_disabled
        && pool_state.in_tax_rate > 0
        && pool_state.tax_mint == ctx.accounts.input_token_mint.key();

    let (out_tax, amount_out_with_tax) = if has_out_tax {
        let amount_out_with_tax = fees::Fees::calculate_pre_fee_amount(
            u128::try_from(amount_out_less_fee).unwrap(),
            pool_state.out_tax_rate,
        )
        .ok_or(ErrorCode::TaxAmountCalculationFailed)?;

        let amount_out_with_tax = u64::try_from(amount_out_with_tax).unwrap();

        let out_tax = amount_out_with_tax
            .checked_sub(amount_out_less_fee)
            .unwrap();

        (out_tax, amount_out_with_tax)
    } else {
        (0, amount_out_less_fee)
    };

    let out_transfer_fee = get_transfer_inverse_fee(
        &ctx.accounts.output_token_mint.to_account_info(),
        amount_out_less_fee,
    )?;

    // after add fee 2022 + tax fee
    let actual_amount_out = amount_out_with_tax.checked_add(out_transfer_fee).unwrap();

    // Calculate the trade amounts
    let (trade_direction, total_input_token_amount, total_output_token_amount) =
        if ctx.accounts.input_vault.key() == pool_state.token_0_vault
            && ctx.accounts.output_vault.key() == pool_state.token_1_vault
        {
            let (total_input_token_amount, total_output_token_amount) = pool_state
                .vault_amount_without_fee(
                    ctx.accounts.input_vault.amount,
                    ctx.accounts.output_vault.amount,
                );

            (
                TradeDirection::ZeroForOne,
                total_input_token_amount,
                total_output_token_amount,
            )
        } else if ctx.accounts.input_vault.key() == pool_state.token_1_vault
            && ctx.accounts.output_vault.key() == pool_state.token_0_vault
        {
            let (total_output_token_amount, total_input_token_amount) = pool_state
                .vault_amount_without_fee(
                    ctx.accounts.output_vault.amount,
                    ctx.accounts.input_vault.amount,
                );

            (
                TradeDirection::OneForZero,
                total_input_token_amount,
                total_output_token_amount,
            )
        } else {
            return err!(ErrorCode::InvalidVault);
        };

    let constant_before = u128::from(total_input_token_amount)
        .checked_mul(u128::from(total_output_token_amount))
        .unwrap();

    let result = CurveCalculator::swap_base_output(
        u128::from(actual_amount_out),
        u128::from(total_input_token_amount),
        u128::from(total_output_token_amount),
        ctx.accounts.amm_config.trade_fee_rate,
        ctx.accounts.amm_config.protocol_fee_rate,
        ctx.accounts.amm_config.fund_fee_rate,
        pool_state.lp_fee_rate,
    )
    .ok_or(ErrorCode::ZeroTradingTokens)?;

    let constant_after = result
        .new_swap_source_amount
        .checked_mul(u128::from(result.new_swap_destination_amount))
        .unwrap();

    require_gte!(constant_after, constant_before);

    // calculate amount out with tax
    let (in_tax, amount_in_with_tax) = if has_in_tax {
        let amount_in_with_tax = fees::Fees::calculate_pre_fee_amount(
            result.source_amount_swapped,
            pool_state.in_tax_rate,
        )
        .ok_or(ErrorCode::TaxAmountCalculationFailed)?;

        let in_tax = amount_in_with_tax
            .checked_sub(result.source_amount_swapped)
            .unwrap();

        (in_tax, amount_in_with_tax)
    } else {
        (0, result.source_amount_swapped)
    };

    // Re-calculate the source amount swapped based on what the curve says
    let input_transfer_amount = {
        let source_amount_swapped = u64::try_from(amount_in_with_tax).unwrap();
        require_gt!(source_amount_swapped, 0);
        let transfer_fee = get_transfer_inverse_fee(
            &ctx.accounts.input_token_mint.to_account_info(),
            source_amount_swapped,
        )?;
        let input_transfer_amount = source_amount_swapped.checked_add(transfer_fee).unwrap();

        require_gte!(
            max_amount_in,
            input_transfer_amount,
            ErrorCode::ExceededSlippage
        );

        input_transfer_amount
    };

    require_eq!(
        u64::try_from(result.destination_amount_swapped).unwrap(),
        actual_amount_out
    );

    let output_transfer_amount = actual_amount_out.checked_sub(out_tax).unwrap();

    let protocol_fee = u64::try_from(result.protocol_fee).unwrap();
    let fund_fee = u64::try_from(result.fund_fee).unwrap();

    // update tax in vault
    let tax_amount: u64 = if has_out_tax {
        out_tax
    } else {
        u64::try_from(in_tax).unwrap()
    };

    if pool_state.tax_mint == pool_state.token_0_mint {
        pool_state.tax_amount_0 = pool_state.tax_amount_0.checked_add(tax_amount).unwrap();
    } else {
        pool_state.tax_amount_1 = pool_state.tax_amount_1.checked_add(tax_amount).unwrap();
    }

    #[cfg(feature = "enable-log")]
    msg!(
        "source_amount_swapped:{}, destination_amount_swapped:{},constant_before:{},constant_after:{},tax_use_token_0:{},tax_amount:{},lp_fee:{}",
        result.source_amount_swapped,
        result.destination_amount_swapped,
        constant_before,
        constant_after,
        pool_state.tax_mint == pool_state.token_0_mint,
        tax_amount,
        result.lp_fee
    );

    match trade_direction {
        TradeDirection::ZeroForOne => {
            pool_state.protocol_fees_token_0 = pool_state
                .protocol_fees_token_0
                .checked_add(protocol_fee)
                .unwrap();
            pool_state.fund_fees_token_0 =
                pool_state.fund_fees_token_0.checked_add(fund_fee).unwrap();
        }
        TradeDirection::OneForZero => {
            pool_state.protocol_fees_token_1 = pool_state
                .protocol_fees_token_1
                .checked_add(protocol_fee)
                .unwrap();
            pool_state.fund_fees_token_1 =
                pool_state.fund_fees_token_1.checked_add(fund_fee).unwrap();
        }
    };

    transfer_from_user_to_pool_vault(
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.input_token_account.to_account_info(),
        ctx.accounts.input_vault.to_account_info(),
        ctx.accounts.input_token_mint.to_account_info(),
        ctx.accounts.input_token_program.to_account_info(),
        input_transfer_amount,
        ctx.accounts.input_token_mint.decimals,
    )?;

    transfer_from_pool_vault_to_user(
        ctx.accounts.authority.to_account_info(),
        ctx.accounts.output_vault.to_account_info(),
        ctx.accounts.output_token_account.to_account_info(),
        ctx.accounts.output_token_mint.to_account_info(),
        ctx.accounts.output_token_program.to_account_info(),
        output_transfer_amount,
        ctx.accounts.output_token_mint.decimals,
        &[&[crate::AUTH_SEED.as_bytes(), &[pool_state.auth_bump]]],
    )?;

    ctx.accounts.input_vault.reload()?;
    ctx.accounts.output_vault.reload()?;

    let (reserve_0, reserve_1) = match trade_direction {
        TradeDirection::ZeroForOne => pool_state.vault_amount_without_fee(
            ctx.accounts.input_vault.amount,
            ctx.accounts.output_vault.amount,
        ),
        TradeDirection::OneForZero => pool_state.vault_amount_without_fee(
            ctx.accounts.output_vault.amount,
            ctx.accounts.input_vault.amount,
        ),
    };

    emit_cpi!(SwapEvent {
        pool_id,
        token_in: ctx.accounts.input_token_account.mint,
        token_out: ctx.accounts.output_token_account.mint,
        amount_in: u64::try_from(result.source_amount_swapped).unwrap(),
        amount_out: u64::try_from(result.destination_amount_swapped).unwrap(),
        reserve_0,
        reserve_1,
    });

    Ok(())
}
