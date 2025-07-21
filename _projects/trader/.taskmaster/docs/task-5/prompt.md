# Task 5: Implement Redis Price Caching and Event Streaming - Autonomous Prompt

You are implementing a high-performance Redis integration for a Solana trading platform. The system must provide sub-millisecond price caching and support real-time event streaming at 10Hz for UI updates.

## Context

The trading platform requires ultra-low latency access to current prices and real-time event propagation. Key requirements:
- Price cache reads must complete in <1ms
- Cache TTL of 1-2 seconds for data freshness
- Event streams support 10Hz updates (100ms intervals)
- Automatic stream trimming at 10,000 entries
- Support for all MVP tokens (SOL, USDC, BONK, JitoSOL, RAY)

## Your Objectives

1. **Implement Connection Pool Management**
   - Create Redis connection pool with bb8
   - Configure 20 connections for concurrency
   - Add connection health checking
   - Implement proper error handling

2. **Build Price Cache System**
   - Key format: `price:{token}`
   - Store price data with timestamp
   - 2-second TTL on all entries
   - Batch operations for efficiency
   - Monitor latency to ensure <1ms

3. **Create Event Streaming**
   - Use Redis Streams (XADD/XREAD)
   - Automatic trimming with MAXLEN ~10000
   - Support multiple event types
   - Consumer group functionality
   - 10Hz update capability

4. **Develop Pool State Cache**
   - Support MEV risk calculations
   - Key format: `pool:{address}`
   - Store liquidity and reserves
   - Same 2-second TTL as prices

## Implementation Requirements

### Data Structures
```rust
PriceData {
    token: String,
    price: Decimal,
    timestamp: DateTime<Utc>,
    source: String,
}

PoolState {
    pool_address: String,
    liquidity: f64,
    reserve_a: f64,
    reserve_b: f64,
    volume_24h: f64,
    timestamp: DateTime<Utc>,
}

TradingEvent {
    PriceUpdate { token, price, timestamp },
    TradeExecuted { id, action, token_pair, amount, price, timestamp },
    PositionUpdate { token, amount, pnl, timestamp },
    SystemStatus { component, status, message, timestamp },
}
```

### Redis Key Schema
```
Prices: price:{token} -> JSON PriceData (TTL: 2s)
Pools: pool:{address} -> JSON PoolState (TTL: 2s)
Streams: 
  - stream:price-updates
  - stream:trade-events
  - stream:ui-events
```

### Performance Requirements
- Price reads: <1ms average latency
- Batch operations for multiple tokens
- Pipeline commands where possible
- Connection pool to avoid bottlenecks
- Async/await throughout

### Stream Configuration
- MAXLEN ~ 10000 (approximate trimming)
- Block timeout: 100ms for 10Hz
- Batch size: 100 events per read
- Consumer groups for multiple readers
- Auto-acknowledge processed messages

## Testing Strategy

Create comprehensive tests for:

1. **Latency Benchmarks**:
   - 1000 sequential reads <1ms average
   - Concurrent access performance
   - Cache hit/miss scenarios
   - Connection pool saturation

2. **TTL Behavior**:
   - Verify 2-second expiration
   - Grace period handling
   - Refresh on update

3. **Stream Performance**:
   - 10Hz publishing rate
   - Consumer keep-up verification
   - Trimming at 10,000 entries
   - Multiple consumer groups

4. **Error Scenarios**:
   - Connection failures
   - Pool exhaustion
   - Serialization errors
   - Network timeouts

## Deliverables

1. **Core Implementation**:
   - `redis_pool.rs` with connection management
   - `price_cache.rs` with <1ms operations
   - `event_stream.rs` with 10Hz support
   - `pool_state_cache.rs` for MEV data

2. **Service Layer**:
   - Price update service
   - Event publisher utilities
   - Subscription manager
   - Metrics collection

3. **Tests**:
   - Unit tests for all components
   - Integration tests with real Redis
   - Performance benchmarks
   - Load testing scenarios

4. **Documentation**:
   - Key schema reference
   - Performance tuning guide
   - Monitoring setup
   - Troubleshooting steps

## Success Criteria

- Average price read latency <1ms
- 99th percentile latency <2ms
- Stream supports sustained 10Hz updates
- Automatic trimming maintains 10k limit
- All MVP tokens supported
- Zero memory leaks over 24h operation
- Graceful handling of Redis failures
- Comprehensive test coverage >90%