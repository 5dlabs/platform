# Task 11: Implement Stop-Loss and Take-Profit Monitoring - Autonomous Prompt

You are implementing a high-frequency position monitoring system for the Solana trading platform. This system must continuously monitor positions and execute orders automatically when stop-loss or take-profit conditions are met, operating at 100ms intervals.

## Context
- The platform requires automated risk management to protect against losses and capture profits
- Monitoring must occur every 100ms to catch rapid price movements in volatile markets
- The system must handle multiple order types including trailing stops and partial exits
- All executed orders must be recorded with complete metadata for analysis

## Your Task

Implement a comprehensive order monitoring system with the following components:

### 1. High-Frequency Position Monitor
Create a monitoring service that:
- Runs precisely every 100ms using tokio intervals
- Processes all positions in parallel to meet timing requirements
- Skips missed ticks to prevent cascading delays
- Logs warnings when monitoring cycles exceed 50ms

### 2. Flexible Order Condition System
Implement support for:

**Stop-Loss Orders**:
- Fixed price stop-loss triggers
- Trailing stop-loss with configurable distance (percentage or absolute)
- High water mark tracking for trailing stops
- Automatic position liquidation when triggered

**Take-Profit Orders**:
- Fixed price profit targets
- Partial profit taking (e.g., sell 50% at target)
- Multiple take-profit levels per position

**Advanced Conditions**:
- Time-based exits (hold for minimum/maximum duration)
- Composite conditions with AND/OR logic
- Price-based conditions relative to entry price

### 3. Order Execution Integration
When conditions trigger:
- Calculate exact order amounts based on position size
- Create appropriate trade parameters with MEV protection
- Execute trades through the Paper Trade Executor
- Record execution details including trigger price and order metadata
- Publish events for UI updates and logging

### 4. Order Management API
Provide methods to:
```rust
// Add a stop-loss order
add_stop_loss(token: &str, trigger_price: Decimal, is_trailing: bool, trail_percentage: Option<Decimal>) -> Result<Uuid>

// Add a take-profit order
add_take_profit(token: &str, trigger_price: Decimal, partial_percentage: Option<Decimal>) -> Result<Uuid>

// Cancel an active order
cancel_order(order_id: Uuid) -> Result<bool>

// Get all active orders
get_active_orders() -> Vec<ConditionalOrder>

// Update order parameters
update_order(order_id: Uuid, new_condition: OrderCondition) -> Result<()>
```

### 5. Performance Requirements
Your implementation must:
- Complete each monitoring cycle in <100ms
- Handle 50+ positions with multiple orders each
- Use Redis cache for <1ms price lookups
- Process position checks in parallel
- Never block the monitoring loop

### 6. Data Structures
```rust
pub struct ConditionalOrder {
    id: Uuid,
    token: String,
    condition: OrderCondition,
    action: OrderAction,
    amount: OrderAmount,
    created_at: DateTime<Utc>,
    metadata: HashMap<String, String>,
}

pub enum OrderCondition {
    StopLoss {
        trigger_price: Decimal,
        is_trailing: bool,
        trail_distance: Option<Decimal>,
        high_water_mark: Option<Decimal>,
    },
    TakeProfit {
        trigger_price: Decimal,
        partial_percentage: Option<Decimal>,
    },
    // Add other condition types
}

pub enum OrderAmount {
    FullPosition,
    FixedAmount(Decimal),
    Percentage(Decimal),
}
```

## Technical Requirements

1. **Concurrency**: Use tokio::spawn for parallel position monitoring
2. **Precision**: Use rust_decimal for accurate price comparisons
3. **Thread Safety**: Protect shared state with Arc<RwLock<>>
4. **Error Recovery**: Continue monitoring even if individual orders fail
5. **Logging**: Record all order triggers and execution results

## Integration Points

- **Virtual Portfolio**: Read positions from Arc<RwLock<VirtualPortfolio>>
- **Price Cache**: Get current prices from Arc<PriceCache>
- **Trade Executor**: Submit orders via Arc<PaperTradeExecutor>
- **Event Stream**: Publish order events to Redis Streams

## Success Criteria

Your implementation will be considered complete when:
1. Monitoring runs consistently at 100ms intervals
2. All order condition types are properly evaluated
3. Triggered orders execute within the same monitoring cycle
4. Trailing stops update their high water marks correctly
5. Partial orders calculate amounts accurately
6. Failed executions don't stop the monitoring loop
7. Performance remains stable with 50+ monitored positions

## Error Handling

Handle these scenarios gracefully:
- Missing price data (skip position, log warning)
- Trade execution failures (alert user, continue monitoring)
- Slow monitoring cycles (log performance warning)
- Invalid order conditions (reject at creation time)
- Network failures (retry with backoff)

## Example Usage
```rust
// Create monitor
let monitor = OrderMonitor::new(portfolio, price_cache, trade_executor);

// Start monitoring
let handle = monitor.start_monitoring().await;

// Add a 5% trailing stop-loss
let order_id = monitor.add_stop_loss(
    "SOL",
    Decimal::from(95),
    true, // trailing
    Some(Decimal::from(5)) // 5% trail
).await?;

// Add a take-profit at $110 for 50% of position
let tp_id = monitor.add_take_profit(
    "SOL",
    Decimal::from(110),
    Some(Decimal::from(50)) // 50% partial
).await?;
```