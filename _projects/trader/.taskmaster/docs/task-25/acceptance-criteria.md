# Task 25: gRPC Testing Client - Acceptance Criteria

## CLI Interface

### 1. Command Structure
- [ ] Main binary compiles successfully
- [ ] Help text displays all commands
- [ ] Global flags work (--endpoint, --auth-token, --verbose)
- [ ] Subcommands accessible (trade, batch, template, perf, history, positions, health)
- [ ] Invalid commands show helpful error
- [ ] Environment variables work for auth token
- [ ] Verbose flag enables debug logging

### 2. Trade Command
- [ ] Accepts all required parameters
- [ ] Token symbols resolve to addresses
- [ ] Amount parsing handles decimals
- [ ] Slippage validation (0-10000 bps)
- [ ] Mode selection works (paper/live)
- [ ] Priority fee optional parameter
- [ ] Stream flag enables streaming
- [ ] Output shows trade results

### 3. Batch Command
- [ ] Loads JSON file successfully
- [ ] Validates all trades in batch
- [ ] Concurrent execution works
- [ ] Delay between requests applied
- [ ] Progress shown during execution
- [ ] Summary statistics displayed
- [ ] Errors don't stop batch

### 4. Template Command
- [ ] Loads TOML configuration file
- [ ] Template lookup by name works
- [ ] Default values applied
- [ ] Template validation on load
- [ ] Missing template shows error
- [ ] Override parameters work

### 5. Performance Command
- [ ] Request count parameter works
- [ ] Concurrency limit applied
- [ ] Duration-based tests work
- [ ] Rate limiting functions
- [ ] Metrics collected accurately
- [ ] Report generated at end

## Core Functionality

### 6. Client Connection
- [ ] Connects to gRPC service
- [ ] Timeout configuration works
- [ ] Connection reuse implemented
- [ ] Error on connection failure
- [ ] Retry logic for transient errors
- [ ] TLS support if configured

### 7. Token Resolution
- [ ] SOL maps to correct address
- [ ] USDC maps to correct address
- [ ] BONK maps to correct address
- [ ] JitoSOL maps to correct address
- [ ] RAY maps to correct address
- [ ] Unknown symbols rejected
- [ ] Direct addresses accepted

### 8. Amount Conversion
- [ ] SOL uses 9 decimals
- [ ] USDC uses 6 decimals
- [ ] BONK uses 5 decimals
- [ ] Conversion accurate
- [ ] No precision loss
- [ ] Large amounts handled
- [ ] Zero amounts rejected

## Execution Features

### 9. Single Trade Execution
- [ ] Request built correctly
- [ ] Auth token included
- [ ] Response parsed properly
- [ ] Success/failure detected
- [ ] Transaction ID captured
- [ ] Execution price shown
- [ ] Slippage calculated
- [ ] Latency measured

### 10. Streaming Execution
- [ ] Stream connection established
- [ ] Updates received in real-time
- [ ] Progress percentage shown
- [ ] Status messages displayed
- [ ] Stream closes on completion
- [ ] Errors handled gracefully
- [ ] Final result captured

### 11. Batch Processing
- [ ] All trades executed
- [ ] Concurrency respected
- [ ] Results collected
- [ ] Statistics calculated
- [ ] Failures isolated
- [ ] Progress updated
- [ ] Memory efficient

## Template System

### 12. Configuration Loading
- [ ] TOML file parsed correctly
- [ ] Templates section found
- [ ] All fields validated
- [ ] Defaults applied
- [ ] Invalid config rejected
- [ ] File not found handled

### 13. Template Execution
- [ ] Template parameters used
- [ ] Priority fee modes work
- [ ] Trading mode respected
- [ ] Description shown
- [ ] Success reported
- [ ] Errors clear

### 14. Priority Fee Modes
- [ ] Low = 1,000 lamports
- [ ] Medium = 5,000 lamports
- [ ] High = 10,000 lamports
- [ ] Custom values work
- [ ] Invalid modes rejected

## Performance Testing

