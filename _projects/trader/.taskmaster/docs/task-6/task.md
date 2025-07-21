# Task 6: Implement Circuit Breaker for Latency-Based Trading Pause

## Overview

This task implements a sophisticated circuit breaker system that automatically pauses trading when node latency exceeds the 200ms P99 threshold. The circuit breaker protects the trading platform from executing trades under degraded conditions, ensuring that performance issues don't lead to poor execution or financial losses.

## Architecture Context

The circuit breaker is a critical component of the resilient infrastructure layer:

- **Automated Protection**: Prevents trades when system performance degrades
- **P99 Latency Monitoring**: Real-time calculation of 99th percentile latency
- **Graceful Recovery**: Intelligent testing of system health before resuming
- **Integration Points**: Works with gRPC client, Jupiter failover, and trading engines

This implementation directly supports the PRD's requirement for maintaining sub-200ms latency and protecting against degraded performance scenarios.

## Implementation Details

### 1. Core Circuit Breaker State Machine

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,    // Normal operation - requests allowed
    Open,      // Circuit broken - requests blocked
    HalfOpen,  // Testing recovery - limited requests allowed
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,        // Number of failures to trigger open
    pub recovery_timeout: Duration,      // Time before attempting recovery
    pub half_open_requests: usize,       // Requests allowed in half-open state
    pub latency_threshold_ms: u64,       // 200ms as per PRD
    pub error_rate_threshold: f64,       // Error rate to trigger open (e.g., 0.5 = 50%)
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            half_open_requests: 3,
            latency_threshold_ms: 200,  // PRD requirement
            error_rate_threshold: 0.5,
        }
    }
}

pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitBreakerState>>,
    config: CircuitBreakerConfig,
    failure_count: AtomicUsize,
    success_count: AtomicUsize,
    last_failure_time: AtomicU64,
    half_open_request_count: AtomicUsize,
    state_listeners: Arc<RwLock<Vec<Box<dyn StateChangeListener + Send + Sync>>>>,
    metrics: Arc<CircuitBreakerMetrics>,
}

#[async_trait]
pub trait StateChangeListener: Send + Sync {
    async fn on_state_change(&self, from: CircuitBreakerState, to: CircuitBreakerState);
}

