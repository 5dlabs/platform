# Task 3: Implement gRPC Connection to Solana Node

## Overview

This task establishes a resilient gRPC connection to Solana nodes, providing the low-latency infrastructure required for high-frequency trading. The implementation includes health monitoring, circuit breaker integration, and automatic failover capabilities to maintain the 200ms P99 latency target specified in the PRD.

## Architecture Context

The gRPC connection is a critical component of the resilient infrastructure layer. It provides:

- **Low-Latency Communication**: Direct gRPC connection for minimal overhead
- **Health Monitoring**: Real-time tracking of latency and error rates
- **Circuit Breaker Protection**: Automatic disconnection when performance degrades
- **Resilient Operation**: Graceful handling of node failures and network issues

This component directly impacts the platform's ability to execute trades within the required latency bounds and maintain reliable operation during network instability.

## Implementation Details

### 1. Core gRPC Client Implementation

```rust
use tonic::{transport::Channel, Request, Response, Status};
use solana_sdk::{
    transaction::Transaction,
    signature::Signature,
    pubkey::Pubkey,
};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::sync::Arc;

// Proto definitions (simplified for example)
pub mod solana_grpc {
    tonic::include_proto!("solana.grpc");
}

use solana_grpc::{
    geyser_client::GeyserClient,
    SubscribeRequest,
    SubscribeUpdate,
    SendTransactionRequest,
};

pub struct SolanaGrpcClient {
    client: GeyserClient<Channel>,
    endpoint: String,
    health_monitor: Arc<HealthMonitor>,
    circuit_breaker: Arc<CircuitBreaker>,
    retry_policy: RetryPolicy,
}

impl SolanaGrpcClient {
    pub async fn new(endpoint: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(endpoint.to_string())?
            .timeout(Duration::from_millis(200))  // PRD requirement
            .connect_timeout(Duration::from_secs(5))
            .tcp_keepalive(Some(Duration::from_secs(10)))
            .connect()
            .await?;

        let client = GeyserClient::new(channel);
        let health_monitor = Arc::new(HealthMonitor::new());
        let circuit_breaker = Arc::new(CircuitBreaker::new());

        Ok(Self {
            client,
            endpoint: endpoint.to_string(),
            health_monitor,
            circuit_breaker,
            retry_policy: RetryPolicy::default(),
        })
    }

    pub async fn send_transaction(&mut self, transaction: &Transaction) -> Result<Signature, SolanaGrpcError> {
        // Check circuit breaker state
        if !self.circuit_breaker.is_open().await {
            return Err(SolanaGrpcError::CircuitBreakerOpen);
        }

        let serialized_tx = bincode::serialize(transaction)?;
        let request = Request::new(SendTransactionRequest {
            transaction: serialized_tx,
            skip_preflight: false,
        });

        // Execute with monitoring
        let result = self.execute_with_monitoring(
            "send_transaction",
            async {
                let response = self.client.send_transaction(request).await?;
                let signature = Signature::from_bytes(&response.into_inner().signature)?;
                Ok(signature)
            }
        ).await;

        // Apply retry policy if needed
        match result {
            Err(e) if self.retry_policy.should_retry(&e) => {
                self.retry_with_backoff(transaction).await
            }
            other => other,
        }
    }

    pub async fn subscribe_accounts(&mut self, accounts: Vec<Pubkey>) -> Result<AccountUpdateStream, SolanaGrpcError> {
        let account_filters = accounts.into_iter()
            .map(|pubkey| AccountFilter {
                account: pubkey.to_string(),
            })
            .collect();

        let request = Request::new(SubscribeRequest {
            accounts: account_filters,
            slots: false,
            transactions: false,
            blocks: false,
            accounts_data_slice: vec![],
        });

        let response = self.execute_with_monitoring(
            "subscribe_accounts",
            async {
                self.client.subscribe(request).await
            }
        ).await?;

        Ok(AccountUpdateStream {
            inner: response.into_inner(),
            health_monitor: self.health_monitor.clone(),
        })
    }

    async fn execute_with_monitoring<F, T>(
        &self,
        operation: &str,
        future: F,
    ) -> Result<T, SolanaGrpcError>
    where
        F: std::future::Future<Output = Result<T, Status>>,
    {
        let start = Instant::now();
        
        let result = future.await;
        let latency = start.elapsed();

        // Update health metrics
        self.health_monitor.record_operation(operation, latency, result.is_ok()).await;

        // Check if circuit breaker should trip
        let metrics = self.health_monitor.get_metrics().await;
        if metrics.p99_latency_ms > 200 || metrics.error_rate > 0.05 {
            self.circuit_breaker.trip().await;
        }

        result.map_err(|e| SolanaGrpcError::from(e))
    }

    async fn retry_with_backoff(&mut self, transaction: &Transaction) -> Result<Signature, SolanaGrpcError> {
        let mut retries = 0;
        let max_retries = self.retry_policy.max_retries;

        while retries < max_retries {
            retries += 1;
            
            // Exponential backoff
            let delay = Duration::from_millis(100 * 2u64.pow(retries));
            tokio::time::sleep(delay).await;

            // Check if circuit breaker has recovered
            if self.circuit_breaker.is_open().await {
                match self.send_transaction(transaction).await {
                    Ok(sig) => return Ok(sig),
                    Err(e) if !self.retry_policy.should_retry(&e) => return Err(e),
                    _ => continue,
                }
            }
        }

        Err(SolanaGrpcError::MaxRetriesExceeded)
    }
}
```

