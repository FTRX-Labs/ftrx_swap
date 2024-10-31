use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Burn, Mint, Token, TokenAccount, Transfer},
};
use fixed::types::I64F64;
use fixed_sqrt::FixedSqrt;

use crate::{
    constants::{AUTHORITY_SEED, LIQUIDITY_SEED, MINIMUM_LIQUIDITY},
    errors::FTRXSwapError,
    state::SimplePool,
};

pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount: u64, amount_expected_a: u64, amount_expected_b: u64) -> Result<()> {
   


    let actual_pool=&ctx.accounts.pool;
    let lp_fee=actual_pool.lp_fee.to_le_bytes();
    
    let pool_a = &ctx.accounts.pool_account_a;
    let pool_b = &ctx.accounts.pool_account_b;


    let amount_a_before=I64F64::from_num(pool_a.amount);
    let amount_b_before=I64F64::from_num(pool_b.amount);


    let authority_seeds = &[
        actual_pool.mint_a.as_ref(),
        actual_pool.mint_b.as_ref(),
        actual_pool.admin.as_ref(),
        lp_fee.as_ref(),
        &[actual_pool.pool_bump],
    ];
    let signer_seeds = &[&authority_seeds[..]];





    let  mint_liquidity_supply_before = ctx.accounts.mint_liquidity.supply + MINIMUM_LIQUIDITY;


    
    // Transfer tokens from the pool
    let amount_a = I64F64::from_num(amount)
        .checked_mul(I64F64::from_num(ctx.accounts.pool_account_a.amount))
        .unwrap()
        .checked_div(I64F64::from_num(
            mint_liquidity_supply_before 
        ))
        .unwrap()
        .floor()
        .to_num::<u64>();
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_account_a.to_account_info(),
                to: ctx.accounts.depositor_account_a.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        amount_a,
    )?;

    let amount_b = I64F64::from_num(amount)
        .checked_mul(I64F64::from_num(ctx.accounts.pool_account_b.amount))
        .unwrap()
        .checked_div(I64F64::from_num(
            mint_liquidity_supply_before,
        ))
        .unwrap()
        .floor()
        .to_num::<u64>();


    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_account_b.to_account_info(),
                to: ctx.accounts.depositor_account_b.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        amount_b,
    )?;

    // Burn the liquidity tokens
    // It will fail if the amount is invalid
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint_liquidity.to_account_info(),
                from: ctx.accounts.depositor_account_liquidity.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount,
    )?;




    if amount_expected_a>amount_a{
        return err!(FTRXSwapError::SlippageExceeded);
    }

    if amount_expected_b>amount_b{
        return err!(FTRXSwapError::SlippageExceeded);
    }



    // Checks : we want to have liquidity_reduction/liquidity_before>amount_a_withdrawn/amount_a_before, and same for b
    // We want the liquidity reduction to be at least more important than the reduction of a and b
    let ratio_liquidity_check=I64F64::from_num(amount).checked_div(I64F64::from_num(mint_liquidity_supply_before)).unwrap();

    let ratio_token_a_check=I64F64::from_num(amount_a).checked_div(I64F64::from_num(amount_a_before)).unwrap();
    let ratio_token_b_check=I64F64::from_num(amount_b).checked_div(I64F64::from_num(amount_b_before)).unwrap();
    //If thats not true for token a or token b we raise exception
    if ratio_liquidity_check<ratio_token_a_check || ratio_liquidity_check<ratio_token_b_check{
        return err!(FTRXSwapError::InconsistentPriceRatioLiquidity);
    }


    
    // NEED TO CHECK inconsistancy between ratio_liquidity_check and ratio_liquidity_check
    
    msg!(
        " liquidity ratio {} token a ratio {} token b ratio {}",
        ratio_liquidity_check,
        ratio_token_a_check,
        ratio_token_b_check
    );

   

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {

    #[account(
        seeds = [
       
        mint_a.key().as_ref(),
        mint_b.key().as_ref(),
        pool.admin.key().as_ref(),
        &pool.lp_fee.to_le_bytes(),

        ],
        bump,
        has_one = mint_a,
        has_one = mint_b,
    )]
    pub pool: Account<'info, SimplePool>,



    #[account(
        mut,
        seeds = [
        
        mint_a.key().as_ref(),
        mint_b.key().as_ref(),
        pool.admin.key().as_ref(),
        LIQUIDITY_SEED.as_ref(),

        ],
        bump,
    )]
    pub mint_liquidity: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub mint_a: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub mint_b: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = mint_a,
        token::authority = pool,
        seeds = [
        mint_a.key().as_ref(),
        pool.key().as_ref(),
        ],
        bump
    )]
    pub pool_account_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = mint_b,
        token::authority = pool,
        seeds = [
        mint_b.key().as_ref(),
        pool.key().as_ref(),
        ],
        bump
    )]
    pub pool_account_b: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_liquidity,
        associated_token::authority = payer,
    )]
    pub depositor_account_liquidity: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = payer,
    )]
    pub depositor_account_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = payer,
    )]
    pub depositor_account_b: Box<Account<'info, TokenAccount>>,

    /// The account paying for all rents
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana ecosystem accounts
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}