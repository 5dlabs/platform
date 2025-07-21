# Task 5: Implement Redis Price Caching and Event Streaming

## Overview

This task implements a high-performance Redis integration that provides sub-millisecond price caching and real-time event streaming for the trading platform. The system enables the TUI to display live updates at 10Hz while maintaining efficient resource usage through intelligent caching and stream management.

## Architecture Context

The Redis integration serves as the real-time data backbone of the platform:

- **Price Cache**: Sub-millisecond access to current token prices with 1-2 second TTL
- **Event Streams**: 10Hz update capability for UI responsiveness
- **Pool State Cache**: MEV risk calculation support with fresh liquidity data
- **Circuit Breaker Integration**: Maintains system stability under high load

This component is critical for achieving the PRD's latency requirements while supporting high-frequency updates across all platform components.

## Implementation Details

### 1. Redis Connection Pool Management

```rust
use redis::{aio::ConnectionManager, Client, RedisError};
use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use std::time::Duration;
use std::sync::Arc;

pub struct RedisPool {
    pool: Pool<RedisConnectionManager>,
    config: RedisConfig,
}

#[derive(Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout: Duration,
    pub price_ttl: Duration,
    pub pool_ttl: Duration,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            pool_size: 20,
            connection_timeout: Duration::from_secs(2),
            price_ttl: Duration::from_secs(2),  // PRD requirement
            pool_ttl: Duration::from_secs(2),   // PRD requirement
        }
    }
}

impl RedisPool {
    pub async fn new(config: RedisConfig) -> Result<Self, RedisError> {
        let manager = RedisConnectionManager::new(config.url.clone())?;
        
        let pool = Pool::builder()
            .max_size(config.pool_size)
            .connection_timeout(config.connection_timeout)
            .test_on_check_out(true)
            .build(manager)
            .await
            .map_err(|e| RedisError::from((redis::ErrorKind::IoError, "Pool creation failed", e.to_string())))?;

        Ok(Self { pool, config })
    }

    pub async fn get_connection(&self) -> Result<PooledConnection<'_, RedisConnectionManager>, RedisError> {
        self.pool.get().await
            .map_err(|e| RedisError::from((redis::ErrorKind::IoError, "Failed to get connection", e.to_string())))
    }

    pub fn get_pool(&self) -> Pool<RedisConnectionManager> {
        self.pool.clone()
    }
}
```

### 2. High-Performance Price Cache

