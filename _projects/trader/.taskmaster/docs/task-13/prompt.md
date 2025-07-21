# Task 13: Implement Paper Trader Binary with CLI Interface - Autonomous Prompt

You are implementing the main paper trader binary that serves as the entry point for the Solana paper trading system. This binary must orchestrate all components, handle configuration, and provide a user-friendly command-line interface.

## Context
- The paper trader simulates real trades without using actual funds
- It must initialize multiple services in the correct order
- Configuration can come from files, environment variables, or CLI arguments
- The system includes databases, monitoring services, and a TUI
- Graceful shutdown is critical to prevent data loss

## Your Task

Implement a complete paper trader binary with the following features:

### 1. Command-Line Interface
Create a comprehensive CLI using clap that supports:

```bash
paper-trader [OPTIONS]

OPTIONS:
    -c, --config <FILE>              Configuration file path [default: config.yaml]
    --initial-sol <AMOUNT>           Initial SOL allocation
    --initial-usdc <AMOUNT>          Initial USDC allocation
    --slippage <PERCENT>             Slippage tolerance (0.5-2.0%) [default: 1.0]
    --mev-protection <true|false>    Enable MEV protection [default: true]
    --tokens <LIST>                  Comma-separated tokens to monitor [default: SOL,USDC,BONK,JitoSOL,RAY]
    -l, --log-level <LEVEL>          Logging level [default: info]
    --no-tui                         Run in headless mode
    --metrics-port <PORT>            Prometheus metrics port [default: 9090]
    -h, --help                       Print help information
    -V, --version                    Print version information
```

### 2. Configuration Management
Implement a layered configuration system:

**Priority Order** (highest to lowest):
1. Command-line arguments
2. Environment variables
3. Configuration file
4. Default values

**Configuration Structure**:
```yaml
trading:
  slippage_tolerance: 1.0
  monitored_tokens: [SOL, USDC, BONK, JitoSOL, RAY]
  order_monitoring_interval_ms: 100
  max_position_size: 10000.0
  max_daily_loss: 1000.0

portfolio:
  initial_allocation:
    SOL: 100.0
    USDC: 10000.0

mev:
  protection_enabled: true
  default_priority_fee: 5000
  sandwich_simulation: true
  max_priority_fee: 50000

database:
  questdb_url: "http://localhost:9000"
  postgres_url: "postgresql://user:pass@localhost/trader"
  redis_url: "redis://localhost:6379"
  connection_pool_size: 10

jupiter:
  self_hosted_url: "http://jupiter-self:8080"
  public_url: "https://quote-api.jup.ag/v6"
  timeout_ms: 200
  use_self_hosted: true

monitoring:
  latency_threshold_ms: 200
  error_rate_threshold: 0.05
  
logging:
  level: info
  file: logs/paper-trader.log
  max_size_mb: 100
  max_files: 10
```

### 3. Component Initialization
Initialize all components in the correct order:

```rust
1. Parse CLI arguments and load configuration
2. Initialize logging system
3. Create database connections (QuestDB, PostgreSQL, Redis)
4. Initialize price cache
5. Create Jupiter client with failover
6. Initialize virtual portfolio
7. Create MEV simulator
8. Create paper trade executor
9. Initialize order monitor
10. Create system health monitor
11. Start all background services
12. Launch TUI or run headless
```

### 4. Service Management
Start and manage background services:

- **Price Updater**: Fetches prices for monitored tokens
- **Order Monitor**: Checks stop-loss/take-profit conditions
- **Batch Writer**: Writes to QuestDB every 100ms
- **Retention Manager**: Cleans data older than 30 days
- **Health Monitor**: Tracks system health metrics

### 5. Error Handling and Recovery
Implement robust error handling:

- Retry database connections with exponential backoff
- Handle configuration validation errors clearly
- Gracefully degrade if optional services fail
- Log all errors with appropriate context
- Ensure data integrity during shutdown

### 6. Shutdown Handling
Implement graceful shutdown:

```rust
// On SIGINT (Ctrl+C) or SIGTERM:
1. Stop accepting new trades
2. Complete any in-flight operations
3. Flush all pending database writes
4. Save current portfolio state
5. Stop all background services
6. Close database connections
7. Exit with appropriate code
```

### 7. Metrics and Monitoring
Expose Prometheus metrics:

```
# Trading metrics
paper_trader_trades_total{status="success|failed"}
paper_trader_portfolio_value{token="SOL|USDC"}
paper_trader_pnl_percentage

# System metrics
paper_trader_order_monitoring_latency_seconds
paper_trader_batch_write_duration_seconds
paper_trader_active_connections{type="questdb|postgres|redis"}

# Health metrics
paper_trader_up
paper_trader_circuit_breaker_status{state="open|closed|half_open"}
```

## Technical Requirements

1. **Async Runtime**: Use tokio with multi-threaded runtime
2. **Signal Handling**: Properly handle SIGINT and SIGTERM
3. **Logging**: Use env_logger with configurable levels
4. **Configuration**: Support YAML and TOML formats
5. **Validation**: Validate all configuration parameters

## Success Criteria

Your implementation will be considered complete when:
1. CLI accepts all specified arguments
2. Configuration loads from file, env, and CLI correctly
3. All components initialize in proper order
4. Background services start without errors
5. TUI launches when not in headless mode
6. Graceful shutdown completes cleanly
7. Metrics endpoint serves Prometheus format
8. Error cases are handled appropriately

## Example Usage

```bash
# Basic usage with defaults
./paper-trader

# Custom configuration
./paper-trader -c custom-config.yaml --initial-sol 200 --initial-usdc 20000

# Headless mode with high verbosity
./paper-trader --no-tui -l debug --metrics-port 9091

# Override MEV settings
./paper-trader --mev-protection false --slippage 2.0

# Monitor specific tokens
./paper-trader --tokens "SOL,USDC,ORCA,MNGO"
```

## Error Scenarios to Handle

1. **Missing Configuration File**: Use defaults or create template
2. **Database Connection Failure**: Retry with backoff, clear error message
3. **Invalid Configuration**: Show specific validation errors
4. **Port Already in Use**: Suggest alternative or auto-increment
5. **Insufficient Permissions**: Guide user to fix permissions
6. **Component Init Failure**: Roll back and clean up partially initialized state

## Integration Testing

Test the complete initialization flow:
```rust
#[tokio::test]
async fn test_full_initialization() {
    // Create test config
    // Initialize all components
    // Verify services start
    // Send shutdown signal
    // Verify clean shutdown
}
```