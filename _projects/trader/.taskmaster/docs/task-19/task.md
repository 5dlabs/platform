# Task 19: Implement Database Trait Abstractions for Modular Data Access

## Overview

This task creates database trait abstractions that provide clean interfaces for data access across different storage systems (PostgreSQL, QuestDB, Redis). These abstractions enable modular data access patterns and facilitate testing through mock implementations, ensuring the trading platform can efficiently interact with its multi-database architecture.

## Architecture Context

The trading platform uses three different database systems, each optimized for specific use cases:

- **PostgreSQL**: Configuration, trader profiles, order rules, and audit logs
- **QuestDB**: Time-series data for trades, positions, and metrics
- **Redis**: High-speed caching for prices and pool states

The trait abstractions provide:
- **Unified Interfaces**: Consistent APIs regardless of underlying storage
- **Type Safety**: Compile-time guarantees for database operations
- **Testability**: Easy mocking for unit and integration tests
- **Performance**: Optimized implementations for each database type
- **Flexibility**: Easy addition of new storage backends

## Implementation Details

### 1. Core Database Traits

#### TradeStore Trait
```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

#[async_trait]
pub trait TradeStore: Send + Sync + 'static {
    /// Save a single trade to the database
    async fn save_trade(&self, trade: &Trade) -> Result<(), DbError>;
    
    /// Retrieve trades for a specific token
    async fn get_trades_by_token(
        &self,
        token: &Pubkey,
        limit: usize,
    ) -> Result<Vec<Trade>, DbError>;
    
    /// Get trades within a time range
    async fn get_trades_in_timerange(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Trade>, DbError>;
    
    /// Batch save multiple trades for efficiency
    async fn batch_save_trades(&self, trades: &[Trade]) -> Result<(), DbError>;
    
    /// Get trade by request ID (for correlation)
    async fn get_trade_by_request_id(
        &self,
        request_id: &str,
    ) -> Result<Option<Trade>, DbError>;
    
    /// Get trades with pagination
    async fn get_trades_paginated(
        &self,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<TradePage, DbError>;
    
    /// Delete trades older than specified time (for data retention)
    async fn delete_trades_before(
        &self,
        timestamp: DateTime<Utc>,
    ) -> Result<u64, DbError>;
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub trader_id: String,
    pub mode: TradingMode,
    pub action: TradeAction,
    pub base_token: Pubkey,
    pub quote_token: Pubkey,
    pub amount: f64,
    pub price: f64,
    pub slippage: f64,
    pub fee: f64,
    pub priority_fee: Option<u64>,
    pub tx_signature: Option<String>,
    pub transfer_fee: Option<f64>,
    pub latency_ms: u32,
    pub mev_protected: bool,
    pub request_id: Option<String>,
    pub source_service: Option<String>,
}

#[derive(Debug)]
pub struct TradePage {
    pub trades: Vec<Trade>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}
```

#### MetricsStore Trait
```rust
#[async_trait]
pub trait MetricsStore: Send + Sync + 'static {
    /// Record operation latency
    async fn record_latency(
        &self,
        operation: &str,
        latency_ms: f64,
    ) -> Result<(), DbError>;
    
    /// Record slippage for analysis
    async fn record_slippage(
        &self,
        token: &Pubkey,
        slippage_bps: i32,
    ) -> Result<(), DbError>;
    
    /// Record MEV events
    async fn record_mev_event(&self, event: &MevEvent) -> Result<(), DbError>;
    
    /// Get P99 latency for an operation
    async fn get_p99_latency(
        &self,
        operation: &str,
        window: Duration,
    ) -> Result<f64, DbError>;
    
    /// Get aggregated metrics
    async fn get_metrics_summary(
        &self,
        window: Duration,
    ) -> Result<MetricsSummary, DbError>;
    
    /// Record custom metric
    async fn record_metric(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<(), DbError>;
    
    /// Query metrics with filters
    async fn query_metrics(
        &self,
        query: MetricsQuery,
    ) -> Result<Vec<MetricPoint>, DbError>;
}

#[derive(Debug, Clone)]
pub struct MevEvent {
    pub timestamp: DateTime<Utc>,
    pub trade_id: Uuid,
    pub sandwich_probability: f64,
    pub estimated_loss_bps: u16,
    pub avoided: bool,
    pub priority_fee: u64,
}

#[derive(Debug)]
pub struct MetricsSummary {
    pub total_trades: u64,
    pub avg_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub avg_slippage_bps: f64,
    pub mev_events_avoided: u64,
    pub mev_events_suffered: u64,
}

#[derive(Debug)]
pub struct MetricsQuery {
    pub metric_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub labels: HashMap<String, String>,
    pub aggregation: Option<AggregationType>,
}
```

