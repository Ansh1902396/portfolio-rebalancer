use anchor_lang::prelude::*;

#[account]
pub struct Portfolio {
    /// Portfolio manager authority
    pub manager: Pubkey,
    /// Deviation threshold for rebalancing (percentage)
    pub rebalance_threshold: u8,
    /// Minimum time between rebalances (seconds)
    pub min_rebalance_interval: i64,
    /// Last rebalance timestamp
    pub last_rebalance: i64,
    /// Total value locked in portfolio
    pub total_value: u64,
    /// Number of active strategies
    pub strategy_count: u8,
    /// Portfolio creation timestamp
    pub created_at: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl Portfolio {
    pub const SPACE: usize = 8 + // discriminator
        32 + // manager
        1 +  // rebalance_threshold
        8 +  // min_rebalance_interval
        8 +  // last_rebalance
        8 +  // total_value
        1 +  // strategy_count
        8 +  // created_at
        1;   // bump

    pub fn can_rebalance(&self, current_time: i64) -> bool {
        current_time >= self.last_rebalance + self.min_rebalance_interval
    }
}

#[account]
pub struct Strategy {
    /// Associated portfolio
    pub portfolio: Pubkey,
    /// Unique strategy identifier
    pub strategy_id: Pubkey,
    /// Protocol type (Lending, DEX, etc.)
    pub protocol_type: ProtocolType,
    /// Current balance in strategy
    pub current_balance: u64,
    /// Target allocation percentage (basis points)
    pub target_allocation: u16,
    /// Strategy creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Strategy status
    pub is_active: bool,
    /// Bump seed for PDA
    pub bump: u8,
}

impl Strategy {
    pub const SPACE: usize = 8 + // discriminator
        32 + // portfolio
        32 + // strategy_id
        1 +  // protocol_type
        8 +  // current_balance
        2 +  // target_allocation
        8 +  // created_at
        8 +  // updated_at
        1 +  // is_active
        1;   // bump

    pub fn calculate_allocation_percentage(&self, total_value: u64) -> Result<u16> {
        if total_value == 0 {
            return Ok(0);
        }
        
        let percentage = (self.current_balance as u128)
            .checked_mul(10000)
            .ok_or(error!(crate::errors::RebalancerError::MathOverflow))?
            .checked_div(total_value as u128)
            .ok_or(error!(crate::errors::RebalancerError::DivisionByZero))?;
            
        Ok(percentage as u16)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProtocolType {
    Lending,
    Dex,
    Staking,
    Vault,
    Synthetic,
}

impl ProtocolType {
    pub fn as_u8(&self) -> u8 {
        match self {
            ProtocolType::Lending => 0,
            ProtocolType::Dex => 1,
            ProtocolType::Staking => 2,
            ProtocolType::Vault => 3,
            ProtocolType::Synthetic => 4,
        }
    }
}

#[account]
pub struct RebalanceEvent {
    /// Associated portfolio
    pub portfolio: Pubkey,
    /// Event timestamp
    pub timestamp: i64,
    /// Portfolio value before rebalance
    pub value_before: u64,
    /// Portfolio value after rebalance
    pub value_after: u64,
    /// Number of strategies rebalanced
    pub strategies_count: u8,
    /// Gas cost for rebalancing
    pub gas_cost: u64,
    /// Event sequence number
    pub sequence: u64,
}

impl RebalanceEvent {
    pub const SPACE: usize = 8 + // discriminator
        32 + // portfolio
        8 +  // timestamp
        8 +  // value_before
        8 +  // value_after
        1 +  // strategies_count
        8 +  // gas_cost
        8;   // sequence
}