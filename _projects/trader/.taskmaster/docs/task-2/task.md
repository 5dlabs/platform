# Task 2: Set Up Database Infrastructure

## Overview

This task establishes the data persistence layer for the Solana Trading Platform by configuring three complementary database systems: QuestDB for time-series data, PostgreSQL for configuration and metadata, and Redis for high-performance caching and event streaming. Each system is optimized for its specific use case to achieve the performance targets outlined in the PRD.

## Architecture Context

According to the system architecture, the database infrastructure provides:

- **QuestDB**: Time-series storage for trades, metrics, and performance data with 30-day retention
- **PostgreSQL**: Relational storage for configuration, token metadata, and Token-2022 extension data
- **Redis**: Sub-millisecond price caching and real-time event streaming at 10Hz

This infrastructure must support the platform's latency requirements (<1ms cache reads) and handle high-frequency updates from both paper and live trading modes.

## Implementation Details

### 1. QuestDB Setup for Time-Series Data

#### Schema Definition
```sql
-- Trades table with daily partitioning
CREATE TABLE trades (
    timestamp TIMESTAMP,
    trader_id SYMBOL,
    mode SYMBOL,  -- 'paper' or 'live'
    action SYMBOL,  -- 'buy', 'sell', 'swap'
    base_token SYMBOL,
    quote_token SYMBOL,
    amount DOUBLE,
    price DOUBLE,
    slippage DOUBLE,
    fee DOUBLE,
    priority_fee LONG,  -- MEV protection fee in lamports
    transfer_fee DOUBLE,  -- Token-2022 fees
    tx_signature STRING,
    latency_ms INT,
    mev_protected BOOLEAN
) timestamp(timestamp) PARTITION BY DAY;

-- Positions table with hourly partitioning for frequent updates
CREATE TABLE positions (
    timestamp TIMESTAMP,
    trader_id SYMBOL,
    token SYMBOL,
    amount DOUBLE,
    cost_basis DOUBLE,
    current_price DOUBLE,
    unrealized_pnl DOUBLE
) timestamp(timestamp) PARTITION BY HOUR;

-- Metrics table for system performance
CREATE TABLE metrics (
    timestamp TIMESTAMP,
    metric_name SYMBOL,
    value DOUBLE,
    labels STRING  -- JSON format for flexibility
) timestamp(timestamp) PARTITION BY DAY;

-- MEV events tracking
CREATE TABLE mev_events (
    timestamp TIMESTAMP,
    trader_id SYMBOL,
    sandwich_probability DOUBLE,
    estimated_loss_bps INT,
    avoided BOOLEAN,
    priority_fee LONG
) timestamp(timestamp) PARTITION BY DAY;
```

#### Batch Writer Implementation
```rust
use questdb::ingress::{Buffer, Sender, SenderBuilder};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

pub struct QuestDBBatchWriter {
    sender: Sender,
    buffer: Buffer,
    batch_interval: Duration,
    rx: mpsc::Receiver<TradeData>,
}

impl QuestDBBatchWriter {
    pub fn new(host: &str, port: u16) -> (Self, mpsc::Sender<TradeData>) {
        let (tx, rx) = mpsc::channel(1000);
        
        let sender = SenderBuilder::new(host, port)
            .connect()
            .expect("Failed to connect to QuestDB");
            
        let writer = Self {
            sender,
            buffer: Buffer::new(),
            batch_interval: Duration::from_millis(100), // PRD requirement
            rx,
        };
        
        (writer, tx)
    }
    
    pub async fn run(&mut self) {
        let mut interval = time::interval(self.batch_interval);
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.flush_batch().await;
                }
                Some(trade) = self.rx.recv() => {
                    self.buffer_trade(trade);
                }
            }
        }
    }
    
    fn buffer_trade(&mut self, trade: TradeData) {
        self.buffer
            .table("trades")
            .symbol("trader_id", &trade.trader_id)
            .symbol("mode", &trade.mode)
            .symbol("action", &trade.action)
            .symbol("base_token", &trade.base_token)
            .symbol("quote_token", &trade.quote_token)
            .column_f64("amount", trade.amount)
            .column_f64("price", trade.price)
            .column_f64("slippage", trade.slippage)
            .column_f64("fee", trade.fee)
            .column_i64("priority_fee", trade.priority_fee as i64)
            .column_f64("transfer_fee", trade.transfer_fee.unwrap_or(0.0))
            .column_str("tx_signature", &trade.tx_signature.unwrap_or_default())
            .column_i64("latency_ms", trade.latency_ms as i64)
            .column_bool("mev_protected", trade.mev_protected)
            .at_timestamp(trade.timestamp);
    }
    
    async fn flush_batch(&mut self) {
        if self.buffer.len() > 0 {
            match self.sender.flush(&mut self.buffer) {
                Ok(_) => self.buffer.clear(),
                Err(e) => eprintln!("Failed to flush to QuestDB: {}", e),
            }
        }
    }
}
```