```rust
use rust_decimal::Decimal;
use redis::{AsyncCommands, Pipeline};
use tokio::time::{interval, Duration as TokioDuration};
use chrono::{DateTime, Utc};

pub struct PriceCache {
    pool: Arc<RedisPool>,
    metrics: Arc<Metrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub token: String,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

impl PriceCache {
    pub fn new(pool: Arc<RedisPool>) -> Self {
        Self {
            pool,
            metrics: Arc::new(Metrics::new()),
        }
    }

    pub async fn set_price(&self, token: &str, price: Decimal) -> Result<(), RedisError> {
        let start = std::time::Instant::now();
        let mut conn = self.pool.get_connection().await?;
        
        let key = format!("price:{}", token);
        let price_data = PriceData {
            token: token.to_string(),
            price,
            timestamp: Utc::now(),
            source: "jupiter".to_string(),
        };
        
        let serialized = serde_json::to_string(&price_data)
            .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string())))?;
        
        conn.set_ex(&key, serialized, self.pool.config.price_ttl.as_secs() as usize).await?;
        
        self.metrics.record_write_latency(start.elapsed());
        Ok(())
    }

    pub async fn get_price(&self, token: &str) -> Result<Option<PriceData>, RedisError> {
        let start = std::time::Instant::now();
        let mut conn = self.pool.get_connection().await?;
        
        let key = format!("price:{}", token);
        let value: Option<String> = conn.get(&key).await?;
        
        let result = match value {
            Some(v) => {
                let price_data: PriceData = serde_json::from_str(&v)
                    .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed", e.to_string())))?;
                Some(price_data)
            }
            None => None,
        };
        
        let latency = start.elapsed();
        self.metrics.record_read_latency(latency);
        
        // Verify <1ms requirement
        if latency.as_millis() > 1 {
            tracing::warn!("Price cache read exceeded 1ms: {:?}", latency);
        }
        
        Ok(result)
    }

    pub async fn set_multiple_prices(&self, prices: Vec<(String, Decimal)>) -> Result<(), RedisError> {
        let start = std::time::Instant::now();
        let mut conn = self.pool.get_connection().await?;
        
        let mut pipe = Pipeline::new();
        
        for (token, price) in prices {
            let key = format!("price:{}", token);
            let price_data = PriceData {
                token: token.clone(),
                price,
                timestamp: Utc::now(),
                source: "jupiter".to_string(),
            };
            
            let serialized = serde_json::to_string(&price_data)
                .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string())))?;
            
            pipe.set_ex(&key, serialized, self.pool.config.price_ttl.as_secs() as usize);
        }
        
        pipe.query_async(&mut *conn).await?;
        
        self.metrics.record_batch_write_latency(start.elapsed());
        Ok(())
    }

    pub async fn get_all_prices(&self, tokens: &[String]) -> Result<Vec<Option<PriceData>>, RedisError> {
        let start = std::time::Instant::now();
        let mut conn = self.pool.get_connection().await?;
        
        let keys: Vec<String> = tokens.iter()
            .map(|token| format!("price:{}", token))
            .collect();
        
        let values: Vec<Option<String>> = conn.get(keys).await?;
        
        let results = values.into_iter()
            .map(|value| {
                value.and_then(|v| {
                    serde_json::from_str::<PriceData>(&v).ok()
                })
            })
            .collect();
        
        self.metrics.record_batch_read_latency(start.elapsed());
        Ok(results)
    }
}

// Pool state cache for MEV calculations
pub struct PoolStateCache {
    pool: Arc<RedisPool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    pub pool_address: String,
    pub liquidity: f64,
    pub reserve_a: f64,
    pub reserve_b: f64,
    pub volume_24h: f64,
    pub timestamp: DateTime<Utc>,
}

impl PoolStateCache {
    pub fn new(pool: Arc<RedisPool>) -> Self {
        Self { pool }
    }

    pub async fn set_pool_state(&self, pool_address: &str, state: PoolState) -> Result<(), RedisError> {
        let mut conn = self.pool.get_connection().await?;
        let key = format!("pool:{}", pool_address);
        
        let serialized = serde_json::to_string(&state)
            .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string())))?;
        
        conn.set_ex(&key, serialized, self.pool.config.pool_ttl.as_secs() as usize).await
    }

    pub async fn get_pool_state(&self, pool_address: &str) -> Result<Option<PoolState>, RedisError> {
        let mut conn = self.pool.get_connection().await?;
        let key = format!("pool:{}", pool_address);
        
        let value: Option<String> = conn.get(&key).await?;
        
        match value {
            Some(v) => {
                let state: PoolState = serde_json::from_str(&v)
                    .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed", e.to_string())))?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }
}
```

### 3. Event Streaming Implementation

