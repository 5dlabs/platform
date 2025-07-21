# Task 21: Implement Live Trader Binary with Command-Line Interface - Autonomous Prompt

You are tasked with implementing the main executable binary for a Solana trading bot's live trading system. This binary serves as the primary command-line interface for executing real trades on the blockchain, integrating with secure wallet management, risk controls, and monitoring infrastructure.

## Context

You are building a live trader CLI that mirrors the paper trader's interface while adding production-ready features for real fund management. The system must integrate with previously built components including an Enhanced Wallet Manager (Task 16), Live Trade Executor (Task 17), Risk Management System (Task 13), and Monitoring Infrastructure (Task 18).

## Current State

The following components are already implemented and available for integration:
- `EnhancedWalletManager` - Handles encrypted wallet storage and transaction signing
- `LiveTradeExecutor` - Executes trades with MEV protection
- `RiskManager` - Validates trades against position and loss limits
- `CircuitBreaker` - Monitors system health and pauses trading when needed
- `MetricsCollector` - Tracks performance and generates audit logs

## Requirements

### 1. CLI Structure
Implement a comprehensive command-line interface using the `clap` crate with:
- Main commands: buy, sell, swap, monitor, positions, health
- Global options: --config, --wallet, --mode (live/paper), --dry-run
- Safety features: --emergency-stop flag
- Proper help text and usage examples

### 2. Configuration System
Create a flexible configuration system that:
- Supports both YAML and TOML formats
- Validates all parameters before use
- Allows environment variable overrides for sensitive data
- Implements secure handling of API keys and wallet paths

### 3. Trade Execution Commands
Implement trading commands that:
- Validate all parameters before execution
- Check circuit breaker status
- Enforce risk limits through the RiskManager
- Show clear trade details before execution
- Require confirmation for live trades
- Display comprehensive results including MEV protection status

### 4. Monitoring Features
Create monitoring capabilities that:
- Show real-time position updates
- Display system health metrics
- Track P&L and risk exposure
- Update at configurable intervals
- Support graceful shutdown

### 5. Safety Features
Implement critical safety mechanisms:
- Emergency stop functionality that halts all trading
- Dry-run mode for testing without execution
- Confirmation prompts for live trading
- Comprehensive audit logging with sensitive data redaction
- Graceful shutdown with position reporting

### 6. Error Handling
Ensure robust error handling with:
- Clear, actionable error messages
- Context for troubleshooting
- Graceful degradation where possible
- Proper cleanup on failures

## Technical Specifications

### Dependencies
```toml
[dependencies]
clap = "2.34"
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
toml = "0.8"
anyhow = "1.0"
chrono = "0.4"
env_logger = "0.11"
log = "0.4"
ctrlc = "3.4"
```

### Key Structures
- `LiveTraderApp` - Main application struct holding all components
- `LiveTraderConfig` - Configuration structure with validation
- `TradingMode` - Enum for live/paper/dry-run modes
- Command handlers for each CLI subcommand

### Integration Requirements
- Use Arc<T> for thread-safe component sharing
- Implement async/await for all I/O operations
- Use broadcast channels for shutdown coordination
- Maintain consistent error types with Result<T>

## Implementation Guidelines

1. **Start with CLI Definition**: Build the complete command-line interface structure first
2. **Configuration Loading**: Implement secure configuration management with validation
3. **Component Integration**: Wire up all existing components (wallet, executor, risk, monitoring)
4. **Command Implementation**: Build each command handler with proper error handling
5. **Safety Features**: Add emergency stop and confirmation prompts
6. **Testing**: Create unit tests for CLI parsing and integration tests for commands

## Example Usage

```bash
# Buy command with custom priority fee
./live_trader --mode live --wallet wallet.json buy BONK 10 --slippage 150 --priority-fee 5000

# Monitor positions in real-time
./live_trader monitor --interval 500

# Emergency stop
./live_trader --emergency-stop

# Dry-run mode for testing
./live_trader --dry-run swap SOL USDC 5.0
```

## Success Criteria

The implementation is complete when:
1. All CLI commands parse correctly and show proper help
2. Configuration loading supports YAML/TOML with validation
3. Trading commands integrate with all required components
4. Live mode requires confirmation and shows warnings
5. Emergency stop immediately halts all operations
6. Monitoring shows real-time updates
7. All operations generate audit logs
8. Error messages are clear and actionable
9. The binary handles signals for graceful shutdown

## Additional Considerations

- Log sensitive operations but redact private keys and API tokens
- Implement connection pooling for database operations
- Use colored output for better visibility of warnings and errors
- Support both absolute and relative paths for configuration files
- Validate token symbols/addresses before trade execution
- Show estimated gas costs before confirming trades
- Implement command history for audit purposes