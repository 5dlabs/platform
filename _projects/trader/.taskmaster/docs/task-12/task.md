# Task 12: Implement QuestDB Integration for Trade Data and Metrics

## Overview
This task implements a high-performance time-series data storage system using QuestDB for recording trade data, performance metrics, and latency measurements. The system uses batch writing with 100ms intervals and maintains a 30-day retention policy as specified in the PRD.

## Architecture Context
According to the architecture.md, QuestDB serves as the primary time-series database for:
- Complete trade history with MEV protection status
- Performance metrics including latency measurements
- System health data and circuit breaker events
- 30-day retention at full resolution for analysis

## Implementation Requirements

### 1. QuestDB Client with Connection Pooling

Implement a robust client with efficient batch writing:

```rust
use questdb::{QuestDbConnectionManager, Pool};
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct QuestDbClient {
    pool: Pool<QuestDbConnectionManager>,
    batch_writer: Arc<BatchWriter>,
    config: QuestDbConfig,
}

pub struct QuestDbConfig {
    connection_string: String,
    max_pool_size: u32,
    batch_interval_ms: u64,    // 100ms as per PRD
    max_batch_size: usize,     // Maximum items per batch
    retention_days: u32,       // 30 days as per PRD
}

impl QuestDbClient {
    pub async fn new(config: QuestDbConfig) -> Result<Self> {
        // Create connection pool
        let manager = QuestDbConnectionManager::new(&config.connection_string)?;
        let pool = Pool::builder()
            .max_size(config.max_pool_size)
            .connection_timeout(Duration::from_secs(5))
            .idle_timeout(Some(Duration::from_secs(300)))
            .build(manager)?;
        
        // Initialize database schema
        Self::initialize_schema(&pool).await?;
        
        // Create and start batch writer
        let batch_writer = Arc::new(BatchWriter::new(pool.clone(), config.batch_interval_ms));
        let writer_handle = batch_writer.start_background_task();
        
        Ok(Self {
            pool,
            batch_writer,
            config,
        })
    }
    
    async fn initialize_schema(pool: &Pool<QuestDbConnectionManager>) -> Result<()> {
        let mut conn = pool.get().await?;
        
        // Create trades table with all MEV fields
        conn.execute("
            CREATE TABLE IF NOT EXISTS trades (
                timestamp TIMESTAMP,
                trader_id SYMBOL,
                mode SYMBOL,
                action SYMBOL,
                base_token SYMBOL,
                quote_token SYMBOL,
                base_amount DOUBLE,
                quote_amount DOUBLE,
                executed_price DOUBLE,
                transaction_fee LONG,
                priority_fee LONG,
                expected_slippage DOUBLE,
                actual_slippage DOUBLE,
                mev_status SYMBOL,
                mev_protected BOOLEAN,
                transfer_fees DOUBLE,
                tx_signature STRING,
                metadata STRING
            ) TIMESTAMP(timestamp) PARTITION BY DAY WITH maxUncommittedRows=10000;
        ").await?;
        
        // Create performance metrics table
        conn.execute("
            CREATE TABLE IF NOT EXISTS performance_metrics (
                timestamp TIMESTAMP,
                metric_type SYMBOL,
                operation SYMBOL,
                value DOUBLE,
                percentile INT,
                tags STRING
            ) TIMESTAMP(timestamp) PARTITION BY DAY WITH maxUncommittedRows=5000;
        ").await?;
        
        // Create MEV metrics table
        conn.execute("
            CREATE TABLE IF NOT EXISTS mev_metrics (
                timestamp TIMESTAMP,
                token_pair SYMBOL,
                trade_size DOUBLE,
                sandwich_probability DOUBLE,
                estimated_loss_bps INT,
                actual_loss_bps INT,
                protection_used BOOLEAN,
                priority_fee_lamports LONG,
                avoided BOOLEAN
            ) TIMESTAMP(timestamp) PARTITION BY DAY WITH maxUncommittedRows=5000;
        ").await?;
        
        // Create latency tracking table
        conn.execute("
            CREATE TABLE IF NOT EXISTS latency_metrics (
                timestamp TIMESTAMP,
                service SYMBOL,
                operation SYMBOL,
                latency_ms LONG,
                success BOOLEAN,
                error_type SYMBOL
            ) TIMESTAMP(timestamp) PARTITION BY HOUR WITH maxUncommittedRows=20000;
        ").await?;
        
        Ok(())
    }
}
```

