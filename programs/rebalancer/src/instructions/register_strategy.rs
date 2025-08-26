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
        space = Strategy::SPACE,
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
    
    // Check if portfolio can accommodate more strategies
    require!(
        portfolio.strategy_count < 255,
        RebalancerError::InvalidAllocationPercentage
    );
    
    // Initialize strategy
    strategy.portfolio = portfolio.key();
    strategy.strategy_id = strategy_id;
    strategy.protocol_type = protocol_type;
    strategy.current_balance = initial_balance;
    strategy.target_allocation = 0; // Will be set separately
    strategy.created_at = clock.unix_timestamp;
    strategy.updated_at = clock.unix_timestamp;
    strategy.is_active = true;
    strategy.bump = ctx.bumps.strategy;
    
    // Update portfolio
    portfolio.strategy_count = portfolio.strategy_count
        .checked_add(1)
        .ok_or(RebalancerError::MathOverflow)?;
    
    portfolio.total_value = portfolio.total_value
        .checked_add(initial_balance)
        .ok_or(RebalancerError::MathOverflow)?;
    
    msg!(
        "Strategy registered: ID={}, Protocol={:?}, Balance={}",
        strategy_id,
        protocol_type,
        initial_balance
    );
    
    Ok(())
}
