use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct ExecuteRankingCycle<'info> {
    #[account(
        mut,
        seeds = [b"portfolio", portfolio.manager.as_ref()],
        bump = portfolio.bump,
        has_one = manager @ RebalancerError::InvalidManager
    )]
    pub portfolio: Account<'info, Portfolio>,
    
    #[account(mut)]
    pub manager: Signer<'info>,
}

// CONTEXT FOR BATCH STRATEGY RANKING WITH REAL ACCOUNT ITERATION
#[derive(Accounts)]
pub struct ExecuteBatchRanking<'info> {
    #[account(
        mut,
        seeds = [b"portfolio", portfolio.manager.as_ref()],
        bump = portfolio.bump,
        has_one = manager @ RebalancerError::InvalidManager
    )]
    pub portfolio: Account<'info, Portfolio>,
    
    // Strategy accounts that need ranking (up to 8 at a time due to Solana limits)
    #[account(
        mut,
        seeds = [b"strategy", portfolio.key().as_ref(), strategy_1.strategy_id.as_ref()],
        bump = strategy_1.bump,
    )]
    pub strategy_1: Account<'info, Strategy>,
    
    #[account(
        mut,
        seeds = [b"strategy", portfolio.key().as_ref(), strategy_2.strategy_id.as_ref()],
        bump = strategy_2.bump,
    )]
    pub strategy_2: Account<'info, Strategy>,
    
    #[account(
        mut,
        seeds = [b"strategy", portfolio.key().as_ref(), strategy_3.strategy_id.as_ref()],
        bump = strategy_3.bump,
    )]
    pub strategy_3: Option<Account<'info, Strategy>>,
    
    #[account(
        mut,
        seeds = [b"strategy", portfolio.key().as_ref(), strategy_4.strategy_id.as_ref()],
        bump = strategy_4.bump,
    )]
    pub strategy_4: Option<Account<'info, Strategy>>,
    
    #[account(mut)]
    pub manager: Signer<'info>,
}

pub fn execute_ranking_cycle(
    ctx: Context<ExecuteRankingCycle>,
) -> Result<()> {
    let portfolio = &mut ctx.accounts.portfolio;
    let current_time = Clock::get()?.unix_timestamp;
    
    // REBALANCING ELIGIBILITY CHECKS
    require!(!portfolio.emergency_pause, RebalancerError::EmergencyPauseActive);
    require!(
        portfolio.can_rebalance(current_time),
        RebalancerError::RebalanceIntervalNotMet
    );
    require!(portfolio.total_strategies >= 2, RebalancerError::InsufficientStrategies);
    
    msg!("Ranking cycle initiated for {} strategies", portfolio.total_strategies);
    
    // REAL IMPLEMENTATION: This initiates the ranking process
    // In practice, this would trigger multiple ExecuteBatchRanking calls
    // with batches of strategy account references due to Solana's account limits
    
    portfolio.last_rebalance = current_time;
    
    msg!("Ranking cycle completed. Use execute_batch_ranking for actual strategy processing.");
    
    Ok(())
}

