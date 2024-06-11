use crate::curve::fees::FEE_RATE_DENOMINATOR_VALUE;
use crate::error::ErrorCode;
use crate::states::*;
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
pub struct UpdateTaxConfig<'info> {
    /// owner of pool
    #[account(address = pool_state.load()?.tax_authority @ ErrorCode::InvalidOwner)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,

    #[account(address = pool_state.load()?.amm_config)]
    pub amm_config: Box<Account<'info, AmmConfig>>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct TransferTaxAuthority<'info> {
    /// owner of pool or admin
    #[account(constraint = (owner.key() == pool_state.load()?.tax_authority || owner.key() == crate::admin::id()) @ ErrorCode::InvalidOwner)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,
}

pub fn update_tax(
    ctx: Context<UpdateTaxConfig>,
    tax_use_token_0: bool,
    in_tax_rate: u64,
    out_tax_rate: u64,
) -> Result<()> {
    assert!(in_tax_rate <= FEE_RATE_DENOMINATOR_VALUE);
    assert!(out_tax_rate <= FEE_RATE_DENOMINATOR_VALUE);

    let pool_state = &mut ctx.accounts.pool_state.load_mut()?;

    // get tax mint
    let tax_mint = if tax_use_token_0 {
        pool_state.token_0_mint
    } else {
        pool_state.token_1_mint
    };

    pool_state.tax_mint = tax_mint;
    pool_state.in_tax_rate = in_tax_rate;
    pool_state.out_tax_rate = out_tax_rate;

    emit_cpi!(TaxConfigUpdatedEvent {
        pool_id: ctx.accounts.pool_state.key(),
        tax_mint: tax_mint,
        tax_authority: pool_state.tax_authority,
        in_tax_rate: in_tax_rate,
        out_tax_rate: out_tax_rate,
        tax_disabled: pool_state.tax_disabled,
    });

    Ok(())
}

pub fn transfer_tax_authority(
    ctx: Context<TransferTaxAuthority>,
    new_authority: Pubkey,
) -> Result<()> {
    let pool_state = &mut ctx.accounts.pool_state.load_mut()?;

    require!(
        new_authority != pool_state.tax_authority,
        ErrorCode::InvalidInput
    );

    pool_state.tax_authority = new_authority;

    emit_cpi!(TaxConfigUpdatedEvent {
        pool_id: ctx.accounts.pool_state.key(),
        tax_mint: pool_state.tax_mint,
        tax_authority: new_authority,
        in_tax_rate: pool_state.in_tax_rate,
        out_tax_rate: pool_state.out_tax_rate,
        tax_disabled: pool_state.tax_disabled,
    });

    Ok(())
}
