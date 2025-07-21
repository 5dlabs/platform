# Task 4: Develop Jupiter Failover Client - Autonomous Prompt

You are implementing a resilient Jupiter V6 API client with automatic failover capabilities for a Solana trading platform. The client must prioritize a self-hosted Jupiter instance for low latency while seamlessly falling back to the public API when needed.

## Context

The trading platform requires reliable access to Jupiter for trade routing and price quotes. Key requirements:
- Self-hosted instance provides <200ms response times
- Public instance (lite-api.jup.ag/v6) serves as backup
- Automatic failover on timeout or errors
- Background health monitoring for recovery
- Support for MEV protection parameters

## Your Objectives

1. **Implement Dual-Instance Client**
   - Create HTTP clients for both self-hosted and public endpoints
   - Configure 200ms timeout for self-hosted, 500ms for public
   - Use connection pooling for performance
   - Track health status with atomic boolean

2. **Build Failover Logic**
   - Always try self-hosted first if healthy
   - Failover to public on timeout or error
   - Mark self-hosted unhealthy on failure
   - Continue with public until recovery

3. **Add Health Monitoring**
   - Background task checking every 30 seconds
   - Only check when marked unhealthy
   - Test with health endpoint
   - Restore service when responsive

4. **Implement Quote and Swap APIs**
   - Support all MVP tokens (SOL, USDC, BONK, JitoSOL, RAY)
   - Include MEV protection parameters
   - Handle Token-2022 extension fees
   - Validate token pairs before requests

## Implementation Requirements

### API Endpoints
```
Quote: GET /quote
- inputMint: Token mint address
- outputMint: Token mint address  
- amount: Amount in smallest units
- slippageBps: Slippage in basis points

Swap: POST /swap
- userPublicKey: User's wallet address
- wrapAndUnwrapSol: true (MEV protection)
- useSharedAccounts: true (MEV protection)
- prioritizationFeeLamports: 1000-10000 (dynamic)
- quoteResponse: Quote object from /quote
```

### MEV Protection Integration
- Always set `wrapAndUnwrapSol: true`
- Always set `useSharedAccounts: true`
- Include priority fee (1000-10000 lamports)
- Use compute unit price for additional protection

### Token Registry
```rust
MVP Tokens:
- SOL: So11111111111111111111111111111111111111112
- USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
- BONK: DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263
- JitoSOL: J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn
- RAY: 4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R
```

### Error Handling
- Distinguish between transient and permanent errors
- Preserve error context for debugging
- Don't retry on client errors (4xx)
- Log failover events with details

## Testing Strategy

Create comprehensive tests for:

1. **Failover Scenarios**:
   - Self-hosted timeout triggers failover
   - Self-hosted error triggers failover
   - Public API continues working
   - Health restored after recovery

2. **MEV Protection**:
   - Verify all protection parameters included
   - Test priority fee within range
   - Confirm wrapped SOL settings

3. **Performance Tests**:
   - Measure latency difference
   - Verify 200ms timeout enforcement
   - Test connection pool efficiency

4. **Token Validation**:
   - Test all MVP token pairs
   - Reject invalid tokens
   - Handle same token errors

## Deliverables

1. **Core Implementation**:
   - `jupiter_client.rs` with failover logic
   - Request/response types with serde
   - Health monitoring task
   - Token registry with validation

2. **Integration Features**:
   - Metrics collection per endpoint
   - Circuit breaker integration
   - Configurable timeouts
   - Connection pool management

3. **Tests**:
   - Unit tests with mocked responses
   - Integration tests with timeouts
   - Performance benchmarks
   - Failover scenario tests

4. **Documentation**:
   - API usage examples
   - Configuration guide
   - Failover behavior explanation
   - Troubleshooting guide

## Success Criteria

- Self-hosted requests complete in <200ms
- Failover occurs within 250ms total
- Health recovery works automatically
- All MVP tokens supported
- MEV protection parameters included
- Zero data races in health status
- Comprehensive error handling
- All tests pass including integration tests