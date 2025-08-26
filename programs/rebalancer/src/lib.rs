use anchor_lang::prelude::*;

declare_id!("85q2t4aLdDPQABM9kwjdCvFynWi6C75Q3sjcWT1avKVG");

pub mod state;
pub mod instructions;
pub mod errors;
pub mod constants;

use instructions::*;
use state::*;

#[program]
pub mod rebalancer {
    use super::*;
    
    pub fn initialize_portfolio(
        ctx: Context<InitializePortfolio>,
        manager: Pubkey,
        rebalance_threshold: u8,
        min_rebalance_interval: i64,
    ) -> Result<()> {
        instructions::initialize_portfolio(ctx, manager, rebalance_threshold, min_rebalance_interval)
    }
    
    pub fn register_strategy(
        ctx: Context<RegisterStrategy>,
        strategy_id: Pubkey,
        protocol_type: ProtocolType,
        initial_balance: u64,
    ) -> Result<()> {
        instructions::register_strategy(ctx, strategy_id, protocol_type, initial_balance)
    }
    
    // Legacy initialize function for backward compatibility
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::handler(ctx)
    }
}