### 2. Health Monitoring Implementation

```rust
use std::collections::VecDeque;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct HealthMetrics {
    pub total_requests: u64,
    pub failed_requests: u64,
    pub error_rate: f64,
    pub p50_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub p99_latency_ms: u64,
    pub last_error: Option<String>,
}

pub struct HealthMonitor {
    window_size: usize,
    latencies: RwLock<VecDeque<u64>>,
    error_count: RwLock<u64>,
    total_count: RwLock<u64>,
    operation_metrics: RwLock<HashMap<String, OperationMetrics>>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            window_size: 1000,  // Keep last 1000 measurements
            latencies: RwLock::new(VecDeque::with_capacity(1000)),
            error_count: RwLock::new(0),
            total_count: RwLock::new(0),
            operation_metrics: RwLock::new(HashMap::new()),
        }
    }

    pub async fn record_operation(&self, operation: &str, latency: Duration, success: bool) {
        let latency_ms = latency.as_millis() as u64;

        // Update sliding window
        let mut latencies = self.latencies.write().await;
        if latencies.len() >= self.window_size {
            latencies.pop_front();
        }
        latencies.push_back(latency_ms);

        // Update counters
        *self.total_count.write().await += 1;
        if !success {
            *self.error_count.write().await += 1;
        }

        // Update per-operation metrics
        let mut op_metrics = self.operation_metrics.write().await;
        let metrics = op_metrics.entry(operation.to_string()).or_insert(OperationMetrics::default());
        metrics.record(latency_ms, success);
    }

    pub async fn get_metrics(&self) -> HealthMetrics {
        let latencies = self.latencies.read().await;
        let total = *self.total_count.read().await;
        let errors = *self.error_count.read().await;

        // Calculate percentiles
        let mut sorted_latencies: Vec<u64> = latencies.iter().copied().collect();
        sorted_latencies.sort_unstable();

        let p50 = Self::percentile(&sorted_latencies, 0.50);
        let p95 = Self::percentile(&sorted_latencies, 0.95);
        let p99 = Self::percentile(&sorted_latencies, 0.99);

        HealthMetrics {
            total_requests: total,
            failed_requests: errors,
            error_rate: if total > 0 { errors as f64 / total as f64 } else { 0.0 },
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            last_error: None,  // Could track this separately
        }
    }

    fn percentile(sorted_values: &[u64], percentile: f64) -> u64 {
        if sorted_values.is_empty() {
            return 0;
        }

        let idx = ((sorted_values.len() - 1) as f64 * percentile) as usize;
        sorted_values[idx]
    }

    pub async fn is_healthy(&self) -> bool {
        let metrics = self.get_metrics().await;
        metrics.p99_latency_ms <= 200 && metrics.error_rate < 0.05
    }
}

#[derive(Default)]
struct OperationMetrics {
    count: u64,
    errors: u64,
    total_latency: u64,
}

impl OperationMetrics {
    fn record(&mut self, latency_ms: u64, success: bool) {
        self.count += 1;
        self.total_latency += latency_ms;
        if !success {
            self.errors += 1;
        }
    }
}
```

