use crate::states::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdatePoolStatus<'info> {
    #[account(
        address = crate::admin::id()
    )]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,
}

pub fn update_pool_status(ctx: Context<UpdatePoolStatus>, status: u8) -> Result<()> {
    require_gte!(255, status);
    let mut pool_state = ctx.accounts.pool_state.load_mut()?;
    pool_state.set_status(status);
    Ok(())
}

pub fn update_pool_tax_status(ctx: Context<UpdatePoolStatus>, tax_disabled: bool) -> Result<()> {
    let mut pool_state = ctx.accounts.pool_state.load_mut()?;
    pool_state.set_tax_status(tax_disabled);

    Ok(())
}

pub fn transfer_pool_owner(ctx: Context<UpdatePoolStatus>, new_owner: Pubkey) -> Result<()> {
    require_keys_neq!(new_owner, Pubkey::default());

    let mut pool_state = ctx.accounts.pool_state.load_mut()?;

    #[cfg(feature = "enable-log")]
    msg!(
        "pool_state, old_pool_owner:{}, new_pool_owner:{}",
        pool_state.pool_creator.to_string(),
        new_owner.key().to_string()
    );

    pool_state.pool_creator = new_owner;

    Ok(())
}
