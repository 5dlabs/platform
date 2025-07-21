# Task 10: Develop Terminal-Based User Interface (TUI) - Autonomous Prompt

You are implementing a sophisticated terminal-based user interface (TUI) for the Solana trading platform. This interface must provide real-time trading data visualization with a 10Hz refresh rate using Redis Streams.

## Context
- The platform consists of paper and live trading services requiring identical interfaces
- Traders need real-time position monitoring, trade execution, and system health visibility
- The TUI must handle high-frequency updates without lag or flicker
- MEV protection status and latency metrics are critical for trading decisions

## Your Task

Implement a complete TUI system with the following components:

### 1. Core Framework
- Use Rust's ratatui library for terminal rendering
- Implement a dual event loop handling both keyboard input and data updates
- Achieve 10Hz (100ms) refresh rate without screen flicker
- Create tab-based navigation: Portfolio, Trades, Order Entry, System Health

### 2. Visual Components
Implement these UI elements:

**Portfolio View**:
- Position list with token, amount, entry price, current price, P&L percentage
- Color coding: green for profit, red for loss
- P&L sparkline chart showing historical performance
- Total portfolio value and overall P&L

**Trade History**:
- Table showing last 20 trades with timestamp, action, pair, amount, price, slippage
- MEV status indicators (protected/at-risk/impacted)
- Priority fee display for MEV-protected trades

**System Health Dashboard**:
- Circuit breaker status (Open/Closed/Half-Open) with color indicators
- Latency metrics: Node P99, Trade Execution P99, Jupiter Response time
- Visual alerts when latency exceeds 200ms threshold
- Uptime percentage and error rates

**Order Entry Form**:
- Token pair selection
- Amount input with validation
- Slippage tolerance setting (0.5-2%)
- MEV protection toggle
- Submit/Cancel buttons

### 3. Real-time Integration
- Subscribe to Redis Streams on "ui-events" channel
- Implement consumer group for reliable event processing
- Handle events: price updates, trade executions, position changes, system alerts
- Buffer updates within 100ms window to batch renders

### 4. Keyboard Navigation
```
Tab       - Switch between views
Arrow Keys - Navigate within lists/tables
Enter     - Submit forms/select items
q         - Quit application
r         - Force refresh
/         - Search in current view
Esc       - Cancel current operation
```

### 5. Performance Requirements
- Render time must be <10ms per frame
- Memory usage should remain stable over long sessions
- CPU usage should stay below 5% during normal operation
- Handle terminal resize gracefully

### 6. Error Handling
- Display connection errors without crashing
- Show stale data indicators when updates stop
- Provide user feedback for all actions
- Log errors to file while keeping UI responsive

## Technical Specifications

```rust
// Event structure from Redis
pub enum UiEvent {
    PriceUpdate { token: String, price: f64 },
    TradeExecuted(Trade),
    PositionChanged { token: String, position: Position },
    SystemAlert { level: AlertLevel, message: String },
    CircuitBreakerChanged(CircuitState),
}

// Required refresh rate
const REFRESH_INTERVAL_MS: u64 = 100; // 10Hz

// Redis Stream configuration
const STREAM_KEY: &str = "ui-events";
const CONSUMER_GROUP: &str = "tui-consumer";
const MAX_PENDING: usize = 1000;
```

## Integration Requirements

1. **Virtual Portfolio Access**: Read positions via Arc<RwLock<VirtualPortfolio>>
2. **Trade Submission**: Send orders through Arc<PaperTradeExecutor>
3. **System Health**: Monitor Arc<SystemHealth> for circuit breaker status
4. **Event Stream**: Use Redis XREADGROUP for guaranteed delivery

## Success Criteria

Your implementation will be considered complete when:
1. All four main views (Portfolio, Trades, Order Entry, System Health) are functional
2. 10Hz refresh rate is achieved without visual artifacts
3. Keyboard navigation works smoothly across all views
4. Real-time data updates are reflected within 100ms
5. System health alerts are prominently displayed
6. Memory and CPU usage remain within specified limits
7. The interface remains responsive under high event load

## Additional Notes
- Use color sparingly but effectively (red/green for P&L, status indicators)
- Ensure all text is readable on both dark and light terminal backgrounds
- Include help text in the footer showing available keyboard shortcuts
- Test with various terminal sizes (minimum 80x24)