#### PriceCache Trait
```rust
#[async_trait]
pub trait PriceCache: Send + Sync + 'static {
    /// Cache a token price with TTL
    async fn cache_price(
        &self,
        token: &Pubkey,
        price: f64,
        ttl: Duration,
    ) -> Result<(), DbError>;
    
    /// Get cached price if available
    async fn get_price(&self, token: &Pubkey) -> Result<Option<f64>, DbError>;
    
    /// Cache pool state for slippage calculations
    async fn cache_pool_state(
        &self,
        pool: &Pubkey,
        state: PoolState,
        ttl: Duration,
    ) -> Result<(), DbError>;
    
    /// Get cached pool state
    async fn get_pool_state(&self, pool: &Pubkey) -> Result<Option<PoolState>, DbError>;
    
    /// Batch get prices for multiple tokens
    async fn get_prices_batch(
        &self,
        tokens: &[Pubkey],
    ) -> Result<HashMap<Pubkey, f64>, DbError>;
    
    /// Invalidate cache entries
    async fn invalidate_token(&self, token: &Pubkey) -> Result<(), DbError>;
    
    /// Get cache statistics
    async fn get_cache_stats(&self) -> Result<CacheStats, DbError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    pub pool_address: Pubkey,
    pub liquidity: f64,
    pub reserve_a: f64,
    pub reserve_b: f64,
    pub volume_24h: f64,
    pub fee_rate: f64,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_keys: u64,
    pub memory_usage_bytes: u64,
    pub hit_rate: f64,
    pub eviction_count: u64,
}
```

#### ConfigStore Trait
```rust
#[async_trait]
pub trait ConfigStore: Send + Sync + 'static {
    /// Save trader configuration
    async fn save_trader_config(&self, config: &TraderConfig) -> Result<(), DbError>;
    
    /// Get trader configuration by ID
    async fn get_trader_config(&self, id: &str) -> Result<Option<TraderConfig>, DbError>;
    
    /// List all trader configurations
    async fn list_trader_configs(&self) -> Result<Vec<TraderConfig>, DbError>;
    
    /// Update specific config fields
    async fn update_trader_config(
        &self,
        id: &str,
        updates: TraderConfigUpdate,
    ) -> Result<(), DbError>;
    
    /// Delete trader configuration
    async fn delete_trader_config(&self, id: &str) -> Result<bool, DbError>;
    
    /// Save order rule
    async fn save_order_rule(&self, rule: &OrderRule) -> Result<(), DbError>;
    
    /// Get order rules for a trader
    async fn get_order_rules(&self, trader_id: &str) -> Result<Vec<OrderRule>, DbError>;
    
    /// Mark order rule as triggered
    async fn trigger_order_rule(
        &self,
        rule_id: &Uuid,
        tx_signature: &str,
    ) -> Result<(), DbError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraderConfig {
    pub id: String,
    pub name: String,
    pub mode: TradingMode,
    pub max_position_size: f64,
    pub max_daily_loss: f64,
    pub max_slippage_bps: u16,
    pub mev_protection_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct OrderRule {
    pub id: Uuid,
    pub trader_id: String,
    pub position_token: Pubkey,
    pub rule_type: OrderRuleType,
    pub trigger_price: f64,
    pub amount: f64,
    pub created_at: DateTime<Utc>,
    pub triggered_at: Option<DateTime<Utc>>,
    pub tx_signature: Option<String>,
}
```

### 2. PostgreSQL Implementations