### 2. Batch Writer Implementation

Efficient batch writing with 100ms intervals:

```rust
pub struct BatchWriter {
    queue: Arc<Mutex<BatchQueue>>,
    pool: Pool<QuestDbConnectionManager>,
    interval_ms: u64,
}

pub struct BatchQueue {
    trades: Vec<TradeRecord>,
    metrics: Vec<MetricRecord>,
    mev_events: Vec<MevRecord>,
    latencies: Vec<LatencyRecord>,
}

pub enum BatchItem {
    Trade(TradeRecord),
    Metric(MetricRecord),
    MevEvent(MevRecord),
    Latency(LatencyRecord),
}

impl BatchWriter {
    pub fn new(pool: Pool<QuestDbConnectionManager>, interval_ms: u64) -> Self {
        Self {
            queue: Arc::new(Mutex::new(BatchQueue::default())),
            pool,
            interval_ms,
        }
    }
    
    pub async fn add(&self, item: BatchItem) -> Result<()> {
        let mut queue = self.queue.lock().await;
        
        match item {
            BatchItem::Trade(trade) => queue.trades.push(trade),
            BatchItem::Metric(metric) => queue.metrics.push(metric),
            BatchItem::MevEvent(mev) => queue.mev_events.push(mev),
            BatchItem::Latency(latency) => queue.latencies.push(latency),
        }
        
        // Check if we should flush early due to size
        if queue.should_flush_early() {
            drop(queue);
            self.flush().await?;
        }
        
        Ok(())
    }
    
    pub fn start_background_task(&self) -> JoinHandle<()> {
        let writer = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(writer.interval_ms));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            
            loop {
                interval.tick().await;
                
                let start = Instant::now();
                if let Err(e) = writer.flush().await {
                    error!("Batch write failed: {}", e);
                }
                
                let elapsed = start.elapsed();
                if elapsed > Duration::from_millis(50) {
                    warn!("Batch write took {}ms", elapsed.as_millis());
                }
            }
        })
    }
    
    async fn flush(&self) -> Result<()> {
        let queue = {
            let mut queue = self.queue.lock().await;
            std::mem::take(&mut *queue)
        };
        
        if queue.is_empty() {
            return Ok(());
        }
        
        let mut conn = self.pool.get().await?;
        let mut tx = conn.transaction().await?;
        
        // Batch insert trades
        if !queue.trades.is_empty() {
            let values: Vec<String> = queue.trades.iter()
                .map(|t| format!(
                    "('{}', '{}', '{}', '{}', '{}', '{}', {}, {}, {}, {}, {}, {}, {}, '{}', {}, {}, '{}', '{}')",
                    t.timestamp.to_rfc3339(),
                    t.trader_id,
                    t.mode,
                    t.action,
                    t.base_token,
                    t.quote_token,
                    t.base_amount,
                    t.quote_amount,
                    t.executed_price,
                    t.transaction_fee,
                    t.priority_fee.unwrap_or(0),
                    t.expected_slippage,
                    t.actual_slippage,
                    t.mev_status,
                    t.mev_protected,
                    t.transfer_fees.unwrap_or(0.0),
                    t.tx_signature.as_deref().unwrap_or(""),
                    t.metadata.as_deref().unwrap_or("{}")
                ))
                .collect();
            
            let query = format!(
                "INSERT INTO trades VALUES {}",
                values.join(", ")
            );
            
            tx.execute(&query).await?;
        }
        
        // Batch insert other record types similarly...
        
        tx.commit().await?;
        
        debug!("Flushed {} trades, {} metrics, {} MEV events, {} latencies",
            queue.trades.len(),
            queue.metrics.len(),
            queue.mev_events.len(),
            queue.latencies.len()
        );
        
        Ok(())
    }
}
```