pub struct CircuitBreakerMetrics {
    state_changes: AtomicU64,
    total_requests: AtomicU64,
    blocked_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            config,
            failure_count: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            last_failure_time: AtomicU64::new(0),
            half_open_request_count: AtomicUsize::new(0),
            state_listeners: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(CircuitBreakerMetrics {
                state_changes: AtomicU64::new(0),
                total_requests: AtomicU64::new(0),
                blocked_requests: AtomicU64::new(0),
                successful_requests: AtomicU64::new(0),
                failed_requests: AtomicU64::new(0),
            }),
        }
    }

    pub async fn add_state_listener(&self, listener: Box<dyn StateChangeListener + Send + Sync>) {
        self.state_listeners.write().await.push(listener);
    }

    pub async fn can_execute(&self) -> bool {
        self.metrics.total_requests.fetch_add(1, Ordering::Relaxed);
        
        let current_state = *self.state.read().await;
        
        match current_state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if recovery timeout has elapsed
                let last_failure = self.last_failure_time.load(Ordering::Relaxed);
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                if now - last_failure >= self.config.recovery_timeout.as_secs() {
                    // Transition to half-open
                    self.transition_to_state(CircuitBreakerState::HalfOpen).await;
                    self.half_open_request_count.store(1, Ordering::Relaxed);
                    true
                } else {
                    self.metrics.blocked_requests.fetch_add(1, Ordering::Relaxed);
                    false
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Allow limited requests in half-open state
                let count = self.half_open_request_count.fetch_add(1, Ordering::Relaxed);
                if count < self.config.half_open_requests {
                    true
                } else {
                    self.metrics.blocked_requests.fetch_add(1, Ordering::Relaxed);
                    false
                }
            }
        }
    }

    pub async fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.metrics.successful_requests.fetch_add(1, Ordering::Relaxed);
        
        let current_state = *self.state.read().await;
        
        match current_state {
            CircuitBreakerState::HalfOpen => {
                // Successful request in half-open state - close the circuit
                self.failure_count.store(0, Ordering::Relaxed);
                self.success_count.store(0, Ordering::Relaxed);
                self.transition_to_state(CircuitBreakerState::Closed).await;
                
                tracing::info!("Circuit breaker closed after successful recovery");
            }
            CircuitBreakerState::Closed => {
                // Reset failure count on success in closed state
                if self.failure_count.load(Ordering::Relaxed) > 0 {
                    self.failure_count.store(0, Ordering::Relaxed);
                }
            }
            _ => {}
        }
    }

    pub async fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        self.metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
        
        self.last_failure_time.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed
        );
        
        let current_state = *self.state.read().await;
        
        match current_state {
            CircuitBreakerState::Closed => {
                // Check if we should open the circuit
                if failures >= self.config.failure_threshold {
                    self.transition_to_state(CircuitBreakerState::Open).await;
                    
                    tracing::warn!(
                        "Circuit breaker opened after {} failures",
                        failures
                    );
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Failed in half-open state - reopen the circuit
                self.transition_to_state(CircuitBreakerState::Open).await;
                self.half_open_request_count.store(0, Ordering::Relaxed);
                
                tracing::warn!("Circuit breaker reopened after failure in half-open state");
            }
            _ => {}
        }
    }

    pub async fn record_latency(&self, latency_ms: u64) {
        if latency_ms > self.config.latency_threshold_ms {
            // Treat high latency as a failure
            self.record_failure().await;
        } else {
            self.record_success().await;
        }
    }

    async fn transition_to_state(&self, new_state: CircuitBreakerState) {
        let mut state = self.state.write().await;
        let old_state = *state;
        
        if old_state != new_state {
            *state = new_state;
            self.metrics.state_changes.fetch_add(1, Ordering::Relaxed);
            
            // Notify listeners
            let listeners = self.state_listeners.read().await;
            for listener in listeners.iter() {
                listener.on_state_change(old_state, new_state).await;
            }
        }
    }

    pub async fn get_state(&self) -> CircuitBreakerState {
        *self.state.read().await
    }

    pub fn get_metrics(&self) -> CircuitBreakerSnapshot {
        CircuitBreakerSnapshot {
            state_changes: self.metrics.state_changes.load(Ordering::Relaxed),
            total_requests: self.metrics.total_requests.load(Ordering::Relaxed),
            blocked_requests: self.metrics.blocked_requests.load(Ordering::Relaxed),
            successful_requests: self.metrics.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.metrics.failed_requests.load(Ordering::Relaxed),
            current_failure_count: self.failure_count.load(Ordering::Relaxed),
        }
    }

    pub async fn reset(&self) {
        *self.state.write().await = CircuitBreakerState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        self.half_open_request_count.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CircuitBreakerSnapshot {
    pub state_changes: u64,
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub current_failure_count: usize,
}
```

### 2. Latency Monitoring System

```rust
use std::collections::VecDeque;
use std::sync::Mutex;

pub struct LatencyMonitor {
    window_size: usize,
    samples: Arc<Mutex<VecDeque<u64>>>,
    p99_threshold_ms: u64,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl LatencyMonitor {
    pub fn new(window_size: usize, p99_threshold_ms: u64, circuit_breaker: Arc<CircuitBreaker>) -> Self {
        Self {
            window_size,
            samples: Arc::new(Mutex::new(VecDeque::with_capacity(window_size))),
            p99_threshold_ms,
            circuit_breaker,
        }
    }

    pub async fn record_latency(&self, latency_ms: u64) {
        // Add to rolling window
        {
            let mut samples = self.samples.lock().unwrap();
            if samples.len() >= self.window_size {
                samples.pop_front();
            }
            samples.push_back(latency_ms);
        }

        // Calculate P99 and check threshold
        let p99 = self.calculate_p99();
        
        if p99 > self.p99_threshold_ms {
            tracing::warn!("P99 latency {}ms exceeds threshold {}ms", p99, self.p99_threshold_ms);
            self.circuit_breaker.record_failure().await;
        } else {
            self.circuit_breaker.record_success().await;
        }
    }

    pub fn calculate_p99(&self) -> u64 {
        let samples = self.samples.lock().unwrap();
        
        if samples.is_empty() {
            return 0;
        }

        let mut sorted: Vec<u64> = samples.iter().copied().collect();
        sorted.sort_unstable();

        let index = ((sorted.len() - 1) as f64 * 0.99) as usize;
        sorted[index]
    }

    pub fn get_statistics(&self) -> LatencyStatistics {
        let samples = self.samples.lock().unwrap();
        
        if samples.is_empty() {
            return LatencyStatistics::default();
        }

        let mut sorted: Vec<u64> = samples.iter().copied().collect();
        sorted.sort_unstable();

        let sum: u64 = sorted.iter().sum();
        let count = sorted.len();

        LatencyStatistics {
            count: count as u64,
            mean: sum / count as u64,
            min: sorted[0],
            max: sorted[count - 1],
            p50: sorted[count / 2],
            p95: sorted[((count - 1) as f64 * 0.95) as usize],
            p99: sorted[((count - 1) as f64 * 0.99) as usize],
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct LatencyStatistics {
    pub count: u64,
    pub mean: u64,
    pub min: u64,
    pub max: u64,
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
}

// Integration with trading operations
pub struct TradingCircuitBreaker {
    circuit_breaker: Arc<CircuitBreaker>,
    latency_monitor: Arc<LatencyMonitor>,
}

impl TradingCircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(config));
        let latency_monitor = Arc::new(LatencyMonitor::new(
            1000,  // Keep last 1000 samples
            200,   // 200ms P99 threshold from PRD
            circuit_breaker.clone(),
        ));

        Self {
            circuit_breaker,
            latency_monitor,
        }
    }

    pub async fn execute_with_breaker<F, T, E>(
        &self,
        operation: F,
    ) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        // Check if circuit allows execution
        if !self.circuit_breaker.can_execute().await {
            return Err(CircuitBreakerError::CircuitOpen);
        }

        let start = std::time::Instant::now();
        
        // Execute the operation
        match operation.await {
            Ok(result) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                self.latency_monitor.record_latency(latency_ms).await;
                Ok(result)
            }
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                Err(CircuitBreakerError::OperationFailed(e))
            }
        }
    }

    pub fn get_circuit_state(&self) -> impl Future<Output = CircuitBreakerState> {
        self.circuit_breaker.get_state()
    }

    pub fn get_latency_stats(&self) -> LatencyStatistics {
        self.latency_monitor.get_statistics()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open")]
    CircuitOpen,
    #[error("Operation failed: {0}")]
    OperationFailed(E),
}
```

### 3. Recovery Mechanism with Exponential Backoff

```rust
pub struct RecoveryPolicy {
    base_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
    jitter_factor: f64,
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(300),  // 5 minutes max
            multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

pub struct RecoveryManager {
    policy: RecoveryPolicy,
    circuit_breaker: Arc<CircuitBreaker>,
    health_checker: Arc<dyn HealthChecker + Send + Sync>,
    retry_count: AtomicUsize,
}

#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn check_health(&self) -> Result<HealthStatus, Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub latency_ms: Option<u64>,
    pub error_rate: Option<f64>,
    pub details: String,
}

