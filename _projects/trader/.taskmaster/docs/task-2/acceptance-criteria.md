# Task 2: Acceptance Criteria

## Functional Requirements

### 1. QuestDB Configuration
- [ ] **Tables Created**:
  - `trades` table with all fields from architecture.md
  - `positions` table for portfolio tracking
  - `metrics` table for performance data
  - `mev_events` table for MEV tracking
- [ ] **Partitioning Configured**:
  - Daily partitions for trades, metrics, mev_events
  - Hourly partitions for positions (frequent updates)
- [ ] **Batch Writer Implementation**:
  - Flushes every 100ms as per PRD
  - Buffers up to 1000 records
  - Handles connection failures gracefully
  - Clears buffer after successful flush
- [ ] **Retention Policy**:
  - 30-day retention at full resolution
  - Automatic partition cleanup

### 2. PostgreSQL Setup
- [ ] **Schema Created**:
  - `tokens` table with Token-2022 extension support (JSONB)
  - `trader_config` table with constraints
  - `order_rules` table with foreign keys
  - `node_health` table for monitoring
- [ ] **Indexes Configured**:
  - Index on tokens.symbol for fast lookups
  - Primary keys on all tables
  - Foreign key constraints enforced
- [ ] **Connection Pool**:
  - Max 20 connections configured
  - Min 5 connections maintained
  - 3-second connection timeout
  - 10-minute idle timeout
- [ ] **Triggers and Functions**:
  - Auto-update timestamp trigger
  - UUID generation for primary keys

### 3. Redis Configuration
- [ ] **Price Cache**:
  - Key format: `price:{token_symbol}`
  - TTL: 1-2 seconds (configurable)
  - Decimal precision maintained
  - Atomic get/set operations
- [ ] **Pool State Cache**:
  - Key format: `pool:{pool_address}`
  - JSON serialization of pool data
  - 2-second TTL
- [ ] **Event Streams**:
  - Stream keys: `stream:price-updates`, `stream:trade-events`, `stream:ui-events`
  - MAXLEN: 10,000 entries
  - Consumer group support
  - Block read with timeout

## Non-Functional Requirements

### Performance
- [ ] QuestDB batch writes complete in <100ms
- [ ] Redis cache reads average <1ms
- [ ] PostgreSQL queries complete in <50ms (indexed)
- [ ] Event stream publishing <10ms

### Reliability
- [ ] Connection retry logic implemented
- [ ] Circuit breaker integration for failures
- [ ] Graceful degradation on cache miss
- [ ] Transaction rollback on errors

### Scalability
- [ ] QuestDB handles 10,000 trades/second
- [ ] Redis supports 100,000 ops/second
- [ ] PostgreSQL connection pool scales to load
- [ ] Event streams handle 10Hz updates

## Test Cases

### QuestDB Tests
```rust
// Test 1: Batch write performance
Input: 1000 trade records
Expected: All records written within 100ms

// Test 2: Retention policy
Input: Data older than 30 days
Expected: Automatically removed by partition drop

// Test 3: Query performance
Input: Query last 1000 trades
Expected: Results returned in <100ms
```

### PostgreSQL Tests
```rust
// Test 1: Concurrent connections
Input: 50 simultaneous queries
Expected: All complete successfully with pool

// Test 2: Token upsert
Input: Insert/update token with extensions
Expected: JSONB data properly stored

// Test 3: Trigger execution
Input: Update trader_config
Expected: updated_at timestamp changes
```

### Redis Tests
```rust
// Test 1: Cache latency
Input: 1000 sequential price reads
Expected: Average latency <1ms

// Test 2: TTL expiration
Input: Set price with 2s TTL
Expected: Key expires after 2 seconds

// Test 3: Stream trimming
Input: Publish 15,000 events
Expected: Stream maintains 10,000 limit
```

## Integration Tests

### End-to-End Flow Test
```rust
// Test complete data flow
1. Write trade to QuestDB
2. Update position in PostgreSQL
3. Cache price in Redis
4. Publish event to stream
5. Verify all data consistency
```

### Failover Test
```rust
// Test database failures
1. Disconnect QuestDB
2. Verify batch writer buffers data
3. Reconnect QuestDB
4. Verify buffered data written
5. Check no data loss
```

### Performance Benchmark
```rust
// Test under load
1. Generate 10,000 trades/second
2. Update 1,000 positions/second
3. Cache 5,000 prices/second
4. Publish 100 events/second
5. Verify all SLAs met
```

## Definition of Done

- [ ] All database schemas deployed successfully
- [ ] Connection code compiles without warnings
- [ ] All unit tests pass
- [ ] Integration tests demonstrate data flow
- [ ] Performance benchmarks meet requirements
- [ ] Error handling covers all failure modes
- [ ] Monitoring queries provided
- [ ] Documentation includes:
  - Schema diagrams
  - Connection examples
  - Performance tuning guide
  - Troubleshooting steps
- [ ] Code reviewed for:
  - SQL injection prevention
  - Connection leak prevention
  - Proper async/await usage
  - Resource cleanup