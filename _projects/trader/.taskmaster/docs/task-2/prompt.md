# Task 2: Set Up Database Infrastructure - Autonomous Prompt

You are setting up the database infrastructure for a high-performance Solana Trading Platform. The system requires three complementary databases optimized for specific use cases: QuestDB for time-series data, PostgreSQL for configuration/metadata, and Redis for sub-millisecond caching and event streaming.

## Context

The platform processes high-frequency trading data from both paper and live trading modes. Performance requirements include:
- QuestDB: 100ms batch writes for time-series data
- PostgreSQL: Reliable storage for configuration and Token-2022 metadata
- Redis: <1ms cache reads and 10Hz event streaming

## Your Objectives

1. **Configure QuestDB for Time-Series Storage**
   - Create tables for trades, positions, metrics, and MEV events
   - Implement daily partitioning for efficient data management
   - Build a batch writer that flushes every 100ms
   - Set up 30-day retention policy

2. **Set Up PostgreSQL for Configuration**
   - Design schema for token metadata with Token-2022 support
   - Create tables for trader configuration and order rules
   - Implement node health tracking table
   - Configure connection pooling with 20 max connections

3. **Configure Redis for Caching and Streaming**
   - Implement price cache with 1-2 second TTL
   - Set up pool state caching for MEV calculations
   - Create event streams with 10,000 entry limit
   - Build pub/sub mechanism for 10Hz updates

## Implementation Requirements

### Directory Structure
```
database/
├── questdb/
│   ├── schema.sql
│   └── batch_writer.rs
├── postgres/
│   ├── migrations/
│   │   └── 001_initial_schema.sql
│   └── connection.rs
└── redis/
    ├── cache.rs
    └── streams.rs
```

### QuestDB Schema Requirements
- Trades table: timestamp, trader_id, mode, action, tokens, amounts, fees, MEV status
- Positions table: current holdings with P&L calculations
- Metrics table: system performance data with JSON labels
- MEV events table: track sandwich attack probabilities and outcomes

### PostgreSQL Schema Requirements
- Tokens table: address, symbol, decimals, Token-2022 extensions (JSONB)
- Trader config: settings, limits, MEV protection preferences
- Order rules: stop-loss and take-profit triggers
- Node health: latency tracking and circuit breaker status

### Redis Key Structure
```
price:{token_symbol} -> decimal price (TTL: 2s)
pool:{pool_address} -> JSON pool state (TTL: 2s)
stream:price-updates -> price update events
stream:trade-events -> trade execution events
stream:ui-events -> UI update events
```

### Performance Requirements
- Batch writes to QuestDB must complete within 100ms
- Redis reads must average <1ms latency
- PostgreSQL connection pool must handle 50 concurrent queries
- Event streams must support 10Hz update frequency

## Testing Strategy

Create comprehensive tests for:
1. **Load Testing**: Verify databases handle expected throughput
2. **Latency Testing**: Confirm <1ms Redis reads, 100ms QuestDB writes
3. **Failover Testing**: Test connection pool behavior under failure
4. **Data Integrity**: Verify proper schema constraints and data types
5. **Retention Testing**: Confirm 30-day retention and proper partitioning

## Deliverables

1. **Schema Files**:
   - QuestDB table definitions with partitioning
   - PostgreSQL migrations with indexes
   - Redis key documentation

2. **Connection Code**:
   - QuestDB batch writer with 100ms intervals
   - PostgreSQL pool with error handling
   - Redis cache with TTL management

3. **Event Streaming**:
   - Redis Streams producer/consumer
   - 10Hz update capability
   - 10,000 entry limit enforcement

4. **Integration Tests**:
   - Performance benchmarks
   - Concurrent access tests
   - Failure recovery scenarios

## Success Criteria

- All schemas deployed without errors
- Batch writer achieves 100ms flush interval
- Redis cache delivers <1ms read latency
- PostgreSQL handles 50 concurrent connections
- Event streams maintain 10Hz update rate
- All tests pass with expected performance
- Proper error handling and logging implemented
- Documentation complete for all schemas and APIs