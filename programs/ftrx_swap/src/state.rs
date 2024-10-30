use anchor_lang::prelude::*;



#[account]
#[derive(Default)]
pub struct SimplePool {
    /// Primary key of the AMM
    pub pool_bump:u8,
    pub vault_a_bump:u8,
    pub vault_b_bump:u8,
    pub treas_a_bump:u8,
    pub treas_b_bump:u8,

    pub creator: Pubkey,

    pub admin: Pubkey,

    /// The LP fee taken on each trade, in basis points
    pub lp_fee: u16,
    pub protocol_fee: u16,

    /// Mint of token A
    pub mint_a: Pubkey,
    pub vault_mint_a: Pubkey,
    pub treasury_mint_a: Pubkey,

    /// Mint of token B
    pub mint_b: Pubkey,
    pub vault_mint_b: Pubkey,
    pub treasury_mint_b: Pubkey,
}

impl SimplePool {
    pub const LEN: usize = 8+ 5 + 8*2 + 32*8;
}