```rust
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::time::Duration;

pub struct PostgresConfigStore {
    pool: PgPool,
}

impl PostgresConfigStore {
    pub async fn new(database_url: &str) -> Result<Self, DbError> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<(), DbError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| DbError::MigrationError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl ConfigStore for PostgresConfigStore {
    async fn save_trader_config(&self, config: &TraderConfig) -> Result<(), DbError> {
        let query = r#"
            INSERT INTO trader_config (
                id, name, mode, max_position_size, max_daily_loss,
                max_slippage_bps, mev_protection_enabled, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                mode = EXCLUDED.mode,
                max_position_size = EXCLUDED.max_position_size,
                max_daily_loss = EXCLUDED.max_daily_loss,
                max_slippage_bps = EXCLUDED.max_slippage_bps,
                mev_protection_enabled = EXCLUDED.mev_protection_enabled,
                updated_at = EXCLUDED.updated_at
        "#;

        sqlx::query(query)
            .bind(&config.id)
            .bind(&config.name)
            .bind(&config.mode.to_string())
            .bind(config.max_position_size)
            .bind(config.max_daily_loss)
            .bind(config.max_slippage_bps as i32)
            .bind(config.mev_protection_enabled)
            .bind(config.created_at)
            .bind(config.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        Ok(())
    }

    async fn get_trader_config(&self, id: &str) -> Result<Option<TraderConfig>, DbError> {
        let query = r#"
            SELECT id, name, mode, max_position_size, max_daily_loss,
                   max_slippage_bps, mev_protection_enabled, created_at, updated_at
            FROM trader_config
            WHERE id = $1
        "#;

        let result = sqlx::query(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        Ok(result.map(|row| TraderConfig {
            id: row.get("id"),
            name: row.get("name"),
            mode: row.get::<String, _>("mode").parse().unwrap(),
            max_position_size: row.get("max_position_size"),
            max_daily_loss: row.get("max_daily_loss"),
            max_slippage_bps: row.get::<i32, _>("max_slippage_bps") as u16,
            mev_protection_enabled: row.get("mev_protection_enabled"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    async fn save_order_rule(&self, rule: &OrderRule) -> Result<(), DbError> {
        let query = r#"
            INSERT INTO order_rules (
                id, trader_id, position_token, rule_type, trigger_price,
                amount, created_at, triggered_at, tx_signature
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;

        sqlx::query(query)
            .bind(rule.id)
            .bind(&rule.trader_id)
            .bind(rule.position_token.to_string())
            .bind(rule.rule_type.to_string())
            .bind(rule.trigger_price)
            .bind(rule.amount)
            .bind(rule.created_at)
            .bind(rule.triggered_at)
            .bind(&rule.tx_signature)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        Ok(())
    }

    // Additional implementations...
}

// Transaction support
impl PostgresConfigStore {
    pub async fn with_transaction<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, sqlx::Postgres>) -> BoxFuture<'_, Result<R, DbError>>,
    {
        let mut tx = self.pool.begin().await
            .map_err(|e| DbError::TransactionError(e.to_string()))?;

        match f(&mut tx).await {
            Ok(result) => {
                tx.commit().await
                    .map_err(|e| DbError::TransactionError(e.to_string()))?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback().await
                    .map_err(|e| DbError::TransactionError(e.to_string()))?;
                Err(e)
            }
        }
    }
}
```

### 3. QuestDB Implementations

```rust
use questdb::{Client as QuestClient, ColumnType, TableWriter};

pub struct QuestDbTradeStore {
    client: Arc<QuestClient>,
    write_buffer: Arc<Mutex<Vec<Trade>>>,
    flush_interval: Duration,
}

impl QuestDbTradeStore {
    pub async fn new(host: &str, port: u16) -> Result<Self, DbError> {
        let client = QuestClient::new(host, port)
            .await
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;

        let store = Self {
            client: Arc::new(client),
            write_buffer: Arc::new(Mutex::new(Vec::with_capacity(1000))),
            flush_interval: Duration::from_millis(100), // 100ms batch writes
        };

        // Start background flush task
        store.start_flush_task();

        Ok(store)
    }

    fn start_flush_task(&self) {
        let client = self.client.clone();
        let buffer = self.write_buffer.clone();
        let interval = self.flush_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            
            loop {
                interval.tick().await;
                
                let trades = {
                    let mut buf = buffer.lock().unwrap();
                    if buf.is_empty() {
                        continue;
                    }
                    std::mem::take(&mut *buf)
                };

                if let Err(e) = Self::flush_trades(&client, &trades).await {
                    error!("Failed to flush trades: {}", e);
                }
            }
        });
    }

    async fn flush_trades(client: &QuestClient, trades: &[Trade]) -> Result<(), DbError> {
        let mut writer = TableWriter::new("trades");

        for trade in trades {
            writer
                .add_timestamp("timestamp", trade.timestamp)
                .add_symbol("trader_id", &trade.trader_id)
                .add_symbol("mode", &trade.mode.to_string())
                .add_symbol("action", &trade.action.to_string())
                .add_symbol("base_token", &trade.base_token.to_string())
                .add_symbol("quote_token", &trade.quote_token.to_string())
                .add_double("amount", trade.amount)
                .add_double("price", trade.price)
                .add_double("slippage", trade.slippage)
                .add_double("fee", trade.fee)
                .add_long("priority_fee", trade.priority_fee.unwrap_or(0) as i64)
                .add_string("tx_signature", trade.tx_signature.as_deref().unwrap_or(""))
                .add_int("latency_ms", trade.latency_ms as i32)
                .add_boolean("mev_protected", trade.mev_protected);

            writer.next_row();
        }

        client.write(writer).await
            .map_err(|e| DbError::WriteError(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl TradeStore for QuestDbTradeStore {
    async fn save_trade(&self, trade: &Trade) -> Result<(), DbError> {
        let mut buffer = self.write_buffer.lock().unwrap();
        buffer.push(trade.clone());
        
        // Force flush if buffer is getting large
        if buffer.len() >= 1000 {
            drop(buffer);
            self.force_flush().await?;
        }
        
        Ok(())
    }

    async fn batch_save_trades(&self, trades: &[Trade]) -> Result<(), DbError> {
        let mut buffer = self.write_buffer.lock().unwrap();
        buffer.extend_from_slice(trades);
        
        if buffer.len() >= 1000 {
            drop(buffer);
            self.force_flush().await?;
        }
        
        Ok(())
    }

    async fn get_trades_in_timerange(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Trade>, DbError> {
        let query = format!(
            "SELECT * FROM trades WHERE timestamp >= '{}' AND timestamp <= '{}' ORDER BY timestamp DESC",
            start.to_rfc3339(),
            end.to_rfc3339()
        );

        let results = self.client.query(&query).await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        Ok(results.into_iter().map(Self::row_to_trade).collect())
    }

    // Additional implementations...
}

pub struct QuestDbMetricsStore {
    client: Arc<QuestClient>,
}

#[async_trait]
impl MetricsStore for QuestDbMetricsStore {
    async fn record_latency(
        &self,
        operation: &str,
        latency_ms: f64,
    ) -> Result<(), DbError> {
        let mut writer = TableWriter::new("metrics");
        
        writer
            .add_timestamp("timestamp", Utc::now())
            .add_symbol("metric_name", "latency")
            .add_symbol("operation", operation)
            .add_double("value", latency_ms)
            .next_row();

        self.client.write(writer).await
            .map_err(|e| DbError::WriteError(e.to_string()))?;

        Ok(())
    }

    async fn get_p99_latency(
        &self,
        operation: &str,
        window: Duration,
    ) -> Result<f64, DbError> {
        let start = Utc::now() - window;
        let query = format!(
            r#"
            SELECT percentile_cont(value, 0.99) as p99
            FROM metrics
            WHERE metric_name = 'latency'
              AND operation = '{}'
              AND timestamp >= '{}'
            "#,
            operation,
            start.to_rfc3339()
        );

        let results = self.client.query(&query).await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        results.first()
            .and_then(|row| row.get("p99"))
            .ok_or_else(|| DbError::NotFound("No latency data found".to_string()))
    }

    // Additional implementations...
}
```