### 15. Load Generation
- [ ] Specified request count executed
- [ ] Concurrent requests limited
- [ ] Rate limiting works
- [ ] Duration-based tests stop on time
- [ ] Request distribution even

### 16. Metrics Collection
- [ ] Total requests counted
- [ ] Success/failure tracked
- [ ] Latency histogram populated
- [ ] Percentiles calculated (P50, P90, P95, P99)
- [ ] Throughput measured
- [ ] No metrics lost

### 17. Performance Report
- [ ] Summary statistics shown
- [ ] Latency percentiles displayed
- [ ] Success rate calculated
- [ ] Throughput reported
- [ ] Duration accurate
- [ ] Max latency captured

## Error Testing

### 18. Error Scenarios
- [ ] Invalid token test passes
- [ ] Excessive slippage test passes
- [ ] Invalid amount test passes
- [ ] Missing auth test passes
- [ ] Timeout test passes
- [ ] All scenarios run

### 19. Error Validation
- [ ] Expected errors match actual
- [ ] Error types categorized
- [ ] Pass/fail determined correctly
- [ ] Summary shows results
- [ ] Details available

## Other Commands

### 20. History Query
- [ ] Time range filtering works
- [ ] Results returned
- [ ] Limit respected
- [ ] Output formatted nicely
- [ ] Empty results handled

### 21. Position Query
- [ ] Current positions retrieved
- [ ] P&L calculated
- [ ] Mode parameter works
- [ ] Total value shown
- [ ] Format readable

### 22. Health Check
- [ ] Health status retrieved
- [ ] Component status shown
- [ ] Version displayed
- [ ] Latency reported
- [ ] No auth required

## Output & Reporting

### 23. Result Display
- [ ] Trade results formatted clearly
- [ ] Batch summaries informative
- [ ] Performance reports detailed
- [ ] Error messages helpful
- [ ] Progress indicators work

### 24. Logging
- [ ] Verbose mode shows debug info
- [ ] Errors logged appropriately
- [ ] Request/response logged
- [ ] Timestamps included
- [ ] Sensitive data masked

## Integration Testing

### 25. End-to-End Tests
```rust
#[tokio::test]
async fn test_single_trade_flow() {
    // Execute trade via CLI
    // Verify result
}

#[tokio::test]
async fn test_batch_execution() {
    // Run batch trades
    // Check summary
}

#[tokio::test]
async fn test_performance_run() {
    // Execute perf test
    // Validate metrics
}
```

### 26. Mock Server Testing
```rust
#[tokio::test]
async fn test_against_mock_server() {
    // Start mock server
    // Run client tests
    // Verify behavior
}
```

## Example Files

### 27. Template File
```toml
# Verify this format works
[connection]
endpoint = "http://localhost:50051"

[[templates]]
name = "test_trade"
token_in = "USDC"
token_out = "SOL"
amount = 10.0
```

### 28. Batch File
```json
{
  "trades": [
    {
      "token_in": "USDC",
      "token_out": "SOL",
      "amount": 10.0,
      "slippage_bps": 50,
      "mode": "Paper"
    }
  ]
}
```

## Manual Testing Checklist

### Command Testing
- [ ] Execute single trade
- [ ] Run with streaming
- [ ] Execute from template
- [ ] Run batch file
- [ ] Execute performance test
- [ ] Query history
- [ ] Check positions
- [ ] Verify health

### Error Handling
- [ ] Test with invalid endpoint
- [ ] Test with bad auth token
- [ ] Test with malformed requests
- [ ] Test with network issues
- [ ] Verify error messages

### Performance Verification
- [ ] Run 1000 request test
- [ ] Verify metrics accuracy
- [ ] Check memory usage
- [ ] Monitor CPU usage
- [ ] Validate throughput

## Definition of Done

- [ ] All CLI commands functional
- [ ] Single trades execute correctly
- [ ] Batch processing works
- [ ] Templates load and execute
- [ ] Performance tests generate metrics
- [ ] Error scenarios validate properly
- [ ] Streaming shows real-time updates
- [ ] Output is clear and helpful
- [ ] Integration tests pass
- [ ] Documentation complete