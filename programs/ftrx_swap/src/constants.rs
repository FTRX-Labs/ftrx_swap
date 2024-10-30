use anchor_lang::prelude::*;

#[constant]
pub const MINIMUM_LIQUIDITY: u64 = 1000;



#[constant]
pub const FEE_MULTIPLIER: u64 = 100000;


#[constant]
pub const AUTHORITY_SEED: &str = "authority";

#[constant]
pub const LIQUIDITY_SEED: &str = "liquidity";

#[constant]
pub const TREASURY_SEED: &str = "treasury";