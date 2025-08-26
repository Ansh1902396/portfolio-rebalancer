use anchor_lang::prelude::*;

#[error_code]
pub enum RebalancerError {
    #[msg("Portfolio manager authority mismatch")]
    InvalidManager = 6000,
    
    #[msg("Rebalance threshold must be between 1 and 50")]
    InvalidRebalanceThreshold,
    
    #[msg("Minimum rebalance interval must be between 3600 and 86400 seconds")]
    InvalidRebalanceInterval,
    
    #[msg("Portfolio has not reached minimum rebalance interval")]
    RebalanceIntervalNotMet,
    
    #[msg("Portfolio deviation below threshold")]
    DeviationBelowThreshold,
    
    #[msg("Strategy already registered")]
    StrategyAlreadyExists,
    
    #[msg("Strategy not found")]
    StrategyNotFound,
    
    #[msg("Invalid protocol type or configuration")]
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
    
    #[msg("Invalid allocation percentage or parameter")]
    InvalidAllocationPercentage,
    
    #[msg("Total allocation must equal 100%")]
    InvalidTotalAllocation,
    
    #[msg("Emergency pause is active")]
    EmergencyPauseActive,
    
    #[msg("Excessive yield rate specified")]
    ExcessiveYieldRate,
    
    #[msg("Balance would cause overflow")]
    BalanceOverflow,
    
    #[msg("Invalid volatility score")]
    InvalidVolatilityScore,
    
    #[msg("Invalid pool identifier")]
    InvalidPoolId,
    
    #[msg("Invalid reserve address")]
    InvalidReserveAddress,
    
    #[msg("Invalid utilization percentage")]
    InvalidUtilization,
    
    #[msg("Invalid pair identifier")]
    InvalidPairId,
    
    #[msg("Duplicate token mints not allowed")]
    DuplicateTokenMints,
    
    #[msg("Invalid reward multiplier")]
    InvalidRewardMultiplier,
    
    #[msg("Invalid fee tier")]
    InvalidFeeTier,
    
    #[msg("Invalid validator identifier")]
    InvalidValidatorId,
    
    #[msg("Invalid stake pool")]
    InvalidStakePool,
    
    #[msg("Invalid commission rate")]
    InvalidCommission,
    
    #[msg("Invalid unstake delay")]
    InvalidUnstakeDelay,
    
    #[msg("Insufficient strategies for rebalancing (minimum 2 required)")]
    InsufficientStrategies,
}
