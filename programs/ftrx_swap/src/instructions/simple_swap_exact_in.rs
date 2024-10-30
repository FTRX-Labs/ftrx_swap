use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use fixed::types::I64F64;

use crate::{
    constants::AUTHORITY_SEED,
    constants::FEE_MULTIPLIER,
    constants::TREASURY_SEED,
    errors::*,
    state::{SimplePool},
};

pub fn simple_swap_exact_in(
    ctx: Context<SimpleSwapExactIn>,
    swap_a: bool,
    input_amount: u64,
    min_output_amount: u64,
) -> Result<()> {
    // Prevent depositing assets the depositor does not own
    let input = if swap_a && input_amount > ctx.accounts.trader_account_a.amount {
        ctx.accounts.trader_account_a.amount
    } else if !swap_a && input_amount > ctx.accounts.trader_account_b.amount {
        ctx.accounts.trader_account_b.amount
    } else {
        input_amount
    };

    // Apply trading fee for the treasury
    let actual_pool=&ctx.accounts.pool;


    let to_treasury_fee=input.checked_mul(actual_pool.protocol_fee as u64).unwrap().checked_div(FEE_MULTIPLIER).unwrap();
    let lp_fee_amount= input.checked_mul(actual_pool.lp_fee as u64).unwrap().checked_div(FEE_MULTIPLIER).unwrap();
    let taxed_input = input.checked_sub(to_treasury_fee).unwrap().checked_sub(lp_fee_amount).unwrap();

    let pool_a = &ctx.accounts.pool_account_a;
    let pool_b = &ctx.accounts.pool_account_b;


    let raw_output =if swap_a{

    let new_pool_a_with_lp_fees=pool_a.
    amount.
    checked_add(lp_fee_amount).unwrap();

    let new_k_after_fees=new_pool_a_with_lp_fees.
    checked_mul(pool_b.amount)
    .ok_or(FTRXSwapError::MathOverflow)?;



    let new_pool_a = new_pool_a_with_lp_fees
    .checked_add(taxed_input)
    .unwrap();

    let theoretical_new_pool_b = I64F64::from_num(new_k_after_fees)
    .checked_div(I64F64::from_num(new_pool_a))
    .unwrap().ceil().to_num::<u64>();

     pool_b
    .amount
    .checked_sub(theoretical_new_pool_b)
    .unwrap()

    }else{

        
    let new_pool_b_with_lp_fees=pool_b.
    amount.
    checked_add(lp_fee_amount).unwrap();


    let new_k_after_fees=pool_a.amount.
    checked_mul(new_pool_b_with_lp_fees)
    .ok_or(FTRXSwapError::MathOverflow)?;

    


    let new_pool_b = new_pool_b_with_lp_fees
    .checked_add(taxed_input)
    .unwrap();




    let theoretical_new_pool_a = I64F64::from_num(new_k_after_fees)
    .checked_div(I64F64::from_num(new_pool_b))
    .unwrap().ceil().to_num::<u64>();

     pool_a
    .amount
    .checked_sub(theoretical_new_pool_a)
    .unwrap()

    };


    if raw_output < min_output_amount {
        return err!(FTRXSwapError::OutputTooSmall);
    }

    // Compute the invariant before the trade
    let invariant_before_trade = pool_a.amount.checked_mul(pool_b.amount).unwrap();

    // Transfer tokens to the pool
    let actual_pool=&ctx.accounts.pool;
    let lp_fee=actual_pool.lp_fee.to_le_bytes();
    
    let authority_seeds = &[
        actual_pool.mint_a.as_ref(),
        actual_pool.mint_b.as_ref(),
        actual_pool.admin.as_ref(),
        lp_fee.as_ref(),
        &[actual_pool.pool_bump],
    ];
    let signer_seeds = &[&authority_seeds[..]];
    if swap_a {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.trader_account_a.to_account_info(),
                    to: ctx.accounts.pool_account_a.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            input,
        )?;
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_account_b.to_account_info(),
                    to: ctx.accounts.trader_account_b.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                signer_seeds,
            ),
            raw_output,
        )?;

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_account_a.to_account_info(),
                    to: ctx.accounts.treasury_mint_a.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                signer_seeds,
            ),
            to_treasury_fee,
        )?;

    } else {

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.trader_account_b.to_account_info(),
                    to: ctx.accounts.pool_account_b.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            input,
        )?;

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_account_a.to_account_info(),
                    to: ctx.accounts.trader_account_a.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                signer_seeds,
            ),
            raw_output,
        )?;

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_account_b.to_account_info(),
                    to: ctx.accounts.treasury_mint_b.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                signer_seeds,
            ),
            to_treasury_fee,
        )?;


    }


    // Verify the invariant still holds
    // Reload accounts because of the CPIs
    // We tolerate if the new invariant is higher because it means a rounding error for LPs
    ctx.accounts.pool_account_a.reload()?;
    ctx.accounts.pool_account_b.reload()?;


    if invariant_before_trade > ctx.accounts.pool_account_a.amount * ctx.accounts.pool_account_b.amount {
        return err!(FTRXSwapError::InvariantViolated);
    }




    Ok(())
}

#[derive(Accounts)]
pub struct SimpleSwapExactIn<'info> {

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



    pub mint_a: Box<Account<'info, Mint>>,

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
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_a,
        associated_token::authority = payer,
    )]
    pub trader_account_a: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_b,
        associated_token::authority = payer,
    )]
    pub trader_account_b: Box<Account<'info, TokenAccount>>,


    
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



    /// The account paying for all rents
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana ecosystem accounts
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}