```rust
use redis::streams::{StreamId, StreamKey, StreamMaxlen, StreamReadOptions, StreamReadReply};
use futures_util::stream::{Stream, StreamExt};
use tokio::sync::mpsc;
use std::collections::HashMap;

pub struct EventStream {
    pool: Arc<RedisPool>,
    stream_config: StreamConfig,
}

#[derive(Clone)]
pub struct StreamConfig {
    pub max_length: usize,        // 10,000 as per PRD
    pub block_timeout_ms: usize,  // For XREAD blocking
    pub batch_size: usize,        // Events per read
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            max_length: 10_000,      // PRD requirement
            block_timeout_ms: 100,   // 100ms for 10Hz
            batch_size: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TradingEvent {
    PriceUpdate {
        token: String,
        price: Decimal,
        timestamp: DateTime<Utc>,
    },
    TradeExecuted {
        id: String,
        action: String,
        token_pair: String,
        amount: Decimal,
        price: Decimal,
        timestamp: DateTime<Utc>,
    },
    PositionUpdate {
        token: String,
        amount: Decimal,
        pnl: Decimal,
        timestamp: DateTime<Utc>,
    },
    SystemStatus {
        component: String,
        status: String,
        message: String,
        timestamp: DateTime<Utc>,
    },
}

impl EventStream {
    pub fn new(pool: Arc<RedisPool>, config: StreamConfig) -> Self {
        Self {
            pool,
            stream_config: config,
        }
    }

    pub async fn publish(&self, stream_name: &str, event: TradingEvent) -> Result<String, RedisError> {
        let mut conn = self.pool.get_connection().await?;
        
        let payload = serde_json::to_string(&event)
            .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string())))?;
        
        let event_type = match &event {
            TradingEvent::PriceUpdate { .. } => "price_update",
            TradingEvent::TradeExecuted { .. } => "trade_executed",
            TradingEvent::PositionUpdate { .. } => "position_update",
            TradingEvent::SystemStatus { .. } => "system_status",
        };
        
        let id: String = redis::cmd("XADD")
            .arg(stream_name)
            .arg("MAXLEN")
            .arg("~")
            .arg(self.stream_config.max_length)
            .arg("*")
            .arg("event_type")
            .arg(event_type)
            .arg("timestamp")
            .arg(Utc::now().to_rfc3339())
            .arg("data")
            .arg(payload)
            .query_async(&mut *conn)
            .await?;
        
        Ok(id)
    }

    pub async fn create_consumer_group(
        &self,
        stream_name: &str,
        group_name: &str,
    ) -> Result<(), RedisError> {
        let mut conn = self.pool.get_connection().await?;
        
        // Try to create group, ignore error if it already exists
        let _: Result<String, _> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(stream_name)
            .arg(group_name)
            .arg("$")
            .arg("MKSTREAM")
            .query_async(&mut *conn)
            .await;
        
        Ok(())
    }

    pub fn subscribe(
        &self,
        stream_name: String,
        group_name: String,
        consumer_name: String,
    ) -> impl Stream<Item = Result<Vec<TradingEvent>, RedisError>> {
        let pool = self.pool.clone();
        let config = self.stream_config.clone();
        let (tx, rx) = mpsc::channel(100);
        
        tokio::spawn(async move {
            let mut last_id = ">".to_string();
            
            loop {
                match Self::read_events(&pool, &stream_name, &group_name, &consumer_name, &last_id, &config).await {
                    Ok(events) => {
                        if !events.is_empty() {
                            if let Some(last_event) = events.last() {
                                last_id = last_event.0.clone();
                            }
                            let parsed_events: Vec<TradingEvent> = events.into_iter()
                                .filter_map(|(_, event)| event)
                                .collect();
                            
                            if !parsed_events.is_empty() {
                                if tx.send(Ok(parsed_events)).await.is_err() {
                                    break; // Receiver dropped
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if tx.send(Err(e)).await.is_err() {
                            break; // Receiver dropped
                        }
                    }
                }
                
                // Small delay to achieve ~10Hz
                tokio::time::sleep(TokioDuration::from_millis(100)).await;
            }
        });
        
        tokio_stream::wrappers::ReceiverStream::new(rx)
    }

    async fn read_events(
        pool: &Arc<RedisPool>,
        stream_name: &str,
        group_name: &str,
        consumer_name: &str,
        last_id: &str,
        config: &StreamConfig,
    ) -> Result<Vec<(String, Option<TradingEvent>)>, RedisError> {
        let mut conn = pool.get_connection().await?;
        
        let reply: StreamReadReply = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(group_name)
            .arg(consumer_name)
            .arg("BLOCK")
            .arg(config.block_timeout_ms)
            .arg("COUNT")
            .arg(config.batch_size)
            .arg("STREAMS")
            .arg(stream_name)
            .arg(last_id)
            .query_async(&mut *conn)
            .await?;
        
        let mut events = Vec::new();
        
        for StreamKey { key: _, ids } in reply.keys {
            for StreamId { id, map } in ids {
                let event = Self::parse_event(&map);
                events.push((id, event));
                
                // Acknowledge processed message
                let _: i32 = redis::cmd("XACK")
                    .arg(stream_name)
                    .arg(group_name)
                    .arg(&id)
                    .query_async(&mut *conn)
                    .await?;
            }
        }
        
        Ok(events)
    }

    fn parse_event(map: &HashMap<String, redis::Value>) -> Option<TradingEvent> {
        let data = map.get("data")?.as_string()?;
        serde_json::from_str(&data).ok()
    }
}
```

