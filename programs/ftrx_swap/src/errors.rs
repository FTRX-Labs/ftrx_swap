use anchor_lang::prelude::*;

#[error_code]
pub enum FTRXSwapError {
    #[msg("Invalid fee value")]
    InvalidFee,

    #[msg("Invalid mint for the pool")]
    InvalidMint,

    #[msg("Depositing too little liquidity")]
    DepositTooSmall,

    #[msg("Output is below the minimum expected")]
    OutputTooSmall,

    #[msg("Invariant does not hold")]
    InvariantViolated,

    #[msg("New invariant does not hold")]
    NewInvariantViolated,

    #[msg("OverFlow in swap calculation")]
    MathOverflow,

    #[msg("Inconsistent price ratio vs liquidity impact")]
    InconsistentPriceRatioLiquidity,

    #[msg("Slippage exceeded")]
    SlippageExceeded,

    

}