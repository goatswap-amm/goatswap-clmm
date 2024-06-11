pub mod curve;
pub mod error;
pub mod instructions;
pub mod states;
pub mod utils;

use crate::curve::fees::FEE_RATE_DENOMINATOR_VALUE;
use anchor_lang::prelude::*;
use instructions::*;

#[cfg(feature = "devnet")]
declare_id!("HKwqLZQw1fcnnFds4nkxYAmYK67TvtZ6TnVLUMJviWPL");
#[cfg(not(feature = "devnet"))]
declare_id!("GoatAFSqACoMvJqvgW7aFACFkkArv69ezTJhS8xdEr5H");

pub mod admin {
    use anchor_lang::prelude::declare_id;
    #[cfg(feature = "devnet")]
    declare_id!("EmUVHtpMu7cREkxkHea6NqAa2fJHLRtS8BocJbDzuPRd");
    #[cfg(not(feature = "devnet"))]
    declare_id!("DGoaqkXNhVrzp29VCxNUCR5j9E6iH1rLiNG58Lz9mQyw");
}

pub mod create_pool_fee_reveiver {
    use anchor_lang::prelude::declare_id;
    #[cfg(feature = "devnet")]
    declare_id!("99hJtj5f3WqWrVoPyimrYPzcCA7gM2TYm2ksLxvgDvgB");
    #[cfg(not(feature = "devnet"))]
    declare_id!("DGoaqkXNhVrzp29VCxNUCR5j9E6iH1rLiNG58Lz9mQyw");
}

pub mod whitelisted_address {
    use anchor_lang::prelude::declare_id;
    #[cfg(feature = "devnet")]
    declare_id!("GtHkW9JQf1WZuJ7zQUR6tL4WsgkoRBB1NxaWA8V8RspA");
    #[cfg(not(feature = "devnet"))]
    declare_id!("Pinks8fXRcJ8EpNjK4vCf7saJ5hFeP9zRXGPwZrTukQ");
}

pub const AUTH_SEED: &str = "vault_and_lp_mint_auth_seed";

#[program]
pub mod goatswap {
    use super::*;

    // The configuation of AMM protocol, include trade fee and protocol fee
    /// # Arguments
    ///
    /// * `ctx`- The accounts needed by instruction.
    /// * `index` - The index of amm config, there may be multiple config.
    /// * `trade_fee_rate` - Trade fee rate, can be changed.
    /// * `protocol_fee_rate` - The rate of protocol fee within tarde fee.
    /// * `fund_fee_rate` - The rate of fund fee within tarde fee.
    ///
    pub fn create_amm_config(
        ctx: Context<CreateAmmConfig>,
        index: u16,
        trade_fee_rate: u64,
        protocol_fee_rate: u64,
        fund_fee_rate: u64,
        create_pool_fee: u64,
    ) -> Result<()> {
        assert!(trade_fee_rate < FEE_RATE_DENOMINATOR_VALUE);
        assert!(protocol_fee_rate <= FEE_RATE_DENOMINATOR_VALUE);
        assert!(fund_fee_rate <= FEE_RATE_DENOMINATOR_VALUE);
        assert!(fund_fee_rate + protocol_fee_rate <= FEE_RATE_DENOMINATOR_VALUE);

        instructions::create_amm_config(
            ctx,
            index,
            trade_fee_rate,
            protocol_fee_rate,
            fund_fee_rate,
            create_pool_fee,
        )
    }