### 4. Price Update Service

```rust
pub struct PriceUpdateService {
    price_cache: Arc<PriceCache>,
    event_stream: Arc<EventStream>,
    jupiter_client: Arc<JupiterFailoverClient>,
    tokens: Vec<String>,
    update_interval: Duration,
}

impl PriceUpdateService {
    pub fn new(
        price_cache: Arc<PriceCache>,
        event_stream: Arc<EventStream>,
        jupiter_client: Arc<JupiterFailoverClient>,
        tokens: Vec<String>,
    ) -> Self {
        Self {
            price_cache,
            event_stream,
            jupiter_client,
            tokens,
            update_interval: Duration::from_millis(500), // Update every 500ms for fresh data
        }
    }

    pub async fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = interval(self.update_interval);
            
            loop {
                interval.tick().await;
                
                // Fetch prices for all tokens in parallel
                let mut handles = vec![];
                
                for token in &self.tokens {
                    let jupiter = self.jupiter_client.clone();
                    let token = token.clone();
                    
                    let handle = tokio::spawn(async move {
                        // Get quote for 1 unit to determine price
                        let request = QuoteRequest {
                            input_mint: token.clone(),
                            output_mint: "USDC".to_string(),
                            amount: "1000000000".to_string(), // 1 SOL worth
                            slippage_bps: 50,
                            only_direct_routes: false,
                            as_legacy_transaction: false,
                        };
                        
                        match jupiter.get_quote(&request).await {
                            Ok(quote) => {
                                let price = calculate_price_from_quote(&quote);
                                Some((token, price))
                            }
                            Err(e) => {
                                tracing::error!("Failed to fetch price for {}: {}", token, e);
                                None
                            }
                        }
                    });
                    
                    handles.push(handle);
                }
                
                // Collect results
                let mut prices = vec![];
                for handle in handles {
                    if let Ok(Some(price)) = handle.await {
                        prices.push(price);
                    }
                }
                
                // Update cache and publish events
                if !prices.is_empty() {
                    // Batch update cache
                    if let Err(e) = self.price_cache.set_multiple_prices(prices.clone()).await {
                        tracing::error!("Failed to update price cache: {}", e);
                    }
                    
                    // Publish price update events
                    for (token, price) in prices {
                        let event = TradingEvent::PriceUpdate {
                            token,
                            price,
                            timestamp: Utc::now(),
                        };
                        
                        if let Err(e) = self.event_stream.publish("stream:price-updates", event).await {
                            tracing::error!("Failed to publish price event: {}", e);
                        }
                    }
                }
            }
        })
    }
}
```

## Testing Strategy

