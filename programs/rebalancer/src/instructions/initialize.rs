use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(manager: Pubkey, rebalance_threshold: u8, min_rebalance_interval: i64)]
pub struct InitializePortfolio<'info> {
    #[account(
        init,
        payer = authority,
        space = Portfolio::MAX_SIZE,
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
    // Validate parameters using new validation methods
    Portfolio::validate_rebalance_threshold(rebalance_threshold)?;
    Portfolio::validate_min_interval(min_rebalance_interval)?;
    
    let portfolio = &mut ctx.accounts.portfolio;
    let clock = Clock::get()?;
    
    portfolio.manager = manager;
    portfolio.rebalance_threshold = rebalance_threshold;
    portfolio.total_strategies = 0;
    portfolio.total_capital_moved = 0;
    portfolio.last_rebalance = clock.unix_timestamp;
    portfolio.min_rebalance_interval = min_rebalance_interval;
    portfolio.portfolio_creation = clock.unix_timestamp;
    portfolio.emergency_pause = false;
    portfolio.performance_fee_bps = 200; // Default 2% performance fee
    portfolio.bump = ctx.bumps.portfolio;
    portfolio.reserved = [0; 31];
    
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
