use crate::curve::fees::FEE_RATE_DENOMINATOR_VALUE;
use crate::error::ErrorCode;
use crate::states::*;
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
pub struct UpdateLpFee<'info> {
    /// owner of pool
    #[account(constraint = (owner.key() == pool_state.load()?.pool_creator || owner.key() == crate::admin::id()) @ ErrorCode::InvalidOwner)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,

    #[account(address = pool_state.load()?.amm_config)]
    pub amm_config: Box<Account<'info, AmmConfig>>,
}

pub fn update_lp_fee(ctx: Context<UpdateLpFee>, lp_fee_rate: u64) -> Result<()> {
    assert!(lp_fee_rate + ctx.accounts.amm_config.trade_fee_rate <= FEE_RATE_DENOMINATOR_VALUE);

    let pool_state = &mut ctx.accounts.pool_state.load_mut()?;

    #[cfg(feature = "enable-log")]
    {
        let old_lp_fee_rate = pool_state.lp_fee_rate;
        msg!(
            "old_lp_fee_rate:{}, new_lp_fee_rate:{}",
            old_lp_fee_rate,
            lp_fee_rate
        );
    }

    pool_state.lp_fee_rate = lp_fee_rate;

    Ok(())
}
