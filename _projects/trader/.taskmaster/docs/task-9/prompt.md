# Task 9: Paper Trade Executor Implementation - Autonomous AI Prompt

## Objective

You are tasked with implementing a paper trade execution system that processes external trade requests through gRPC, applies configurable slippage models, integrates MEV simulation, and maintains a virtual portfolio. This executor must provide identical interfaces to the live trader while simulating realistic market conditions for accurate strategy validation.

## Context

The Paper Trade Executor is the central component that brings together price discovery, slippage modeling, MEV simulation, and portfolio management. It must process trades with <100ms latency while maintaining 85-90% accuracy compared to live execution. The system receives trade requests from external sources (strategies, APIs, or UI) and returns comprehensive execution results.

## Technical Requirements

### Core Components to Implement

1. **PaperTradeExecutor struct** with:
   - Virtual portfolio reference (Arc<RwLock<VirtualPortfolio>>)
   - Price cache for <1ms lookups
   - MEV simulator integration
   - Jupiter client with failover
   - QuestDB client for trade recording
   - Configurable slippage models
   - Metrics collection

2. **TradeRequest processing** that:
   - Validates request format (not business logic)
   - Retrieves current prices from cache or Jupiter
   - Calculates expected execution price
   - Applies slippage model
   - Optionally applies MEV simulation
   - Updates virtual portfolio atomically
   - Records trade in QuestDB
   - Returns detailed TradeResult

3. **Slippage models** including:
   - Fixed percentage (0.5-2% configurable for MVP)
   - Framework for future dynamic models
   - Per-request override capability
   - Direction-aware application

### Implementation Guidelines

1. **Request Validation**:
   - Amount must be positive
   - Token symbols must be non-empty and different
   - Priority fee must be reasonable (<1M lamports)
   - Slippage tolerance must be <50%

2. **Price Discovery Flow**:
   ```
   1. Check Redis cache (TTL: 2 seconds)
   2. If miss, fetch from Jupiter with failover
   3. Cache fresh prices for subsequent requests
   4. Track cache hit rate for metrics
   ```

3. **Execution Price Calculation**:
   - Buy: quote_price / base_price
   - Sell: base_price / quote_price
   - Swap: Use Jupiter route-specific pricing

4. **Slippage Application**:
   - Buy: price * (1 + slippage)
   - Sell: price * (1 - slippage)
   - Swap: Depends on input token direction

5. **MEV Integration**:
   - Only apply if request.simulate_mev is true
   - Calculate trade size in USD
   - Get risk assessment from MEV simulator
   - Apply impact if attacked and under-protected
   - Add warnings to result

### API Contracts

```rust
// Input
pub struct TradeRequest {
    pub request_id: Uuid,
    pub action: TradeAction,
    pub base_token: String,
    pub quote_token: String,
    pub amount: Decimal,
    pub is_base_input: bool,
    pub slippage_tolerance_bps: Option<u16>,
    pub priority_fee: u64,
    pub simulate_mev: bool,
    pub max_price_impact_bps: Option<u16>,
    pub metadata: Option<serde_json::Value>,
}

// Output
pub struct TradeResult {
    pub request_id: Uuid,
    pub trade: Trade,
    pub portfolio_state: PortfolioSummary,
    pub execution_metrics: ExecutionMetrics,
    pub warnings: Vec<String>,
}
```

### Performance Requirements

1. **Latency Targets**:
   - Total execution: <100ms
   - Price cache lookup: <1ms
   - Portfolio update: <10ms
   - MEV simulation: <5ms

2. **Concurrency**:
   - Thread-safe for parallel execution
   - No blocking in async methods
   - Atomic portfolio updates

3. **Resource Usage**:
   - Minimal memory allocation per trade
   - Efficient metric collection
   - Non-blocking QuestDB writes

### Error Handling

1. **Validation Errors**: Return immediately with clear message
2. **Price Fetch Errors**: Retry with exponential backoff
3. **Portfolio Update Errors**: Rollback and return error
4. **QuestDB Errors**: Log but don't fail trade
5. **MEV Simulation Errors**: Continue without MEV impact

## Code Structure

```rust
impl PaperTradeExecutor {
    pub async fn execute_trade_request(&self, request: TradeRequest) -> Result<TradeResult> {
        // 1. Start timing
        // 2. Validate request
        // 3. Get prices (cache-first)
        // 4. Calculate expected price
        // 5. Apply slippage model
        // 6. Apply MEV simulation (if enabled)
        // 7. Check price impact limits
        // 8. Create trade record
        // 9. Update portfolio
        // 10. Record to QuestDB (async)
        // 11. Calculate metrics
        // 12. Return comprehensive result
    }
}
```

## Testing Requirements

1. **Unit Tests**:
   - All slippage model configurations
   - Price calculation for each trade action
   - MEV integration with various outcomes
   - Request validation edge cases
   - Metric collection accuracy

2. **Integration Tests**:
   - Full trade flow with mocked dependencies
   - Concurrent trade execution
   - Cache hit/miss scenarios
   - Error propagation
   - Performance under load

3. **Accuracy Tests**:
   - Compare execution prices with expected ranges
   - Verify slippage application correctness
   - Validate MEV impact calculations
   - Ensure portfolio state consistency

## Logging Requirements

Use structured logging with appropriate levels:
- INFO: Trade received, completed
- DEBUG: Price lookups, calculations
- WARN: High slippage, MEV attacks, slow execution
- ERROR: Failed validations, update failures

Include request_id in all log entries for traceability.

## Success Criteria

1. Processes valid trades in <100ms consistently
2. Correctly applies configurable slippage models
3. Integrates MEV simulation seamlessly
4. Maintains accurate portfolio state
5. Provides comprehensive execution metrics
6. Handles errors gracefully without panics
7. Supports 1000+ concurrent trades
8. Achieves >80% cache hit rate in steady state

## Additional Notes

- Focus on correctness over optimization for MVP
- Ensure interface compatibility with future live trader
- Document all assumptions in code
- Make slippage models easily configurable
- Consider adding execution replay capability
- Maintain audit trail for all trades