### 2. PostgreSQL Configuration

#### Schema Definition
```sql
-- Token metadata with Token-2022 support
CREATE TABLE tokens (
    address VARCHAR(50) PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL,
    name VARCHAR(255),
    decimals INTEGER NOT NULL,
    has_transfer_fee BOOLEAN DEFAULT false,
    transfer_fee_bps INTEGER,
    extensions JSONB,  -- Token-2022 extension data
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create index for symbol lookups
CREATE INDEX idx_tokens_symbol ON tokens(symbol);

-- Trader configuration
CREATE TABLE trader_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    mode VARCHAR(20) CHECK (mode IN ('paper', 'live')),
    max_position_size DECIMAL(20, 8),
    max_daily_loss DECIMAL(20, 8),
    max_slippage_bps INTEGER,
    mev_protection_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Order rules for stop-loss and take-profit
CREATE TABLE order_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trader_id UUID REFERENCES trader_config(id),
    position_token VARCHAR(50),
    rule_type VARCHAR(20) CHECK (rule_type IN ('stop_loss', 'take_profit')),
    trigger_price DECIMAL(20, 8),
    amount DECIMAL(20, 8),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    triggered_at TIMESTAMP
);

-- Node health tracking
CREATE TABLE node_health (
    timestamp TIMESTAMP PRIMARY KEY,
    latency_ms INTEGER NOT NULL,
    error_rate DECIMAL(5, 4),
    circuit_breaker_status VARCHAR(20),
    last_failure TIMESTAMP
);

-- Create function to update timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to trader_config
CREATE TRIGGER update_trader_config_updated_at 
BEFORE UPDATE ON trader_config 
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

#### Connection Pool Implementation
```rust
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub struct PostgresDB {
    pool: PgPool,
}

impl PostgresDB {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(600))
            .connect(database_url)
            .await?;
            
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
            
        Ok(Self { pool })
    }
    
    pub async fn get_token_info(&self, symbol: &str) -> Result<Option<TokenInfo>, sqlx::Error> {
        sqlx::query_as!(
            TokenInfo,
            r#"
            SELECT 
                address, 
                symbol, 
                name, 
                decimals,
                has_transfer_fee,
                transfer_fee_bps,
                extensions
            FROM tokens 
            WHERE symbol = $1
            "#,
            symbol
        )
        .fetch_optional(&self.pool)
        .await
    }
    
    pub async fn upsert_token(&self, token: &TokenInfo) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO tokens (address, symbol, name, decimals, has_transfer_fee, transfer_fee_bps, extensions)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (address) 
            DO UPDATE SET 
                symbol = EXCLUDED.symbol,
                name = EXCLUDED.name,
                decimals = EXCLUDED.decimals,
                has_transfer_fee = EXCLUDED.has_transfer_fee,
                transfer_fee_bps = EXCLUDED.transfer_fee_bps,
                extensions = EXCLUDED.extensions,
                last_updated = CURRENT_TIMESTAMP
            "#,
            token.address,
            token.symbol,
            token.name,
            token.decimals,
            token.has_transfer_fee,
            token.transfer_fee_bps,
            token.extensions
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

### 3. Redis Configuration

#### Price Cache Implementation
```rust
use redis::{aio::Connection, AsyncCommands, Client};
use rust_decimal::Decimal;
use std::time::Duration;

pub struct RedisCache {
    client: Client,
    price_ttl: Duration,
}

impl RedisCache {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        
        Ok(Self {
            client,
            price_ttl: Duration::from_secs(2), // 1-2 second TTL as per PRD
        })
    }
    
    pub async fn get_connection(&self) -> Result<Connection, redis::RedisError> {
        self.client.get_async_connection().await
    }
    
    pub async fn cache_price(
        &self,
        token: &str,
        price: Decimal,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.get_connection().await?;
        let key = format!("price:{}", token);
        let value = price.to_string();
        
        conn.set_ex(&key, value, self.price_ttl.as_secs() as usize).await
    }
    
    pub async fn get_cached_price(
        &self,
        token: &str,
    ) -> Result<Option<Decimal>, Box<dyn std::error::Error>> {
        let mut conn = self.get_connection().await?;
        let key = format!("price:{}", token);
        
        let value: Option<String> = conn.get(&key).await?;
        match value {
            Some(v) => Ok(Some(v.parse::<Decimal>()?)),
            None => Ok(None),
        }
    }
    
    pub async fn cache_pool_state(
        &self,
        pool_address: &str,
        state: &PoolState,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.get_connection().await?;
        let key = format!("pool:{}", pool_address);
        let value = serde_json::to_string(state).unwrap();
        
        conn.set_ex(&key, value, 2).await // 2 second TTL
    }
}
```

