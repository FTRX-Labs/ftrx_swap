use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct SimpleAmm {
    /// The primary key of the AMM
    pub id: Pubkey,

    /// Account that has admin authority over the AMM
    pub admin: Pubkey,

    /// The LP fee taken on each trade, in basis points
    pub fee: u16,
}

impl SimpleAmm {
    pub const LEN: usize = 8 + 32 + 32 + 2;
}

#[account]
#[derive(Default)]
pub struct SimplePool {
    /// Primary key of the AMM
    pub amm: Pubkey,

    /// Mint of token A
    pub mint_a: Pubkey,

    /// Mint of token B
    pub mint_b: Pubkey,
}

impl SimplePool {
    pub const LEN: usize = 8 + 32 + 32 + 32;
}