### Performance Benchmarks

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    async fn benchmark_price_cache_read(cache: &PriceCache) {
        let price = cache.get_price("SOL").await.unwrap();
        black_box(price);
    }

    fn bench_redis_latency(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pool = rt.block_on(RedisPool::new(RedisConfig::default())).unwrap();
        let cache = PriceCache::new(Arc::new(pool));
        
        // Warm up cache
        rt.block_on(cache.set_price("SOL", Decimal::from(150))).unwrap();
        
        c.bench_function("redis_price_read", |b| {
            b.to_async(&rt).iter(|| benchmark_price_cache_read(&cache));
        });
    }

    criterion_group!(benches, bench_redis_latency);
    criterion_main!(benches);
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_event_stream_10hz() {
    let pool = Arc::new(RedisPool::new(RedisConfig::default()).await.unwrap());
    let event_stream = Arc::new(EventStream::new(pool.clone(), StreamConfig::default()));
    
    // Create consumer group
    event_stream.create_consumer_group("test-stream", "test-group").await.unwrap();
    
    // Start subscriber
    let mut subscriber = event_stream.subscribe(
        "test-stream".to_string(),
        "test-group".to_string(),
        "test-consumer".to_string(),
    );
    
    // Publish 100 events at 10Hz
    let publisher = event_stream.clone();
    let publish_task = tokio::spawn(async move {
        for i in 0..100 {
            let event = TradingEvent::PriceUpdate {
                token: "SOL".to_string(),
                price: Decimal::from(150 + i),
                timestamp: Utc::now(),
            };
            
            publisher.publish("test-stream", event).await.unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await; // 10Hz
        }
    });
    
    // Verify we receive events at approximately 10Hz
    let start = std::time::Instant::now();
    let mut count = 0;
    
    while let Some(Ok(events)) = subscriber.next().await {
        count += events.len();
        if count >= 100 {
            break;
        }
    }
    
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_secs(9)); // Should take ~10 seconds
    assert!(elapsed <= Duration::from_secs(11)); // With some tolerance
}

#[tokio::test]
async fn test_cache_ttl() {
    let pool = Arc::new(RedisPool::new(RedisConfig::default()).await.unwrap());
    let cache = PriceCache::new(pool);
    
    // Set price
    cache.set_price("SOL", Decimal::from(150)).await.unwrap();
    
    // Should exist immediately
    let price = cache.get_price("SOL").await.unwrap();
    assert!(price.is_some());
    
    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    // Should be gone
    let price = cache.get_price("SOL").await.unwrap();
    assert!(price.is_none());
}

#[tokio::test]
async fn test_stream_trimming() {
    let pool = Arc::new(RedisPool::new(RedisConfig::default()).await.unwrap());
    let mut config = StreamConfig::default();
    config.max_length = 10; // Small limit for testing
    
    let event_stream = EventStream::new(pool, config);
    
    // Publish 20 events
    for i in 0..20 {
        let event = TradingEvent::SystemStatus {
            component: "test".to_string(),
            status: "ok".to_string(),
            message: format!("Event {}", i),
            timestamp: Utc::now(),
        };
        
        event_stream.publish("test-trim-stream", event).await.unwrap();
    }
    
    // Check stream length
    let mut conn = event_stream.pool.get_connection().await.unwrap();
    let len: usize = redis::cmd("XLEN")
        .arg("test-trim-stream")
        .query_async(&mut *conn)
        .await
        .unwrap();
    
    // Should be trimmed to approximately 10
    assert!(len <= 15); // Allow some overhead
}
```

## Dependencies

- **Task 2**: Uses Redis connection from database infrastructure

## Integration Points

- **Paper Trader**: Reads prices from cache for simulations
- **Live Trader**: Same price access for real trades
- **Price Feed**: Updates cache with latest prices
- **TUI**: Subscribes to event streams for real-time updates
- **MEV Simulator**: Uses pool state cache for risk calculations

## Performance Considerations

- Connection pooling with 20 connections
- Pipeline operations for batch updates
- 1-2 second TTL balances freshness and performance
- Stream trimming prevents unbounded memory growth
- Async processing for all operations

## Operational Requirements

- Redis configured with appropriate memory limits
- Monitoring for cache hit rates
- Alerts for high latency (>1ms)
- Regular connection pool health checks
- Stream size monitoring

## Future Enhancements

- Redis Cluster support for scaling
- Lua scripts for atomic operations
- Compression for large payloads
- Priority queues for critical events
- Historical price retention in separate store