# Task 25: Implement gRPC Testing Client for Trade Request Simulation - Autonomous Prompt

You are tasked with creating a comprehensive testing client for the gRPC Trade Execution Service. This client will simulate external trading decision services, enabling thorough testing of the trading platform during development and integration phases.

## Context

The gRPC Trade Execution Service (Task 24) is now implemented and needs a sophisticated testing client. This client must support various testing scenarios including single trades, batch operations, performance testing, and error validation. It will be used by developers for debugging, QA teams for validation, and can serve as a reference implementation for external integrations.

## Current State

You have access to:
- Protocol buffer definitions from Task 24
- Trade request and response message structures
- The gRPC service endpoint specification
- Common trading models from Task 1

## Requirements

### 1. CLI Application Structure
Build a command-line application with these subcommands:
- `trade` - Execute a single trade with specified parameters
- `batch` - Execute multiple trades from a JSON file
- `template` - Execute trades using predefined templates
- `perf` - Run performance tests with configurable load
- `history` - Query trade history
- `positions` - Get current positions
- `health` - Check service health

### 2. Trade Execution Features
Implement comprehensive trade execution capabilities:
- Support all MVP tokens (SOL, USDC, BONK, JitoSOL, RAY)
- Token symbol to address resolution
- Proper decimal conversion for amounts
- Configurable slippage tolerance (0.1% to 5%)
- MEV protection with priority fee settings
- Both paper and live trading modes
- Request ID generation or manual specification

### 3. Template System
Create a reusable template system:
- TOML configuration file support
- Named templates for common trades
- Priority fee modes (low/medium/high/custom)
- Default values with overrides
- Template validation on load

### 4. Real-time Monitoring
Implement execution monitoring features:
- Connect to status streaming endpoint
- Display real-time progress updates
- Show completion percentage
- Final result summary with metrics
- Latency measurement and reporting

### 5. Batch Processing
Build batch execution capabilities:
- Load trades from JSON files
- Configurable concurrency limits
- Optional delays between requests
- Progress tracking for batch operations
- Result aggregation and reporting

### 6. Performance Testing
Create comprehensive performance testing:
- Configurable request patterns (fixed/random/weighted)
- Target RPS (requests per second) support
- Concurrent request execution
- Latency histogram collection (P50, P90, P95, P99)
- Throughput measurement
- Detailed performance reports

### 7. Error Scenario Testing
Implement negative testing capabilities:
- Invalid token addresses
- Excessive slippage values
- Invalid amounts (negative, too large)
- Missing authentication
- Network timeout simulation
- Expected vs actual error validation

### 8. Metrics and Reporting
Build comprehensive metrics collection:
- Request/response logging
- Success/failure rates
- Latency statistics
- Error categorization
- Export to various formats

## Technical Specifications

### Dependencies
```toml
[dependencies]
clap = "2.34"
tokio = { version = "1.35", features = ["full"] }
tonic = "0.10"
prost = "0.12"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
uuid = { version = "1.6", features = ["v4"] }
chrono = "0.4"
anyhow = "1.0"
thiserror = "1.0"
futures = "0.3"
hdrhistogram = "7.5"
env_logger = "0.11"
log = "0.4"

[build-dependencies]
tonic-build = "0.10"
```

### Key Components
- `TestClient` - Main client wrapper around gRPC client
- `TradeParams` - Structured trade parameters
- `RequestTemplate` - Template system for reusable trades
- `PerformanceTester` - Performance testing engine
- `ErrorScenarioTester` - Negative testing framework
- `MetricsCollector` - Metrics aggregation

## Implementation Guidelines

1. **Start with CLI**: Build complete command structure with clap
2. **Implement Client**: Create TestClient with connection management
3. **Add Basic Trade**: Implement single trade execution
4. **Build Templates**: Add template loading and execution
5. **Add Streaming**: Implement status stream handling
6. **Batch Support**: Add batch processing with concurrency
7. **Performance Tests**: Build load testing capabilities
8. **Error Testing**: Add negative test scenarios
9. **Polish Output**: Format results nicely for users

## Example Usage Patterns

```bash
# Single trade
./test-client trade --token-in USDC --token-out SOL --amount 10 --slippage 50

# With streaming
./test-client trade --token-in USDC --token-out SOL --amount 10 --stream

# From template
./test-client template --name small_sol_buy --config templates.toml

# Batch execution
./test-client batch --file trades.json --concurrent 10 --delay 100

# Performance test
./test-client perf --requests 1000 --concurrent 50 --rate 100

# Error testing
./test-client test-errors --all
```

## Configuration Example

```toml
[connection]
endpoint = "http://localhost:50051"
timeout_ms = 5000

[default_trade]
token_in = "USDC"
token_out = "SOL"
amount = 10.0
slippage_bps = 50
priority_fee_mode = "medium"

[[templates]]
name = "small_sol_buy"
token_in = "USDC"
token_out = "SOL"
amount = 5.0
slippage_bps = 30
priority_fee_mode = "low"
trading_mode = "paper"
```

## Success Criteria

The testing client is complete when:
1. All CLI commands parse and execute correctly
2. Single trades execute with proper parameter handling
3. Templates load from configuration files
4. Streaming shows real-time updates
5. Batch processing handles concurrent requests
6. Performance tests generate accurate metrics
7. Error scenarios validate expected failures
8. Output is clear and well-formatted
9. Metrics provide actionable insights
10. Documentation includes usage examples

## Additional Considerations

- Support environment variables for sensitive data (auth tokens)
- Add request retry logic with exponential backoff
- Implement request deduplication for batch mode
- Support output formats (JSON, CSV, human-readable)
- Add interactive mode for repeated testing
- Consider adding a web UI for visualization
- Support recording and replaying test scenarios
- Add comparison tools for regression testing