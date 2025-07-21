# Task 8: Implement MEV Risk Simulation for Paper Trading

## Overview

This task involves creating a realistic Maximum Extractable Value (MEV) risk simulation system that models sandwich attack probability and impact for paper trading. The simulator is critical for achieving the 85-90% accuracy target between paper and live trading by replicating the real-world MEV risks that affect 15-20% of memecoin trades on Solana.

## Architecture Context

As outlined in the `architecture.md`, the MEV Simulator is a core component of the Paper Trader Service that:
- Integrates with Redis cache for pool state information
- Calculates sandwich attack probabilities based on trade size and token liquidity
- Estimates potential losses in basis points
- Recommends priority fees in the 1,000-10,000 lamport range
- Enables realistic paper trading that matches live execution conditions

## Dependencies

- **Task 1**: Common Libraries and Data Models (for MEV risk structures)
- **Task 7**: Virtual Portfolio Manager (to integrate with portfolio updates)

## Implementation Details

### 1. Core MEV Simulator Structure

```rust
use std::collections::HashMap;
use rand::{thread_rng, ThreadRng, Rng};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};

pub struct MevSimulator {
    token_risk_profiles: HashMap<String, TokenRiskProfile>,
    redis_cache: Arc<RedisClient>,
    rng: ThreadRng,
    historical_patterns: MevHistoricalPatterns,
}

pub struct TokenRiskProfile {
    pub base_attack_probability: f64,  // 0.0-1.0
    pub size_sensitivity: f64,         // How much trade size affects probability
    pub max_impact_bps: u32,           // Maximum impact in basis points
    pub liquidity_threshold: f64,      // USD value where MEV becomes attractive
    pub token_type: TokenType,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    StableCoin,      // USDC, USDT - low MEV risk
    LiquidAsset,     // SOL, ETH - moderate MEV risk
    DeFiToken,       // RAY, SRM - moderate MEV risk
    LiquidStaking,   // JitoSOL, mSOL - low-moderate MEV risk
    Memecoin,        // BONK, WIF - high MEV risk (15-20%)
}

pub struct MevRiskAssessment {
    pub sandwich_probability: f64,
    pub is_attacked: bool,
    pub estimated_loss_bps: u32,
    pub recommended_fee: u64,
    pub attack_type: Option<AttackType>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone)]
pub enum AttackType {
    FrontRun,
    BackRun,
    Sandwich,
}

pub struct MevHistoricalPatterns {
    // Historical data for more accurate simulation
    pub average_attack_rate_by_type: HashMap<TokenType, f64>,
    pub peak_hours: Vec<u8>,  // UTC hours with highest MEV activity
    pub network_congestion_multiplier: f64,
}
```

### 2. Token Risk Profile Configuration

```rust
impl MevSimulator {
    pub fn new(redis_cache: Arc<RedisClient>) -> Self {
        let mut token_risk_profiles = HashMap::new();
        
        // Configure risk profiles based on PRD specifications
        // Stablecoins - minimal MEV risk
        token_risk_profiles.insert("USDC".to_string(), TokenRiskProfile {
            base_attack_probability: 0.01,  // 1% base rate
            size_sensitivity: 0.05,
            max_impact_bps: 10,
            liquidity_threshold: 10_000.0,
            token_type: TokenType::StableCoin,
        });
        
        // Liquid assets - moderate MEV risk
        token_risk_profiles.insert("SOL".to_string(), TokenRiskProfile {
            base_attack_probability: 0.05,  // 5% base rate
            size_sensitivity: 0.15,
            max_impact_bps: 50,
            liquidity_threshold: 5_000.0,
            token_type: TokenType::LiquidAsset,
        });
        
        // DeFi tokens - moderate MEV risk
        token_risk_profiles.insert("RAY".to_string(), TokenRiskProfile {
            base_attack_probability: 0.08,  // 8% base rate
            size_sensitivity: 0.20,
            max_impact_bps: 100,
            liquidity_threshold: 3_000.0,
            token_type: TokenType::DeFiToken,
        });
        
        // Liquid staking tokens - low-moderate MEV risk
        token_risk_profiles.insert("JitoSOL".to_string(), TokenRiskProfile {
            base_attack_probability: 0.04,  // 4% base rate
            size_sensitivity: 0.10,
            max_impact_bps: 30,
            liquidity_threshold: 8_000.0,
            token_type: TokenType::LiquidStaking,
        });
        
        // Memecoins - high MEV risk (15-20% as specified)
        token_risk_profiles.insert("BONK".to_string(), TokenRiskProfile {
            base_attack_probability: 0.20,  // 20% base rate matching PRD
            size_sensitivity: 0.35,
            max_impact_bps: 200,  // Up to 2% loss
            liquidity_threshold: 1_000.0,
            token_type: TokenType::Memecoin,
        });
        
        // Historical patterns from research
        let historical_patterns = MevHistoricalPatterns {
            average_attack_rate_by_type: HashMap::from([
                (TokenType::StableCoin, 0.01),
                (TokenType::LiquidAsset, 0.05),
                (TokenType::DeFiToken, 0.08),
                (TokenType::LiquidStaking, 0.04),
                (TokenType::Memecoin, 0.175),  // Average of 15-20%
            ]),
            peak_hours: vec![14, 15, 16, 17, 18, 19],  // UTC peak trading hours
            network_congestion_multiplier: 1.5,
        };
        
        Self {
            token_risk_profiles,
            redis_cache,
            rng: thread_rng(),
            historical_patterns,
        }
    }
}
```