impl RecoveryManager {
    pub fn new(
        policy: RecoveryPolicy,
        circuit_breaker: Arc<CircuitBreaker>,
        health_checker: Arc<dyn HealthChecker + Send + Sync>,
    ) -> Self {
        Self {
            policy,
            circuit_breaker,
            health_checker,
            retry_count: AtomicUsize::new(0),
        }
    }

    pub async fn start_recovery_loop(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                let state = self.circuit_breaker.get_state().await;
                
                if state == CircuitBreakerState::Open {
                    // Wait for recovery timeout
                    let delay = self.calculate_backoff_delay();
                    tokio::time::sleep(delay).await;
                    
                    // Attempt health check
                    match self.health_checker.check_health().await {
                        Ok(status) if status.is_healthy => {
                            tracing::info!("Health check passed, allowing circuit breaker recovery");
                            self.retry_count.store(0, Ordering::Relaxed);
                        }
                        Ok(status) => {
                            tracing::warn!("Health check failed: {}", status.details);
                            self.retry_count.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(e) => {
                            tracing::error!("Health check error: {}", e);
                            self.retry_count.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                } else {
                    // Not in open state, reset retry count
                    self.retry_count.store(0, Ordering::Relaxed);
                }
                
                // Check every 5 seconds when not recovering
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
    }

    fn calculate_backoff_delay(&self) -> Duration {
        let retry_count = self.retry_count.load(Ordering::Relaxed) as u32;
        
        // Exponential backoff with jitter
        let base_delay = self.policy.base_delay.as_millis() as f64;
        let delay = base_delay * self.policy.multiplier.powi(retry_count as i32);
        
        // Add jitter
        let jitter = delay * self.policy.jitter_factor * rand::random::<f64>();
        let final_delay = delay + jitter;
        
        // Cap at max delay
        let final_delay_ms = final_delay.min(self.policy.max_delay.as_millis() as f64) as u64;
        
        Duration::from_millis(final_delay_ms)
    }
}

// Example health checker implementation
pub struct SolanaHealthChecker {
    grpc_client: Arc<SolanaGrpcClient>,
}

#[async_trait]
impl HealthChecker for SolanaHealthChecker {
    async fn check_health(&self) -> Result<HealthStatus, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        
        // Perform a simple health check operation
        match self.grpc_client.get_slot().await {
            Ok(_) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                Ok(HealthStatus {
                    is_healthy: latency_ms < 200,  // PRD threshold
                    latency_ms: Some(latency_ms),
                    error_rate: None,
                    details: format!("Health check completed in {}ms", latency_ms),
                })
            }
            Err(e) => Ok(HealthStatus {
                is_healthy: false,
                latency_ms: None,
                error_rate: None,
                details: format!("Health check failed: {}", e),
            }),
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_state_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(1),
            ..Default::default()
        };
        
        let breaker = CircuitBreaker::new(config);
        
        // Initially closed
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Closed);
        assert!(breaker.can_execute().await);
        
        // Record failures
        for _ in 0..3 {
            breaker.record_failure().await;
        }
        
        // Should be open
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Open);
        assert!(!breaker.can_execute().await);
        
        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Should allow one request (half-open)
        assert!(breaker.can_execute().await);
        assert_eq!(breaker.get_state().await, CircuitBreakerState::HalfOpen);
        