### 4. Redis Implementations

```rust
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use std::time::Duration;

pub struct RedisPriceCache {
    conn: ConnectionManager,
    default_ttl: Duration,
}

impl RedisPriceCache {
    pub async fn new(redis_url: &str, default_ttl: Duration) -> Result<Self, DbError> {
        let client = Client::open(redis_url)
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;
        
        let conn = ConnectionManager::new(client).await
            .map_err(|e| DbError::ConnectionError(e.to_string()))?;

        Ok(Self { conn, default_ttl })
    }

    fn price_key(token: &Pubkey) -> String {
        format!("price:{}", token)
    }

    fn pool_key(pool: &Pubkey) -> String {
        format!("pool:{}", pool)
    }
}

#[async_trait]
impl PriceCache for RedisPriceCache {
    async fn cache_price(
        &self,
        token: &Pubkey,
        price: f64,
        ttl: Duration,
    ) -> Result<(), DbError> {
        let key = Self::price_key(token);
        let ttl_secs = ttl.as_secs() as usize;

        self.conn.clone()
            .set_ex(key, price.to_string(), ttl_secs)
            .await
            .map_err(|e| DbError::CacheError(e.to_string()))?;

        Ok(())
    }

    async fn get_price(&self, token: &Pubkey) -> Result<Option<f64>, DbError> {
        let key = Self::price_key(token);

        let value: Option<String> = self.conn.clone()
            .get(key)
            .await
            .map_err(|e| DbError::CacheError(e.to_string()))?;

        Ok(value.and_then(|v| v.parse().ok()))
    }

    async fn cache_pool_state(
        &self,
        pool: &Pubkey,
        state: PoolState,
        ttl: Duration,
    ) -> Result<(), DbError> {
        let key = Self::pool_key(pool);
        let ttl_secs = ttl.as_secs() as usize;
        
        let serialized = serde_json::to_string(&state)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        self.conn.clone()
            .set_ex(key, serialized, ttl_secs)
            .await
            .map_err(|e| DbError::CacheError(e.to_string()))?;

        Ok(())
    }

    async fn get_pool_state(&self, pool: &Pubkey) -> Result<Option<PoolState>, DbError> {
        let key = Self::pool_key(pool);

        let value: Option<String> = self.conn.clone()
            .get(key)
            .await
            .map_err(|e| DbError::CacheError(e.to_string()))?;

        match value {
            Some(json) => {
                let state = serde_json::from_str(&json)
                    .map_err(|e| DbError::DeserializationError(e.to_string()))?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    async fn get_prices_batch(
        &self,
        tokens: &[Pubkey],
    ) -> Result<HashMap<Pubkey, f64>, DbError> {
        let keys: Vec<String> = tokens.iter().map(Self::price_key).collect();
        
        let values: Vec<Option<String>> = self.conn.clone()
            .mget(&keys)
            .await
            .map_err(|e| DbError::CacheError(e.to_string()))?;

        let mut prices = HashMap::new();
        for (token, value) in tokens.iter().zip(values.iter()) {
            if let Some(price_str) = value {
                if let Ok(price) = price_str.parse::<f64>() {
                    prices.insert(*token, price);
                }
            }
        }

        Ok(prices)
    }

    async fn get_cache_stats(&self) -> Result<CacheStats, DbError> {
        let info: String = redis::cmd("INFO")
            .arg("stats")
            .query_async(&mut self.conn.clone())
            .await
            .map_err(|e| DbError::CacheError(e.to_string()))?;

        // Parse Redis INFO output
        let mut stats = CacheStats {
            total_keys: 0,
            memory_usage_bytes: 0,
            hit_rate: 0.0,
            eviction_count: 0,
        };

        for line in info.lines() {
            if let Some((key, value)) = line.split_once(':') {
                match key {
                    "db0" => {
                        // Parse keys count from db0:keys=100,expires=50
                        if let Some(keys_part) = value.split(',').next() {
                            if let Some(count) = keys_part.strip_prefix("keys=") {
                                stats.total_keys = count.parse().unwrap_or(0);
                            }
                        }
                    }
                    "keyspace_hits" => {
                        let hits: u64 = value.parse().unwrap_or(0);
                        // Calculate hit rate with keyspace_misses
                        // This is simplified - real implementation would track both
                        stats.hit_rate = hits as f64 / (hits + 1) as f64;
                    }
                    "evicted_keys" => {
                        stats.eviction_count = value.parse().unwrap_or(0);
                    }
                    _ => {}
                }
            }
        }

        Ok(stats)
    }
}
```