#### Event Streaming Implementation
```rust
use redis::streams::{StreamId, StreamKey, StreamMaxlen, StreamReadOptions, StreamReadReply};

pub struct RedisEventStream {
    cache: RedisCache,
    stream_key: String,
    max_len: usize,
}

impl RedisEventStream {
    pub fn new(cache: RedisCache, stream_name: &str) -> Self {
        Self {
            cache,
            stream_key: format!("stream:{}", stream_name),
            max_len: 10_000, // PRD requirement
        }
    }
    
    pub async fn publish_event(
        &self,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<String, redis::RedisError> {
        let mut conn = self.cache.get_connection().await?;
        
        let id: String = conn.xadd_maxlen(
            &self.stream_key,
            StreamMaxlen::Approx(self.max_len),
            "*",
            &[
                ("event_type", event_type),
                ("timestamp", &chrono::Utc::now().to_rfc3339()),
                ("data", &data.to_string()),
            ],
        ).await?;
        
        Ok(id)
    }
    
    pub async fn consume_events(
        &self,
        consumer_group: &str,
        consumer_name: &str,
        block_ms: usize,
    ) -> Result<Vec<Event>, redis::RedisError> {
        let mut conn = self.cache.get_connection().await?;
        
        let opts = StreamReadOptions::default()
            .block(block_ms)
            .count(100); // Read up to 100 events at a time
            
        let reply: StreamReadReply = conn.xread_options(
            &[&self.stream_key],
            &[">"],
            &opts,
        ).await?;
        
        let mut events = Vec::new();
        for StreamKey { key: _, ids } in reply.keys {
            for StreamId { id, map } in ids {
                if let Some(event) = Self::parse_event(id, map) {
                    events.push(event);
                }
            }
        }
        
        Ok(events)
    }
    
    fn parse_event(id: String, map: HashMap<String, String>) -> Option<Event> {
        let event_type = map.get("event_type")?;
        let timestamp = map.get("timestamp")?;
        let data: serde_json::Value = serde_json::from_str(map.get("data")?).ok()?;
        
        Some(Event {
            id,
            event_type: event_type.clone(),
            timestamp: timestamp.clone(),
            data,
        })
    }
}
```

## Testing Strategy

### QuestDB Performance Tests
```rust
#[tokio::test]
async fn test_questdb_batch_performance() {
    let (writer, tx) = QuestDBBatchWriter::new("localhost", 9009);
    
    // Spawn writer task
    tokio::spawn(async move {
        writer.run().await;
    });
    
    // Send 1000 trades
    let start = Instant::now();
    for i in 0..1000 {
        let trade = create_test_trade(i);
        tx.send(trade).await.unwrap();
    }
    
    // Wait for batch flush
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(1), "Batch write took too long");
}
```

### Redis Latency Tests
```rust
#[tokio::test]
async fn test_redis_cache_latency() {
    let cache = RedisCache::new("redis://localhost:6379").unwrap();
    
    // Warm up cache
    cache.cache_price("SOL", Decimal::from(150)).await.unwrap();
    
    // Measure read latency
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = cache.get_cached_price("SOL").await.unwrap();
    }
    let elapsed = start.elapsed();
    
    let avg_latency_ms = elapsed.as_millis() / 1000;
    assert!(avg_latency_ms < 1, "Redis latency exceeds 1ms target");
}
```

### PostgreSQL Connection Pool Tests
```rust
#[tokio::test]
async fn test_postgres_connection_pool() {
    let db = PostgresDB::new("postgres://localhost/trading_test").await.unwrap();
    
    // Spawn concurrent queries
    let mut handles = vec![];
    for _ in 0..50 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            db_clone.get_token_info("SOL").await
        });
        handles.push(handle);
    }
    
    // All should complete successfully
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

## Dependencies

- **Task 1**: Uses Trade and MEV models from common libraries

## Integration Points

- **Paper Trader**: Writes trades to QuestDB, reads token info from PostgreSQL
- **Live Trader**: Same database interactions as paper trader
- **Price Feed**: Updates Redis cache with latest prices
- **TUI**: Subscribes to Redis streams for real-time updates

## Performance Considerations

- QuestDB batch writes every 100ms to optimize throughput
- PostgreSQL connection pool sized for concurrent access
- Redis with 2-second TTL balances freshness and performance
- Event streams capped at 10,000 entries to control memory

## Operational Requirements

- QuestDB: 30-day retention policy with daily partitions
- PostgreSQL: Daily backups with point-in-time recovery
- Redis: Persistence disabled for performance (cache only)
- Monitoring: Track latency percentiles for all databases

## Future Enhancements

- QuestDB clustering for high availability
- PostgreSQL read replicas for scaling
- Redis Sentinel for automatic failover
- Compression for historical data