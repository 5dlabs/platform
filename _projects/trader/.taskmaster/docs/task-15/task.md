# Task 15: Implement Performance Benchmarking and Monitoring Tools

## Overview
This task implements comprehensive performance monitoring and benchmarking tools for the Solana trading platform. The system tracks critical metrics including trade execution latency, slippage accuracy, MEV avoidance rates, and system health indicators, providing real-time monitoring and historical analysis capabilities.

## Architecture Context
According to the architecture.md and PRD, performance monitoring is essential for:
- Ensuring sub-100ms trade execution latency
- Tracking P99 latencies below 200ms threshold
- Monitoring MEV protection effectiveness (target: 80%+ avoidance)
- Triggering circuit breakers when performance degrades
- Validating paper trading accuracy through performance metrics

## Implementation Requirements

### 1. Performance Monitoring System

Core monitoring infrastructure with real-time metrics collection:

```rust
use prometheus::{Counter, Histogram, Gauge, Registry};
use std::time::Instant;

pub struct PerformanceMonitor {
    quest_db: Arc<QuestDbClient>,
    redis: Pool<RedisConnectionManager>,
    system_health: Arc<SystemHealth>,
    metrics_registry: Registry,
    collectors: MetricCollectors,
}

pub struct MetricCollectors {
    // Latency metrics
    trade_latency: Histogram,
    node_latency: Histogram,
    jupiter_latency: Histogram,
    db_write_latency: Histogram,
    
    // Trading metrics
    trades_total: Counter,
    trades_failed: Counter,
    slippage_error: Histogram,
    mev_avoided: Counter,
    mev_impacted: Counter,
    
    // System metrics
    active_connections: Gauge,
    circuit_breaker_state: Gauge,
    memory_usage: Gauge,
    cpu_usage: Gauge,
    
    // Business metrics
    portfolio_value: Gauge,
    pnl_percentage: Gauge,
    active_positions: Gauge,
}

impl PerformanceMonitor {
    pub fn new(
        quest_db: Arc<QuestDbClient>,
        redis: Pool<RedisConnectionManager>,
        system_health: Arc<SystemHealth>,
    ) -> Result<Self> {
        let registry = Registry::new();
        
        // Initialize Prometheus metrics
        let collectors = MetricCollectors {
            trade_latency: Histogram::with_opts(
                HistogramOpts::new("trade_execution_latency_seconds", "Trade execution latency")
                    .buckets(vec![0.01, 0.025, 0.05, 0.1, 0.2, 0.5, 1.0])
            )?,
            
            node_latency: Histogram::with_opts(
                HistogramOpts::new("solana_node_latency_seconds", "Solana node RPC latency")
                    .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.2])
            )?,
            
            jupiter_latency: Histogram::with_opts(
                HistogramOpts::new("jupiter_api_latency_seconds", "Jupiter API response time")
                    .buckets(vec![0.01, 0.05, 0.1, 0.2, 0.5])
            )?,
            
            trades_total: Counter::with_opts(
                CounterOpts::new("trades_total", "Total number of trades executed")
            )?,
            
            trades_failed: Counter::with_opts(
                CounterOpts::new("trades_failed", "Number of failed trades")
            )?,
            
            slippage_error: Histogram::with_opts(
                HistogramOpts::new("slippage_error_ratio", "Slippage prediction error")
                    .buckets(vec![0.0, 0.1, 0.2, 0.5, 1.0, 2.0])
            )?,
            
            mev_avoided: Counter::with_opts(
                CounterOpts::new("mev_attacks_avoided", "Number of MEV attacks avoided")
            )?,
            
            circuit_breaker_state: Gauge::with_opts(
                GaugeOpts::new("circuit_breaker_state", "Circuit breaker state (0=closed, 1=open, 2=half-open)")
            )?,
            
            // ... register other metrics
        };
        
        // Register all metrics
        registry.register(Box::new(collectors.trade_latency.clone()))?;
        registry.register(Box::new(collectors.node_latency.clone()))?;
        // ... register all collectors
        
        Ok(Self {
            quest_db,
            redis,
            system_health,
            metrics_registry: registry,
            collectors,
        })
    }
    
    pub fn record_trade_execution(&self, start_time: Instant, success: bool) {
        let duration = start_time.elapsed();
        self.collectors.trade_latency.observe(duration.as_secs_f64());
        
        if success {
            self.collectors.trades_total.inc();
        } else {
            self.collectors.trades_failed.inc();
        }
        
        // Alert if latency exceeds threshold
        if duration > Duration::from_millis(200) {
            warn!("Trade execution latency exceeded 200ms: {:?}", duration);
            self.system_health.record_high_latency_event(duration).await;
        }
    }
    
    pub fn record_slippage_accuracy(&self, expected: f64, actual: f64) {
        let error = ((expected - actual) / expected).abs();
        self.collectors.slippage_error.observe(error);
    }
    
    pub fn record_mev_outcome(&self, avoided: bool, priority_fee: u64) {
        if avoided {
            self.collectors.mev_avoided.inc();
        } else {
            self.collectors.mev_impacted.inc();
        }
        
        // Record priority fee metrics
        tokio::spawn(async move {
            let metric = MetricRecord {
                timestamp: Utc::now(),
                metric_type: "mev_priority_fee".to_string(),
                operation: "protection".to_string(),
                value: priority_fee as f64,
                percentile: None,
                tags: json!({"avoided": avoided}).to_string(),
            };
            
            self.quest_db.record_metric(metric).await?;
        });
    }
}
```

