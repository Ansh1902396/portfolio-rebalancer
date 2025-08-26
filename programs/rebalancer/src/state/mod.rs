use anchor_lang::prelude::*;

#[account]
#[derive(Debug)]
pub struct Portfolio {
    pub manager: Pubkey,                    // 32 bytes - Portfolio manager authority
    pub rebalance_threshold: u8,            // 1 byte - Bottom % for reallocation (1-50)
    pub total_strategies: u32,              // 4 bytes - Current strategy count
    pub total_capital_moved: u64,           // 8 bytes - Lifetime capital rebalanced (lamports)
    pub last_rebalance: i64,                // 8 bytes - Unix timestamp of last rebalance
    pub min_rebalance_interval: i64,        // 8 bytes - Minimum seconds between rebalances
    pub portfolio_creation: i64,            // 8 bytes - Portfolio creation timestamp
    pub emergency_pause: bool,              // 1 byte - Emergency stop flag
    pub performance_fee_bps: u16,           // 2 bytes - Performance fee in basis points
    pub bump: u8,                           // 1 byte - PDA bump seed
    pub reserved: [u8; 31],                 // 31 bytes - Future expansion buffer
}
// Total: 136 bytes

#[account]
#[derive(Debug)]
pub struct Strategy {
    pub strategy_id: Pubkey,                // 32 bytes - Unique strategy identifier
    pub protocol_type: ProtocolType,        // Variable size - Protocol-specific data
    pub current_balance: u64,               // 8 bytes - Current capital allocated (lamports)
    pub yield_rate: u64,                    // 8 bytes - Annual yield in basis points (0-50000)
    pub volatility_score: u32,              // 4 bytes - Risk metric (0-10000, 100.00% max)
    pub performance_score: u64,             // 8 bytes - Calculated composite score
    pub percentile_rank: u8,                // 1 byte - 0-100 ranking position
    pub last_updated: i64,                  // 8 bytes - Last metric update timestamp
    pub status: StrategyStatus,             // 1 byte - Current strategy status
    pub total_deposits: u64,                // 8 bytes - Lifetime deposits tracking
    pub total_withdrawals: u64,             // 8 bytes - Lifetime withdrawals tracking
    pub creation_time: i64,                 // 8 bytes - Strategy creation timestamp
    pub bump: u8,                           // 1 byte - PDA bump seed
    pub reserved: [u8; 23],                 // 23 bytes - Future expansion
}
// Total: ~144 bytes + protocol_type size

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum ProtocolType {
    StableLending { 
        pool_id: Pubkey,                    // 32 bytes - Solend pool identifier
        utilization: u16,                   // 2 bytes - Pool utilization in basis points
        reserve_address: Pubkey,            // 32 bytes - Reserve account address
    },  // 66 bytes total
    YieldFarming { 
        pair_id: Pubkey,                    // 32 bytes - Orca pair identifier
        reward_multiplier: u8,              // 1 byte - Reward boost (1-10x)
        token_a_mint: Pubkey,               // 32 bytes - Token A mint address
        token_b_mint: Pubkey,               // 32 bytes - Token B mint address
        fee_tier: u16,                      // 2 bytes - Pool fee in basis points
    },  // 99 bytes total
    LiquidStaking { 
        validator_id: Pubkey,               // 32 bytes - Marinade validator
        commission: u16,                    // 2 bytes - Validator commission (basis points)
        stake_pool: Pubkey,                 // 32 bytes - Stake pool address
        unstake_delay: u32,                 // 4 bytes - Unstaking delay in epochs
    },  // 70 bytes total
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum StrategyStatus {
    Active,      // Normal operation, participates in rebalancing
    Paused,      // Temporarily disabled, no new allocations
    Deprecated,  // Marked for removal, extract capital when possible
}

#[account]
#[derive(Debug)]
pub struct CapitalPosition {
    pub strategy_id: Pubkey,                // 32 bytes - Reference to strategy
    pub token_a_amount: u64,                // 8 bytes - Token A quantity
    pub token_b_amount: u64,                // 8 bytes - Token B quantity (0 for single asset)
    pub lp_tokens: u64,                     // 8 bytes - LP tokens held
    pub platform_controlled_lp: u64,       // 8 bytes - LP tokens under platform control
    pub position_type: PositionType,        // 1 byte - Position classification
    pub entry_price_a: u64,                 // 8 bytes - Entry price token A (6 decimals)
    pub entry_price_b: u64,                 // 8 bytes - Entry price token B (6 decimals)
    pub last_rebalance: i64,                // 8 bytes - Last position update
    pub accrued_fees: u64,                  // 8 bytes - Accumulated fees in position
    pub impermanent_loss: i64,              // 8 bytes - IL tracking (can be negative)
    pub bump: u8,                           // 1 byte - PDA bump seed
    pub reserved: [u8; 15],                 // 15 bytes - Future expansion
}
// Total: 145 bytes

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum PositionType {
    SingleAsset,
    LiquidityPair,
    StakedPosition,
}

impl Portfolio {
    pub const MAX_SIZE: usize = 8 + 136;
    
    pub fn validate_rebalance_threshold(threshold: u8) -> Result<()> {
        require!((1..=50).contains(&threshold), crate::errors::RebalancerError::InvalidRebalanceThreshold);
        Ok(())
    }
    
    pub fn can_rebalance(&self, current_time: i64) -> bool {
        !self.emergency_pause && 
        current_time >= self.last_rebalance.saturating_add(self.min_rebalance_interval)
    }
    
    pub fn validate_min_interval(interval: i64) -> Result<()> {
        require!((3600..=86400).contains(&interval), crate::errors::RebalancerError::InvalidRebalanceInterval);
        Ok(())
    }
}

impl Strategy {
    pub const MAX_SIZE: usize = 8 + 200; // Account for largest protocol type
    
    pub fn validate_yield_rate(rate: u64) -> Result<()> {
        require!(rate <= 50000, crate::errors::RebalancerError::InvalidAllocationPercentage);
        Ok(())
    }
    
    pub fn validate_balance_update(new_balance: u64) -> Result<()> {
        require!(new_balance < u64::MAX / 1000, crate::errors::RebalancerError::MathOverflow);
        Ok(())
    }
    
    pub fn validate_volatility_score(score: u32) -> Result<()> {
        require!(score <= 10000, crate::errors::RebalancerError::InvalidAllocationPercentage);
        Ok(())
    }
}

impl ProtocolType {
    pub fn validate(&self) -> Result<()> {
        match self {
            ProtocolType::StableLending { pool_id, utilization, reserve_address } => {
                require!(*pool_id != Pubkey::default(), crate::errors::RebalancerError::InvalidProtocolType);
                require!(*reserve_address != Pubkey::default(), crate::errors::RebalancerError::InvalidProtocolType);
                require!(*utilization <= 10000, crate::errors::RebalancerError::InvalidAllocationPercentage);
                Ok(())
            },
            ProtocolType::YieldFarming { 
                pair_id, reward_multiplier, token_a_mint, token_b_mint, fee_tier 
            } => {
                require!(*pair_id != Pubkey::default(), crate::errors::RebalancerError::InvalidProtocolType);
                require!(*token_a_mint != Pubkey::default(), crate::errors::RebalancerError::InvalidTokenMint);
                require!(*token_b_mint != Pubkey::default(), crate::errors::RebalancerError::InvalidTokenMint);
                require!(*token_a_mint != *token_b_mint, crate::errors::RebalancerError::InvalidTokenMint);
                require!(*reward_multiplier >= 1 && *reward_multiplier <= 10, crate::errors::RebalancerError::InvalidAllocationPercentage);
                require!(*fee_tier <= 1000, crate::errors::RebalancerError::InvalidAllocationPercentage);
                Ok(())
            },
            ProtocolType::LiquidStaking { 
                validator_id, commission, stake_pool, unstake_delay 
            } => {
                require!(*validator_id != Pubkey::default(), crate::errors::RebalancerError::InvalidProtocolType);
                require!(*stake_pool != Pubkey::default(), crate::errors::RebalancerError::InvalidProtocolType);
                require!(*commission <= 1000, crate::errors::RebalancerError::InvalidAllocationPercentage);
                require!(*unstake_delay <= 50, crate::errors::RebalancerError::InvalidAllocationPercentage);
                Ok(())
            },
        }
    }
    
    pub fn get_protocol_name(&self) -> &'static str {
        match self {
            ProtocolType::StableLending { .. } => "Stable Lending",
            ProtocolType::YieldFarming { .. } => "Yield Farming",
            ProtocolType::LiquidStaking { .. } => "Liquid Staking",
        }
    }
}

impl CapitalPosition {
    pub const MAX_SIZE: usize = 8 + 145;
}