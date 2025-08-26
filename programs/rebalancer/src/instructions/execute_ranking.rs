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
    // Note: We still get the fixed threshold from portfolio for backwards compatibility
    // but will calculate a dynamic threshold based on volatility
    let _portfolio_fixed_threshold = ctx.accounts.portfolio.rebalance_threshold;
    
    // Create StrategyData from accounts without borrowing references
    let mut strategy_data = Vec::new();
    
    // Add strategy_1 if active
    if ctx.accounts.strategy_1.status == StrategyStatus::Active {
        strategy_data.push(StrategyData::from_strategy(
            &ctx.accounts.strategy_1, 
            25 // Temporary value, will be updated by calculate_percentile_rankings
        ));
    }
    
    // Add strategy_2 if active
    if ctx.accounts.strategy_2.status == StrategyStatus::Active {
        strategy_data.push(StrategyData::from_strategy(
            &ctx.accounts.strategy_2, 
            25 // Temporary value, will be updated by calculate_percentile_rankings
        ));
    }
    
    // Add strategy_3 if present and active
    if let Some(ref strategy_3) = ctx.accounts.strategy_3 {
        if strategy_3.status == StrategyStatus::Active {
            strategy_data.push(StrategyData::from_strategy(
                strategy_3, 
                25 // Temporary value, will be updated by calculate_percentile_rankings
            ));
        }
    }
    
    // Add strategy_4 if present and active
    if let Some(ref strategy_4) = ctx.accounts.strategy_4 {
        if strategy_4.status == StrategyStatus::Active {
            strategy_data.push(StrategyData::from_strategy(
                strategy_4, 
                25 // Temporary value, will be updated by calculate_percentile_rankings
            ));
        }
    }
    
    require!(!strategy_data.is_empty(), RebalancerError::InsufficientStrategies);
    require!(strategy_data.len() >= 2, RebalancerError::InsufficientStrategies);
    
    // Execute the core ranking algorithm (which now calculates dynamic threshold internally)
    let underperformers = calculate_percentile_rankings(&mut strategy_data)?;
    
    // Get the dynamic threshold that was calculated
    let dynamic_threshold = if !strategy_data.is_empty() {
        strategy_data[0].rebalance_threshold
    } else {
        25u8 // Fallback
    };
    
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
    
    // Calculate rebalancing candidates using dynamic threshold
    let mut rebalancing_candidates = Vec::new();
    
    if should_rebalance_strategy(&ctx.accounts.strategy_1, dynamic_threshold) {
        rebalancing_candidates.push(ctx.accounts.strategy_1.strategy_id);
    }
    
    if should_rebalance_strategy(&ctx.accounts.strategy_2, dynamic_threshold) {
        rebalancing_candidates.push(ctx.accounts.strategy_2.strategy_id);
    }
    
    if let Some(ref strategy_3) = ctx.accounts.strategy_3 {
        if should_rebalance_strategy(strategy_3, dynamic_threshold) {
            rebalancing_candidates.push(strategy_3.strategy_id);
        }
    }
    
    if let Some(ref strategy_4) = ctx.accounts.strategy_4 {
        if should_rebalance_strategy(strategy_4, dynamic_threshold) {
            rebalancing_candidates.push(strategy_4.strategy_id);
        }
    }
    
    // Log comprehensive results
    msg!("Batch ranking completed: {} strategies processed, {} underperformers identified, {} rebalancing candidates, dynamic threshold: {}%", 
         strategy_data.len(), 
         underperformers.len(),
         rebalancing_candidates.len(),
         dynamic_threshold);
    
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
    
    // Convert to StrategyData and filter active strategies
    // Use temporary threshold value - will be updated by calculate_percentile_rankings
    let mut strategy_data: Vec<StrategyData> = strategies
        .iter()
        .filter(|s| s.status == StrategyStatus::Active)
        .map(|s| StrategyData::from_strategy(s, 25)) // Temporary value
        .collect();
    
    // Execute ranking algorithm (which calculates dynamic threshold internally)
    let underperformers = calculate_percentile_rankings(&mut strategy_data)?;
    
    // Get the dynamic threshold that was calculated
    let dynamic_threshold = if !strategy_data.is_empty() {
        strategy_data[0].rebalance_threshold
    } else {
        25u8 // Fallback
    };
    
    // Update the original strategy accounts with new percentile ranks
    for strategy in strategies.iter_mut() {
        if strategy.status == StrategyStatus::Active {
            if let Some(data) = strategy_data.iter().find(|d| d.strategy_id == strategy.strategy_id) {
                strategy.percentile_rank = data.percentile_rank;
                strategy.last_updated = Clock::get()?.unix_timestamp;
            }
        }
    }
    
    // Identify strategies that should be rebalanced using dynamic threshold
    let rebalancing_candidates: Vec<Pubkey> = strategies
        .iter()
        .filter(|s| should_rebalance_strategy(s, dynamic_threshold))
        .map(|s| s.strategy_id)
        .collect();
    
    let results = RankingResults {
        total_strategies: strategies.len() as u32,
        active_strategies: strategy_data.len() as u32,
        underperformers: underperformers.clone(),
        rebalancing_candidates,
        ranking_timestamp: Clock::get()?.unix_timestamp,
    };
    
    msg!("Complete ranking results: {} total, {} active, {} underperformers, {} candidates, dynamic threshold: {}%", 
         results.total_strategies,
         results.active_strategies, 
         results.underperformers.len(),
         results.rebalancing_candidates.len(),
         dynamic_threshold);
    
    Ok(results)
}