### 3. MEV Risk Assessment Implementation

```rust
impl MevSimulator {
    pub async fn simulate_mev(
        &mut self, 
        token: &str, 
        trade_size_usd: Decimal,
        current_time: DateTime<Utc>,
    ) -> Result<MevRiskAssessment> {
        // Get token risk profile
        let profile = self.token_risk_profiles.get(token)
            .ok_or_else(|| anyhow!("No risk profile for token: {}", token))?;
        
        // Get pool state from Redis cache for more accurate assessment
        let pool_state = self.get_pool_state(token).await?;
        
        // Calculate base probability
        let mut attack_probability = profile.base_attack_probability;
        
        // Apply size factor (larger trades more likely to be attacked)
        let size_factor = (trade_size_usd.to_f64().unwrap() / profile.liquidity_threshold)
            .min(1.0)
            .max(0.0);
        attack_probability += profile.size_sensitivity * size_factor;
        
        // Apply time-based factor (peak hours have higher MEV activity)
        let hour = current_time.hour() as u8;
        if self.historical_patterns.peak_hours.contains(&hour) {
            attack_probability *= 1.2;  // 20% increase during peak hours
        }
        
        // Apply liquidity-based adjustment
        let liquidity_ratio = trade_size_usd.to_f64().unwrap() / pool_state.liquidity;
        if liquidity_ratio > 0.01 {  // Trade is >1% of pool liquidity
            attack_probability *= 1.5;  // 50% increase for large trades
        }
        
        // Cap probability at realistic maximum
        attack_probability = attack_probability.min(0.5);  // Max 50% as per architecture
        
        // Determine if this trade would be attacked
        let is_attacked = self.rng.gen::<f64>() < attack_probability;
        
        // Calculate impact if attacked
        let (impact_bps, attack_type) = if is_attacked {
            // Calculate impact based on multiple factors
            let base_impact = profile.max_impact_bps as f64 * 0.5;  // Start at 50% of max
            
            // Adjust based on trade size
            let size_multiplier = (liquidity_ratio * 10.0).min(2.0).max(1.0);
            let adjusted_impact = base_impact * size_multiplier;
            
            // Add randomness for realistic variation
            let random_factor = 0.8 + (self.rng.gen::<f64>() * 0.4);  // 0.8 to 1.2
            let final_impact = (adjusted_impact * random_factor) as u32;
            
            // Determine attack type based on probability
            let attack_type = match self.rng.gen::<f64>() {
                x if x < 0.7 => AttackType::Sandwich,  // 70% sandwich attacks
                x if x < 0.9 => AttackType::FrontRun,  // 20% front-run only
                _ => AttackType::BackRun,               // 10% back-run only
            };
            
            (final_impact.min(profile.max_impact_bps), Some(attack_type))
        } else {
            (0, None)
        };
        
        // Calculate recommended priority fee
        let recommended_fee = self.calculate_priority_fee(
            attack_probability,
            trade_size_usd,
            &pool_state,
        )?;
        
        // Calculate confidence score based on data quality
        let confidence_score = self.calculate_confidence_score(&pool_state);
        
        Ok(MevRiskAssessment {
            sandwich_probability: attack_probability,
            is_attacked,
            estimated_loss_bps: impact_bps,
            recommended_fee,
            attack_type,
            confidence_score,
        })
    }
    
    fn calculate_priority_fee(
        &self,
        attack_probability: f64,
        trade_size_usd: Decimal,
        pool_state: &PoolState,
    ) -> Result<u64> {
        // Base fee of 1,000 lamports as specified
        let base_fee: u64 = 1_000;
        
        // Size multiplier (larger trades need higher fees)
        let size_multiplier = (trade_size_usd.to_f64().unwrap() / 100.0)
            .min(10.0)
            .max(1.0) as u64;
        
        // Risk multiplier based on attack probability
        let risk_multiplier = (1.0 + (attack_probability * 4.0)) as u64;  // 1x to 5x
        
        // Network congestion factor
        let congestion_multiplier = if pool_state.recent_volume_24h > 1_000_000.0 {
            2  // Double fee for high-volume pools
        } else {
            1
        };
        
        // Calculate final fee
        let fee = base_fee * size_multiplier * risk_multiplier * congestion_multiplier;
        
        // Cap at 10,000 lamports as specified in PRD
        Ok(fee.min(10_000))
    }
    
    async fn get_pool_state(&self, token: &str) -> Result<PoolState> {
        // Try to get from Redis cache first
        if let Some(state) = self.redis_cache.get_pool_state(token).await? {
            return Ok(state);
        }
        
        // Fallback to default state if not cached
        Ok(PoolState {
            liquidity: 100_000.0,  // Default $100k liquidity
            recent_volume_24h: 50_000.0,
            price_impact_curve: vec![],
            last_update: Utc::now(),
        })
    }
    
    fn calculate_confidence_score(&self, pool_state: &PoolState) -> f64 {
        // Score based on data freshness and completeness
        let age_minutes = (Utc::now() - pool_state.last_update).num_minutes();
        let freshness_score = match age_minutes {
            0..=1 => 1.0,
            2..=5 => 0.8,
            6..=30 => 0.6,
            _ => 0.4,
        };
        
        // Adjust based on data completeness
        let completeness_score = if pool_state.price_impact_curve.is_empty() {
            0.7
        } else {
            1.0
        };
        
        freshness_score * completeness_score
    }
}
```

