use crate::curve::calculator::CurveCalculator;
use crate::curve::TradeDirection;
use crate::error::ErrorCode;
use crate::states::*;
use crate::utils::tax_amount;
use crate::utils::token::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[event_cpi]
#[derive(Accounts)]
pub struct Swap<'info> {
    /// The user performing the swap
    pub payer: Signer<'info>,

    /// CHECK: pool vault and lp mint authority
    #[account(
        seeds = [
            crate::AUTH_SEED.as_bytes(),
        ],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    /// The factory state to read protocol fees
    #[account(address = pool_state.load()?.amm_config)]
    pub amm_config: Box<Account<'info, AmmConfig>>,

    /// The program account of the pool in which the swap will be performed
    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,

    /// The user token account for input token
    #[account(mut, token::authority = payer, token::mint = input_token_mint)]
    pub input_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The user token account for output token
    #[account(mut, token::authority = payer, token::mint = output_token_mint)]
    pub output_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for input token
    #[account(
        mut,
        constraint = input_vault.key() == pool_state.load()?.token_0_vault || input_vault.key() == pool_state.load()?.token_1_vault
    )]
    pub input_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for output token
    #[account(
        mut,
        constraint = output_vault.key() == pool_state.load()?.token_0_vault || output_vault.key() == pool_state.load()?.token_1_vault
    )]
    pub output_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// SPL program for input token transfers
    pub input_token_program: Interface<'info, TokenInterface>,

    /// SPL program for output token transfers
    pub output_token_program: Interface<'info, TokenInterface>,

    /// The mint of input token
    #[account(
        address = input_vault.mint,
        token::token_program = input_token_program
    )]
    pub input_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of output token
    #[account(
        address = output_vault.mint,
        token::token_program = output_token_program
    )]
    pub output_token_mint: Box<InterfaceAccount<'info, Mint>>,
}

pub fn swap_base_input(ctx: Context<Swap>, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
    let block_timestamp = solana_program::clock::Clock::get()?.unix_timestamp as u64;
    let pool_id = ctx.accounts.pool_state.key();
    let pool_state = &mut ctx.accounts.pool_state.load_mut()?;
    if !pool_state.get_status_by_bit(PoolStatusBitIndex::Swap)
        || block_timestamp < pool_state.open_time
    {
        return err!(ErrorCode::NotApproved);
    }

    let transfer_fee =
        get_transfer_fee(&ctx.accounts.input_token_mint.to_account_info(), amount_in)?;

    // Take transfer fees into account for actual amount transferred in
    let amount_in_without_transfer_fee = amount_in.saturating_sub(transfer_fee);

    // check in|out tax
    let has_out_tax = !pool_state.tax_disabled
        && pool_state.out_tax_rate > 0
        && pool_state.tax_mint == ctx.accounts.output_token_mint.key();
    let has_in_tax = !pool_state.tax_disabled
        && pool_state.in_tax_rate > 0
        && pool_state.tax_mint == ctx.accounts.input_token_mint.key();

    let in_tax = if has_in_tax {
        let in_tax = tax_amount(amount_in_without_transfer_fee, pool_state.in_tax_rate).unwrap();
        u64::try_from(in_tax).unwrap()
    } else {
        0
    };

    let actual_amount_in = amount_in_without_transfer_fee.saturating_sub(in_tax);

    require_gt!(actual_amount_in, 0);

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

    let result = CurveCalculator::swap_base_input(
        u128::from(actual_amount_in),
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

    require_eq!(
        u64::try_from(result.source_amount_swapped).unwrap(),
        actual_amount_in
    );
    let input_transfer_amount = amount_in;

    // calculate amount out with tax
    let out_tax = if has_out_tax {
        tax_amount(
            u64::try_from(result.destination_amount_swapped).unwrap(),
            pool_state.out_tax_rate,
        )
        .ok_or(ErrorCode::TaxAmountCalculationFailed)?
    } else {
        0
    };

    // check minimum amount out
    let output_transfer_amount = {
        let destination_amount_swapped_post_tax = result
            .destination_amount_swapped
            .checked_sub(out_tax)
            .unwrap();

        let amount_out: u64 = u64::try_from(destination_amount_swapped_post_tax).unwrap();
        let transfer_fee = get_transfer_fee(
            &ctx.accounts.output_token_mint.to_account_info(),
            amount_out,
        )?;
        let amount_received = amount_out.checked_sub(transfer_fee).unwrap();
        require_gt!(amount_received, 0);
        require_gte!(
            amount_received,
            minimum_amount_out,
            ErrorCode::ExceededSlippage
        );
        amount_out
    };

    let protocol_fee = u64::try_from(result.protocol_fee).unwrap();
    let fund_fee = u64::try_from(result.fund_fee).unwrap();

    // update tax in vault
    let tax_amount: u64 = if has_in_tax {
        in_tax
    } else {
        u64::try_from(out_tax).unwrap()
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

    // update fee in vault
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