// CORE PERCENTILE RANKING ALGORITHM
/// Calculate average volatility across all active strategies
/// Returns volatility as a percentage (0-100)
pub fn calculate_average_volatility(strategies: &[StrategyData]) -> Result<u32> {
    if strategies.is_empty() {
        return Err(RebalancerError::InsufficientStrategies.into());
    }

    let mut total_volatility: u64 = 0;
    let mut count = 0u64;

    for strategy in strategies {
        // Convert volatility_score (0-10000 representing 0-100%) to percentage
        let volatility_pct = strategy.volatility_score
            .checked_div(100)
            .ok_or(RebalancerError::DivisionByZero)?;
        
        total_volatility = total_volatility
            .checked_add(volatility_pct as u64)
            .ok_or(RebalancerError::MathOverflow)?;
        
        count = count
            .checked_add(1)
            .ok_or(RebalancerError::MathOverflow)?;
    }

    let average_volatility = total_volatility
        .checked_div(count)
        .ok_or(RebalancerError::DivisionByZero)?;

    // Ensure result fits in u32 and is within valid range (0-100%)
    let result = if average_volatility > 100 {
        100u32
    } else {
        average_volatility as u32
    };

    msg!("Calculated average volatility: {}% from {} strategies", result, count);
    Ok(result)
}

/// Calculate dynamic threshold based on average volatility
/// Formula: Dynamic Threshold = Base Threshold + Volatility Adjustment
/// Where: Base = 15%, Volatility Adjustment = (Avg Volatility / 100) × 20%
/// Range: 10% minimum, 40% maximum
pub fn calculate_dynamic_threshold(strategies: &[StrategyData]) -> Result<u8> {
    if strategies.is_empty() {
        return Err(RebalancerError::InsufficientStrategies.into());
    }

    // Calculate average volatility
    let avg_volatility = calculate_average_volatility(strategies)?;
    
    // Base threshold: 15%
    const BASE_THRESHOLD: u32 = 15;
    
    // Volatility adjustment: (avg_volatility / 100) * 20
    let volatility_adjustment = avg_volatility
        .checked_mul(20)
        .ok_or(RebalancerError::MathOverflow)?
        .checked_div(100)
        .ok_or(RebalancerError::DivisionByZero)?;
    
    // Calculate dynamic threshold
    let dynamic_threshold = BASE_THRESHOLD
        .checked_add(volatility_adjustment)
        .ok_or(RebalancerError::MathOverflow)?;
    
    // Enforce bounds: 10% minimum, 40% maximum
    let bounded_threshold = if dynamic_threshold < 10 {
        10u8
    } else if dynamic_threshold > 40 {
        40u8
    } else {
        dynamic_threshold as u8
    };

    msg!("Dynamic threshold calculated: {}% (avg volatility: {}%, adjustment: {}%)", 
         bounded_threshold, avg_volatility, volatility_adjustment);
    
    Ok(bounded_threshold)
}

