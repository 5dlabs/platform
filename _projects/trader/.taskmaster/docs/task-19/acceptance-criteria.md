# Task 19: Acceptance Criteria

## Functional Requirements

### 1. Core Database Traits
- [ ] **TradeStore Trait**:
  - `save_trade()` persists single trade
  - `get_trades_by_token()` returns trades for specific token
  - `get_trades_in_timerange()` queries by time window
  - `batch_save_trades()` handles bulk inserts
  - `get_trade_by_request_id()` for correlation
  - All methods return `Result<T, DbError>`
- [ ] **MetricsStore Trait**:
  - `record_latency()` stores operation timings
  - `record_slippage()` tracks slippage data
  - `record_mev_event()` logs MEV occurrences
  - `get_p99_latency()` calculates percentiles
  - `get_metrics_summary()` returns aggregates
- [ ] **PriceCache Trait**:
  - `cache_price()` stores with TTL
  - `get_price()` returns if not expired
  - `cache_pool_state()` for complex data
  - `get_prices_batch()` for efficiency
  - `invalidate_token()` removes entries
- [ ] **ConfigStore Trait**:
  - `save_trader_config()` with upsert
  - `get_trader_config()` by ID
  - `list_trader_configs()` returns all
  - `save_order_rule()` for stop/take profit
  - `trigger_order_rule()` marks as executed

### 2. PostgreSQL Implementation
- [ ] **Connection Management**:
  - Uses `sqlx` with compile-time query checking
  - Connection pool with 20 max connections
  - 30-second connection timeout
  - 10-minute idle timeout
  - Automatic reconnection on failure
- [ ] **Features**:
  - Prepared statement caching
  - Transaction support with rollback
  - Migration system via sqlx::migrate!
  - Proper NULL handling
  - JSON/JSONB for extension data
- [ ] **Error Handling**:
  - Maps sqlx errors to DbError types
  - Distinguishes constraint violations
  - Handles connection pool exhaustion
  - Provides meaningful error context

### 3. QuestDB Implementation
- [ ] **Time-Series Optimization**:
  - Appropriate column types (SYMBOL, TIMESTAMP)
  - Partition by day for trades table
  - Efficient time-range queries
  - Optimized for append-only workload
- [ ] **Batch Operations**:
  - Write buffer with 1000 record capacity
  - 100ms flush interval
  - Background flush task
  - Force flush on buffer full
  - Handles partial batch failures
- [ ] **Query Performance**:
  - Uses native QuestDB SQL extensions
  - Efficient aggregation queries
  - Proper timestamp handling
  - Index usage for common patterns

### 4. Redis Implementation
- [ ] **Caching Strategy**:
  - Default 2-second TTL for prices
  - Per-key TTL override support
  - Automatic expiry handling
  - Memory-efficient storage
- [ ] **Connection Management**:
  - ConnectionManager for auto-reconnect
  - Pipeline support for batch ops
  - Proper error recovery
  - Connection pool via redis-rs
- [ ] **Data Serialization**:
  - Primitive types as strings
  - Complex types as JSON
  - Efficient batch operations
  - Consistent key naming scheme

### 5. Mock Implementations
- [ ] **MockTradeStore**:
  - In-memory Vec storage
  - Thread-safe with Arc<Mutex>
  - Configurable failure modes
  - Latency simulation
  - Preserves insertion order
- [ ] **MockPriceCache**:
  - Simulates TTL expiry
  - Preset price injection
  - Cache hit/miss tracking
  - Thread-safe operations
- [ ] **Testing Features**:
  - `with_failure()` constructor
  - `with_latency(ms)` constructor
  - `with_preset_data()` options
  - Verification helpers

### 6. Error Handling
- [ ] **Error Types**:
  - Comprehensive DbError enum
  - Transient vs permanent classification
  - Detailed error messages
  - Source error preservation
- [ ] **Retry Logic**:
  - Helper for transient errors
  - Exponential backoff
  - Max retry configuration
  - Skip permanent errors

### 7. Connection Management
- [ ] **Health Checks**:
  - Individual database checks
  - Combined health status
  - Timeout on health queries
  - Non-blocking checks
- [ ] **Resource Management**:
  - Graceful shutdown support
  - Connection pool draining
  - Pending operation completion
  - Resource leak prevention

### 8. Database Factory
- [ ] **Factory Methods**:
  - Creates appropriate implementation
  - Based on configuration enum
  - Returns trait objects (Arc<dyn>)
  - Consistent error handling
- [ ] **Configuration**:
  - Supports all database types
  - Connection string validation
  - Pool size configuration
  - Feature flag support

## Non-Functional Requirements