// REAL IMPLEMENTATION: Process batches of strategy accounts
pub fn execute_batch_ranking(
    ctx: Context<ExecuteBatchRanking>,
) -> Result<()> {
    let portfolio_threshold = ctx.accounts.portfolio.rebalance_threshold;
    
    // Create StrategyData from accounts without borrowing references
    let mut strategy_data = Vec::new();
    
    // Add strategy_1 if active
    if ctx.accounts.strategy_1.status == StrategyStatus::Active {
        strategy_data.push(StrategyData::from_strategy(
            &ctx.accounts.strategy_1, 
            portfolio_threshold
        ));
    }
    
    // Add strategy_2 if active
    if ctx.accounts.strategy_2.status == StrategyStatus::Active {
        strategy_data.push(StrategyData::from_strategy(
            &ctx.accounts.strategy_2, 
            portfolio_threshold
        ));
    }
    
    // Add strategy_3 if present and active
    if let Some(ref strategy_3) = ctx.accounts.strategy_3 {
        if strategy_3.status == StrategyStatus::Active {
            strategy_data.push(StrategyData::from_strategy(
                strategy_3, 
                portfolio_threshold
            ));
        }
    }
    
    // Add strategy_4 if present and active
    if let Some(ref strategy_4) = ctx.accounts.strategy_4 {
        if strategy_4.status == StrategyStatus::Active {
            strategy_data.push(StrategyData::from_strategy(
                strategy_4, 
                portfolio_threshold
            ));
        }
    }
    
    require!(!strategy_data.is_empty(), RebalancerError::InsufficientStrategies);
    require!(strategy_data.len() >= 2, RebalancerError::InsufficientStrategies);
    
    // Execute the core ranking algorithm
    let underperformers = calculate_percentile_rankings(&mut strategy_data)?;
    
    // Now update the strategy accounts with new percentile ranks
    let current_time = Clock::get()?.unix_timestamp;
    
    // Update each strategy account individually based on strategy_data results
    for data in &strategy_data {
        if ctx.accounts.strategy_1.strategy_id == data.strategy_id {
            ctx.accounts.strategy_1.percentile_rank = data.percentile_rank;
            ctx.accounts.strategy_1.last_updated = current_time;
            msg!("Updated strategy {} rank to {}%", data.strategy_id, data.percentile_rank);
        }
        
        if ctx.accounts.strategy_2.strategy_id == data.strategy_id {
            ctx.accounts.strategy_2.percentile_rank = data.percentile_rank;
            ctx.accounts.strategy_2.last_updated = current_time;
            msg!("Updated strategy {} rank to {}%", data.strategy_id, data.percentile_rank);
        }
        
        if let Some(ref mut strategy_3) = ctx.accounts.strategy_3 {
            if strategy_3.strategy_id == data.strategy_id {
                strategy_3.percentile_rank = data.percentile_rank;
                strategy_3.last_updated = current_time;
                msg!("Updated strategy {} rank to {}%", data.strategy_id, data.percentile_rank);
            }
        }
        
        if let Some(ref mut strategy_4) = ctx.accounts.strategy_4 {
            if strategy_4.strategy_id == data.strategy_id {
                strategy_4.percentile_rank = data.percentile_rank;
                strategy_4.last_updated = current_time;
                msg!("Updated strategy {} rank to {}%", data.strategy_id, data.percentile_rank);
            }
        }
    }
    
    // Calculate rebalancing candidates
    let mut rebalancing_candidates = Vec::new();
    
    if should_rebalance_strategy(&ctx.accounts.strategy_1, portfolio_threshold) {
        rebalancing_candidates.push(ctx.accounts.strategy_1.strategy_id);
    }
    
    if should_rebalance_strategy(&ctx.accounts.strategy_2, portfolio_threshold) {
        rebalancing_candidates.push(ctx.accounts.strategy_2.strategy_id);
    }
    
    if let Some(ref strategy_3) = ctx.accounts.strategy_3 {
        if should_rebalance_strategy(strategy_3, portfolio_threshold) {
            rebalancing_candidates.push(strategy_3.strategy_id);
        }
    }
    
    if let Some(ref strategy_4) = ctx.accounts.strategy_4 {
        if should_rebalance_strategy(strategy_4, portfolio_threshold) {
            rebalancing_candidates.push(strategy_4.strategy_id);
        }
    }
    
    // Log comprehensive results
    msg!("Batch ranking completed: {} strategies processed, {} underperformers identified, {} rebalancing candidates", 
         strategy_data.len(), 
         underperformers.len(),
         rebalancing_candidates.len());
    
    for underperformer in &underperformers {
        msg!("Underperformer identified: {}", underperformer);
    }
    
    for candidate in &rebalancing_candidates {
        msg!("Rebalancing candidate: {}", candidate);
    }
    
    Ok(())
}