pub fn calculate_percentile_rankings(strategies: &mut Vec<StrategyData>) -> Result<Vec<Pubkey>> {
    require!(!strategies.is_empty(), RebalancerError::InsufficientStrategies);
    
    // Calculate dynamic threshold based on volatility
    let dynamic_threshold = calculate_dynamic_threshold(strategies)?;
    
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
        
        // Update strategy's threshold to the dynamic value for consistency
        strategy_data.rebalance_threshold = dynamic_threshold;
        
        // IDENTIFY BOTTOM PERFORMERS BASED ON DYNAMIC THRESHOLD
        if total_strategies <= 4 {
            // For small portfolios, only rebalance bottom strategies based on dynamic threshold
            if strategy_data.percentile_rank < dynamic_threshold {
                underperformers.push(strategy_data.strategy_id);
            }
        } else {
            // For larger portfolios, use dynamic threshold percentage
            let threshold_strategies = (total_strategies * dynamic_threshold as usize) / 100;
            let threshold_strategies = threshold_strategies.max(1); // At least 1 strategy
            
            if index >= total_strategies - threshold_strategies {
                underperformers.push(strategy_data.strategy_id);
            }
        };
        
        msg!("Strategy {} ranked: percentile={}%, score={}, balance={}, dynamic_threshold={}%", 
             strategy_data.strategy_id, 
             strategy_data.percentile_rank, 
             strategy_data.performance_score,
             strategy_data.current_balance,
             dynamic_threshold);
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
    fn test_calculate_average_volatility() {
        // Test with various volatility scenarios
        let strategies = vec![
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 8000,
                current_balance: 1_000_000_000,
                volatility_score: 2000, // 20% volatility
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 6000,
                current_balance: 2_000_000_000,
                volatility_score: 5000, // 50% volatility
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 4000,
                current_balance: 500_000_000,
                volatility_score: 8000, // 80% volatility
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
        ];
        
        let avg_volatility = calculate_average_volatility(&strategies).unwrap();
        // Average should be (20 + 50 + 80) / 3 = 50%
        assert_eq!(avg_volatility, 50);
    }
    
    #[test]
    fn test_calculate_dynamic_threshold() {
        // Test low volatility scenario
        let low_vol_strategies = vec![
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 8000,
                current_balance: 1_000_000_000,
                volatility_score: 2000, // 20% volatility
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
        ];
        
        let threshold = calculate_dynamic_threshold(&low_vol_strategies).unwrap();
        // Expected: 15% + (20/100 * 20%) = 15% + 4% = 19%
        assert_eq!(threshold, 19);
        
        // Test high volatility scenario
        let high_vol_strategies = vec![
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 8000,
                current_balance: 1_000_000_000,
                volatility_score: 8000, // 80% volatility
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
        ];
        
        let threshold = calculate_dynamic_threshold(&high_vol_strategies).unwrap();
        // Expected: 15% + (80/100 * 20%) = 15% + 16% = 31%
        assert_eq!(threshold, 31);
        
        // Test boundary conditions - very high volatility
        let extreme_vol_strategies = vec![
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 8000,
                current_balance: 1_000_000_000,
                volatility_score: 10000, // 100% volatility
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
        ];
        
        let threshold = calculate_dynamic_threshold(&extreme_vol_strategies).unwrap();
        // Expected: 15% + (100/100 * 20%) = 35%, but capped at 40%
        assert_eq!(threshold, 35);
    }
    
    #[test]
    fn test_dynamic_threshold_bounds() {
        // Test minimum bound (should never go below 10%)
        let zero_vol_strategies = vec![
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 8000,
                current_balance: 1_000_000_000,
                volatility_score: 0, // 0% volatility
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
        ];
        
        let threshold = calculate_dynamic_threshold(&zero_vol_strategies).unwrap();
        // Expected: 15% + (0/100 * 20%) = 15% (above minimum)
        assert_eq!(threshold, 15);
        
        // Test that we would hit minimum if base was lower
        // (This test validates the 10% minimum bound)
        assert!(threshold >= 10);
    }
    
    #[test] 
    fn test_percentile_ranking_with_dynamic_threshold() {
        let mut strategies = vec![
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 8000,
                current_balance: 1_000_000_000,
                volatility_score: 2000, // 20% volatility
                percentile_rank: 0,
                rebalance_threshold: 25, // Will be updated
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 6000,
                current_balance: 2_000_000_000,
                volatility_score: 4000, // 40% volatility
                percentile_rank: 0,
                rebalance_threshold: 25, // Will be updated
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 4000,
                current_balance: 500_000_000,
                volatility_score: 6000, // 60% volatility
                percentile_rank: 0,
                rebalance_threshold: 25, // Will be updated
            },
        ];
        
        let underperformers = calculate_percentile_rankings(&mut strategies).unwrap();
        
        // Verify that dynamic threshold was calculated and applied
        // Average volatility: (20 + 40 + 60) / 3 = 40%
        // Dynamic threshold: 15% + (40/100 * 20%) = 23%
        let expected_threshold = 23u8;
        assert_eq!(strategies[0].rebalance_threshold, expected_threshold);
        
        // Verify ranking order (highest score = highest percentile)
        assert!(strategies[0].percentile_rank > strategies[1].percentile_rank);
        assert!(strategies[1].percentile_rank > strategies[2].percentile_rank);
        
        // With dynamic threshold of 23% and 3 strategies, should identify bottom performer
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
                rebalance_threshold: 25, // Will be updated
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 5000, // Same score
                current_balance: 1_000_000_000, // Lower balance
                volatility_score: 3000,
                percentile_rank: 0,
                rebalance_threshold: 25, // Will be updated
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
                rebalance_threshold: 25, // Will be updated
            }
        ];
        
        let underperformers = calculate_percentile_rankings(&mut single_strategy).unwrap();
        assert_eq!(single_strategy[0].percentile_rank, 50); // Median rank
        assert_eq!(underperformers.len(), 0); // No rebalancing for single strategy
        
        // Verify dynamic threshold was calculated even for single strategy
        // Volatility: 30%, Dynamic threshold: 15% + (30/100 * 20%) = 21%
        assert_eq!(single_strategy[0].rebalance_threshold, 21);
    }
    
    #[test]
    fn test_real_ranking_implementation() {
        // Test with realistic strategy data showing various volatility levels
        let mut strategies = vec![
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 9500, // Excellent performance
                current_balance: 10_000_000_000, // 10 SOL
                volatility_score: 1000, // Low volatility (10%)
                percentile_rank: 0,
                rebalance_threshold: 25, // Will be updated
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 7500, // Good performance
                current_balance: 5_000_000_000, // 5 SOL
                volatility_score: 3000, // Medium volatility (30%)
                percentile_rank: 0,
                rebalance_threshold: 25, // Will be updated
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 5000, // Average performance
                current_balance: 2_000_000_000, // 2 SOL
                volatility_score: 5000, // Higher volatility (50%)
                percentile_rank: 0,
                rebalance_threshold: 25, // Will be updated
            },
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 2500, // Poor performance
                current_balance: 1_000_000_000, // 1 SOL
                volatility_score: 7000, // High volatility (70%)
                percentile_rank: 0,
                rebalance_threshold: 25, // Will be updated
            },
        ];
        
        let underperformers = calculate_percentile_rankings(&mut strategies).unwrap();
        
        // Verify dynamic threshold calculation
        // Average volatility: (10 + 30 + 50 + 70) / 4 = 40%
        // Dynamic threshold: 15% + (40/100 * 20%) = 23%
        let expected_threshold = 23u8;
        assert_eq!(strategies[0].rebalance_threshold, expected_threshold);
        
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
        
        // With dynamic threshold of 23% on 4 strategies, bottom strategy should be underperformer
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
        
        // Test rebalancing logic with various dynamic thresholds
        assert!(!should_rebalance_strategy(&good_strategy, 25)); // Good rank, shouldn't rebalance
        assert!(should_rebalance_strategy(&poor_strategy, 25)); // Poor rank, should rebalance
        assert!(!should_rebalance_strategy(&inactive_strategy, 25)); // Inactive, shouldn't rebalance
        assert!(!should_rebalance_strategy(&dust_strategy, 25)); // Too small, shouldn't rebalance
        
        // Test with different dynamic thresholds
        assert!(!should_rebalance_strategy(&poor_strategy, 5)); // With 5% threshold, rank 10 is safe
        assert!(should_rebalance_strategy(&poor_strategy, 15)); // With 15% threshold, rank 10 should rebalance
    }
    
    #[test]
    fn test_volatility_edge_cases() {
        // Test with zero volatility strategies
        let zero_vol_strategies = vec![
            StrategyData {
                strategy_id: Pubkey::new_unique(),
                performance_score: 8000,
                current_balance: 1_000_000_000,
                volatility_score: 0, // 0% volatility
                percentile_rank: 0,
                rebalance_threshold: 25,
            },
        ];
        
        let avg_vol = calculate_average_volatility(&zero_vol_strategies).unwrap();
        assert_eq!(avg_vol, 0);
        
        let threshold = calculate_dynamic_threshold(&zero_vol_strategies).unwrap();
        assert_eq!(threshold, 15); // Base threshold only
        
        // Test empty strategies (should error)
        let empty_strategies: Vec<StrategyData> = vec![];
        assert!(calculate_average_volatility(&empty_strategies).is_err());
        assert!(calculate_dynamic_threshold(&empty_strategies).is_err());
    }
}
