# Task 6: Implement Circuit Breaker for Latency-Based Trading Pause - Autonomous Prompt

You are implementing a sophisticated circuit breaker system that automatically pauses trading when node latency exceeds 200ms P99. This is critical for protecting the trading platform from executing trades under degraded performance conditions.

## Context

The trading platform requires automatic protection against high latency scenarios that could lead to poor trade execution. The circuit breaker must:
- Monitor P99 latency in real-time with rolling window
- Trigger open state when P99 > 200ms (PRD requirement)
- Implement three states: Closed (normal), Open (blocked), Half-Open (testing)
- Use exponential backoff for recovery attempts
- Integrate seamlessly with trading operations

## Your Objectives

1. **Implement State Machine**
   - Thread-safe state transitions
   - Closed → Open on failure threshold
   - Open → Half-Open after timeout
   - Half-Open → Closed on success
   - Half-Open → Open on failure
   - State change notifications

2. **Build Latency Monitor**
   - Rolling window of 1000 samples
   - Real-time P99 calculation
   - Trigger circuit on threshold breach
   - Efficient percentile algorithm
   - Thread-safe updates

3. **Create Recovery System**
   - Exponential backoff delays
   - Health check integration
   - Configurable retry policy
   - Jitter to prevent thundering herd
   - Automatic recovery attempts

4. **Trading Integration**
   - Wrap operations with breaker
   - Record operation latencies
   - Block when circuit open
   - Minimal performance overhead

## Implementation Requirements

### Configuration
```rust
CircuitBreakerConfig {
    failure_threshold: 5,           // Failures to trigger open
    recovery_timeout: 30s,          // Initial recovery delay
    half_open_requests: 3,          // Test requests allowed
    latency_threshold_ms: 200,      // P99 threshold (PRD)
    error_rate_threshold: 0.5,      // 50% error rate
}
```

### State Transitions
- **Closed → Open**: 5 consecutive failures OR P99 > 200ms
- **Open → Half-Open**: After recovery timeout expires
- **Half-Open → Closed**: Single successful request
- **Half-Open → Open**: Any failure in half-open state

### Monitoring Window
- Keep last 1000 latency samples
- Update P99 on each new sample
- Efficient insertion and calculation
- Memory-bounded structure

### Recovery Policy
```rust
RecoveryPolicy {
    base_delay: 1s,
    max_delay: 5m,
    multiplier: 2.0,
    jitter_factor: 0.1,
}
```

## Testing Strategy

Create comprehensive tests for:

1. **State Machine Tests**:
   - All state transitions
   - Concurrent state changes
   - State persistence
   - Listener notifications

2. **Latency Monitoring**:
   - P99 calculation accuracy
   - Window size limits
   - Threshold detection
   - Performance under load

3. **Recovery Tests**:
   - Exponential backoff calculation
   - Jitter application
   - Max delay capping
   - Health check integration

4. **Integration Tests**:
   - Trading operation wrapping
   - Latency-based triggering
   - Circuit breaker metrics
   - End-to-end scenarios

## Deliverables

1. **Core Components**:
   - `circuit_breaker.rs` with state machine
   - `latency_monitor.rs` with P99 tracking
   - `recovery_manager.rs` with backoff
   - Error types and metrics

2. **Integration Layer**:
   - `TradingCircuitBreaker` wrapper
   - Health checker trait
   - State change listeners
   - Metrics collection

3. **Tests**:
   - Unit tests for all components
   - Integration tests with mocks
   - Performance benchmarks
   - Concurrent operation tests

4. **Documentation**:
   - State diagram
   - Configuration guide
   - Integration examples
   - Monitoring setup

## Success Criteria

- Circuit opens when P99 > 200ms
- State transitions are atomic and correct
- Recovery uses exponential backoff
- <100μs overhead on normal operations
- No false positives under normal load
- Proper logging of state changes
- Metrics exported for monitoring
- All tests pass including concurrent scenarios