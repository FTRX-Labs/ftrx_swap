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

    pub fn create_amm(ctx: Context<CreateAmm>, id: Pubkey, fee: u16) -> Result<()> {
        instructions::create_amm(ctx, id, fee)
    }

    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
        instructions::create_pool(ctx)
    }

    pub fn deposit_liquidity(
        ctx: Context<DepositLiquidity>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        instructions::deposit_liquidity(ctx, amount_a, amount_b)
    }

    pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount: u64) -> Result<()> {
        instructions::withdraw_liquidity(ctx, amount)
    }

    pub fn simple_swap(ctx: Context<SimpleSwap>,swap_a: bool,input_amount: u64,min_output_amount: u64)-> Result<()> {
        instructions::simple_swap(ctx,swap_a,input_amount,min_output_amount)
    }


    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
    
}


#[derive(Accounts)]
pub struct Initialize {}