    /// Updates the owner of the amm config
    /// Must be called by the current owner or admin
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `trade_fee_rate`- The new trade fee rate of amm config, be set when `param` is 0
    /// * `protocol_fee_rate`- The new protocol fee rate of amm config, be set when `param` is 1
    /// * `fund_fee_rate`- The new fund fee rate of amm config, be set when `param` is 2
    /// * `new_procotol_owner`- The config's new owner, be set when `param` is 3
    /// * `new_fund_owner`- The config's new fund owner, be set when `param` is 4
    /// * `create_pool_fee`- The config's new owner, be set when `param` is 5
    /// * `disable_create_pool`- The config's new fund owner, be set when `param` is 6
    /// * `param`- The vaule can be 0 | 1 | 2 | 3 | 4 | 5 | 6, otherwise will report a error
    ///
    pub fn update_amm_config(ctx: Context<UpdateAmmConfig>, param: u8, value: u64) -> Result<()> {
        instructions::update_amm_config(ctx, param, value)
    }

    /// Update pool status for given vaule
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `status` - The vaule of status
    ///
    pub fn update_pool_status(ctx: Context<UpdatePoolStatus>, status: u8) -> Result<()> {
        instructions::update_pool_status(ctx, status)
    }

    /// Update pool tax status for given vaule
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `tax_disabled` - The vaule of tax status of pool
    ///
    pub fn update_pool_tax_status(
        ctx: Context<UpdatePoolStatus>,
        tax_disabled: bool,
    ) -> Result<()> {
        instructions::update_pool_tax_status(ctx, tax_disabled)
    }

    /// Update tax
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `tax_use_token_0` - tax use token0 or token1
    /// * `tax_fee_in_rate` - new tax fee rate of pool when swap in
    /// * `tax_fee_out_rate` - new tax fee rate of pool when swap out
    ///
    pub fn update_tax(
        ctx: Context<UpdateTaxConfig>,
        tax_use_token_0: bool,
        in_tax_rate: u64,
        out_tax_rate: u64,
    ) -> Result<()> {
        instructions::update_tax(ctx, tax_use_token_0, in_tax_rate, out_tax_rate)
    }

    /// Update tax
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `lp_fee_rate` - lp fee rate value
    ///
    pub fn update_lp_fee(ctx: Context<UpdateLpFee>, lp_fee_rate: u64) -> Result<()> {
        instructions::update_lp_fee(ctx, lp_fee_rate)
    }

    /// Update tax authority
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `new_authority` - new tax transfer authority
    ///
    pub fn transfer_tax_authority(
        ctx: Context<TransferTaxAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::transfer_tax_authority(ctx, new_authority)
    }

    /// Transfer pool owner
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `new_owner` - The new owner of pool
    ///
    pub fn transfer_pool_owner(ctx: Context<UpdatePoolStatus>, new_owner: Pubkey) -> Result<()> {
        instructions::transfer_pool_owner(ctx, new_owner)
    }

    /// Collect the protocol fee accrued to the pool
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of accounts
    /// * `amount_0_requested` - The maximum amount of token_0 to send, can be 0 to collect fees in only token_1
    /// * `amount_1_requested` - The maximum amount of token_1 to send, can be 0 to collect fees in only token_0
    ///
    pub fn collect_protocol_fee(
        ctx: Context<CollectProtocolFee>,
        amount_0_requested: u64,
        amount_1_requested: u64,
    ) -> Result<()> {
        instructions::collect_protocol_fee(ctx, amount_0_requested, amount_1_requested)
    }

    /// Collect the fund fee accrued to the pool
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of accounts
    /// * `amount_0_requested` - The maximum amount of token_0 to send, can be 0 to collect fees in only token_1
    /// * `amount_1_requested` - The maximum amount of token_1 to send, can be 0 to collect fees in only token_0
    ///
    pub fn collect_fund_fee(
        ctx: Context<CollectFundFee>,
        amount_0_requested: u64,
        amount_1_requested: u64,
    ) -> Result<()> {
        instructions::collect_fund_fee(ctx, amount_0_requested, amount_1_requested)
    }

    /// Collect the tax accrued to the pool
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of accounts
    ///
    pub fn collect_tax(ctx: Context<CollectTax>) -> Result<()> {
        instructions::collect_tax(ctx)
    }

