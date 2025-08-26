use anchor_lang::prelude::*;

#[error_code]
pub enum RebalancerError {
    #[msg("Portfolio manager authority mismatch")]
    InvalidManager = 6000,
    
    #[msg("Rebalance threshold must be between 1 and 100")]
    InvalidRebalanceThreshold,
    
    #[msg("Minimum rebalance interval must be positive")]
    InvalidRebalanceInterval,
    
    #[msg("Portfolio has not reached minimum rebalance interval")]
    RebalanceIntervalNotMet,
    
    #[msg("Portfolio deviation below threshold")]
    DeviationBelowThreshold,
    
    #[msg("Strategy already registered")]
    StrategyAlreadyExists,
    
    #[msg("Strategy not found")]
    StrategyNotFound,
    
    #[msg("Invalid protocol type")]
    InvalidProtocolType,
    
    #[msg("Insufficient balance for rebalancing")]
    InsufficientBalance,
    
    #[msg("Mathematical overflow in calculations")]
    MathOverflow,
    
    #[msg("Division by zero")]
    DivisionByZero,
    
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    
    #[msg("Token account owner mismatch")]
    TokenAccountOwnerMismatch,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Portfolio not initialized")]
    PortfolioNotInitialized,
    
    #[msg("Invalid allocation percentage")]
    InvalidAllocationPercentage,
    
    #[msg("Total allocation must equal 100%")]
    InvalidTotalAllocation,
}
