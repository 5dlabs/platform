# Task 19: Implement Database Trait Abstractions for Modular Data Access - Autonomous Prompt

You are implementing database trait abstractions for a Solana trading platform that uses multiple storage systems. Your goal is to create clean, type-safe interfaces that unify access to PostgreSQL, QuestDB, and Redis while enabling easy testing through mock implementations.

## Context

The trading platform uses three databases optimized for different purposes:
- **PostgreSQL**: Configuration, trader profiles, order rules, audit logs
- **QuestDB**: Time-series data for trades, positions, and metrics
- **Redis**: High-speed caching for prices and pool states (1-2 second TTL)

Your abstractions must provide consistent interfaces while allowing each implementation to leverage database-specific optimizations.

## Your Objectives

1. **Design Core Database Traits**
   - Create `TradeStore` trait for trade data persistence
   - Define `MetricsStore` trait for time-series metrics
   - Build `PriceCache` trait for high-speed price caching
   - Implement `ConfigStore` trait for configuration data
   - Ensure all traits are `Send + Sync + 'static`

2. **Implement PostgreSQL Adapters**
   - Use `sqlx` for type-safe queries
   - Implement connection pooling (20 connections max)
   - Add transaction support for atomic operations
   - Include migration support
   - Handle prepared statements efficiently

3. **Build QuestDB Implementations**
   - Optimize for time-series data patterns
   - Implement batch operations (100ms flush interval)
   - Use appropriate column types for performance
   - Support efficient time-range queries
   - Handle high-throughput ingestion

4. **Create Redis Implementations**
   - Implement TTL-based caching (1-2 seconds default)
   - Support batch operations for efficiency
   - Use appropriate serialization (JSON for complex types)
   - Implement cache statistics tracking
   - Handle connection pooling via ConnectionManager

5. **Develop Mock Implementations**
   - In-memory storage for all traits
   - Configurable failure modes for testing
   - Latency simulation capabilities
   - Expiry simulation for cache testing
   - Thread-safe implementations

## Implementation Requirements

### Trait Definitions
```rust
// All traits must be async and object-safe
#[async_trait]
pub trait TradeStore: Send + Sync + 'static {
    async fn save_trade(&self, trade: &Trade) -> Result<(), DbError>;
    async fn get_trades_by_token(&self, token: &Pubkey, limit: usize) -> Result<Vec<Trade>, DbError>;
    async fn get_trades_in_timerange(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Trade>, DbError>;
    async fn batch_save_trades(&self, trades: &[Trade]) -> Result<(), DbError>;
}
```

### Error Handling
```rust
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Query error: {0}")]
    QueryError(String),
    // ... other variants
}

impl DbError {
    pub fn is_transient(&self) -> bool;
    pub fn is_permanent(&self) -> bool;
}
```

### Implementation Structure
```
database/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── traits/
│   │   ├── mod.rs
│   │   ├── trade_store.rs
│   │   ├── metrics_store.rs
│   │   ├── price_cache.rs
│   │   └── config_store.rs
│   ├── postgres/
│   │   ├── mod.rs
│   │   ├── trade_store.rs
│   │   └── config_store.rs
│   ├── questdb/
│   │   ├── mod.rs
│   │   ├── trade_store.rs
│   │   └── metrics_store.rs
│   ├── redis/
│   │   ├── mod.rs
│   │   └── price_cache.rs
│   ├── mock/
│   │   └── mod.rs
│   ├── factory.rs
│   └── errors.rs
├── migrations/
│   └── postgres/
```

### Key Implementation Details

1. **Batch Operations**:
   - QuestDB: Buffer writes for 100ms or 1000 records
   - PostgreSQL: Use COPY for bulk inserts
   - Redis: Pipeline commands for efficiency

2. **Connection Management**:
   - Connection pooling for all databases
   - Health check endpoints
   - Graceful shutdown procedures
   - Automatic reconnection logic

3. **Performance Optimizations**:
   - Prepared statements for PostgreSQL
   - Columnar storage benefits in QuestDB
   - Redis pipelining and Lua scripts
   - Efficient serialization formats

4. **Testing Support**:
   ```rust
   // Easy mock creation
   let mock_store = MockTradeStore::new();
   let mock_store = MockTradeStore::with_failure();
   let mock_store = MockTradeStore::with_latency(50);
   ```

### Database Factory Pattern
```rust
pub struct DatabaseFactory;

impl DatabaseFactory {
    pub async fn create_trade_store(config: &DatabaseConfig) -> Result<Arc<dyn TradeStore>, DbError>;
    pub async fn create_metrics_store(config: &DatabaseConfig) -> Result<Arc<dyn MetricsStore>, DbError>;
    pub async fn create_price_cache(config: &DatabaseConfig) -> Result<Arc<dyn PriceCache>, DbError>;
    pub async fn create_config_store(config: &DatabaseConfig) -> Result<Arc<dyn ConfigStore>, DbError>;
}
```

### Testing Requirements

1. **Unit Tests**:
   - Mock implementations for all scenarios
   - Error handling verification
   - Retry logic testing
   - Cache expiry behavior

2. **Integration Tests**:
   - Docker-based test databases
   - Migration testing
   - Connection pool behavior
   - Cross-database consistency

3. **Performance Tests**:
   - Batch operation benchmarks
   - Concurrent access patterns
   - Memory usage tracking
   - Latency measurements

## Deliverables

1. Complete trait definitions for all database operations
2. PostgreSQL implementation with migrations
3. QuestDB implementation with batch optimization
4. Redis implementation with TTL support
5. Comprehensive mock implementations
6. Database factory for easy instantiation
7. Integration test suite with Docker
8. Performance benchmarks

## Success Criteria

- All trait implementations pass integration tests
- Mock implementations enable unit testing
- Batch operations achieve >10k records/second
- Connection pooling handles 100+ concurrent requests
- Error handling distinguishes transient vs permanent
- Factory pattern simplifies database selection
- Zero unsafe code or memory leaks
- Documentation includes usage examples