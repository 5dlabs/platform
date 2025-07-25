# Task 21: Live Trader Binary with Command-Line Interface - Acceptance Criteria

## Functional Requirements

### 1. CLI Interface Structure
- [ ] Binary executable named `live_trader` builds successfully
- [ ] Help command (`--help`) displays all available commands and options
- [ ] Version flag (`--version`) shows correct version information
- [ ] All subcommands (buy, sell, swap, monitor, positions, health) are accessible
- [ ] Global flags (--config, --wallet, --mode, --dry-run, --emergency-stop) work correctly
- [ ] Invalid commands show helpful error messages
- [ ] Tab completion configuration can be generated for common shells

### 2. Configuration Management
- [ ] YAML configuration files load successfully
- [ ] TOML configuration files load successfully
- [ ] Invalid configuration files produce clear error messages
- [ ] Environment variables override configuration file values
- [ ] Missing required configuration shows specific error
- [ ] Configuration validation catches invalid parameters
- [ ] Default configuration values are applied when not specified
- [ ] Sensitive configuration values can be loaded from environment variables

### 3. Trading Commands

#### Buy Command
- [ ] Accepts token symbol or address as parameter
- [ ] Validates amount is positive number
- [ ] Accepts optional slippage parameter (defaults to 100 bps)
- [ ] Accepts optional priority fee parameter
- [ ] Shows quote details before execution
- [ ] Requires confirmation prompt in live mode
- [ ] Executes trade through LiveTradeExecutor
- [ ] Displays transaction signature on success
- [ ] Shows actual execution price and slippage
- [ ] Updates position tracking after execution

#### Sell Command
- [ ] Accepts token symbol or address as parameter
- [ ] Validates user has sufficient token balance
- [ ] Accepts amount to sell
- [ ] Shows expected proceeds before execution
- [ ] Requires confirmation in live mode
- [ ] Executes trade with MEV protection
- [ ] Reports execution results

#### Swap Command
- [ ] Accepts source and destination tokens
- [ ] Validates swap amount
- [ ] Shows conversion rate before execution
- [ ] Applies slippage protection
- [ ] Requires confirmation in live mode
- [ ] Reports swap results with both token amounts

### 4. Monitoring Commands

#### Monitor Command
- [ ] Starts real-time position monitoring
- [ ] Updates at specified interval (default 1000ms)
- [ ] Shows current positions with P&L
- [ ] Displays system health status
- [ ] Shows risk metrics (daily P&L, exposure)
- [ ] Responds to Ctrl+C for graceful exit
- [ ] Clears screen between updates for clean display

#### Positions Command
- [ ] Lists all current positions
- [ ] Shows token, amount, cost basis, current price
- [ ] Calculates and displays unrealized P&L
- [ ] Shows P&L percentage for each position
- [ ] Displays totals row when multiple positions exist
- [ ] Shows "No open positions" when empty

#### Health Command
- [ ] Displays circuit breaker status
- [ ] Shows current node latency
- [ ] Reports error rate percentage
- [ ] Shows wallet address and SOL balance
- [ ] Displays risk limit usage
- [ ] Indicates if trading is currently possible

### 5. Safety Features

#### Emergency Stop
- [ ] `--emergency-stop` flag immediately halts all trading
- [ ] Opens circuit breaker to prevent new trades
- [ ] Cancels any pending transactions
- [ ] Reports all current positions
- [ ] Saves emergency state to timestamped JSON file
- [ ] Cannot be combined with trading commands
- [ ] Shows clear message about trading being halted

#### Live Mode Warnings
- [ ] Live mode shows prominent warning message
- [ ] 5-second countdown before proceeding
- [ ] Ctrl+C during countdown cancels operation
- [ ] Each trade shows "LIVE TRADE" warning
- [ ] Confirmation prompt appears before execution
- [ ] "N" or empty response cancels trade

#### Dry Run Mode
- [ ] `--dry-run` flag prevents actual execution
- [ ] Shows what would be executed
- [ ] Validates all parameters
- [ ] Displays estimated results
- [ ] Clearly indicates "DRY RUN" in output
- [ ] No wallet required in dry-run mode

### 6. Error Handling
- [ ] Missing wallet path shows clear error
- [ ] Invalid token symbols produce helpful message
- [ ] Network connection failures show retry behavior
- [ ] Circuit breaker open state prevents trades
- [ ] Risk limit violations show specific limits exceeded
- [ ] Configuration errors indicate exact problem
- [ ] File permission errors are clearly reported

### 7. Logging and Audit
- [ ] All trades logged with timestamp and details
- [ ] Sensitive data (keys, passwords) redacted in logs
- [ ] Log level configurable via environment variable
- [ ] Audit entries include user action and result
- [ ] Failed operations logged with error details
- [ ] Emergency stops logged with state snapshot

### 8. Integration Requirements
- [ ] Integrates with EnhancedWalletManager for signing
- [ ] Uses LiveTradeExecutor for trade execution
- [ ] Validates trades through RiskManager
- [ ] Respects CircuitBreaker status
- [ ] Reports metrics to MetricsCollector
- [ ] All async operations handle cancellation

## Non-Functional Requirements

### Performance
- [ ] CLI responds to commands within 100ms
- [ ] Configuration loads in under 50ms
- [ ] Monitor updates don't lag with 100ms interval
- [ ] Graceful shutdown completes within 5 seconds

### Security
- [ ] No sensitive data in command history
- [ ] Wallet passwords never shown in output
- [ ] API keys loaded only from environment
- [ ] Audit logs stored with restricted permissions
- [ ] No sensitive data in error messages

### Usability
- [ ] Help text is clear and comprehensive
- [ ] Error messages suggest corrective actions
- [ ] Output uses consistent formatting
- [ ] Colors used appropriately (red for errors, green for success)
- [ ] Progress indicators for long operations

## Test Scenarios

### Unit Tests
```rust
#[test]
fn test_cli_parsing() {
    // Test all command combinations parse correctly
}

#[test]
fn test_config_validation() {
    // Test configuration validation rules
}

#[test]
fn test_mode_selection() {
    // Test live/paper/dry-run mode logic
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_buy_command_flow() {
    // Test complete buy command execution
}

#[tokio::test]
async fn test_emergency_stop() {
    // Test emergency stop functionality
}

#[tokio::test]
async fn test_monitor_shutdown() {
    // Test graceful shutdown during monitoring
}
```

### Manual Testing Checklist
- [ ] Execute buy command in dry-run mode
- [ ] Execute sell command with confirmation
- [ ] Run monitor and verify real-time updates
- [ ] Trigger emergency stop during operation
- [ ] Test with invalid configuration file
- [ ] Verify Ctrl+C handling in all modes
- [ ] Test with missing environment variables
- [ ] Verify audit log generation

## Definition of Done
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Code coverage exceeds 80%
- [ ] Documentation includes usage examples
- [ ] Binary runs on target platform
- [ ] No security vulnerabilities in dependencies
- [ ] Audit logging verified working
- [ ] Emergency stop tested thoroughly
- [ ] Live mode warnings are prominent
- [ ] All commands have help text