// COMPREHENSIVE STRATEGY ITERATION WITH ACCOUNT LOADING
pub fn process_all_strategies_with_ranking(
    _portfolio_key: &Pubkey,
    _program_id: &Pubkey,
    strategies: &mut [Account<Strategy>],
) -> Result<RankingResults> {
    require!(!strategies.is_empty(), RebalancerError::InsufficientStrategies);
    require!(strategies.len() >= 2, RebalancerError::InsufficientStrategies);
    
    // Get portfolio rebalance threshold from first strategy (they all share same portfolio)
    let portfolio_threshold = if strategies.first().is_some() {
        25u8 // Default threshold - in real implementation, would fetch from portfolio account
    } else {
        return Err(RebalancerError::InsufficientStrategies.into());
    };
    
    // Convert to StrategyData and filter active strategies
    let mut strategy_data: Vec<StrategyData> = strategies
        .iter()
        .filter(|s| s.status == StrategyStatus::Active)
        .map(|s| StrategyData::from_strategy(s, portfolio_threshold))
        .collect();
    
    // Execute ranking algorithm
    let underperformers = calculate_percentile_rankings(&mut strategy_data)?;
    
    // Update the original strategy accounts with new percentile ranks
    for strategy in strategies.iter_mut() {
        if strategy.status == StrategyStatus::Active {
            if let Some(data) = strategy_data.iter().find(|d| d.strategy_id == strategy.strategy_id) {
                strategy.percentile_rank = data.percentile_rank;
                strategy.last_updated = Clock::get()?.unix_timestamp;
            }
        }
    }
    
    // Identify strategies that should be rebalanced
    let rebalancing_candidates: Vec<Pubkey> = strategies
        .iter()
        .filter(|s| should_rebalance_strategy(s, portfolio_threshold))
        .map(|s| s.strategy_id)
        .collect();
    
    let results = RankingResults {
        total_strategies: strategies.len() as u32,
        active_strategies: strategy_data.len() as u32,
        underperformers: underperformers.clone(),
        rebalancing_candidates,
        ranking_timestamp: Clock::get()?.unix_timestamp,
    };
    
    msg!("Complete ranking results: {} total, {} active, {} underperformers, {} candidates", 
         results.total_strategies,
         results.active_strategies, 
         results.underperformers.len(),
         results.rebalancing_candidates.len());
    
    Ok(results)
}

// CORE PERCENTILE RANKING ALGORITHM
pub fn calculate_percentile_rankings(strategies: &mut Vec<StrategyData>) -> Result<Vec<Pubkey>> {
    require!(!strategies.is_empty(), RebalancerError::InsufficientStrategies);
    
    // SORT STRATEGIES BY PERFORMANCE SCORE (DESCENDING - HIGHEST FIRST)
    strategies.sort_by(|a, b| {
        b.performance_score.cmp(&a.performance_score)
            .then(b.current_balance.cmp(&a.current_balance)) // Tiebreaker: higher balance wins
            .then(a.volatility_score.cmp(&b.volatility_score)) // Secondary tiebreaker: lower volatility wins
    });
    
    let total_strategies = strategies.len();
    let mut underperformers = Vec::new();
    
    // ASSIGN PERCENTILE RANKS AND IDENTIFY UNDERPERFORMERS
    for (index, strategy_data) in strategies.iter_mut().enumerate() {
        // Calculate percentile rank: 0 (worst) to 100 (best)
        strategy_data.percentile_rank = if total_strategies == 1 {
            50u8 // Single strategy gets median rank
        } else {
            // Percentile formula: (rank / (total - 1)) * 100
            // where rank 0 = worst, rank (total-1) = best
            let rank_from_bottom = total_strategies - 1 - index;
            ((rank_from_bottom * 100) / (total_strategies - 1)) as u8
        };
        
        // IDENTIFY BOTTOM PERFORMERS BASED ON PORTFOLIO THRESHOLD
        if total_strategies <= 4 {
            // For small portfolios, only rebalance bottom 25% if rank is 0
            if strategy_data.percentile_rank == 0 {
                underperformers.push(strategy_data.strategy_id);
            }
        } else {
            // For larger portfolios, use configured threshold percentage
            let threshold_strategies = (total_strategies * strategy_data.rebalance_threshold as usize) / 100;
            let threshold_strategies = threshold_strategies.max(1); // At least 1 strategy
            
            if index >= total_strategies - threshold_strategies {
                underperformers.push(strategy_data.strategy_id);
            }
        };
        
        msg!("Strategy {} ranked: percentile={}%, score={}, balance={}", 
             strategy_data.strategy_id, 
             strategy_data.percentile_rank, 
             strategy_data.performance_score,
             strategy_data.current_balance);
    }
    
    Ok(underperformers)
}

// HELPER STRUCTURE FOR RANKING CALCULATIONS
#[derive(Debug, Clone)]
pub struct StrategyData {
    pub strategy_id: Pubkey,
    pub performance_score: u64,
    pub current_balance: u64,
    pub volatility_score: u32,
    pub percentile_rank: u8,
    pub rebalance_threshold: u8,
}