### 5. Mock Implementations

```rust
use std::collections::HashMap;
use std::sync::Mutex;

pub struct MockTradeStore {
    trades: Arc<Mutex<Vec<Trade>>>,
    fail_on_save: bool,
    latency_ms: Option<u64>,
}

impl MockTradeStore {
    pub fn new() -> Self {
        Self {
            trades: Arc::new(Mutex::new(Vec::new())),
            fail_on_save: false,
            latency_ms: None,
        }
    }

    pub fn with_failure() -> Self {
        Self {
            fail_on_save: true,
            ..Self::new()
        }
    }

    pub fn with_latency(latency_ms: u64) -> Self {
        Self {
            latency_ms: Some(latency_ms),
            ..Self::new()
        }
    }

    async fn simulate_latency(&self) {
        if let Some(latency) = self.latency_ms {
            tokio::time::sleep(Duration::from_millis(latency)).await;
        }
    }
}

#[async_trait]
impl TradeStore for MockTradeStore {
    async fn save_trade(&self, trade: &Trade) -> Result<(), DbError> {
        self.simulate_latency().await;

        if self.fail_on_save {
            return Err(DbError::WriteError("Mock failure".to_string()));
        }

        let mut trades = self.trades.lock().unwrap();
        trades.push(trade.clone());
        Ok(())
    }

    async fn get_trades_by_token(
        &self,
        token: &Pubkey,
        limit: usize,
    ) -> Result<Vec<Trade>, DbError> {
        self.simulate_latency().await;

        let trades = self.trades.lock().unwrap();
        let filtered: Vec<Trade> = trades
            .iter()
            .filter(|t| t.base_token == *token || t.quote_token == *token)
            .take(limit)
            .cloned()
            .collect();

        Ok(filtered)
    }

    async fn batch_save_trades(&self, trades: &[Trade]) -> Result<(), DbError> {
        for trade in trades {
            self.save_trade(trade).await?;
        }
        Ok(())
    }

    // Additional mock implementations...
}

pub struct MockPriceCache {
    prices: Arc<RwLock<HashMap<Pubkey, (f64, Instant)>>>,
    default_ttl: Duration,
}

impl MockPriceCache {
    pub fn new() -> Self {
        Self {
            prices: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Duration::from_secs(2),
        }
    }

    pub fn with_preset_prices(prices: HashMap<Pubkey, f64>) -> Self {
        let now = Instant::now();
        let prices_with_expiry = prices
            .into_iter()
            .map(|(k, v)| (k, (v, now + Duration::from_secs(3600))))
            .collect();

        Self {
            prices: Arc::new(RwLock::new(prices_with_expiry)),
            default_ttl: Duration::from_secs(2),
        }
    }
}

#[async_trait]
impl PriceCache for MockPriceCache {
    async fn cache_price(
        &self,
        token: &Pubkey,
        price: f64,
        ttl: Duration,
    ) -> Result<(), DbError> {
        let expiry = Instant::now() + ttl;
        self.prices.write().await.insert(*token, (price, expiry));
        Ok(())
    }

    async fn get_price(&self, token: &Pubkey) -> Result<Option<f64>, DbError> {
        let prices = self.prices.read().await;
        
        match prices.get(token) {
            Some((price, expiry)) if *expiry > Instant::now() => Ok(Some(*price)),
            _ => Ok(None),
        }
    }

    // Additional mock implementations...
}
```

