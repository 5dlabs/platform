# Task 8: MEV Risk Simulation Implementation - Autonomous AI Prompt

## Objective

You are tasked with implementing a Maximum Extractable Value (MEV) risk simulation system for the Solana trading platform's paper trading mode. This simulator must accurately model sandwich attack probabilities and their financial impact to achieve 85-90% correlation between paper and live trading results.

## Context

MEV attacks, particularly sandwich attacks, affect 15-20% of memecoin trades on Solana. Your implementation must simulate these attacks realistically to ensure paper trading results accurately reflect live trading conditions. The simulator will be integrated with the paper trade executor to apply MEV-based price impacts during simulated trades.

## Technical Requirements

### Core Components to Implement

1. **MevSimulator struct** with:
   - Token risk profiles mapping
   - Redis cache integration for pool states
   - Random number generator for probabilistic simulation
   - Historical pattern tracking

2. **TokenRiskProfile configuration** including:
   - Base attack probability (0.0-1.0)
   - Size sensitivity factor
   - Maximum impact in basis points
   - Liquidity thresholds
   - Token type classification

3. **Risk assessment algorithm** that:
   - Calculates attack probability based on token type, trade size, and market conditions
   - Determines if a specific trade would be attacked
   - Estimates financial impact in basis points
   - Recommends priority fees (1,000-10,000 lamports range)

### Implementation Guidelines

1. **Token Risk Profiles**:
   - USDC: 1% base attack rate (stablecoin)
   - SOL: 5% base attack rate (liquid asset)
   - RAY: 8% base attack rate (DeFi token)
   - JitoSOL: 4% base attack rate (liquid staking)
   - BONK: 20% base attack rate (memecoin, matching PRD specification)

2. **Probability Calculation**:
   ```
   attack_probability = base_rate + (size_sensitivity * size_factor)
   ```
   - Apply time-based adjustments (20% increase during peak hours)
   - Apply liquidity-based adjustments (50% increase for trades >1% of pool)
   - Cap maximum probability at 50%

3. **Impact Calculation**:
   - Start at 50% of token's maximum impact
   - Scale with trade size relative to pool liquidity
   - Add random variation (±20%) for realism
   - Return impact in basis points

4. **Priority Fee Calculation**:
   ```
   fee = base_fee * size_multiplier * risk_multiplier * congestion_multiplier
   ```
   - Base fee: 1,000 lamports
   - Size multiplier: 1-10x based on trade size
   - Risk multiplier: 1-5x based on attack probability
   - Cap at 10,000 lamports maximum

### Integration Requirements

1. **Redis Cache**:
   - Fetch pool state with <1ms latency
   - Use cached liquidity data for risk calculations
   - Fallback to default values if cache miss

2. **Async Operations**:
   - All public methods must be async
   - Handle Redis timeouts gracefully
   - Maintain <5ms total simulation time

3. **Error Handling**:
   - Return Result<T> for all fallible operations
   - Provide meaningful error messages
   - Never panic on invalid input

## Code Structure

```rust
pub struct MevSimulator {
    token_risk_profiles: HashMap<String, TokenRiskProfile>,
    redis_cache: Arc<RedisClient>,
    rng: ThreadRng,
    historical_patterns: MevHistoricalPatterns,
}

impl MevSimulator {
    pub fn new(redis_cache: Arc<RedisClient>) -> Self { ... }
    
    pub async fn simulate_mev(
        &mut self,
        token: &str,
        trade_size_usd: Decimal,
        current_time: DateTime<Utc>,
    ) -> Result<MevRiskAssessment> { ... }
    
    fn calculate_priority_fee(...) -> Result<u64> { ... }
    
    async fn get_pool_state(&self, token: &str) -> Result<PoolState> { ... }
}
```

## Testing Requirements

1. **Statistical Validation**:
   - Verify memecoin attack rates fall within 15-20% range over 1000+ simulations
   - Confirm priority fees stay within 1,000-10,000 lamport range
   - Test impact calculations produce realistic values

2. **Edge Cases**:
   - Unknown tokens (use default profile)
   - Zero or negative trade sizes (return error)
   - Redis cache failures (use fallback values)
   - Extreme market conditions

3. **Performance**:
   - Simulation completes in <5ms
   - Concurrent simulations don't interfere
   - Memory usage remains constant

## Success Criteria

1. Memecoin trades show 15-20% attack rate as specified in PRD
2. Priority fee recommendations align with network conditions
3. Impact calculations produce realistic slippage adjustments
4. Integration with paper trade executor works seamlessly
5. All tests pass with >90% code coverage

## Additional Notes

- Focus on statistical accuracy over complex modeling for MVP
- Document all assumptions in code comments
- Provide configuration options for easy adjustment
- Consider future enhancements but don't over-engineer
- Maintain consistency with architecture.md specifications