use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    constants::{AUTHORITY_SEED, LIQUIDITY_SEED,TREASURY_SEED},
    errors::*,
    state::{SimplePool},
};

pub fn create_pool(ctx: Context<CreatePool>,lp_fee:u16,bump_pool:u8,bump_vault_a:u8,bump_vault_b:u8,bump_treas_a:u8,bump_treas_b:u8) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.pool_bump=bump_pool;
    pool.vault_a_bump=bump_vault_a;
    pool.vault_b_bump=bump_vault_b;
    pool.treas_a_bump=bump_treas_a;
    pool.treas_b_bump=bump_treas_b;
    pool.creator = ctx.accounts.payer.key();
    pool.mint_a = ctx.accounts.mint_a.key();
    pool.mint_b = ctx.accounts.mint_b.key();
    pool.vault_mint_a = ctx.accounts.pool_account_a.key();
    pool.vault_mint_b = ctx.accounts.pool_account_b.key();
    pool.treasury_mint_a = ctx.accounts.treasury_mint_a.key();
    pool.treasury_mint_b = ctx.accounts.treasury_mint_b.key();
    pool.admin = ctx.accounts.admin.key();
    pool.lp_fee = lp_fee;
    pool.protocol_fee = 1;

    Ok(())
}

#[derive(Accounts)]
#[instruction(lp_fee: u16)]
pub struct CreatePool<'info> {
 
    #[account(
        init,
        payer = payer,
        space = SimplePool::LEN,
        seeds = [
            mint_a.key().as_ref(),
            mint_b.key().as_ref(),
            admin.key().as_ref(),
            &lp_fee.to_le_bytes(),
        ],
        bump,
        constraint = mint_a.key() < mint_b.key() @ FTRXSwapError::InvalidMint,
        constraint = lp_fee < 2000 @ FTRXSwapError::InvalidFee,
      
       
    )]
    pub pool: Account<'info, SimplePool>,

    /// CHECK: Read only authority
    pub admin: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [
            mint_a.key().as_ref(),
            mint_b.key().as_ref(),
            admin.key().as_ref(),
            LIQUIDITY_SEED.as_ref(),
        ],
        bump,
        mint::decimals = 6,
        mint::authority = pool,
    )]
    pub mint_liquidity: Box<Account<'info, Mint>>,

    pub mint_a: Box<Account<'info, Mint>>,

    pub mint_b: Box<Account<'info, Mint>>,

    #[account(init,
        token::mint = mint_a,
        token::authority = pool,
        seeds = [
        mint_a.key().as_ref(),
        pool.key().as_ref(),
        ],
        bump,
        payer = payer
      )]
    pub pool_account_a: Box<Account<'info, TokenAccount>>,

    #[account(init,
        token::mint = mint_b,
        token::authority = pool,
        seeds = [
        mint_b.key().as_ref(),
        pool.key().as_ref(),
        ],
        bump,
        payer = payer
      )]
    pub pool_account_b: Box<Account<'info, TokenAccount>>,



    
    #[account(init,
        token::mint = mint_a,
        token::authority = pool,
        seeds = [
        mint_a.key().as_ref(),
        pool.key().as_ref(),
        TREASURY_SEED.as_ref(),
        admin.key().as_ref(),
        ],
        bump,
        payer = payer
      )]
    pub treasury_mint_a: Box<Account<'info, TokenAccount>>,

    #[account(init,
        token::mint = mint_b,
        token::authority = pool,
        seeds = [
        mint_b.key().as_ref(),
        pool.key().as_ref(),
        TREASURY_SEED.as_ref(),
        admin.key().as_ref(),
        ],
        bump,
        payer = payer
      )]
    pub treasury_mint_b: Box<Account<'info, TokenAccount>>,

    /// The account paying for all rents
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana ecosystem accounts
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}