### 6. Database Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Query error: {0}")]
    QueryError(String),
    
    #[error("Write error: {0}")]
    WriteError(String),
    
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    #[error("Migration error: {0}")]
    MigrationError(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Pool exhausted")]
    PoolExhausted,
}

impl DbError {
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            DbError::ConnectionError(_) | 
            DbError::TimeoutError(_) | 
            DbError::PoolExhausted
        )
    }

    pub fn is_permanent(&self) -> bool {
        matches!(
            self,
            DbError::QueryError(_) | 
            DbError::SerializationError(_) | 
            DbError::DeserializationError(_)
        )
    }
}

// Retry helper for transient errors
pub async fn retry_db_operation<F, T>(
    mut operation: F,
    max_retries: u32,
) -> Result<T, DbError>
where
    F: FnMut() -> BoxFuture<'static, Result<T, DbError>>,
{
    let mut last_error = None;
    
    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if !e.is_transient() {
                    return Err(e);
                }
                
                last_error = Some(e);
                let delay = Duration::from_millis(100 * 2u64.pow(attempt));
                tokio::time::sleep(delay).await;
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

### 7. Connection Management

```rust
pub struct ConnectionManager {
    postgres_pool: PgPool,
    questdb_client: Arc<QuestClient>,
    redis_manager: ConnectionManager,
}

impl ConnectionManager {
    pub async fn new(config: DatabaseConfig) -> Result<Self, DbError> {
        // PostgreSQL pool
        let postgres_pool = PgPoolOptions::new()
            .max_connections(config.postgres_max_connections)
            .connect_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .connect(&config.postgres_url)
            .await?;

        // QuestDB client
        let questdb_client = Arc::new(
            QuestClient::new(&config.questdb_host, config.questdb_port).await?
        );

        // Redis connection manager
        let redis_client = Client::open(config.redis_url)?;
        let redis_manager = ConnectionManager::new(redis_client).await?;

        Ok(Self {
            postgres_pool,
            questdb_client,
            redis_manager,
        })
    }

    pub async fn health_check(&self) -> Result<HealthStatus, DbError> {
        let mut status = HealthStatus::default();

        // Check PostgreSQL
        match sqlx::query("SELECT 1").fetch_one(&self.postgres_pool).await {
            Ok(_) => status.postgres = true,
            Err(e) => warn!("PostgreSQL health check failed: {}", e),
        }

        // Check QuestDB
        match self.questdb_client.query("SELECT 1").await {
            Ok(_) => status.questdb = true,
            Err(e) => warn!("QuestDB health check failed: {}", e),
        }

        // Check Redis
        match redis::cmd("PING")
            .query_async::<_, String>(&mut self.redis_manager.clone())
            .await
        {
            Ok(_) => status.redis = true,
            Err(e) => warn!("Redis health check failed: {}", e),
        }

        Ok(status)
    }

    pub async fn graceful_shutdown(&self) -> Result<(), DbError> {
        // Close PostgreSQL connections
        self.postgres_pool.close().await;

        // QuestDB and Redis connections close on drop
        info!("Database connections closed gracefully");
        
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct HealthStatus {
    pub postgres: bool,
    pub questdb: bool,
    pub redis: bool,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        self.postgres && self.questdb && self.redis
    }
}
```

### 8. Database Factory

```rust
pub struct DatabaseFactory;

impl DatabaseFactory {
    pub async fn create_trade_store(
        config: &DatabaseConfig,
    ) -> Result<Arc<dyn TradeStore>, DbError> {
        match config.trade_store_type {
            StoreType::Postgres => {
                let store = PostgresTradeStore::new(&config.postgres_url).await?;
                Ok(Arc::new(store))
            }
            StoreType::QuestDb => {
                let store = QuestDbTradeStore::new(
                    &config.questdb_host,
                    config.questdb_port,
                ).await?;
                Ok(Arc::new(store))
            }
            StoreType::Mock => {
                Ok(Arc::new(MockTradeStore::new()))
            }
        }
    }

    pub async fn create_metrics_store(
        config: &DatabaseConfig,
    ) -> Result<Arc<dyn MetricsStore>, DbError> {
        match config.metrics_store_type {
            StoreType::QuestDb => {
                let store = QuestDbMetricsStore::new(
                    &config.questdb_host,
                    config.questdb_port,
                ).await?;
                Ok(Arc::new(store))
            }
            StoreType::Mock => {
                Ok(Arc::new(MockMetricsStore::new()))
            }
            _ => Err(DbError::ConnectionError(
                "Metrics store only supports QuestDB".to_string()
            )),
        }
    }

    pub async fn create_price_cache(
        config: &DatabaseConfig,
    ) -> Result<Arc<dyn PriceCache>, DbError> {
        match config.cache_type {
            StoreType::Redis => {
                let cache = RedisPriceCache::new(
                    &config.redis_url,
                    Duration::from_secs(2),
                ).await?;
                Ok(Arc::new(cache))
            }
            StoreType::Mock => {
                Ok(Arc::new(MockPriceCache::new()))
            }
            _ => Err(DbError::ConnectionError(
                "Price cache only supports Redis".to_string()
            )),
        }
    }

    pub async fn create_config_store(
        config: &DatabaseConfig,
    ) -> Result<Arc<dyn ConfigStore>, DbError> {
        match config.config_store_type {
            StoreType::Postgres => {
                let store = PostgresConfigStore::new(&config.postgres_url).await?;
                Ok(Arc::new(store))
            }
            StoreType::Mock => {
                Ok(Arc::new(MockConfigStore::new()))
            }
            _ => Err(DbError::ConnectionError(
                "Config store only supports PostgreSQL".to_string()
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub postgres_url: String,
    pub postgres_max_connections: u32,
    pub questdb_host: String,
    pub questdb_port: u16,
    pub redis_url: String,
    pub trade_store_type: StoreType,
    pub metrics_store_type: StoreType,
    pub cache_type: StoreType,
    pub config_store_type: StoreType,
}

#[derive(Debug, Clone)]
pub enum StoreType {
    Postgres,
    QuestDb,
    Redis,
    Mock,
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_trade_store() {
        let store = MockTradeStore::new();
        let trade = create_test_trade();

        // Save trade
        store.save_trade(&trade).await.unwrap();

        // Retrieve trades
        let trades = store.get_trades_by_token(&trade.base_token, 10).await.unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].id, trade.id);
    }

    #[tokio::test]
    async fn test_mock_price_cache_expiry() {
        let cache = MockPriceCache::new();
        let token = Pubkey::new_unique();

        // Cache price with 100ms TTL
        cache.cache_price(&token, 100.0, Duration::from_millis(100)).await.unwrap();

        // Should exist immediately
        assert_eq!(cache.get_price(&token).await.unwrap(), Some(100.0));

        // Should expire after TTL
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get_price(&token).await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_db_error_classification() {
        let transient_error = DbError::ConnectionError("timeout".to_string());
        assert!(transient_error.is_transient());
        assert!(!transient_error.is_permanent());

        let permanent_error = DbError::QueryError("syntax error".to_string());
        assert!(!permanent_error.is_transient());
        assert!(permanent_error.is_permanent());
    }

    #[tokio::test]
    async fn test_retry_transient_errors() {
        let mut attempt = 0;
        
        let result = retry_db_operation(|| {
            attempt += 1;
            Box::pin(async move {
                if attempt < 3 {
                    Err(DbError::ConnectionError("timeout".to_string()))
                } else {
                    Ok("success")
                }
            })
        }, 5).await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempt, 3);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_postgres_trade_store_integration() {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://test:test@localhost/test".to_string());

    let store = PostgresTradeStore::new(&database_url).await.unwrap();
    store.run_migrations().await.unwrap();

    let trade = create_test_trade();

    // Test save
    store.save_trade(&trade).await.unwrap();

    // Test retrieve by token
    let trades = store.get_trades_by_token(&trade.base_token, 10).await.unwrap();
    assert!(!trades.is_empty());

    // Test time range query
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now();
    let trades = store.get_trades_in_timerange(start, end).await.unwrap();
    assert!(!trades.is_empty());
}

#[tokio::test]
async fn test_questdb_metrics_integration() {
    let client = QuestDbMetricsStore::new("localhost", 9000).await.unwrap();

    // Record some latencies
    for i in 0..100 {
        let latency = 10.0 + (i as f64 * 0.5);
        client.record_latency("test_operation", latency).await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Get P99 latency
    let p99 = client.get_p99_latency("test_operation", Duration::from_secs(60))
        .await
        .unwrap();

    assert!(p99 > 50.0); // Should be around 59.5
}

#[tokio::test]
async fn test_redis_cache_integration() {
    let redis_url = std::env::var("TEST_REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache = RedisPriceCache::new(&redis_url, Duration::from_secs(2)).await.unwrap();

    let token = Pubkey::new_unique();
    let price = 123.45;

    // Cache price
    cache.cache_price(&token, price, Duration::from_secs(60)).await.unwrap();

    // Retrieve price
    let cached_price = cache.get_price(&token).await.unwrap();
    assert_eq!(cached_price, Some(price));

    // Test batch get
    let tokens = vec![token, Pubkey::new_unique(), Pubkey::new_unique()];
    let prices = cache.get_prices_batch(&tokens).await.unwrap();
    assert_eq!(prices.get(&token), Some(&price));
}

#[tokio::test]
async fn test_database_factory() {
    let config = DatabaseConfig {
        postgres_url: "postgres://test:test@localhost/test".to_string(),
        postgres_max_connections: 10,
        questdb_host: "localhost".to_string(),
        questdb_port: 9000,
        redis_url: "redis://localhost:6379".to_string(),
        trade_store_type: StoreType::Mock,
        metrics_store_type: StoreType::Mock,
        cache_type: StoreType::Mock,
        config_store_type: StoreType::Mock,
    };

    let trade_store = DatabaseFactory::create_trade_store(&config).await.unwrap();
    let metrics_store = DatabaseFactory::create_metrics_store(&config).await.unwrap();
    let price_cache = DatabaseFactory::create_price_cache(&config).await.unwrap();
    let config_store = DatabaseFactory::create_config_store(&config).await.unwrap();

    // All should be mock implementations
    assert!(trade_store.save_trade(&create_test_trade()).await.is_ok());
    assert!(metrics_store.record_latency("test", 10.0).await.is_ok());
    assert!(price_cache.cache_price(&Pubkey::new_unique(), 100.0, Duration::from_secs(60)).await.is_ok());
    assert!(config_store.list_trader_configs().await.is_ok());
}
```

### Performance Tests

```rust
#[tokio::test]
async fn test_batch_write_performance() {
    let store = QuestDbTradeStore::new("localhost", 9000).await.unwrap();

    let trades: Vec<Trade> = (0..10000)
        .map(|i| {
            let mut trade = create_test_trade();
            trade.timestamp = Utc::now() - Duration::seconds(i);
            trade
        })
        .collect();

    let start = Instant::now();
    store.batch_save_trades(&trades).await.unwrap();
    let elapsed = start.elapsed();

    println!("Batch wrote 10,000 trades in {:?}", elapsed);
    assert!(elapsed < Duration::from_secs(1)); // Should be much faster
}

#[tokio::test]
async fn test_concurrent_database_access() {
    let store = Arc::new(MockTradeStore::new());
    let mut handles = vec![];

    // Spawn 100 concurrent operations
    for i in 0..100 {
        let store_clone = store.clone();
        let handle = tokio::spawn(async move {
            let mut trade = create_test_trade();
            trade.id = Uuid::new_v4();
            trade.amount = i as f64;
            store_clone.save_trade(&trade).await
        });
        handles.push(handle);
    }

    // All should complete successfully
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_cache_performance() {
    let cache = MockPriceCache::new();
    let tokens: Vec<Pubkey> = (0..1000).map(|_| Pubkey::new_unique()).collect();

    // Populate cache
    for (i, token) in tokens.iter().enumerate() {
        cache.cache_price(token, i as f64, Duration::from_secs(60)).await.unwrap();
    }

    // Measure batch get performance
    let start = Instant::now();
    let prices = cache.get_prices_batch(&tokens).await.unwrap();
    let elapsed = start.elapsed();

    assert_eq!(prices.len(), tokens.len());
    println!("Batch retrieved 1000 prices in {:?}", elapsed);
    assert!(elapsed < Duration::from_millis(10));
}
```

## Dependencies

- **Task 2**: Database setup and schemas

## Integration Points

- **Paper Trader**: Uses trait abstractions for trade storage
- **Live Trader**: Records real trades through same interfaces
- **Monitoring System**: Queries metrics via trait methods
- **TUI**: Reads data through abstraction layer
- **Risk Manager**: Accesses configuration via traits

## Performance Considerations

- Batch operations for high-throughput scenarios
- Connection pooling for all databases
- Efficient serialization with minimal allocations
- Asynchronous operations throughout
- Caching layer for frequently accessed data

## Security Considerations

- Connection strings stored securely
- Prepared statements prevent SQL injection
- Connection encryption for all databases
- Audit trail for configuration changes
- No sensitive data in error messages

## Future Enhancements

- Read replicas for PostgreSQL scaling
- Sharding support for QuestDB
- Redis cluster support
- Change data capture (CDC) integration
- GraphQL API over trait abstractions
- Multi-region database support