        // Success closes the circuit
        breaker.record_success().await;
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_latency_monitoring() {
        let breaker = Arc::new(CircuitBreaker::new(CircuitBreakerConfig::default()));
        let monitor = LatencyMonitor::new(100, 200, breaker.clone());
        
        // Record various latencies
        for i in 1..=100 {
            monitor.record_latency(i * 2).await;
        }
        
        let stats = monitor.get_statistics();
        assert_eq!(stats.count, 100);
        assert_eq!(stats.min, 2);
        assert_eq!(stats.max, 200);
        assert_eq!(stats.p99, 198);  // 99th percentile
    }

    #[test]
    fn test_exponential_backoff() {
        let policy = RecoveryPolicy {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            jitter_factor: 0.0,  // No jitter for predictable testing
        };
        
        let breaker = Arc::new(CircuitBreaker::new(CircuitBreakerConfig::default()));
        let health_checker = Arc::new(MockHealthChecker { healthy: false });
        let recovery = RecoveryManager::new(policy, breaker, health_checker);
        
        // Test backoff progression
        recovery.retry_count.store(0, Ordering::Relaxed);
        assert_eq!(recovery.calculate_backoff_delay(), Duration::from_secs(1));
        
        recovery.retry_count.store(1, Ordering::Relaxed);
        assert_eq!(recovery.calculate_backoff_delay(), Duration::from_secs(2));
        
        recovery.retry_count.store(2, Ordering::Relaxed);
        assert_eq!(recovery.calculate_backoff_delay(), Duration::from_secs(4));
        
        recovery.retry_count.store(10, Ordering::Relaxed);
        assert_eq!(recovery.calculate_backoff_delay(), Duration::from_secs(60)); // Max
    }

