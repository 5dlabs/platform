# Task 15: Implement Performance Benchmarking and Monitoring Tools - Autonomous Prompt

You are implementing comprehensive performance monitoring and benchmarking tools for the Solana trading platform. The system must track critical metrics, provide real-time monitoring, and alert when performance degrades below acceptable thresholds.

## Context
- The platform requires sub-100ms trade execution for competitiveness
- Node latency must stay below 200ms P99 to prevent circuit breaker activation
- MEV protection effectiveness directly impacts trading profitability
- Real-time monitoring enables rapid response to performance issues
- Benchmarking validates optimizations and tracks improvements

## Your Task

Implement a complete performance monitoring system with the following components:

### 1. Metric Collection Infrastructure
Create comprehensive metric collectors using Prometheus:

**Latency Metrics**:
```rust
// Track with histograms (buckets in seconds)
- trade_execution_latency: [0.01, 0.025, 0.05, 0.1, 0.2, 0.5, 1.0]
- solana_node_latency: [0.001, 0.005, 0.01, 0.05, 0.1, 0.2]
- jupiter_api_latency: [0.01, 0.05, 0.1, 0.2, 0.5]
- database_write_latency: [0.001, 0.005, 0.01, 0.05]
```

**Trading Metrics**:
```rust
// Counters and gauges
- trades_total{status="success|failed"}
- slippage_error_ratio (histogram)
- mev_attacks_avoided (counter)
- mev_attacks_impacted (counter)
- priority_fees_paid (counter)
```

**System Metrics**:
```rust
// Real-time system health
- circuit_breaker_state (0=closed, 1=open, 2=half-open)
- active_connections{type="questdb|postgres|redis"}
- memory_usage_bytes
- cpu_usage_percent
```

### 2. Latency Tracking System
Implement precise latency measurement:

```rust
// Wrap all async operations
let result = latency_tracker.track("operation_name", async {
    // Actual operation
}).await?;

// Automatic percentile calculation
// Store P50, P95, P99, P999 every minute
// Alert if P99 > threshold
```

### 3. MEV Protection Monitoring
Track MEV avoidance effectiveness:

**Metrics to Track**:
- Sandwich attack detection rate
- Successful avoidance percentage (target: 80%+)
- Average loss when impacted
- Priority fee ROI calculation
- Optimal fee recommendations

**Analysis Features**:
```rust
// Real-time MEV analysis
- Track by token pair
- Correlate with trade size
- Identify vulnerability patterns
- Calculate protection cost/benefit
```

### 4. Real-time Dashboard Updates
Push metrics to Redis for live dashboards:

**Update Frequency**: Every 5 seconds
**Data Points**:
```rust
dashboard:trade_latency_p99 -> milliseconds
dashboard:node_latency_p99 -> milliseconds
dashboard:slippage_accuracy -> percentage
dashboard:mev_avoidance_rate -> percentage
dashboard:trades_per_minute -> count
dashboard:success_rate -> percentage
dashboard:circuit_breaker_state -> enum
dashboard:active_positions -> count
dashboard:total_pnl -> percentage
```

**Time Series Storage**:
```rust
// Store for charting (7-day retention)
dashboard:timeseries:YYYYMMDD -> ZADD with timestamp scores
```

### 5. Alerting System
Implement multi-channel alerts:

**Alert Conditions**:
```rust
// Critical Alerts
- Node latency P99 > 200ms
- Circuit breaker activated
- Trade failure rate > 10%
- Database connection lost

// Warning Alerts
- MEV avoidance rate < 80%
- Trade failure rate > 5%
- Memory usage > 80%
- Slippage accuracy < 70%

// Info Alerts
- Daily performance summary
- Benchmark results available
```

**Alert Channels**:
- Console logging (always enabled)
- File logging with rotation
- Webhook notifications (configurable)
- Metrics endpoint for Grafana

**Suppression Logic**:
- Same alert suppressed for 5 minutes
- Escalation after 3 occurrences
- Clear notification when resolved

### 6. Performance Benchmarking
Create standardized benchmarks:

**Benchmark Scenarios**:
```rust
1. Trade Throughput Benchmark
   - Execute 1000 trades
   - Vary concurrency (1, 10, 50, 100)
   - Measure operations/second
   - Track latency percentiles

2. Latency Under Load
   - Sustained 100 trades/second
   - Measure latency degradation
   - Identify bottlenecks
   
3. MEV Simulation Accuracy
   - Compare predicted vs actual MEV
   - Validate protection effectiveness
   
4. Database Performance
   - Batch write throughput
   - Query response times
   - Connection pool efficiency
```

**Benchmark Output**:
```rust
pub struct BenchmarkResult {
    scenario_name: String,
    duration: Duration,
    operations_count: u64,
    operations_per_second: f64,
    latency_p50: Duration,
    latency_p95: Duration,
    latency_p99: Duration,
    error_rate: f64,
    recommendations: Vec<String>,
}
```

### 7. Metrics Endpoint
Expose Prometheus-compatible endpoint:

```
GET /metrics

# HELP trade_execution_latency_seconds Trade execution latency
# TYPE trade_execution_latency_seconds histogram
trade_execution_latency_seconds_bucket{le="0.01"} 245
trade_execution_latency_seconds_bucket{le="0.025"} 892
...

# HELP mev_attacks_avoided Number of MEV attacks successfully avoided
# TYPE mev_attacks_avoided counter
mev_attacks_avoided 1823
```

## Technical Requirements

1. **Non-blocking**: Metric collection must not impact trading performance
2. **Thread-safe**: All collectors must handle concurrent access
3. **Efficient Storage**: Use circular buffers for in-memory metrics
4. **Batch Operations**: Write to QuestDB in batches
5. **Graceful Degradation**: Continue operating if monitoring fails

## Success Criteria

Your implementation will be considered complete when:
1. All specified metrics are collected accurately
2. Dashboard updates every 5 seconds reliably
3. Alerts trigger at correct thresholds
4. Benchmarks provide actionable insights
5. Monitoring overhead is <1% CPU
6. Memory usage is stable over 24 hours
7. Integration with Prometheus/Grafana works

## Example Usage

```rust
// Initialize monitoring
let monitor = PerformanceMonitor::new(quest_db, redis, registry)?;

// Start automatic monitoring
monitor.start_monitoring().await?;

// Record custom metric
monitor.record_trade_execution(start_time, success);

// Run benchmarks
let results = monitor.run_benchmarks(vec![
    Box::new(TradeThroughputBenchmark::new(1000, 10)),
    Box::new(LatencyUnderLoadBenchmark::new(Duration::from_secs(60))),
]).await?;

// Check current performance
let metrics = monitor.get_current_metrics().await?;
if metrics.trade_latency_p99 > Duration::from_millis(100) {
    warn!("Performance degradation detected");
}
```

## Alert Examples

```
[CRITICAL] High Node Latency Detected
Solana node P99 latency is 245ms (threshold: 200ms)
Impact: Circuit breaker may activate
Action: Check node health or switch to backup

[WARNING] Low MEV Avoidance Rate
MEV avoidance rate dropped to 72.3% (target: 80%+)
Recent losses: $1,234 over 500 trades
Recommendation: Increase priority fees for large trades
```

## Dashboard Integration

The monitoring system should provide data that can be visualized in dashboards showing:
- Real-time latency graphs
- Trade success/failure rates
- MEV protection effectiveness
- System resource usage
- Historical performance trends