### 3. Query Interfaces

Implement efficient query methods for analysis:

```rust
impl QuestDbClient {
    pub async fn get_trades(&self, filter: TradeFilter) -> Result<Vec<Trade>> {
        let mut conn = self.pool.get().await?;
        
        let mut query = String::from("SELECT * FROM trades WHERE 1=1");
        let mut params = Vec::new();
        
        if let Some(start) = filter.start_time {
            query.push_str(" AND timestamp >= $1");
            params.push(start.to_rfc3339());
        }
        
        if let Some(end) = filter.end_time {
            query.push_str(&format!(" AND timestamp <= ${}", params.len() + 1));
            params.push(end.to_rfc3339());
        }
        
        if let Some(token) = filter.token {
            query.push_str(&format!(" AND (base_token = ${} OR quote_token = ${})", 
                params.len() + 1, params.len() + 1));
            params.push(token);
        }
        
        if let Some(trader_id) = filter.trader_id {
            query.push_str(&format!(" AND trader_id = ${}", params.len() + 1));
            params.push(trader_id);
        }
        
        query.push_str(" ORDER BY timestamp DESC");
        
        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        let rows = conn.query(&query, &params).await?;
        
        // Map rows to Trade structs
        rows.into_iter()
            .map(|row| Trade::from_row(&row))
            .collect()
    }
    
    pub async fn get_performance_metrics(&self, 
        metric_type: &str, 
        duration: Duration
    ) -> Result<PerformanceStats> {
        let mut conn = self.pool.get().await?;
        
        let start_time = Utc::now() - duration;
        
        let query = "
            SELECT 
                percentile_cont(0.5) WITHIN GROUP (ORDER BY value) as p50,
                percentile_cont(0.95) WITHIN GROUP (ORDER BY value) as p95,
                percentile_cont(0.99) WITHIN GROUP (ORDER BY value) as p99,
                avg(value) as mean,
                min(value) as min,
                max(value) as max,
                count(*) as count
            FROM performance_metrics
            WHERE metric_type = $1 
                AND timestamp >= $2
        ";
        
        let row = conn.query_one(query, &[&metric_type, &start_time]).await?;
        
        Ok(PerformanceStats {
            p50: row.get("p50"),
            p95: row.get("p95"),
            p99: row.get("p99"),
            mean: row.get("mean"),
            min: row.get("min"),
            max: row.get("max"),
            count: row.get("count"),
        })
    }
    
    pub async fn get_mev_analysis(&self, duration: Duration) -> Result<MevAnalysis> {
        let mut conn = self.pool.get().await?;
        
        let start_time = Utc::now() - duration;
        
        let query = "
            SELECT 
                count(*) as total_trades,
                sum(CASE WHEN avoided THEN 1 ELSE 0 END) as avoided_count,
                avg(sandwich_probability) as avg_risk,
                sum(actual_loss_bps * trade_size / 10000) as total_loss_usd,
                avg(priority_fee_lamports) as avg_priority_fee
            FROM mev_metrics
            WHERE timestamp >= $1
        ";
        
        let row = conn.query_one(query, &[&start_time]).await?;
        
        Ok(MevAnalysis {
            total_trades: row.get("total_trades"),
            avoidance_rate: row.get::<i64>("avoided_count") as f64 / row.get::<i64>("total_trades") as f64,
            average_risk: row.get("avg_risk"),
            total_loss_usd: row.get("total_loss_usd"),
            average_priority_fee: row.get("avg_priority_fee"),
        })
    }
    
    pub async fn get_latency_percentiles(&self, 
        service: &str, 
        operation: &str,
        duration: Duration
    ) -> Result<LatencyPercentiles> {
        let mut conn = self.pool.get().await?;
        
        let start_time = Utc::now() - duration;
        
        let query = "
            SELECT 
                percentile_cont(0.5) WITHIN GROUP (ORDER BY latency_ms) as p50,
                percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms) as p95,
                percentile_cont(0.99) WITHIN GROUP (ORDER BY latency_ms) as p99,
                percentile_cont(0.999) WITHIN GROUP (ORDER BY latency_ms) as p999
            FROM latency_metrics
            WHERE service = $1 
                AND operation = $2
                AND timestamp >= $3
                AND success = true
        ";
        
        let row = conn.query_one(query, &[&service, &operation, &start_time]).await?;
        
        Ok(LatencyPercentiles {
            p50: Duration::from_millis(row.get("p50")),
            p95: Duration::from_millis(row.get("p95")),
            p99: Duration::from_millis(row.get("p99")),
            p999: Duration::from_millis(row.get("p999")),
        })
    }
}
```

