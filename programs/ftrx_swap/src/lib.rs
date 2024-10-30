use anchor_lang::prelude::*;

mod constants;
mod errors;
mod instructions;
mod state;

pub use instructions::*;

// Set the correct key here
declare_id!("2DpiziPDmTsZStzv7qtRfsetz2Dx1zCcronBKx2Jq192");

#[program]
pub mod ftrx_swap {
    use super::*;



    pub fn create_pool(ctx: Context<CreatePool>,lp_fee:u16,bump_pool:u8,bump_vault_a:u8,bump_vault_b:u8,bump_treas_a:u8,bump_treas_b:u8) -> Result<()> {
        instructions::create_pool(ctx,lp_fee,bump_pool,bump_vault_a,bump_vault_b,bump_treas_a,bump_treas_b)
    }

    pub fn deposit_liquidity(
        ctx: Context<DepositLiquidity>,
        amount_a: u64,
        amount_b: u64,
        expected_lp_token:u64
    ) -> Result<()> {
        instructions::deposit_liquidity(ctx, amount_a, amount_b,expected_lp_token)
    }

    pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount: u64, amount_expected_a: u64, amount_expected_b: u64) -> Result<()> {
        instructions::withdraw_liquidity(ctx, amount,amount_expected_a,amount_expected_b)
    }

    pub fn simple_swap_exact_in(ctx: Context<SimpleSwapExactIn>,swap_a: bool,input_amount: u64,min_output_amount: u64)-> Result<()> {
        instructions::simple_swap_exact_in(ctx,swap_a,input_amount,min_output_amount)
    }

    pub fn admin_gets_treasury(ctx: Context<AdminGetsTreasury>,amount_a: u64,amount_b: u64)-> Result<()> {
        instructions::admin_gets_treasury(ctx,amount_a,amount_b)
    }


    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
    
}


#[derive(Accounts)]
pub struct Initialize {}