### 2. Latency Tracking System

Comprehensive latency measurement across all operations:

```rust
pub struct LatencyTracker {
    trackers: Arc<RwLock<HashMap<String, OperationTracker>>>,
    quest_db: Arc<QuestDbClient>,
}

pub struct OperationTracker {
    operation_name: String,
    samples: VecDeque<Duration>,
    max_samples: usize,
    last_flush: Instant,
}

impl LatencyTracker {
    pub async fn track<F, Fut, T>(
        &self,
        operation: &str,
        f: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        
        // Record to in-memory tracker
        {
            let mut trackers = self.trackers.write().await;
            let tracker = trackers.entry(operation.to_string())
                .or_insert_with(|| OperationTracker::new(operation));
            
            tracker.add_sample(duration);
            
            // Check if we should flush to database
            if tracker.should_flush() {
                let metrics = tracker.calculate_percentiles();
                self.flush_metrics(operation, metrics).await?;
                tracker.reset();
            }
        }
        
        // Record to Prometheus
        if let Some(histogram) = self.get_histogram(operation) {
            histogram.observe(duration.as_secs_f64());
        }
        
        result
    }
    
    async fn flush_metrics(&self, operation: &str, metrics: PercentileMetrics) -> Result<()> {
        let records = vec![
            MetricRecord {
                timestamp: Utc::now(),
                metric_type: "latency".to_string(),
                operation: operation.to_string(),
                value: metrics.p50.as_secs_f64() * 1000.0, // Convert to ms
                percentile: Some(50),
                tags: String::new(),
            },
            MetricRecord {
                timestamp: Utc::now(),
                metric_type: "latency".to_string(),
                operation: operation.to_string(),
                value: metrics.p95.as_secs_f64() * 1000.0,
                percentile: Some(95),
                tags: String::new(),
            },
            MetricRecord {
                timestamp: Utc::now(),
                metric_type: "latency".to_string(),
                operation: operation.to_string(),
                value: metrics.p99.as_secs_f64() * 1000.0,
                percentile: Some(99),
                tags: String::new(),
            },
        ];
        
        for record in records {
            self.quest_db.record_metric(record).await?;
        }
        
        // Check P99 against threshold
        if operation == "node_rpc" && metrics.p99 > Duration::from_millis(200) {
            self.trigger_latency_alert(operation, metrics.p99).await?;
        }
        
        Ok(())
    }
}

impl OperationTracker {
    fn calculate_percentiles(&self) -> PercentileMetrics {
        let mut samples: Vec<Duration> = self.samples.iter().cloned().collect();
        samples.sort();
        
        let len = samples.len();
        PercentileMetrics {
            p50: samples[len * 50 / 100],
            p95: samples[len * 95 / 100],
            p99: samples[len * 99 / 100],
            p999: samples[len * 999 / 1000],
            mean: Duration::from_nanos(
                (samples.iter().map(|d| d.as_nanos()).sum::<u128>() / len as u128) as u64
            ),
            count: len,
        }
    }
}
```