    /// Creates a pool for the given token pair and the initial price
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `init_amount_0` - the initial amount_0 to deposit
    /// * `init_amount_1` - the initial amount_1 to deposit
    /// * `open_time` - the timestamp allowed for swap
    ///
    pub fn initialize(
        ctx: Context<Initialize>,
        init_amount_0: u64,
        init_amount_1: u64,
        open_time: u64,
        tax_use_token_0: bool,
        in_tax_rate: u64,
        out_tax_rate: u64,
        lp_fee_rate: Option<u64>,
    ) -> Result<()> {
        instructions::initialize(
            ctx,
            init_amount_0,
            init_amount_1,
            open_time,
            tax_use_token_0,
            in_tax_rate,
            out_tax_rate,
            lp_fee_rate,
        )
    }

    /// (For whitelisted program only) Creates a pool for the given token pair and the initial price
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `init_amount_0` - the initial amount_0 to deposit
    /// * `init_amount_1` - the initial amount_1 to deposit
    /// * `open_time` - the timestamp allowed for swap
    ///
    pub fn initialize_whitelisted(
        ctx: Context<InitializeWhitelisted>,
        init_amount_0: u64,
        init_amount_1: u64,
        open_time: u64,
        tax_use_token_0: bool,
        in_tax_rate: u64,
        out_tax_rate: u64,
        lp_fee_rate: Option<u64>,
    ) -> Result<()> {
        instructions::initialize_whitelisted(
            ctx,
            init_amount_0,
            init_amount_1,
            open_time,
            tax_use_token_0,
            in_tax_rate,
            out_tax_rate,
            lp_fee_rate,
        )
    }

    /// Creates a pool for the given token pair and the initial price
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `lp_token_amount` - Pool token amount to transfer. token_a and token_b amount are set by the current exchange rate and size of the pool
    /// * `maximum_token_0_amount` -  Maximum token 0 amount to deposit, prevents excessive slippage
    /// * `maximum_token_1_amount` - Maximum token 1 amount to deposit, prevents excessive slippage
    ///
    pub fn deposit(
        ctx: Context<Deposit>,
        lp_token_amount: u64,
        maximum_token_0_amount: u64,
        maximum_token_1_amount: u64,
    ) -> Result<()> {
        instructions::deposit(
            ctx,
            lp_token_amount,
            maximum_token_0_amount,
            maximum_token_1_amount,
        )
    }

    /// Withdraw lp for token0 ande token1
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `lp_token_amount` - Amount of pool tokens to burn. User receives an output of token a and b based on the percentage of the pool tokens that are returned.
    /// * `minimum_token_0_amount` -  Minimum amount of token 0 to receive, prevents excessive slippage
    /// * `minimum_token_1_amount` -  Minimum amount of token 1 to receive, prevents excessive slippage
    ///
    pub fn withdraw(
        ctx: Context<Withdraw>,
        lp_token_amount: u64,
        minimum_token_0_amount: u64,
        minimum_token_1_amount: u64,
    ) -> Result<()> {
        instructions::withdraw(
            ctx,
            lp_token_amount,
            minimum_token_0_amount,
            minimum_token_1_amount,
        )
    }

    /// Swap the tokens in the pool base input amount
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `amount_in` -  input amount to transfer, output to DESTINATION is based on the exchange rate
    /// * `minimum_amount_out` -  Minimum amount of output token, prevents excessive slippage
    ///
    pub fn swap_base_input(
        ctx: Context<Swap>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        instructions::swap_base_input(ctx, amount_in, minimum_amount_out)
    }

    /// Swap the tokens in the pool base output amount
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `max_amount_in` -  input amount prevents excessive slippage
    /// * `amount_out` -  amount of output token
    ///
    pub fn swap_base_output(ctx: Context<Swap>, max_amount_in: u64, amount_out: u64) -> Result<()> {
        instructions::swap_base_output(ctx, max_amount_in, amount_out)
    }
}
