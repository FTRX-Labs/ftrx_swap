use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Burn, Mint, Token, TokenAccount, Transfer},
};
use fixed::types::I64F64;
use fixed_sqrt::FixedSqrt;

use crate::{
    constants::{AUTHORITY_SEED, LIQUIDITY_SEED, MINIMUM_LIQUIDITY},
    constants::TREASURY_SEED,
    errors::FTRXSwapError,
    state::SimplePool,
};

pub fn admin_gets_treasury(ctx: Context<AdminGetsTreasury>, amount_a: u64,amount_b: u64) -> Result<()> {
   


    Ok(())
}

#[derive(Accounts)]
pub struct AdminGetsTreasury<'info> {

    #[account(
        seeds = [
       
        mint_a.key().as_ref(),
        mint_b.key().as_ref(),
        admin.key().as_ref(),
        &pool.lp_fee.to_le_bytes(),

        ],
        bump,
        has_one = mint_a,
        has_one = mint_b,
    )]
    pub pool: Account<'info, SimplePool>,


    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = admin,
    )]
    pub depositor_account_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = admin,
    )]
    pub depositor_account_b: Box<Account<'info, TokenAccount>>,


        
    #[account(mut,
        token::mint = mint_a,
        token::authority = pool,
        seeds = [
        pool.mint_a.key().as_ref(),
        pool.key().as_ref(),
        TREASURY_SEED.as_ref(),
        pool.admin.key().as_ref(),
        ],
        bump,

      )]
    pub treasury_mint_a: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        token::mint = mint_b,
        token::authority = pool,
        seeds = [
        pool.mint_b.key().as_ref(),
        pool.key().as_ref(),
        TREASURY_SEED.as_ref(),
        pool.admin.key().as_ref(),
        ],
        bump,
   
      )]
    pub treasury_mint_b: Box<Account<'info, TokenAccount>>,


    pub mint_a: Box<Account<'info, Mint>>,

    pub mint_b: Box<Account<'info, Mint>>,


    /// The account paying for all rents
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Solana ecosystem accounts
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}