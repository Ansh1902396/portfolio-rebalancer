use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
#[instruction(manager: Pubkey, rebalance_threshold: u8, min_rebalance_interval: i64)]
pub struct InitializePortfolio<'info> {
    #[account(
        init,
        payer = authority,
        space = Portfolio::SPACE,
        seeds = [b"portfolio", manager.as_ref()],
        bump
    )]
    pub portfolio: Account<'info, Portfolio>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn initialize_portfolio(
    ctx: Context<InitializePortfolio>,
    manager: Pubkey,
    rebalance_threshold: u8,
    min_rebalance_interval: i64,
) -> Result<()> {
    // Validate parameters
    require!(
        rebalance_threshold > 0 && rebalance_threshold <= 100,
        RebalancerError::InvalidRebalanceThreshold
    );
    
    require!(
        min_rebalance_interval > 0,
        RebalancerError::InvalidRebalanceInterval
    );
    
    let portfolio = &mut ctx.accounts.portfolio;
    let clock = Clock::get()?;
    
    portfolio.manager = manager;
    portfolio.rebalance_threshold = rebalance_threshold;
    portfolio.min_rebalance_interval = min_rebalance_interval;
    portfolio.last_rebalance = clock.unix_timestamp;
    portfolio.total_value = 0;
    portfolio.strategy_count = 0;
    portfolio.created_at = clock.unix_timestamp;
    portfolio.bump = ctx.bumps.portfolio;
    
    msg!(
        "Portfolio initialized for manager: {}, threshold: {}%, interval: {}s",
        manager,
        rebalance_threshold,
        min_rebalance_interval
    );
    
    Ok(())
}

// Legacy initialize function for backward compatibility
#[derive(Accounts)]
pub struct Initialize {}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    msg!("Legacy initialize called from: {:?}", ctx.program_id);
    Ok(())
}