### 4. Data Retention Management

Implement 30-day retention policy:

```rust
impl QuestDbClient {
    pub async fn start_retention_manager(&self) -> JoinHandle<()> {
        let pool = self.pool.clone();
        let retention_days = self.config.retention_days;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_hours(24));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::cleanup_old_data(&pool, retention_days).await {
                    error!("Retention cleanup failed: {}", e);
                }
            }
        })
    }
    
    async fn cleanup_old_data(pool: &Pool<QuestDbConnectionManager>, retention_days: u32) -> Result<()> {
        let mut conn = pool.get().await?;
        let cutoff_date = Utc::now() - Duration::from_days(retention_days as i64);
        
        // Drop old partitions for each table
        let tables = ["trades", "performance_metrics", "mev_metrics", "latency_metrics"];
        
        for table in &tables {
            let query = format!(
                "ALTER TABLE {} DROP PARTITION WHERE timestamp < '{}'",
                table,
                cutoff_date.format("%Y-%m-%d")
            );
            
            match conn.execute(&query).await {
                Ok(_) => info!("Dropped old partitions from {}", table),
                Err(e) => warn!("Failed to drop partitions from {}: {}", table, e),
            }
        }
        
        Ok(())
    }
}
```

### 5. Performance Optimization

Optimize for high-throughput writes:

1. **Partition by Day**: Enables efficient data retention and queries
2. **Batch Writes**: Reduce connection overhead with 100ms batches
3. **Connection Pooling**: Reuse connections for better performance
4. **Async Operations**: Non-blocking database operations
5. **Index Strategy**: Appropriate indexes for common queries

## Error Handling

```rust
pub enum QuestDbError {
    ConnectionError(String),
    SchemaError(String),
    WriteError(String),
    QueryError(String),
    PoolExhausted,
}

impl QuestDbClient {
    async fn handle_write_error(&self, error: QuestDbError, items: BatchQueue) {
        match error {
            QuestDbError::ConnectionError(_) => {
                // Retry with exponential backoff
                self.retry_queue.push(items).await;
            }
            QuestDbError::WriteError(msg) => {
                // Log and potentially dead-letter
                error!("Permanent write error: {}", msg);
                self.dead_letter_queue.push(items).await;
            }
            _ => {
                error!("Unhandled QuestDB error: {:?}", error);
            }
        }
    }
}
```

## Testing Strategy

1. **Unit Tests**: Test batch queue operations and data transformations
2. **Integration Tests**: Verify schema creation and data persistence
3. **Performance Tests**: Benchmark write throughput and query latency
4. **Load Tests**: Verify 100ms batch intervals under high load
5. **Retention Tests**: Verify 30-day cleanup works correctly

## Dependencies
- Task 2: Data Models (Trade, MEV, and metric structures)
- Task 9: Paper Trade Executor (source of trade data)