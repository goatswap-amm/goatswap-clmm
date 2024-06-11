use crate::error::ErrorCode;
use crate::states::*;
use crate::utils::token::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::Mint;
use anchor_spl::token_interface::Token2022;
use anchor_spl::token_interface::TokenAccount;

#[event_cpi]
#[derive(Accounts)]
pub struct CollectTax<'info> {
    /// owner of pool
    #[account(address = pool_state.load()?.pool_creator @ ErrorCode::InvalidOwner)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,

    /// CHECK: pool vault and lp mint authority
    #[account(
        seeds = [
            crate::AUTH_SEED.as_bytes(),
        ],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    /// Amm config account stores fund_owner
    #[account(address = pool_state.load()?.amm_config)]
    pub amm_config: Account<'info, AmmConfig>,

    /// The address that holds pool tokens for token_0
    #[account(
        mut,
        constraint = token_0_vault.key() == pool_state.load()?.token_0_vault
    )]
    pub token_0_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_1
    #[account(
        mut,
        constraint = token_1_vault.key() == pool_state.load()?.token_1_vault
    )]
    pub token_1_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The mint of token_0 vault
    #[account(
        address = token_0_vault.mint
    )]
    pub vault_0_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token_1 vault
    #[account(
        address = token_1_vault.mint
    )]
    pub vault_1_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The address that receives the collected token_0 tax
    #[account(mut)]
    pub recipient_token_0_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that receives the collected token_1 tax
    #[account(mut)]
    pub recipient_token_1_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The SPL program to perform token transfers
    pub token_program: Program<'info, Token>,

    /// The SPL program 2022 to perform token transfers
    pub token_program_2022: Program<'info, Token2022>,
}

pub fn collect_tax(ctx: Context<CollectTax>) -> Result<()> {
    let amount_0: u64;
    let amount_1: u64;
    let auth_bump: u8;
    {
        let mut pool_state = ctx.accounts.pool_state.load_mut()?;

        require!(!pool_state.tax_disabled, ErrorCode::TaxDisabled);

        amount_0 = pool_state.tax_amount_0;
        amount_1 = pool_state.tax_amount_1;

        require!(amount_0 > 0 || amount_1 > 0, ErrorCode::NoPendingTax);

        pool_state.tax_amount_0 = 0;
        pool_state.tax_amount_1 = 0;
        auth_bump = pool_state.auth_bump;
    }

    if amount_0 > 0 {
        transfer_from_pool_vault_to_user(
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.token_0_vault.to_account_info(),
            ctx.accounts.recipient_token_0_account.to_account_info(),
            ctx.accounts.vault_0_mint.to_account_info(),
            if ctx.accounts.vault_0_mint.to_account_info().owner == ctx.accounts.token_program.key {
                ctx.accounts.token_program.to_account_info()
            } else {
                ctx.accounts.token_program_2022.to_account_info()
            },
            amount_0,
            ctx.accounts.vault_0_mint.decimals,
            &[&[crate::AUTH_SEED.as_bytes(), &[auth_bump]]],
        )?;
    }

    if amount_1 > 0 {
        transfer_from_pool_vault_to_user(
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.token_1_vault.to_account_info(),
            ctx.accounts.recipient_token_1_account.to_account_info(),
            ctx.accounts.vault_1_mint.to_account_info(),
            if ctx.accounts.vault_1_mint.to_account_info().owner == ctx.accounts.token_program.key {
                ctx.accounts.token_program.to_account_info()
            } else {
                ctx.accounts.token_program_2022.to_account_info()
            },
            amount_1,
            ctx.accounts.vault_1_mint.decimals,
            &[&[crate::AUTH_SEED.as_bytes(), &[auth_bump]]],
        )?;
    }

    emit_cpi!(TaxCollectEvent {
        pool_id: ctx.accounts.pool_state.key(),
        amount_0,
        amount_1,
    });

    Ok(())
}
