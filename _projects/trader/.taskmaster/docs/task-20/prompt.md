# Task 20: Implement Real-time Pool State Monitoring and Dynamic Slippage - Autonomous Prompt

You are implementing a real-time pool monitoring system for a Solana trading platform that tracks DEX liquidity pools and calculates dynamic slippage based on actual market conditions. This system will significantly improve trade simulation accuracy by using live liquidity data instead of static models.

## Context

The trading platform currently uses fixed slippage models (0.5-2%) which don't reflect actual market conditions. Your system will monitor pool states across multiple DEXs (Raydium, Orca, etc.), cache data in Redis with 1-2 second TTL, and provide dynamic slippage calculations based on real liquidity depth.

## Your Objectives

1. **Build Pool State Monitoring Service**
   - Subscribe to pool account updates via Solana gRPC
   - Support multiple DEX protocols (Raydium, Orca, etc.)
   - Parse protocol-specific pool layouts
   - Cache states in Redis (1-2 second TTL)
   - Store snapshots in QuestDB for analysis

2. **Implement DEX-Specific Parsers**
   - Create trait for pool state parsing
   - Implement Raydium AMM V4 parser
   - Implement Orca pool parser
   - Support concentrated liquidity pools
   - Extract reserves, fees, and metadata

3. **Develop Liquidity Depth Analysis**
   - Calculate available liquidity at price points
   - Analyze liquidity concentration
   - Estimate price impact for various trade sizes
   - Track volume-to-liquidity ratios
   - Generate pool health scores

4. **Create Dynamic Slippage Calculator**
   - Replace fixed slippage with real calculations
   - Factor in current pool reserves
   - Include historical slippage patterns
   - Account for trade size impact
   - Provide confidence scores

5. **Build Feedback Collection System**
   - Compare predicted vs actual slippage
   - Store results for model improvement
   - Analyze prediction accuracy
   - Identify systematic biases
   - Enable continuous refinement

## Implementation Requirements

### Architecture Overview
```
pool_monitoring/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── monitor.rs      # Pool subscription service
│   ├── parsers/        # DEX-specific parsers
│   │   ├── mod.rs
│   │   ├── raydium.rs
│   │   ├── orca.rs
│   │   └── traits.rs
│   ├── analysis/       # Liquidity analysis
│   │   ├── depth.rs
│   │   ├── health.rs
│   │   └── volume.rs
│   ├── slippage/       # Dynamic calculations
│   │   ├── calculator.rs
│   │   ├── factors.rs
│   │   └── feedback.rs
│   └── registry.rs     # Pool management
```

### Pool State Model
```rust
pub struct PoolState {
    pub pool_address: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_reserve: u64,
    pub token_b_reserve: u64,
    pub fee_rate: f64,
    pub liquidity_usd: f64,
    pub price: f64,
    pub volume_24h: f64,
    pub last_update: DateTime<Utc>,
    pub pool_type: PoolType,
    pub additional_data: Option<Value>,
}
```

### Key Implementation Details

1. **Subscription Management**:
   - Use account subscriptions for each pool
   - Handle reconnections gracefully
   - Process updates within 10ms
   - Batch updates for efficiency

2. **Parser Requirements**:
   - Support all major DEX protocols
   - Handle different pool layouts
   - Extract consistent PoolState
   - Validate data integrity

3. **Slippage Calculation**:
   - Constant product formula: `x * y = k`
   - Account for fees in calculations
   - Consider multi-hop routes
   - Factor in gas costs

4. **Performance Targets**:
   - Pool update processing: <10ms
   - Slippage calculation: <1ms cached
   - Cache hit rate: >90%
   - Support 1000+ pools

### Testing Approach

1. **Unit Tests**:
   - Parser accuracy for each DEX
   - Slippage calculation correctness
   - Cache expiry behavior
   - Error handling

2. **Integration Tests**:
   - Live pool monitoring
   - Multi-DEX support
   - Feedback accuracy
   - Performance under load

3. **Simulation Tests**:
   - Compare with actual trades
   - Verify slippage predictions
   - Test edge cases

## Deliverables

1. Pool state monitoring service with gRPC subscriptions
2. Parsers for major DEX protocols
3. Dynamic slippage calculator with confidence scoring
4. Integration with paper trade executor
5. Feedback collection and analysis system
6. Pool health monitoring and alerts
7. Comprehensive test suite
8. Performance benchmarks

## Success Criteria

- Successfully monitors 100+ pools in real-time
- Pool state updates processed in <10ms
- Dynamic slippage calculations <1ms with cache
- 85-90% correlation with actual execution
- Redis cache hit rate >90%
- QuestDB stores 30 days of pool history
- Alert system for unhealthy pools
- Feedback loop improves accuracy over time