# Task 1: Implement Common Libraries for Trade Models and MEV Structures - Autonomous Prompt

You are implementing the foundation layer of a Solana Trading Platform. Your goal is to create shared Rust crates that define common data structures and functionality for both paper and live trading modes.

## Context

The platform requires resilient trading infrastructure with MEV protection, circuit breakers for latency management, and failover capabilities between self-hosted and public Jupiter instances. All components must handle the 200ms P99 latency target specified in the PRD.

## Your Objectives

1. **Create the Trading Models Crate** (`common/models/`)
   - Implement the Enhanced Trade Model with all fields from the PRD
   - Create the MEV Risk Model with sandwich attack probability calculations (15-20% base rate for memecoins)
   - Build the Circuit Breaker Model with 200ms latency threshold
   - Ensure all models are serializable with serde

2. **Develop the Solana Integration Crate** (`common/solana/`)
   - Create a SolanaClient wrapper with gRPC support
   - Implement retry logic with exponential backoff
   - Add circuit breaker integration for latency monitoring
   - Build dynamic priority fee calculation (1000-10000 lamports range)

3. **Build the Jupiter Client with Failover** (`common/jupiter/`)
   - Create a dual-mode client supporting self-hosted and public endpoints
   - Implement 200ms timeout before failover
   - Add MEV protection parameters to swap requests
   - Include circuit breaker for service health tracking

## Implementation Requirements

### Code Structure
```
common/
├── models/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── trade.rs
│   │   ├── mev.rs
│   │   └── circuit_breaker.rs
├── solana/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
└── jupiter/
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

### Key Implementation Details

1. **MEV Risk Calculation**:
   - Base risk: 5% for small trades
   - Scaled risk: 15-50% based on trade impact (trade_size / pool_liquidity)
   - Priority fee scaling: 1000-10000 lamports based on risk level

2. **Circuit Breaker States**:
   - Open: Normal operation
   - Half-Open: Testing recovery after failure
   - Closed: Blocking operations due to failures

3. **Failover Logic**:
   - Primary: Self-hosted Jupiter (200ms timeout)
   - Fallback: Public Jupiter API
   - Track failures and latency for circuit breaker

### Testing Requirements

Write comprehensive tests for:
- MEV risk calculations with various trade sizes
- Circuit breaker state transitions
- Failover behavior between Jupiter instances
- Model serialization/deserialization
- Retry logic with simulated failures

### Performance Targets

- Circuit breaker triggers at >200ms latency
- Jupiter failover within 250ms total
- Priority fees calculated dynamically
- All operations must be async

## Deliverables

1. Three Rust crates with complete implementations
2. Unit tests with >80% coverage
3. Integration tests for failover scenarios
4. Documentation comments for all public APIs
5. Cargo.toml files with proper dependencies

## Success Criteria

- Models compile without warnings
- All tests pass
- Circuit breaker correctly manages latency
- Failover works seamlessly
- MEV calculations match PRD specifications (15-20% base rate)
- Code is idiomatic Rust with proper error handling