use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount, Transfer},
};
use fixed::types::I64F64;
use fixed_sqrt::FixedSqrt;

use crate::{
    constants::{AUTHORITY_SEED, LIQUIDITY_SEED, MINIMUM_LIQUIDITY},
    errors::FTRXSwapError,
    state::SimplePool,
};

pub fn deposit_liquidity(
    ctx: Context<DepositLiquidity>,
    amount_a: u64,
    amount_b: u64,
    expected_lp_token:u64
) -> Result<()> {

    // Prevent depositing assets the depositor does not own
    let mut amount_a = if amount_a > ctx.accounts.depositor_account_a.amount {
        ctx.accounts.depositor_account_a.amount
    } else {
        amount_a
    };
    let mut amount_b = if amount_b > ctx.accounts.depositor_account_b.amount {
        ctx.accounts.depositor_account_b.amount
    } else {
        amount_b
    };

    let pool_a = &ctx.accounts.pool_account_a;
    let pool_b = &ctx.accounts.pool_account_b;

    //Saving amounts of token a and b for end of instruction checks
    let amount_a_before=I64F64::from_num(pool_a.amount);
    let amount_b_before=I64F64::from_num(pool_b.amount);

    // Is it the first time we deposit amounts in the pool ?
    let pool_creation = pool_a.amount == 0 && pool_b.amount == 0;

    //Calculating price as a ratio before deposit if there is available liquidity
    let mut a_sup_b=false;
    let mut ratio_price_check=I64F64::from_num(1);
    if !pool_creation{
   
        //Calculating ratio before deposit assuming poolb has more token than poola
        //For precision, we'll evaluate a/b if a>b, and b/a otherwise
        if pool_a.amount>pool_b.amount{
            a_sup_b=true;
            ratio_price_check=I64F64::from_num(pool_a.amount).checked_div(I64F64::from_num(pool_b.amount)).unwrap();
        }else{
            ratio_price_check=I64F64::from_num(pool_b.amount).checked_div(I64F64::from_num(pool_a.amount)).unwrap();
        
        }
    }
    //DONE - Calculating price as a ratio before deposit if there is available liquidity

    // Initializing or making sure the price ratio constraint is respected
    (amount_a, amount_b) = if pool_creation {
        // Add as is if there is no liquidity
        (amount_a, amount_b)
    } else {
        if a_sup_b{
            // ratio_price_check is a/b and added_a should be added_b*ratio_price_check
            (
                I64F64::from_num(amount_b)
                    .checked_mul(ratio_price_check)
                    .unwrap().ceil()
                    .to_num::<u64>(),
                amount_b,
            )
            
        } else {
            // ratio_price_check is b/a and added_b should be added_a*ratio_price_check
            (
                amount_a,
                I64F64::from_num(amount_a)
                    .checked_mul(ratio_price_check)
                    .unwrap().ceil()
                    .to_num::<u64>(),
            )
        }
    };


    let  mint_liquidity_supply_before = ctx.accounts.mint_liquidity.supply + MINIMUM_LIQUIDITY;


    // Computing the amount of liquidity about to be deposited
    // This formula will only be used for the pool creation ie the first deposit
    let mut liquidity = I64F64::from_num(amount_a)
        .checked_mul(I64F64::from_num(amount_b))
        .unwrap()
        .sqrt()
        .floor()
        .to_num::<u64>();
    

    // Lock some minimum liquidity on the first deposit
    if pool_creation {
        if liquidity < MINIMUM_LIQUIDITY {
            return err!(FTRXSwapError::DepositTooSmall);
        }

        liquidity -= MINIMUM_LIQUIDITY;
  
    }else{
        // For all deposits after the very first successful deposit, this is the relevant formula 
        // for calculating the amount of LP token to mint
        liquidity =I64F64::from_num(mint_liquidity_supply_before).checked_mul(I64F64::from_num(amount_a)).unwrap().checked_div(I64F64::from_num(pool_a.amount)).unwrap().floor().to_num::<u64>();
    }

    // Transfer tokens to the pool
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.depositor_account_a.to_account_info(),
                to: ctx.accounts.pool_account_a.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        amount_a,
    )?;
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.depositor_account_b.to_account_info(),
                to: ctx.accounts.pool_account_b.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        amount_b,
    )?;

    // Mint the liquidity to user

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
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint_liquidity.to_account_info(),
                to: ctx.accounts.depositor_account_liquidity.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        liquidity,
    )?;

    //Making end of instruction checks
    if liquidity<expected_lp_token{
        return err!(FTRXSwapError::SlippageExceeded);
    }
    //We reload amounts
    ctx.accounts.pool_account_a.reload()?;
    ctx.accounts.pool_account_b.reload()?;
    ctx.accounts.mint_liquidity.reload()?;

    let  mint_liquidity_supply_after = ctx.accounts.mint_liquidity.supply + MINIMUM_LIQUIDITY;


    let new_pool_a_amount=ctx.accounts.pool_account_a.amount;
    let new_pool_b_amount=ctx.accounts.pool_account_b.amount;


    // If its not pool creation, we can make additional checks.

    // Checking the liquidity ratios vs new token ratios are in favor of the lp
    //These are potentially triggering an error preventing from the tx to complete
    // We want to have added_a/a_before > added_lp_token_supply/lp_token_supply
    //and same for b
    if !pool_creation{

        let ratio_a_check_after=I64F64::from_num(amount_a).checked_div(I64F64::from_num(amount_a_before)).unwrap();
        let ratio_b_check_after=I64F64::from_num(amount_b).checked_div(I64F64::from_num(amount_b_before)).unwrap();
        let ratio_supply_check_after=I64F64::from_num(liquidity).checked_div(I64F64::from_num(mint_liquidity_supply_before)).unwrap();
        
        if ratio_supply_check_after>ratio_a_check_after || ratio_supply_check_after>ratio_b_check_after{
            return err!(FTRXSwapError::InconsistentPriceRatioLiquidity);
        }


    }

    Ok(())
}

#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
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


    /// The account paying for all rents
    pub depositor: Signer<'info>,

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
        associated_token::mint = mint_liquidity,
        associated_token::authority = depositor,
    )]
    pub depositor_account_liquidity: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        associated_token::mint = mint_a,
        associated_token::authority = depositor,
    )]
    pub depositor_account_a: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        associated_token::mint = mint_b,
        associated_token::authority = depositor,
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