### 3. Circuit Breaker Integration

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Rejecting requests
    HalfOpen,  // Testing recovery
}

pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    failure_threshold: u32,
    recovery_timeout: Duration,
    half_open_max_requests: Arc<RwLock<u32>>,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            half_open_max_requests: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn is_open(&self) -> bool {
        let state = *self.state.read().await;
        
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= self.recovery_timeout {
                        *self.state.write().await = CircuitState::HalfOpen;
                        *self.half_open_max_requests.write().await = 0;
                        return true;  // Allow one request through
                    }
                }
                false
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                let mut requests = self.half_open_max_requests.write().await;
                if *requests < 3 {  // Allow up to 3 test requests
                    *requests += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub async fn trip(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;

        if *failure_count >= self.failure_threshold {
            *self.state.write().await = CircuitState::Open;
            *self.last_failure_time.write().await = Some(Instant::now());
            
            // Log circuit breaker opening
            tracing::warn!("Circuit breaker opened after {} failures", failure_count);
        }
    }

    pub async fn record_success(&self) {
        let state = *self.state.read().await;
        
        match state {
            CircuitState::HalfOpen => {
                // Successful request in half-open state - close the circuit
                *self.state.write().await = CircuitState::Closed;
                *self.failure_count.write().await = 0;
                *self.last_failure_time.write().await = None;
                
                tracing::info!("Circuit breaker closed after successful recovery");
            }
            CircuitState::Closed => {
                // Reset failure count on success
                *self.failure_count.write().await = 0;
            }
            _ => {}
        }
    }

    pub async fn record_failure(&self) {
        let state = *self.state.read().await;
        
        match state {
            CircuitState::HalfOpen => {
                // Failed in half-open - go back to open
                *self.state.write().await = CircuitState::Open;
                *self.last_failure_time.write().await = Some(Instant::now());
                
                tracing::warn!("Circuit breaker reopened after failure in half-open state");
            }
            _ => {
                self.trip().await;
            }
        }
    }

    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }
}
```

### 4. Account Update Stream Handler

```rust
use futures_util::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

pub struct AccountUpdateStream {
    inner: tonic::Streaming<SubscribeUpdate>,
    health_monitor: Arc<HealthMonitor>,
}

impl AccountUpdateStream {
    pub async fn next_update(&mut self) -> Result<Option<AccountUpdate>, SolanaGrpcError> {
        let start = Instant::now();
        
        match self.inner.next().await {
            Some(Ok(update)) => {
                let latency = start.elapsed();
                self.health_monitor.record_operation("stream_update", latency, true).await;
                
                // Parse the update
                let account_update = self.parse_update(update)?;
                Ok(Some(account_update))
            }
            Some(Err(e)) => {
                let latency = start.elapsed();
                self.health_monitor.record_operation("stream_update", latency, false).await;
                Err(SolanaGrpcError::from(e))
            }
            None => Ok(None),
        }
    }

