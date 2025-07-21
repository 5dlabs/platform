# Task 8: MEV Risk Simulation - Acceptance Criteria

## Functional Requirements

### 1. Token Risk Profile Management
- [ ] System maintains risk profiles for all MVP tokens (SOL, USDC, BONK, JitoSOL, RAY)
- [ ] Each profile includes base attack probability, size sensitivity, and maximum impact
- [ ] Memecoin (BONK) profile shows 20% base attack probability
- [ ] Profiles can be updated at runtime without restarting the system

### 2. Attack Probability Calculation
- [ ] Base probability correctly applies based on token type
- [ ] Trade size factor scales linearly up to liquidity threshold
- [ ] Time-based adjustment increases probability by 20% during peak hours (14:00-19:00 UTC)
- [ ] Liquidity-based adjustment increases probability by 50% for trades >1% of pool
- [ ] Maximum probability capped at 50% regardless of factors

### 3. MEV Impact Simulation
- [ ] Impact calculation starts at 50% of token's maximum impact in basis points
- [ ] Size multiplier correctly scales impact based on trade/liquidity ratio
- [ ] Random variation adds ±20% to final impact for realistic distribution
- [ ] Attack types distributed as: 70% sandwich, 20% front-run, 10% back-run
- [ ] Zero impact returned when attack simulation determines no attack

### 4. Priority Fee Recommendation
- [ ] Base fee starts at 1,000 lamports
- [ ] Size multiplier ranges from 1x to 10x based on trade size
- [ ] Risk multiplier ranges from 1x to 5x based on attack probability
- [ ] Congestion multiplier doubles fee for high-volume pools (>$1M daily volume)
- [ ] Final fee capped at 10,000 lamports maximum

### 5. Redis Integration
- [ ] Successfully fetches pool state from Redis when available
- [ ] Falls back to default pool state on cache miss
- [ ] Pool state queries complete in <1ms
- [ ] Handles Redis connection failures gracefully

## Performance Requirements

### 1. Latency
- [ ] Single simulation completes in <5ms (excluding Redis lookup)
- [ ] Redis cache lookup completes in <1ms
- [ ] Total end-to-end simulation time <6ms with cache hit

### 2. Throughput
- [ ] Supports 1000+ concurrent simulations without degradation
- [ ] Thread-safe for parallel execution
- [ ] No memory leaks during extended operation

### 3. Resource Usage
- [ ] Memory footprint <10MB for base configuration
- [ ] CPU usage scales linearly with simulation count
- [ ] No blocking operations in async methods

## Statistical Accuracy

### 1. Attack Rate Distribution
- [ ] BONK shows 15-20% attack rate over 1000+ simulations
- [ ] SOL shows 4-6% attack rate over 1000+ simulations
- [ ] USDC shows 0.5-1.5% attack rate over 1000+ simulations
- [ ] Attack rates increase with trade size as expected

### 2. Impact Distribution
- [ ] Memecoin impacts range from 50-200 basis points when attacked
- [ ] Stablecoin impacts stay below 10 basis points
- [ ] Impact increases with trade size relative to liquidity
- [ ] Random variation creates realistic distribution curve

### 3. Fee Recommendations
- [ ] All recommended fees fall within 1,000-10,000 lamport range
- [ ] Larger trades consistently recommend higher fees
- [ ] High-risk tokens recommend higher fees than low-risk
- [ ] Fee distribution shows logical progression

## Integration Requirements

### 1. API Compatibility
- [ ] `simulate_mev()` method accepts token, amount, and timestamp
- [ ] Returns `MevRiskAssessment` struct with all required fields
- [ ] All public methods are async and return `Result<T>`
- [ ] Error messages provide actionable information

### 2. Data Model Compliance
- [ ] Uses `Decimal` type for monetary values
- [ ] Uses `DateTime<Utc>` for timestamps
- [ ] Implements all enum types as specified
- [ ] Maintains compatibility with common library structures

### 3. Error Handling
- [ ] Returns error for unknown tokens (with suggestion to use default)
- [ ] Returns error for invalid trade amounts (zero or negative)
- [ ] Handles Redis timeouts without panicking
- [ ] Provides fallback behavior for all external dependencies

## Test Coverage

### 1. Unit Tests
- [ ] Token profile configuration tests
- [ ] Probability calculation with various inputs
- [ ] Impact calculation edge cases
- [ ] Priority fee calculation boundaries
- [ ] Statistical distribution validation

### 2. Integration Tests
- [ ] Redis integration with mock client
- [ ] Concurrent simulation stress test
- [ ] Full simulation flow with all token types
- [ ] Error propagation verification

### 3. Performance Tests
- [ ] Latency benchmarks for various scenarios
- [ ] Memory usage under load
- [ ] Throughput testing with concurrent requests
- [ ] Cache hit/miss performance comparison

## Documentation

### 1. Code Documentation
- [ ] All public methods have comprehensive doc comments
- [ ] Complex algorithms include inline explanations
- [ ] Assumptions clearly stated in comments
- [ ] Examples provided for main use cases

### 2. Configuration Guide
- [ ] Token profile configuration documented
- [ ] Historical pattern setup explained
- [ ] Redis connection requirements specified
- [ ] Performance tuning options described

## Acceptance Test Scenarios

### Scenario 1: Memecoin Large Trade
```
Given: BONK trade of $10,000 during peak hours
When: MEV simulation runs
Then: 
  - Attack probability between 25-35%
  - If attacked, impact 100-200 bps
  - Recommended fee 5,000-10,000 lamports
```

### Scenario 2: Stablecoin Small Trade
```
Given: USDC trade of $500 during off-peak hours
When: MEV simulation runs
Then:
  - Attack probability <2%
  - If attacked, impact <10 bps
  - Recommended fee 1,000-2,000 lamports
```

### Scenario 3: Network Congestion
```
Given: High-volume pool (>$1M daily)
When: Any trade simulated
Then:
  - Priority fee doubles from base calculation
  - Still capped at 10,000 lamports
```

### Scenario 4: Cache Miss Handling
```
Given: Redis cache unavailable
When: MEV simulation requested
Then:
  - Falls back to default pool state
  - Simulation completes successfully
  - Confidence score reflects data staleness
```