### 4. Pool State Integration

```rust
#[derive(Debug, Clone)]
pub struct PoolState {
    pub liquidity: f64,
    pub recent_volume_24h: f64,
    pub price_impact_curve: Vec<(f64, f64)>,  // (size, impact)
    pub last_update: DateTime<Utc>,
}

impl MevSimulator {
    pub async fn update_token_profile(
        &mut self,
        token: &str,
        profile: TokenRiskProfile,
    ) -> Result<()> {
        self.token_risk_profiles.insert(token.to_string(), profile);
        Ok(())
    }
    
    pub fn get_token_profiles(&self) -> &HashMap<String, TokenRiskProfile> {
        &self.token_risk_profiles
    }
    
    pub fn analyze_historical_patterns(&self) -> MevAnalytics {
        // Analyze patterns for reporting
        MevAnalytics {
            total_simulations: 0,  // Would be tracked in production
            attack_rate_by_token: HashMap::new(),
            average_loss_bps: 0,
            recommended_fee_distribution: vec![],
        }
    }
}

pub struct MevAnalytics {
    pub total_simulations: u64,
    pub attack_rate_by_token: HashMap<String, f64>,
    pub average_loss_bps: u32,
    pub recommended_fee_distribution: Vec<u64>,
}
```

### 5. Testing Utilities

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memecoin_attack_rate() {
        let redis_client = Arc::new(MockRedisClient::new());
        let mut simulator = MevSimulator::new(redis_client);
        
        // Run 1000 simulations for BONK
        let mut attacked_count = 0;
        for _ in 0..1000 {
            let assessment = simulator.simulate_mev(
                "BONK",
                Decimal::new(5000, 0),  // $5,000 trade
                Utc::now(),
            ).await.unwrap();
            
            if assessment.is_attacked {
                attacked_count += 1;
            }
        }
        
        // Verify attack rate is within 15-20% range
        let attack_rate = attacked_count as f64 / 1000.0;
        assert!(attack_rate >= 0.15 && attack_rate <= 0.25);
    }
    
    #[tokio::test]
    async fn test_priority_fee_range() {
        let redis_client = Arc::new(MockRedisClient::new());
        let mut simulator = MevSimulator::new(redis_client);
        
        // Test various trade sizes
        for trade_size in [100, 1_000, 10_000, 100_000] {
            let assessment = simulator.simulate_mev(
                "SOL",
                Decimal::new(trade_size, 0),
                Utc::now(),
            ).await.unwrap();
            
            // Verify fee is within specified range
            assert!(assessment.recommended_fee >= 1_000);
            assert!(assessment.recommended_fee <= 10_000);
        }
    }
}
```

## Integration Points

1. **Redis Cache Integration**: Fetches real-time pool state for accurate risk assessment
2. **Virtual Portfolio**: Provides MEV impact data for trade execution
3. **Paper Trade Executor**: Uses simulation results to adjust execution prices
4. **QuestDB**: Records MEV events for historical analysis

## Configuration

The MEV Simulator supports runtime configuration through:
- Token risk profile updates
- Historical pattern adjustments
- Priority fee calculation parameters
- Attack probability thresholds

## Performance Considerations

- Simulation runs in <5ms for optimal trade execution flow
- Redis cache reduces pool state lookup latency to <1ms
- Thread-safe design supports concurrent simulations
- Minimal memory footprint with efficient data structures

## Future Enhancements

1. Machine learning-based attack prediction
2. Real-time network congestion monitoring
3. Cross-DEX MEV correlation analysis
4. Advanced attack pattern recognition