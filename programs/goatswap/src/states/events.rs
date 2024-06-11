use anchor_lang::prelude::*;

/// Emitted when deposit and withdraw
#[event]
pub struct LpChangeEvent {
    #[index]
    pub pool_id: Pubkey,
    pub amount_lp: u64,
    pub amount_0: u64,
    pub amount_1: u64,
    pub reserve_0: u64,
    pub reserve_1: u64,
    // 0: create, 1: add, 2: burn
    pub change_type: u8,
}

/// Emitted when swap
#[event]
pub struct SwapEvent {
    #[index]
    pub pool_id: Pubkey,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub reserve_0: u64,
    pub reserve_1: u64,
}

/// Emitted when init pool, update tax
#[event]
pub struct TaxConfigUpdatedEvent {
    pub pool_id: Pubkey,
    pub tax_mint: Pubkey,
    pub tax_authority: Pubkey,
    pub in_tax_rate: u64,
    pub out_tax_rate: u64,
    pub tax_disabled: bool,
}

/// Emitted when collect tax
#[event]
pub struct TaxCollectEvent {
    pub pool_id: Pubkey,
    pub amount_0: u64,
    pub amount_1: u64,
}
