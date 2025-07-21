# Task 17: Implement Live Trade Executor with Real Transaction Execution - Autonomous Prompt

You are implementing a robust live trading execution engine for a Solana trading platform. The executor must handle real transactions with comprehensive risk management, MEV protection, and reliable confirmation monitoring while integrating with external trade request sources via gRPC.

## Context

The live trading service receives pre-validated trade requests from external strategy services. Your executor focuses on the technical execution aspects: building transactions, managing priority fees, ensuring sufficient balances, and monitoring confirmations. The system must maintain sub-100ms latency to transaction broadcast while protecting against MEV attacks.

## Your Objectives

1. **Build Core Transaction Execution System**
   - Create transaction builder for Jupiter V6 swaps
   - Implement dynamic priority fee calculation (1000-10000 lamports)
   - Add MEV protection parameters to all swaps
   - Support versioned transactions for efficiency
   - Integrate wallet manager for secure signing

2. **Implement gRPC Trade Request Interface**
   - Create `execute_trade_request(TradeRequest)` endpoint
   - Validate request format and required fields
   - Return detailed `TradeResult` with execution metrics
   - Handle errors with appropriate gRPC status codes
   - Log all requests and responses for audit trail

3. **Integrate Risk Management Checks**
   - Verify sufficient balance including fees
   - Check token whitelist status
   - Validate trade size limits
   - Ensure circuit breaker is open
   - Simulate transaction before execution

4. **Develop Confirmation Monitoring**
   - Poll for transaction confirmation status
   - Implement exponential backoff (100ms → 1s)
   - Set configurable timeout (default 30s)
   - Extract execution details from confirmed transaction
   - Handle partial confirmations and failures

5. **Create Comprehensive Error Handling**
   - Define specific error types for each failure mode
   - Implement retry logic for transient errors
   - Distinguish permanent vs temporary failures
   - Provide detailed error context for debugging
   - Track error rates and patterns

## Implementation Requirements

### Code Structure
```
live_trader/executor/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── builder.rs      # Transaction building
│   ├── executor.rs     # Main execution logic
│   ├── grpc.rs         # gRPC service implementation
│   ├── validation.rs   # Pre-trade validation
│   ├── monitoring.rs   # Confirmation tracking
│   ├── recording.rs    # Database integration
│   └── errors.rs       # Error types
├── proto/
│   └── trade_executor.proto
```

### gRPC Interface Definition
```protobuf
service TradeExecutor {
    rpc ExecuteTradeRequest(TradeRequest) returns (TradeResult);
    rpc GetExecutorStatus(Empty) returns (ExecutorStatus);
}

message TradeRequest {
    string request_id = 1;
    string source_service = 2;
    string input_token = 3;
    string output_token = 4;
    uint64 amount_lamports = 5;
    uint32 max_slippage_bps = 6;
    UrgencyLevel urgency_level = 7;
    double expected_price = 8;
    optional string fee_account = 9;
    optional string tracking_account = 10;
}

message TradeResult {
    string request_id = 1;
    TradeStatus status = 2;
    optional string transaction_signature = 3;
    uint64 executed_amount = 4;
    double executed_price = 5;
    uint32 slippage_bps = 6;
    uint64 fees_paid = 7;
    uint64 priority_fee = 8;
    uint64 execution_time_ms = 9;
    optional string error_message = 10;
    bool mev_protected = 11;
    uint64 confirmation_time_ms = 12;
}
```

### Key Implementation Details

1. **Priority Fee Strategy**:
   - Use percentiles based on urgency (50th, 75th, 90th, 95th)
   - Query recent fees from Solana
   - Clamp to 1000-10000 lamports range
   - Cache fee data for performance

2. **MEV Protection**:
   - Always set `wrap_and_unwrap_sol = true`
   - Enable `use_shared_accounts = true`
   - Include priority fee in compute budget
   - Use versioned transactions when possible

3. **Transaction Flow**:
   ```
   Receive Request → Validate → Check Balance → Simulate → 
   Build Transaction → Sign → Send → Monitor → Record
   ```

4. **Error Recovery**:
   - Retry network errors up to 3 times
   - Exponential backoff starting at 100ms
   - Don't retry permanent errors (insufficient funds, invalid tx)
   - Log all retry attempts

### Testing Requirements

1. **Unit Tests**:
   - Priority fee calculation with mock data
   - Request validation edge cases
   - Transaction builder output verification
   - Error type classification

2. **Integration Tests**:
   - gRPC service with mock client
   - Circuit breaker integration
   - Confirmation monitoring with devnet
   - Database recording verification

3. **End-to-End Tests**:
   - Full trade flow from request to confirmation
   - Failure scenarios and recovery
   - Concurrent request handling
   - Performance under load

### Performance Targets

- Request validation: <5ms
- Transaction building: <50ms (including Jupiter API)
- Signing operation: <10ms
- Total to broadcast: <100ms
- Confirmation polling: Async, non-blocking

### Database Schema

```sql
-- QuestDB tables
CREATE TABLE trades (
    timestamp TIMESTAMP,
    request_id SYMBOL,
    trader_id SYMBOL,
    mode SYMBOL,
    action SYMBOL,
    base_token SYMBOL,
    quote_token SYMBOL,
    amount DOUBLE,
    price DOUBLE,
    slippage DOUBLE,
    fee DOUBLE,
    priority_fee LONG,
    tx_signature STRING,
    latency_ms INT,
    mev_protected BOOLEAN,
    source_service SYMBOL,
    execution_time_ms LONG,
    confirmation_time_ms LONG
) timestamp(timestamp) PARTITION BY DAY;

-- PostgreSQL audit log
CREATE TABLE trade_audit_log (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMP,
    event_type VARCHAR(50),
    request_id VARCHAR(100),
    source_service VARCHAR(50),
    request_data JSONB,
    result_data JSONB,
    created_at TIMESTAMP DEFAULT NOW()
);
```

## Deliverables

1. Complete live trade executor with all components
2. gRPC service implementation with protobuf definitions
3. Comprehensive test suite including integration tests
4. Performance benchmarks showing <100ms execution
5. Documentation for API usage and error handling

## Success Criteria

- Successfully executes trades on Solana mainnet
- Maintains <100ms latency to transaction broadcast
- Properly handles all error scenarios with retry logic
- Records all trades in QuestDB with complete metrics
- Integrates circuit breaker for system protection
- Provides detailed execution results via gRPC
- Achieves 95%+ test coverage for critical paths
- Handles 100+ trades per second under load