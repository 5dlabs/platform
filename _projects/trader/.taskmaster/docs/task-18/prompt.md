# Task 18: Implement Risk Management System with Pre-trade Validation - Autonomous Prompt

You are implementing a focused risk management system for a Solana trading platform that validates technical execution aspects of trades. The system acts as a safety layer between external trade requests and the execution engine, ensuring trades can be executed safely from a technical perspective.

## Context

The trading platform receives pre-approved trade requests from external strategy services. Your risk management system focuses purely on technical validation: checking execution parameters, system health, node latency, and circuit breaker status. Business logic and portfolio risk are handled externally.

## Your Objectives

1. **Create Core Risk Management Components**
   - Build `RiskManager` struct that centralizes all risk checks
   - Implement configurable risk thresholds (slippage, latency, etc.)
   - Design persistent configuration system
   - Create validation pipeline for pre-trade checks
   - Calculate technical risk scores for monitoring

2. **Implement Execution Parameter Validation**
   - Validate maximum slippage tolerance
   - Check MEV protection requirements
   - Verify priority fee ranges (1000-10000 lamports)
   - Validate execution timeout parameters
   - Generate warnings for borderline values

3. **Build System Health Monitoring**
   - Monitor Solana node P99 latency (200ms threshold)
   - Track error rates across services
   - Calculate system health scores
   - Check critical service availability (Jupiter, RPC)
   - Integrate with existing health metrics

4. **Integrate Circuit Breaker System**
   - Connect to circuit breakers from common libraries
   - Register technical failures (timeouts, errors)
   - Implement automatic trading halts
   - Support manual override capabilities
   - Track circuit breaker state changes

5. **Create Risk Metrics and Monitoring**
   - Record validation times and outcomes
   - Track risk scores over time
   - Count and categorize violations
   - Export Prometheus metrics
   - Build dashboard data endpoints

## Implementation Requirements

### Code Structure
```
risk_management/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── manager.rs       # Core risk manager
│   ├── validation.rs    # Parameter validation
│   ├── health.rs        # System health checks
│   ├── circuit.rs       # Circuit breaker integration
│   ├── metrics.rs       # Metrics collection
│   ├── override.rs      # Override management
│   └── config.rs        # Configuration types
```

### Risk Configuration Schema
```rust
pub struct RiskConfig {
    // Execution limits
    max_slippage_bps: u16,           // Default: 500 (5%)
    min_confirmation_time_ms: u64,    // Default: 100ms
    max_confirmation_time_ms: u64,    // Default: 30000ms
    
    // System health
    max_node_latency_ms: u64,        // Default: 200ms (PRD)
    min_node_health_score: f64,      // Default: 0.8
    max_error_rate: f64,             // Default: 0.05
    
    // MEV protection
    mev_protection_required: bool,    // Default: true
    min_priority_fee: u64,           // Default: 1000
    max_priority_fee: u64,           // Default: 10000
    
    // Circuit breaker
    max_consecutive_failures: u32,    // Default: 5
    failure_window_secs: u64,        // Default: 60
    recovery_timeout_secs: u64,      // Default: 300
}
```

### Validation Flow
```
1. Check circuit breaker status
2. Validate execution parameters
3. Check system health metrics
4. Verify MEV protection
5. Calculate risk score
6. Apply any active overrides
7. Return validation result
```

### Key Implementation Details

1. **Risk Score Calculation**:
   - Base score: 1.0 (perfect)
   - Deduct for high slippage tolerance
   - Deduct for urgency level
   - Multiply by system health score
   - Range: 0.0 to 1.0

2. **Health Monitoring**:
   - Query node metrics every 10 seconds
   - Calculate rolling error rates
   - Monitor service connectivity
   - Track P99 latencies

3. **Override System**:
   - Temporary overrides with expiration
   - Requires authentication
   - Maximum 24-hour duration
   - Full audit trail

### Testing Requirements

1. **Unit Tests**:
   - Parameter validation edge cases
   - Risk score calculations
   - Circuit breaker triggers
   - Override creation and expiry

2. **Integration Tests**:
   - Full validation pipeline
   - System health degradation
   - Circuit breaker integration
   - Metric collection

3. **Performance Tests**:
   - <1ms validation P99
   - Concurrent validation handling
   - Minimal memory allocation

### Integration Points

```rust
// Integration with trade executor
pub trait RiskValidator: Send + Sync {
    async fn validate_trade_execution(
        &self,
        request: &TradeRequest,
    ) -> Result<ValidationResult, RiskError>;
}

// Integration with monitoring
pub trait RiskMetrics: Send + Sync {
    async fn get_risk_dashboard(&self) -> Result<RiskDashboard, Error>;
    async fn export_prometheus_metrics(&self) -> String;
}
```

### Mock Implementation

Provide a mock risk manager for testing:
```rust
pub struct MockRiskManager;

impl MockRiskManager {
    pub fn always_approve() -> Self { /* ... */ }
    pub fn always_reject() -> Self { /* ... */ }
    pub fn with_warnings() -> Self { /* ... */ }
}
```

## Deliverables

1. Complete risk management system with all validators
2. Integration with circuit breaker system
3. Prometheus metrics export functionality
4. Override management with audit logging
5. Mock implementation for testing
6. Comprehensive test suite
7. Performance benchmarks

## Success Criteria

- Validates all technical execution parameters
- Integrates with system circuit breakers
- Maintains <1ms validation latency P99
- Provides detailed risk scoring
- Supports temporary overrides with audit trail
- Exports comprehensive metrics
- Handles 1000+ validations per second
- Zero false negatives for critical risks
- Mock implementation enables easy testing