# Task 1: Acceptance Criteria

## Functional Requirements

### 1. Trading Models Crate
- [ ] **Trade Model** includes all fields specified in architecture.md:
  - `id`, `timestamp`, `action`, `base_token`, `quote_token`
  - `amount`, `price`, `fee`, `slippage`
  - `priority_fee` (Option<u64>)
  - `tx_signature` (Option<String>)
  - `transfer_fee` for Token-2022 support
  - `mev_protected` boolean flag
  - `latency_ms` tracking
- [ ] **Trade Model Methods**:
  - `calculate_total_cost()` returns accurate total including fees
  - `calculate_slippage_bps()` returns basis points calculation
- [ ] **MEV Risk Model** calculates:
  - Sandwich probability between 0.05 and 0.50
  - Base 15-20% risk for memecoins as per PRD
  - Dynamic priority fee recommendation (1000-10000 lamports)
- [ ] **Circuit Breaker Model**:
  - Three states: Open, HalfOpen, Closed
  - Triggers at 200ms latency threshold (P99 target)
  - Recovery timeout of 30 seconds
  - Failure threshold of 5 consecutive failures

### 2. Solana Integration Crate
- [ ] **SolanaClient** wrapper provides:
  - gRPC connection support
  - Health monitoring integration
  - Latency tracking for all operations
- [ ] **Retry Logic**:
  - Exponential backoff starting at 100ms
  - Maximum retry configurable
  - Circuit breaker integration
- [ ] **Priority Fee Calculation**:
  - Returns fees in 1000-10000 lamports range
  - Based on recent network fees
  - Defaults to 1000 lamports if no data

### 3. Jupiter Client Crate
- [ ] **Dual-Mode Support**:
  - Self-hosted endpoint as primary
  - Public endpoint as fallback
  - Seamless failover on timeout/error
- [ ] **Failover Behavior**:
  - 200ms timeout on self-hosted before failover
  - Circuit breaker tracking for service health
  - Total operation completes within 250ms
- [ ] **MEV Protection Parameters**:
  - `wrap_and_unwrap_sol` flag
  - `use_shared_accounts` flag
  - `priority_fee_lamports` field

## Non-Functional Requirements

### Performance
- [ ] All async operations use tokio runtime
- [ ] Circuit breaker checks complete in <1ms
- [ ] Model serialization/deserialization <10ms
- [ ] No blocking I/O in any public API

### Code Quality
- [ ] Zero compiler warnings
- [ ] All public APIs have documentation comments
- [ ] Error types implement std::error::Error
- [ ] No use of `unwrap()` except in tests

### Testing
- [ ] Unit test coverage >80%
- [ ] Integration tests for failover scenarios
- [ ] Property-based tests for MEV calculations
- [ ] Mock tests for external service calls

## Test Cases

### MEV Risk Calculation Tests
```rust
// Test 1: Low risk scenario
Input: trade_size=100, pool_liquidity=100000
Expected: sandwich_probability < 0.1, priority_fee = 1000

// Test 2: Medium risk scenario  
Input: trade_size=500, pool_liquidity=10000
Expected: sandwich_probability ~0.20, priority_fee > 2000

// Test 3: High risk scenario
Input: trade_size=1000, pool_liquidity=5000
Expected: sandwich_probability > 0.35, priority_fee > 5000
```

### Circuit Breaker Tests
```rust
// Test 1: Normal operation
Input: 5 requests with 150ms latency
Expected: Circuit remains Open

// Test 2: Threshold breach
Input: 5 requests with 250ms latency
Expected: Circuit transitions to Closed

// Test 3: Recovery
Input: Wait 30 seconds after Closed
Expected: Circuit transitions to HalfOpen
```

### Failover Tests
```rust
// Test 1: Successful self-hosted
Input: Self-hosted responds in 150ms
Expected: Use self-hosted response

// Test 2: Self-hosted timeout
Input: Self-hosted times out after 200ms
Expected: Failover to public endpoint

// Test 3: Both fail
Input: Both endpoints fail
Expected: Return error with details
```

## Definition of Done

- [ ] All three crates compile without errors or warnings
- [ ] All acceptance criteria marked as complete
- [ ] All tests pass with `cargo test`
- [ ] Documentation generated with `cargo doc`
- [ ] Code reviewed for idiomatic Rust patterns
- [ ] Integration points verified with mock implementations
- [ ] Performance benchmarks meet targets
- [ ] Security considerations addressed (no sensitive data in logs)