### Performance
- [ ] Batch inserts >10,000 records/second
- [ ] Single record operations <10ms P99
- [ ] Cache lookups <1ms P99
- [ ] Connection pool supports 100+ concurrent requests
- [ ] Minimal memory allocations

### Reliability
- [ ] No panics in database code
- [ ] Graceful degradation on failures
- [ ] Automatic reconnection logic
- [ ] Transaction rollback on errors
- [ ] Data consistency guarantees

### Maintainability
- [ ] Clear trait boundaries
- [ ] Consistent error handling
- [ ] Comprehensive documentation
- [ ] Example usage in tests
- [ ] Migration versioning

## Test Cases

### Trait Implementation Tests
```rust
// Test 1: Save and retrieve trade
let store = create_test_store();
let trade = create_test_trade();
store.save_trade(&trade).await?;
let retrieved = store.get_trade_by_request_id(&trade.request_id).await?;
assert_eq!(retrieved.unwrap().id, trade.id);

// Test 2: Time range query
let trades = store.get_trades_in_timerange(start, end).await?;
assert!(trades.iter().all(|t| t.timestamp >= start && t.timestamp <= end));

// Test 3: Batch operations
let trades = create_test_trades(1000);
store.batch_save_trades(&trades).await?;
// Verify all saved
```

### Mock Implementation Tests
```rust
// Test 1: Failure simulation
let store = MockTradeStore::with_failure();
let result = store.save_trade(&trade).await;
assert!(matches!(result, Err(DbError::WriteError(_))));

// Test 2: Latency simulation
let store = MockTradeStore::with_latency(50);
let start = Instant::now();
store.save_trade(&trade).await?;
assert!(start.elapsed() >= Duration::from_millis(50));

// Test 3: Cache expiry
let cache = MockPriceCache::new();
cache.cache_price(&token, 100.0, Duration::from_millis(100)).await?;
sleep(Duration::from_millis(150)).await;
assert_eq!(cache.get_price(&token).await?, None);
```

### Integration Tests
```rust
// Test 1: PostgreSQL transactions
let store = create_postgres_store().await;
store.with_transaction(|tx| async {
    tx.save_trade(&trade1).await?;
    tx.save_trade(&trade2).await?;
    Err(DbError::QueryError("rollback test"))
}).await;
// Verify neither trade was saved

// Test 2: QuestDB batch performance
let store = create_questdb_store().await;
let trades = create_test_trades(10_000);
let start = Instant::now();
store.batch_save_trades(&trades).await?;
assert!(start.elapsed() < Duration::from_secs(1));

// Test 3: Redis pipeline
let cache = create_redis_cache().await;
let tokens = create_test_tokens(100);
for (token, price) in tokens {
    cache.cache_price(&token, price, Duration::from_secs(60)).await?;
}
let prices = cache.get_prices_batch(&token_list).await?;
assert_eq!(prices.len(), 100);
```

### Error Handling Tests
```rust
// Test 1: Transient error retry
let mut attempts = 0;
retry_db_operation(|| {
    attempts += 1;
    if attempts < 3 {
        Err(DbError::ConnectionError("timeout"))
    } else {
        Ok("success")
    }
}, 5).await?;
assert_eq!(attempts, 3);

// Test 2: Permanent error no retry
let result = retry_db_operation(|| {
    Err(DbError::QueryError("syntax error"))
}, 5).await;
assert!(result.is_err());

// Test 3: Connection pool exhaustion
// Spawn many concurrent operations
let handles = (0..200).map(|_| {
    spawn(async { store.save_trade(&trade).await })
});
// Some should get PoolExhausted error
```

### Factory Tests
```rust
// Test 1: Create each store type
let config = DatabaseConfig { /* ... */ };
let trade_store = DatabaseFactory::create_trade_store(&config).await?;
let metrics_store = DatabaseFactory::create_metrics_store(&config).await?;
let price_cache = DatabaseFactory::create_price_cache(&config).await?;
let config_store = DatabaseFactory::create_config_store(&config).await?;

// Test 2: Mock configuration
let config = DatabaseConfig {
    trade_store_type: StoreType::Mock,
    // ...
};
let store = DatabaseFactory::create_trade_store(&config).await?;
// Verify mock behavior
```

## Definition of Done

- [ ] All trait definitions complete and documented
- [ ] PostgreSQL implementation with migrations
- [ ] QuestDB implementation with batch optimization
- [ ] Redis implementation with TTL support
- [ ] Mock implementations for all traits
- [ ] Database factory pattern implemented
- [ ] Integration tests pass with real databases
- [ ] Performance benchmarks meet targets
- [ ] No memory leaks or unsafe code
- [ ] CI/CD integration with test databases
- [ ] Documentation includes usage examples
- [ ] Error handling covers all edge cases