### 3. MEV Protection Monitoring

Track MEV avoidance effectiveness:

```rust
pub struct MevMonitor {
    quest_db: Arc<QuestDbClient>,
    metrics: Arc<MevMetrics>,
}

pub struct MevMetrics {
    total_trades: AtomicU64,
    avoided_count: AtomicU64,
    impacted_count: AtomicU64,
    total_loss_lamports: AtomicU64,
    total_fees_paid: AtomicU64,
}

impl MevMonitor {
    pub async fn record_trade_outcome(
        &self,
        trade: &Trade,
        mev_analysis: &MevAnalysis,
    ) -> Result<()> {
        self.metrics.total_trades.fetch_add(1, Ordering::Relaxed);
        
        let mev_record = MevRecord {
            timestamp: trade.timestamp,
            token_pair: format!("{}/{}", trade.base_token, trade.quote_token),
            trade_size: trade.base_amount,
            sandwich_probability: mev_analysis.sandwich_probability,
            estimated_loss_bps: mev_analysis.estimated_loss_bps as i32,
            actual_loss_bps: self.calculate_actual_loss(&trade)?,
            protection_used: trade.priority_fee.is_some(),
            priority_fee_lamports: trade.priority_fee.unwrap_or(0),
            avoided: trade.mev_status != "impacted",
        };
        
        if mev_record.avoided {
            self.metrics.avoided_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.metrics.impacted_count.fetch_add(1, Ordering::Relaxed);
            self.metrics.total_loss_lamports.fetch_add(
                (mev_record.actual_loss_bps as u64 * trade.base_amount as u64) / 10000,
                Ordering::Relaxed
            );
        }
        
        if let Some(fee) = trade.priority_fee {
            self.metrics.total_fees_paid.fetch_add(fee, Ordering::Relaxed);
        }
        
        self.quest_db.record_mev_event(mev_record).await?;
        
        Ok(())
    }
    
    pub async fn calculate_avoidance_rate(&self, duration: Duration) -> Result<f64> {
        let total = self.metrics.total_trades.load(Ordering::Relaxed);
        let avoided = self.metrics.avoided_count.load(Ordering::Relaxed);
        
        if total == 0 {
            return Ok(0.0);
        }
        
        let rate = avoided as f64 / total as f64;
        
        // Alert if avoidance rate drops below threshold
        if rate < 0.8 && total > 100 {
            self.send_alert(
                "Low MEV Avoidance Rate",
                &format!("Current rate: {:.2}% (target: 80%+)", rate * 100.0)
            ).await?;
        }
        
        Ok(rate)
    }
    
    pub async fn get_mev_analysis(&self, duration: Duration) -> Result<MevAnalysis> {
        let end_time = Utc::now();
        let start_time = end_time - duration;
        
        let analysis = self.quest_db.get_mev_analysis(duration).await?;
        
        // Calculate cost effectiveness
        let total_loss = self.metrics.total_loss_lamports.load(Ordering::Relaxed);
        let total_fees = self.metrics.total_fees_paid.load(Ordering::Relaxed);
        let roi = if total_fees > 0 {
            (total_loss as f64 - total_fees as f64) / total_fees as f64
        } else {
            0.0
        };
        
        Ok(MevAnalysis {
            avoidance_rate: analysis.avoidance_rate,
            average_risk: analysis.average_risk,
            total_loss_usd: analysis.total_loss_usd,
            average_priority_fee: analysis.average_priority_fee,
            protection_roi: roi,
            recommendations: self.generate_mev_recommendations(&analysis),
        })
    }
}
```