    #[tokio::test]
    async fn test_trading_circuit_breaker() {
        let breaker = TradingCircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        });
        
        // Simulate successful operations
        for _ in 0..5 {
            let result = breaker.execute_with_breaker(async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok::<_, &str>("success")
            }).await;
            assert!(result.is_ok());
        }
        
        // Simulate high latency operations
        for _ in 0..3 {
            let result = breaker.execute_with_breaker(async {
                tokio::time::sleep(Duration::from_millis(250)).await;
                Ok::<_, &str>("slow")
            }).await;
            // First two should succeed, third should be blocked
        }
        
        // Circuit should be open
        assert_eq!(breaker.get_circuit_state().await, CircuitBreakerState::Open);
        
        // Further requests should be blocked
        let result = breaker.execute_with_breaker(async {
            Ok::<_, &str>("blocked")
        }).await;
        assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen)));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_circuit_breaker_with_real_latency() {
    // Create a mock service with variable latency
    let service = MockTradingService::new();
    let breaker = TradingCircuitBreaker::new(CircuitBreakerConfig::default());
    
    // Add state change listener
    let state_log = Arc::new(Mutex::new(Vec::new()));
    let log_clone = state_log.clone();
    
    breaker.circuit_breaker.add_state_listener(Box::new(LoggingListener {
        log: log_clone,
    })).await;
    
    // Simulate normal operations
    for _ in 0..10 {
        service.set_latency(Duration::from_millis(50));
        let _ = breaker.execute_with_breaker(service.execute()).await;
    }
    
    // Simulate latency spike
    for _ in 0..10 {
        service.set_latency(Duration::from_millis(300));
        let _ = breaker.execute_with_breaker(service.execute()).await;
    }
    
    // Verify state transitions
    let log = state_log.lock().unwrap();
    assert!(log.contains(&(CircuitBreakerState::Closed, CircuitBreakerState::Open)));
    
    // Verify metrics
    let metrics = breaker.circuit_breaker.get_metrics();
    assert!(metrics.blocked_requests > 0);
    assert!(metrics.failed_requests >= 5);
}

struct LoggingListener {
    log: Arc<Mutex<Vec<(CircuitBreakerState, CircuitBreakerState)>>>,
}

#[async_trait]
impl StateChangeListener for LoggingListener {
    async fn on_state_change(&self, from: CircuitBreakerState, to: CircuitBreakerState) {
        self.log.lock().unwrap().push((from, to));
        tracing::info!("Circuit breaker state changed from {:?} to {:?}", from, to);
    }
}
```

## Dependencies

- **Task 1**: Uses Circuit Breaker models from common libraries
- **Task 3**: Integrates with gRPC client for latency monitoring

## Integration Points

- **Paper Trader**: Wraps all trading operations with circuit breaker
- **Live Trader**: Same protection for production trades
- **gRPC Client**: Reports latencies to circuit breaker
- **Jupiter Client**: Protected by circuit breaker checks

## Performance Considerations

- Latency calculation uses lock-free atomics where possible
- P99 calculation optimized for rolling window
- State checks are non-blocking reads
- Minimal overhead on successful operations (<100μs)

## Monitoring and Observability

- State change events logged with timestamps
- Metrics exported for monitoring systems
- Latency statistics available via API
- Circuit breaker dashboard integration

## Future Enhancements

- ML-based anomaly detection for latency patterns
- Adaptive thresholds based on time of day
- Multi-level circuit breakers for different operations
- Integration with distributed tracing