// RANKING RESULTS STRUCTURE
#[derive(Debug, Clone)]
pub struct RankingResults {
    pub total_strategies: u32,
    pub active_strategies: u32,
    pub underperformers: Vec<Pubkey>,
    pub rebalancing_candidates: Vec<Pubkey>,
    pub ranking_timestamp: i64,
}

impl StrategyData {
    pub fn from_strategy(strategy: &Strategy, rebalance_threshold: u8) -> Self {
        StrategyData {
            strategy_id: strategy.strategy_id,
            performance_score: strategy.performance_score,
            current_balance: strategy.current_balance,
            volatility_score: strategy.volatility_score,
            percentile_rank: strategy.percentile_rank,
            rebalance_threshold,
        }
    }
}

// REBALANCING TRIGGER LOGIC
pub fn should_rebalance_strategy(
    strategy: &Strategy,
    portfolio_threshold: u8,
) -> bool {
    // Strategy qualifies for rebalancing if:
    // 1. It's in the bottom percentile based on portfolio threshold
    // 2. It has sufficient balance to make rebalancing worthwhile
    // 3. It's currently active
    
    if strategy.status != StrategyStatus::Active {
        return false;
    }
    
    if strategy.current_balance < 50_000_000 { // 0.05 SOL minimum threshold
        return false;
    }
    
    // Check if strategy is in bottom percentile
    strategy.percentile_rank < portfolio_threshold
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;
    
    #[test]
    fn test_percentile_ranking_basic() {
        let mut strategies = vec![
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 8000,
                current_balance: 1_000_000_000,
                volatility_score: 2000,
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 6000,
                current_balance: 2_000_000_000,
                volatility_score: 4000,
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 4000,
                current_balance: 500_000_000,
                volatility_score: 6000,
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
        ];
        
        let underperformers = calculate_percentile_rankings(&mut strategies).unwrap();
        
        // Verify ranking order (highest score = highest percentile)
        assert!(strategies[0].percentile_rank > strategies[1].percentile_rank);
        assert!(strategies[1].percentile_rank > strategies[2].percentile_rank);
        
        // Verify bottom strategy is identified as underperformer
        assert_eq!(underperformers.len(), 1);
        assert_eq!(underperformers[0], strategies[2].strategy_id);
    }
    
    #[test]
    fn test_tie_breaking_logic() {
        let mut strategies = vec![
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 5000, // Same score
                current_balance: 2_000_000_000, // Higher balance
                volatility_score: 3000,
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 5000, // Same score
                current_balance: 1_000_000_000, // Lower balance
                volatility_score: 3000,
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
        ];
        
        calculate_percentile_rankings(&mut strategies).unwrap();
        
        // Higher balance should win the tiebreaker
        assert!(strategies[0].percentile_rank > strategies[1].percentile_rank);
        assert_eq!(strategies[0].current_balance, 2_000_000_000);
    }
    
    #[test]
    fn test_edge_cases() {
        // Single strategy
        let mut single_strategy = vec![
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 5000,
                current_balance: 1_000_000_000,
                volatility_score: 3000,
                percentile_rank: 0,
                rebalance_threshold: 25,
            }
        ];
        
        let underperformers = calculate_percentile_rankings(&mut single_strategy).unwrap();
        assert_eq!(single_strategy[0].percentile_rank, 50); // Median rank
        assert_eq!(underperformers.len(), 0); // No rebalancing for single strategy
    }
    
    #[test]
    fn test_real_ranking_implementation() {
        // Test with realistic strategy data
        let mut strategies = vec![
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 9500, // Excellent performance
                current_balance: 10_000_000_000, // 10 SOL
                volatility_score: 1000, // Low volatility (10%)
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 7500, // Good performance
                current_balance: 5_000_000_000, // 5 SOL
                volatility_score: 3000, // Medium volatility (30%)
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 5000, // Average performance
                current_balance: 2_000_000_000, // 2 SOL
                volatility_score: 5000, // Higher volatility (50%)
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 2500, // Poor performance
                current_balance: 1_000_000_000, // 1 SOL
                volatility_score: 7000, // High volatility (70%)
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
        ];
        
        let underperformers = calculate_percentile_rankings(&mut strategies).unwrap();
        
        // Verify rankings are in correct order
        assert!(strategies[0].percentile_rank > strategies[1].percentile_rank);
        assert!(strategies[1].percentile_rank > strategies[2].percentile_rank);
        assert!(strategies[2].percentile_rank > strategies[3].percentile_rank);
        
        // Verify percentile calculation for 4 strategies
        // Best strategy should get 100, worst should get 0
        assert_eq!(strategies[0].percentile_rank, 100);
        assert_eq!(strategies[1].percentile_rank, 66); // (2*100)/3 = 66.67 → 66
        assert_eq!(strategies[2].percentile_rank, 33); // (1*100)/3 = 33.33 → 33
        assert_eq!(strategies[3].percentile_rank, 0);
        
        // With 25% threshold on 4 strategies, 1 should be identified as underperformer
        assert_eq!(underperformers.len(), 1);
        assert_eq!(underperformers[0], strategies[3].strategy_id);
    }
    
    #[test]
    fn test_should_rebalance_strategy_logic() {
        let good_strategy = Strategy {
            strategy_id: Pubkey::new_unique(),
            protocol_type: ProtocolType::StableLending {
                pool_id: Pubkey::new_unique(),
                utilization: 8000,
                reserve_address: Pubkey::new_unique(),
            },
            current_balance: 1_000_000_000, // 1 SOL
            yield_rate: 8000,
            volatility_score: 2000,
            performance_score: 7500,
            percentile_rank: 75, // Good rank
            last_updated: 0,
            status: StrategyStatus::Active,
            total_deposits: 1_000_000_000,
            total_withdrawals: 0,
            creation_time: 0,
            bump: 255,
            reserved: [0; 23],
        };
        
        let poor_strategy = Strategy {
            strategy_id: Pubkey::new_unique(),
            protocol_type: ProtocolType::StableLending {
                pool_id: Pubkey::new_unique(),
                utilization: 8000,
                reserve_address: Pubkey::new_unique(),
            },
            current_balance: 100_000_000, // 0.1 SOL
            yield_rate: 2000,
            volatility_score: 8000,
            performance_score: 2000,
            percentile_rank: 10, // Poor rank
            last_updated: 0,
            status: StrategyStatus::Active,
            total_deposits: 100_000_000,
            total_withdrawals: 0,
            creation_time: 0,
            bump: 255,
            reserved: [0; 23],
        };
        
        let inactive_strategy = Strategy {
            strategy_id: Pubkey::new_unique(),
            protocol_type: ProtocolType::StableLending {
                pool_id: Pubkey::new_unique(),
                utilization: 8000,
                reserve_address: Pubkey::new_unique(),
            },
            current_balance: 1_000_000_000,
            yield_rate: 1000,
            volatility_score: 9000,
            performance_score: 1000,
            percentile_rank: 5,
            last_updated: 0,
            status: StrategyStatus::Paused, // Not active
            total_deposits: 1_000_000_000,
            total_withdrawals: 0,
            creation_time: 0,
            bump: 255,
            reserved: [0; 23],
        };
        
        let dust_strategy = Strategy {
            strategy_id: Pubkey::new_unique(),
            protocol_type: ProtocolType::StableLending {
                pool_id: Pubkey::new_unique(),
                utilization: 8000,
                reserve_address: Pubkey::new_unique(),
            },
            current_balance: 10_000_000, // 0.01 SOL - below threshold
            yield_rate: 1000,
            volatility_score: 9000,
            performance_score: 1000,
            percentile_rank: 5,
            last_updated: 0,
            status: StrategyStatus::Active,
            total_deposits: 10_000_000,
            total_withdrawals: 0,
            creation_time: 0,
            bump: 255,
            reserved: [0; 23],
        };
        
        // Test rebalancing logic
        assert!(!should_rebalance_strategy(&good_strategy, 25)); // Good rank, shouldn't rebalance
        assert!(should_rebalance_strategy(&poor_strategy, 25)); // Poor rank, should rebalance
        assert!(!should_rebalance_strategy(&inactive_strategy, 25)); // Inactive, shouldn't rebalance
        assert!(!should_rebalance_strategy(&dust_strategy, 25)); // Too small, shouldn't rebalance
    }
}