### 4. Real-time Dashboard Updates

Push metrics to Redis for dashboard consumption:

```rust
pub struct DashboardUpdater {
    redis: Pool<RedisConnectionManager>,
    update_interval: Duration,
}

impl DashboardUpdater {
    pub async fn start_updates(
        &self,
        monitor: Arc<PerformanceMonitor>,
    ) -> JoinHandle<()> {
        let redis = self.redis.clone();
        let interval = self.update_interval;
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            
            loop {
                ticker.tick().await;
                
                if let Err(e) = Self::update_dashboard(&redis, &monitor).await {
                    error!("Dashboard update failed: {}", e);
                }
            }
        })
    }
    
    async fn update_dashboard(
        redis: &Pool<RedisConnectionManager>,
        monitor: &PerformanceMonitor,
    ) -> Result<()> {
        let mut conn = redis.get().await?;
        
        // Collect current metrics
        let metrics = monitor.collect_current_metrics().await?;
        
        // Update Redis keys with TTL
        let updates = vec![
            ("dashboard:trade_latency_p99", metrics.trade_latency_p99.as_millis()),
            ("dashboard:node_latency_p99", metrics.node_latency_p99.as_millis()),
            ("dashboard:slippage_accuracy", (metrics.slippage_accuracy * 100.0) as u64),
            ("dashboard:mev_avoidance_rate", (metrics.mev_avoidance_rate * 100.0) as u64),
            ("dashboard:trades_per_minute", metrics.trades_per_minute),
            ("dashboard:success_rate", (metrics.success_rate * 100.0) as u64),
            ("dashboard:circuit_breaker_state", metrics.circuit_breaker_state as u64),
            ("dashboard:active_connections", metrics.active_connections),
        ];
        
        for (key, value) in updates {
            redis::cmd("SET")
                .arg(key)
                .arg(value)
                .arg("EX")
                .arg(30) // 30 second TTL
                .query_async(&mut conn)
                .await?;
        }
        
        // Publish update notification
        redis::cmd("PUBLISH")
            .arg("dashboard:metrics:updated")
            .arg(Utc::now().timestamp())
            .query_async(&mut conn)
            .await?;
        
        // Store time-series data for charts
        let ts_key = format!("dashboard:timeseries:{}", Utc::now().format("%Y%m%d"));
        redis::cmd("ZADD")
            .arg(&ts_key)
            .arg(Utc::now().timestamp())
            .arg(serde_json::to_string(&metrics)?)
            .query_async(&mut conn)
            .await?;
        
        // Expire old time-series data
        redis::cmd("EXPIRE")
            .arg(&ts_key)
            .arg(86400 * 7) // 7 days
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }
}
```

### 5. Alerting System

Proactive alerts for performance degradation:

