use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
#[instruction(strategy_id: Pubkey, protocol_type: ProtocolType, initial_balance: u64)]
pub struct RegisterStrategy<'info> {
    #[account(
        mut,
        seeds = [b"portfolio", portfolio.manager.as_ref()],
        bump = portfolio.bump,
        constraint = portfolio.manager == authority.key() @ RebalancerError::InvalidManager
    )]
    pub portfolio: Account<'info, Portfolio>,
    
    #[account(
        init,
        payer = authority,
        space = Strategy::MAX_SIZE,
        seeds = [b"strategy", portfolio.key().as_ref(), strategy_id.as_ref()],
        bump
    )]
    pub strategy: Account<'info, Strategy>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn register_strategy(
    ctx: Context<RegisterStrategy>,
    strategy_id: Pubkey,
    protocol_type: ProtocolType,
    initial_balance: u64,
) -> Result<()> {
    let portfolio = &mut ctx.accounts.portfolio;
    let strategy = &mut ctx.accounts.strategy;
    let clock = Clock::get()?;
    
    // Validate protocol type
    protocol_type.validate()?;
    
    // Validate balance update
    Strategy::validate_balance_update(initial_balance)?;
    
    // Check emergency pause
    require!(!portfolio.emergency_pause, RebalancerError::EmergencyPauseActive);
    
    // Check if portfolio can accommodate more strategies
    require!(
        portfolio.total_strategies < u32::MAX,
        RebalancerError::MathOverflow
    );
    
    // Initialize strategy with new structure
    strategy.strategy_id = strategy_id;
    strategy.protocol_type = protocol_type;
    strategy.current_balance = initial_balance;
    strategy.yield_rate = 0; // Will be updated by oracle
    strategy.volatility_score = 0; // Will be calculated
    strategy.performance_score = 0; // Will be calculated
    strategy.percentile_rank = 0; // Will be calculated
    strategy.last_updated = clock.unix_timestamp;
    strategy.status = StrategyStatus::Active;
    strategy.total_deposits = initial_balance;
    strategy.total_withdrawals = 0;
    strategy.creation_time = clock.unix_timestamp;
    strategy.bump = ctx.bumps.strategy;
    strategy.reserved = [0; 23];
    
    // Update portfolio with saturating arithmetic
    portfolio.total_strategies = portfolio.total_strategies
        .saturating_add(1);
    
    portfolio.total_capital_moved = portfolio.total_capital_moved
        .saturating_add(initial_balance);
    
    msg!(
        "Strategy registered: ID={}, Protocol={}, Balance={}",
        strategy_id,
        protocol_type.get_protocol_name(),
        initial_balance
    );
    
    Ok(())
}
