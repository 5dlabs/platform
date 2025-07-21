# Task 4: Acceptance Criteria

## Functional Requirements

### 1. Dual-Instance Client Setup
- [ ] **HTTP Clients Configured**:
  - Self-hosted client with 200ms timeout
  - Public client with 500ms timeout
  - Connection pooling enabled
  - Keep-alive configured
- [ ] **URL Configuration**:
  - Self-hosted URL configurable
  - Public URL: lite-api.jup.ag/v6
  - Health endpoint: /health
  - API version: V6
- [ ] **Health Tracking**:
  - Atomic boolean for health status
  - Initially marked healthy
  - Thread-safe status updates
  - Health status accessible

### 2. Failover Logic
- [ ] **Primary Path**:
  - Always tries self-hosted first when healthy
  - Respects 200ms timeout strictly
  - Returns result on success
  - Updates metrics on success
- [ ] **Failover Path**:
  - Triggers on timeout or error
  - Marks self-hosted unhealthy
  - Attempts public API request
  - Logs failover event
- [ ] **Error Handling**:
  - Distinguishes timeout vs error
  - Preserves error context
  - No retry on 4xx errors
  - Graceful degradation

### 3. Health Monitoring
- [ ] **Background Task**:
  - Runs every 30 seconds
  - Only checks when unhealthy
  - Uses /health endpoint
  - Non-blocking operation
- [ ] **Recovery Logic**:
  - Marks healthy on success
  - Resets circuit breaker
  - Logs recovery event
  - Continues monitoring
- [ ] **Resource Management**:
  - Task spawned on creation
  - Proper shutdown handling
  - No resource leaks
  - Minimal overhead

### 4. Quote API Implementation
- [ ] **Request Parameters**:
  - inputMint: Token address
  - outputMint: Token address
  - amount: String format
  - slippageBps: u16 (basis points)
  - onlyDirectRoutes: boolean
  - asLegacyTransaction: boolean
- [ ] **Response Handling**:
  - Parses Quote object
  - Validates response fields
  - Handles missing data
  - Returns typed result
- [ ] **Token Support**:
  - All MVP tokens work
  - Token validation included
  - Mint addresses correct
  - Decimal handling proper

### 5. Swap API Implementation
- [ ] **MEV Protection**:
  - wrapAndUnwrapSol: always true
  - useSharedAccounts: always true
  - prioritizationFeeLamports: 1000-10000
  - computeUnitPriceMicroLamports: calculated
- [ ] **Request Building**:
  - Includes quote response
  - User public key required
  - All optional fields handled
  - Proper serialization
- [ ] **Response Processing**:
  - Base64 transaction extracted
  - Block height captured
  - Priority fee confirmed
  - Error details preserved

## Non-Functional Requirements

### Performance
- [ ] Self-hosted latency <200ms (P99)
- [ ] Failover completes in <250ms total
- [ ] Health check overhead <1% CPU
- [ ] Memory usage stable over time

### Reliability
- [ ] No panic on network errors
- [ ] Graceful timeout handling
- [ ] Failover rate <5% in normal conditions
- [ ] Recovery time <1 minute

### Observability
- [ ] Endpoint latency metrics
- [ ] Failover count tracking
- [ ] Health status changes logged
- [ ] Error details captured

## Test Cases

### Failover Tests
```rust
// Test 1: Self-hosted timeout
Input: Self-hosted delays 300ms
Expected: Failover to public, request succeeds

// Test 2: Self-hosted error
Input: Self-hosted returns 500
Expected: Failover to public, health marked false

// Test 3: Both endpoints fail
Input: Both return errors
Expected: Error returned with context
```

### Health Recovery Tests
```rust
// Test 1: Recovery detection
Input: Health endpoint returns 200
Expected: Service marked healthy within 35s

// Test 2: Persistent failure
Input: Health endpoint keeps failing
Expected: Remains unhealthy, checks continue

// Test 3: Flapping prevention
Input: Alternating success/failure
Expected: Stable state transitions
```

### MEV Protection Tests
```rust
// Test 1: Parameter inclusion
Input: Swap request with quote
Expected: All MEV parameters present

// Test 2: Priority fee range
Input: Various fee levels
Expected: Fees between 1000-10000 lamports

// Test 3: Compute units
Input: Priority fee 5000
Expected: Compute units = 5000000 microLamports
```

### Token Validation Tests
```rust
// Test 1: Valid pairs
Input: SOL -> USDC
Expected: Request succeeds

// Test 2: Invalid token
Input: SOL -> INVALID
Expected: Validation error

// Test 3: Same token
Input: SOL -> SOL
Expected: Validation error
```

## Integration Tests

### Load Test
```rust
// 100 concurrent quote requests
- 95% use self-hosted
- <5% failover rate
- All complete within timeout
- Memory stable
```

### Failover Simulation
```rust
// Kill self-hosted mid-test
- Existing requests failover
- New requests use public
- No request failures
- Recovery when restored
```

### Network Conditions
```rust
// Variable latency test
- 50-150ms: Use self-hosted
- 200-300ms: Failover occurs
- Metrics accurately reflect
- No dropped requests
```

## Definition of Done

- [ ] All unit tests pass
- [ ] Integration tests demonstrate failover
- [ ] Performance benchmarks meet targets
- [ ] No compiler warnings
- [ ] Documentation includes:
  - Configuration examples
  - Failover explanation
  - Token list reference
  - MEV parameter guide
- [ ] Code reviewed for:
  - Thread safety
  - Error handling
  - Resource management
  - API compatibility
- [ ] Metrics dashboard shows:
  - Endpoint latencies
  - Failover rates
  - Health status
  - Error counts