```rust
pub struct AlertManager {
    alert_channels: Vec<Box<dyn AlertChannel>>,
    alert_history: Arc<Mutex<VecDeque<Alert>>>,
    suppression_window: Duration,
}

pub struct Alert {
    id: Uuid,
    timestamp: DateTime<Utc>,
    severity: AlertSeverity,
    title: String,
    message: String,
    metric_value: Option<f64>,
    threshold: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

pub trait AlertChannel: Send + Sync {
    async fn send_alert(&self, alert: &Alert) -> Result<()>;
}

impl AlertManager {
    pub async fn check_alert_conditions(&self, metrics: &PerformanceMetrics) -> Result<()> {
        // Node latency check
        if metrics.node_latency_p99 > Duration::from_millis(200) {
            self.send_alert(Alert {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                severity: AlertSeverity::Critical,
                title: "High Node Latency Detected".to_string(),
                message: format!(
                    "Solana node P99 latency is {}ms (threshold: 200ms)",
                    metrics.node_latency_p99.as_millis()
                ),
                metric_value: Some(metrics.node_latency_p99.as_millis() as f64),
                threshold: Some(200.0),
            }).await?;
        }
        
        // MEV avoidance rate check
        if metrics.mev_avoidance_rate < 0.8 {
            self.send_alert(Alert {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                severity: AlertSeverity::Warning,
                title: "Low MEV Avoidance Rate".to_string(),
                message: format!(
                    "MEV avoidance rate dropped to {:.1}% (target: 80%+)",
                    metrics.mev_avoidance_rate * 100.0
                ),
                metric_value: Some(metrics.mev_avoidance_rate),
                threshold: Some(0.8),
            }).await?;
        }
        
        // Circuit breaker activation
        if metrics.circuit_breaker_status == CircuitBreakerState::Open {
            self.send_alert(Alert {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                severity: AlertSeverity::Critical,
                title: "Circuit Breaker Activated".to_string(),
                message: "Trading has been paused due to system performance issues".to_string(),
                metric_value: None,
                threshold: None,
            }).await?;
        }
        
        // Trade failure rate
        if metrics.failure_rate > 0.05 {
            self.send_alert(Alert {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                severity: AlertSeverity::Warning,
                title: "High Trade Failure Rate".to_string(),
                message: format!(
                    "Trade failure rate is {:.1}% (threshold: 5%)",
                    metrics.failure_rate * 100.0
                ),
                metric_value: Some(metrics.failure_rate),
                threshold: Some(0.05),
            }).await?;
        }
        
        Ok(())
    }
    
    async fn send_alert(&self, alert: Alert) -> Result<()> {
        // Check suppression window to avoid alert spam
        if self.is_suppressed(&alert).await {
            return Ok(());
        }
        
        // Send through all configured channels
        for channel in &self.alert_channels {
            if let Err(e) = channel.send_alert(&alert).await {
                error!("Failed to send alert through channel: {}", e);
            }
        }
        
        // Record in history
        let mut history = self.alert_history.lock().await;
        history.push_back(alert);
        
        // Limit history size
        while history.len() > 1000 {
            history.pop_front();
        }
        
        Ok(())
    }
}
```

### 6. Benchmarking Tools

Performance benchmarking for optimization:

