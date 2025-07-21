# Task 12: Implement QuestDB Integration for Trade Data and Metrics - Autonomous Prompt

You are implementing a high-performance time-series data storage system using QuestDB for the Solana trading platform. The system must handle trade data, performance metrics, and latency measurements with 100ms batch writing intervals and maintain a 30-day retention policy.

## Context
- QuestDB is chosen for its exceptional time-series performance and SQL compatibility
- The system must handle high-frequency trading data without impacting trade execution
- All data must be retained for 30 days at full resolution for analysis
- Batch writing at 100ms intervals balances performance with data freshness

## Your Task

Implement a comprehensive QuestDB integration with the following components:

### 1. Connection Pool and Client
Create a robust QuestDB client that:
- Maintains a connection pool with configurable size (5-10 connections)
- Handles connection failures with automatic retry
- Implements health checks to detect database issues
- Provides both synchronous and asynchronous interfaces

### 2. Schema Design and Initialization
Define optimized time-series tables:

**Trades Table**:
```sql
CREATE TABLE trades (
    timestamp TIMESTAMP,
    trader_id SYMBOL,
    mode SYMBOL,              -- 'paper' or 'live'
    action SYMBOL,            -- 'buy', 'sell', 'swap'
    base_token SYMBOL,
    quote_token SYMBOL,
    base_amount DOUBLE,
    quote_amount DOUBLE,
    executed_price DOUBLE,
    transaction_fee LONG,
    priority_fee LONG,        -- MEV protection fee
    expected_slippage DOUBLE,
    actual_slippage DOUBLE,
    mev_status SYMBOL,        -- 'protected', 'at_risk', 'impacted'
    transfer_fees DOUBLE,     -- Token-2022 fees
    tx_signature STRING,
    metadata STRING           -- JSON for extensibility
) TIMESTAMP(timestamp) PARTITION BY DAY;
```

**Performance Metrics Table**:
```sql
CREATE TABLE performance_metrics (
    timestamp TIMESTAMP,
    metric_type SYMBOL,       -- 'latency', 'throughput', 'error_rate'
    operation SYMBOL,         -- 'trade_execution', 'price_fetch', etc.
    value DOUBLE,
    percentile INT,          -- 50, 95, 99
    tags STRING              -- JSON tags
) TIMESTAMP(timestamp) PARTITION BY DAY;
```

**MEV Metrics Table**:
```sql
CREATE TABLE mev_metrics (
    timestamp TIMESTAMP,
    token_pair SYMBOL,
    trade_size DOUBLE,
    sandwich_probability DOUBLE,
    estimated_loss_bps INT,
    actual_loss_bps INT,
    protection_used BOOLEAN,
    priority_fee_lamports LONG,
    avoided BOOLEAN
) TIMESTAMP(timestamp) PARTITION BY DAY;
```

### 3. Batch Writer System
Implement efficient batch writing:
- Buffer incoming records in memory
- Flush to database every 100ms
- Handle multiple record types in same batch
- Implement early flush on buffer size limits
- Use prepared statements for performance
- Track batch write latency

### 4. Query Interfaces
Provide comprehensive query methods:

```rust
// Get trades with filtering
get_trades(filter: TradeFilter) -> Result<Vec<Trade>>

// Get performance statistics
get_performance_metrics(metric_type: &str, duration: Duration) -> Result<PerformanceStats>

// Analyze MEV protection effectiveness
get_mev_analysis(duration: Duration) -> Result<MevAnalysis>

// Get latency percentiles
get_latency_percentiles(service: &str, operation: &str, duration: Duration) -> Result<LatencyPercentiles>

// Custom SQL queries
execute_query(sql: &str, params: &[Value]) -> Result<QueryResult>
```

### 5. Data Structures
```rust
pub struct TradeRecord {
    timestamp: DateTime<Utc>,
    trader_id: String,
    mode: TradingMode,
    action: TradeAction,
    base_token: String,
    quote_token: String,
    base_amount: f64,
    quote_amount: f64,
    executed_price: f64,
    transaction_fee: u64,
    priority_fee: Option<u64>,
    expected_slippage: f64,
    actual_slippage: f64,
    mev_status: MevStatus,
    mev_protected: bool,
    transfer_fees: Option<f64>,
    tx_signature: Option<String>,
    metadata: Option<String>,
}

pub struct BatchQueue {
    trades: Vec<TradeRecord>,
    metrics: Vec<MetricRecord>,
    mev_events: Vec<MevRecord>,
    latencies: Vec<LatencyRecord>,
    total_size: usize,
    max_size: usize,
}
```

### 6. Retention Management
Implement automatic data cleanup:
- Run retention job daily at low-activity hours
- Drop partitions older than 30 days
- Log retention operations for audit
- Handle partial failures gracefully
- Monitor disk space usage

### 7. Performance Requirements
Your implementation must achieve:
- Write throughput: >10,000 records/second
- Batch write latency: <50ms for 1000 records
- Query response time: <100ms for 24-hour data
- Connection pool efficiency: >90% utilization
- Memory usage: <100MB for batch buffers

## Technical Requirements

1. **Concurrency**: Use Arc<Mutex<>> for thread-safe batch queue
2. **Error Handling**: Implement retry logic with exponential backoff
3. **Monitoring**: Track batch sizes, write latencies, and errors
4. **Testing**: Include benchmarks for write/read performance
5. **Configuration**: Support environment-based configuration

## Integration Points

- **Paper Trade Executor**: Receives trade records to persist
- **Performance Monitor**: Receives latency and throughput metrics
- **MEV Simulator**: Receives MEV analysis data
- **Correlation Analyzer**: Queries historical data for analysis

## Success Criteria

Your implementation will be considered complete when:
1. Schema initialization completes without errors
2. Batch writer maintains 100ms intervals under load
3. Write throughput exceeds 10,000 records/second
4. Query interfaces return accurate results
5. 30-day retention policy executes automatically
6. Connection pool handles failures gracefully
7. Memory usage remains stable over 24 hours

## Error Scenarios to Handle

1. **Database Unavailable**: Queue writes locally, retry when available
2. **Slow Writes**: Log warning, consider increasing batch interval
3. **Query Timeouts**: Implement query timeout, return partial results
4. **Disk Full**: Alert and stop accepting new writes
5. **Corrupt Data**: Validate before write, log rejected records

## Example Usage
```rust
// Initialize client
let config = QuestDbConfig {
    connection_string: "http://localhost:9000",
    max_pool_size: 10,
    batch_interval_ms: 100,
    max_batch_size: 1000,
    retention_days: 30,
};
let client = QuestDbClient::new(config).await?;

// Record a trade
client.record_trade(trade_record).await?;

// Query recent trades
let filter = TradeFilter {
    start_time: Some(Utc::now() - Duration::hours(1)),
    token: Some("SOL".to_string()),
    limit: Some(100),
};
let trades = client.get_trades(filter).await?;

// Get performance metrics
let metrics = client.get_performance_metrics(
    "trade_execution",
    Duration::hours(24)
).await?;
```