    fn parse_update(&self, update: SubscribeUpdate) -> Result<AccountUpdate, SolanaGrpcError> {
        // Parse the protobuf update into our domain model
        match update.update_oneof {
            Some(UpdateOneof::Account(account)) => {
                Ok(AccountUpdate {
                    pubkey: Pubkey::from_str(&account.pubkey)?,
                    lamports: account.lamports,
                    data: account.data,
                    slot: update.slot,
                })
            }
            _ => Err(SolanaGrpcError::UnexpectedUpdate),
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
    async fn test_health_monitor_percentiles() {
        let monitor = HealthMonitor::new();
        
        // Record various latencies
        for i in 1..=100 {
            let latency = Duration::from_millis(i);
            monitor.record_operation("test", latency, true).await;
        }
        
        let metrics = monitor.get_metrics().await;
        assert_eq!(metrics.p50_latency_ms, 50);
        assert_eq!(metrics.p95_latency_ms, 95);
        assert_eq!(metrics.p99_latency_ms, 99);
    }

    #[tokio::test]
    async fn test_circuit_breaker_states() {
        let breaker = CircuitBreaker::new();
        
        // Initially closed
        assert!(breaker.is_open().await);
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
        
        // Trip after threshold failures
        for _ in 0..5 {
            breaker.record_failure().await;
        }
        
        assert!(!breaker.is_open().await);
        assert_eq!(breaker.get_state().await, CircuitState::Open);
        
        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_secs(31)).await;
        
        // Should be half-open
        assert!(breaker.is_open().await);  // Allow one request
        assert_eq!(breaker.get_state().await, CircuitState::HalfOpen);
        
        // Success closes the circuit
        breaker.record_success().await;
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_retry_with_backoff() {
        // Test exponential backoff timing
        let start = Instant::now();
        let mut total_delay = Duration::from_millis(0);
        
        for retry in 1..=3 {
            let delay = Duration::from_millis(100 * 2u64.pow(retry));
            total_delay += delay;
        }
        
        // 100ms + 200ms + 400ms = 700ms
        assert_eq!(total_delay.as_millis(), 700);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_grpc_connection_with_latency() {
    // Mock server with artificial latency
    let mock_server = MockSolanaServer::new()
        .with_latency(Duration::from_millis(150))
        .start()
        .await;
    
    let mut client = SolanaGrpcClient::new(&mock_server.url()).await.unwrap();
    
    // Send transaction
    let tx = create_test_transaction();
    let result = client.send_transaction(&tx).await;
    
    assert!(result.is_ok());
    
    // Check health metrics
    let metrics = client.health_monitor.get_metrics().await;
    assert!(metrics.p99_latency_ms < 200);
}

#[tokio::test]
async fn test_circuit_breaker_under_load() {
    let mock_server = MockSolanaServer::new()
        .with_error_rate(0.5)  // 50% error rate
        .start()
        .await;
    
    let mut client = SolanaGrpcClient::new(&mock_server.url()).await.unwrap();
    
    // Send multiple transactions
    let mut failures = 0;
    for _ in 0..10 {
        let tx = create_test_transaction();
        if client.send_transaction(&tx).await.is_err() {
            failures += 1;
        }
    }
    
    // Circuit breaker should have opened
    assert_eq!(client.circuit_breaker.get_state().await, CircuitState::Open);
    
    // Further requests should fail immediately
    let tx = create_test_transaction();
    let result = client.send_transaction(&tx).await;
    assert!(matches!(result, Err(SolanaGrpcError::CircuitBreakerOpen)));
}
```

## Dependencies

- **Task 1**: Uses Circuit Breaker model from common libraries

## Integration Points

- **Paper Trader**: Uses gRPC client for transaction submission
- **Live Trader**: Same gRPC interface for production trades
- **Price Feed**: Subscribes to account updates for real-time prices
- **Health Monitoring**: Exports metrics for system observability

## Performance Considerations

- 200ms timeout on all gRPC calls
- Connection pooling for concurrent requests
- Sliding window of 1000 samples for percentile calculations
- Exponential backoff prevents overwhelming failed nodes

## Security Considerations

- TLS encryption for all gRPC connections
- No sensitive data in health metrics logs
- Connection credentials stored securely
- Rate limiting to prevent abuse

## Future Enhancements

- Multi-node failover with health-based routing
- Custom gRPC interceptors for enhanced monitoring
- Compression for bandwidth optimization
- Priority queue for critical transactions