```rust
pub struct BenchmarkRunner {
    scenarios: Vec<Box<dyn BenchmarkScenario>>,
    results_store: Arc<QuestDbClient>,
}

#[async_trait]
pub trait BenchmarkScenario: Send + Sync {
    fn name(&self) -> &str;
    async fn setup(&mut self) -> Result<()>;
    async fn run(&self) -> Result<BenchmarkResult>;
    async fn teardown(&mut self) -> Result<()>;
}

pub struct BenchmarkResult {
    scenario_name: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    operations_count: u64,
    operations_per_second: f64,
    latency_p50: Duration,
    latency_p95: Duration,
    latency_p99: Duration,
    error_count: u64,
    custom_metrics: HashMap<String, f64>,
}

impl BenchmarkRunner {
    pub async fn run_all_benchmarks(&self) -> Result<Vec<BenchmarkResult>> {
        let mut results = Vec::new();
        
        for mut scenario in &mut self.scenarios {
            info!("Running benchmark: {}", scenario.name());
            
            // Setup
            scenario.setup().await?;
            
            // Run benchmark
            let result = scenario.run().await?;
            
            // Store results
            self.store_result(&result).await?;
            results.push(result);
            
            // Teardown
            scenario.teardown().await?;
        }
        
        // Generate comparison report
        self.generate_report(&results).await?;
        
        Ok(results)
    }
}

// Example benchmark scenario
pub struct TradeThroughputBenchmark {
    trade_executor: Arc<PaperTradeExecutor>,
    trade_count: usize,
    concurrent_trades: usize,
}

#[async_trait]
impl BenchmarkScenario for TradeThroughputBenchmark {
    fn name(&self) -> &str {
        "Trade Throughput Benchmark"
    }
    
    async fn run(&self) -> Result<BenchmarkResult> {
        let start = Instant::now();
        let mut handles = Vec::new();
        let latencies = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(AtomicU64::new(0));
        
        // Execute trades concurrently
        for batch in 0..(self.trade_count / self.concurrent_trades) {
            let mut batch_handles = Vec::new();
            
            for _ in 0..self.concurrent_trades {
                let executor = self.trade_executor.clone();
                let latencies = latencies.clone();
                let errors = errors.clone();
                
                let handle = tokio::spawn(async move {
                    let trade_start = Instant::now();
                    
                    let params = TradeParams {
                        action: TradeAction::Buy,
                        base_token: "SOL".to_string(),
                        quote_token: "USDC".to_string(),
                        amount: 1.0,
                        is_base_input: true,
                        simulate_mev: true,
                        priority_fee: 5000,
                        metadata: None,
                    };
                    
                    match executor.execute_trade(params).await {
                        Ok(_) => {
                            let duration = trade_start.elapsed();
                            latencies.lock().await.push(duration);
                        }
                        Err(_) => {
                            errors.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                });
                
                batch_handles.push(handle);
            }
            
            // Wait for batch to complete
            futures::future::join_all(batch_handles).await;
        }
        
        let duration = start.elapsed();
        let latencies = latencies.lock().await;
        let mut sorted_latencies = latencies.clone();
        sorted_latencies.sort();
        
        Ok(BenchmarkResult {
            scenario_name: self.name().to_string(),
            start_time: Utc::now() - chrono::Duration::from_std(duration).unwrap(),
            end_time: Utc::now(),
            operations_count: self.trade_count as u64,
            operations_per_second: self.trade_count as f64 / duration.as_secs_f64(),
            latency_p50: sorted_latencies[sorted_latencies.len() / 2],
            latency_p95: sorted_latencies[sorted_latencies.len() * 95 / 100],
            latency_p99: sorted_latencies[sorted_latencies.len() * 99 / 100],
            error_count: errors.load(Ordering::Relaxed),
            custom_metrics: HashMap::new(),
        })
    }
}
```

## Error Handling

```rust
pub enum MonitoringError {
    MetricRecordingFailed(String),
    AlertDeliveryFailed(String),
    DashboardUpdateFailed(String),
    BenchmarkFailed(String),
}

impl PerformanceMonitor {
    async fn handle_monitoring_error(&self, error: MonitoringError) {
        match error {
            MonitoringError::MetricRecordingFailed(msg) => {
                // Log locally and try alternative storage
                error!("Failed to record metric: {}", msg);
                self.fallback_metric_storage(msg).await;
            }
            MonitoringError::AlertDeliveryFailed(msg) => {
                // Escalate through alternative channels
                error!("Alert delivery failed: {}", msg);
                self.escalate_alert(msg).await;
            }
            _ => {}
        }
    }
}
```

## Testing Strategy

1. **Unit Tests**: Test metric calculations and percentile algorithms
2. **Integration Tests**: Verify end-to-end metric flow
3. **Load Tests**: Ensure monitoring doesn't impact performance
4. **Alert Tests**: Verify alerts trigger at correct thresholds
5. **Benchmark Tests**: Validate benchmark scenarios

## Dependencies
- Task 3: Solana Integration (for latency tracking)
- Task 6: System Health and Circuit Breaker
- Task 12: QuestDB Integration (for metric storage)