# Task 24: Implement gRPC Trade Execution Service Interface

## Overview

This task implements a gRPC service interface that serves as the communication layer between external decision services and the trading platform's execution engines. The service accepts trade requests via gRPC, validates them, routes them to the appropriate executor (paper or live), and provides real-time status updates. This creates a standardized, high-performance API for automated trading systems to interact with the platform.

## Dependencies

This task depends on:
- Task 1: Common Rust Libraries for Trading Models - Provides core trading structures
- Task 9: gRPC Client for Solana Node Communication - Base gRPC infrastructure patterns
- Task 17: Live Trade Executor with MEV Protection - Live trading execution capability
- Task 18: Monitoring and Logging Infrastructure - Metrics and logging integration

## Architecture Context

According to the architecture.md, the gRPC service interface:
- Provides a unified API for both paper and live trading modes
- Supports streaming for real-time execution status updates
- Integrates with the circuit breaker for resilience
- Implements comprehensive authentication and authorization
- Maintains audit trails for all trading requests

## Implementation Details

### 1. Protocol Buffer Definitions

```protobuf
// proto/trade_execution.proto
syntax = "proto3";
package trade_execution;

import "google/protobuf/timestamp.proto";

// Main service definition
service TradeExecutionService {
  // Execute a single trade
  rpc ExecuteTrade(TradeRequest) returns (TradeResult);
  
  // Stream real-time execution status updates
  rpc StreamExecutionStatus(TradeRequest) returns (stream ExecutionStatus);
  
  // Get historical trade data
  rpc GetTradeHistory(HistoryRequest) returns (HistoryResponse);
  
  // Get current positions
  rpc GetPositions(PositionsRequest) returns (PositionsResponse);
  
  // Health check endpoint
  rpc HealthCheck(HealthRequest) returns (HealthResponse);
}

// Trade request message
message TradeRequest {
  string request_id = 1;                   // Unique request identifier
  string token_in = 2;                     // Input token address
  string token_out = 3;                    // Output token address
  string amount = 4;                       // Amount as string for precision
  bool is_exact_in = 5;                    // true: exact input, false: exact output
  double slippage_tolerance = 6;           // Slippage tolerance as percentage
  bool enable_mev_protection = 7;          // Enable MEV protection features
  TradingMode mode = 8;                    // Paper or live trading
  string auth_token = 9;                   // Authentication token
  map<string, string> metadata = 10;       // Additional metadata
  MevProtectionParams mev_params = 11;     // Optional MEV parameters
}

// MEV protection parameters
message MevProtectionParams {
  uint64 priority_fee = 1;                 // Priority fee in lamports
  bool wrap_sol = 2;                       // Wrap/unwrap SOL
  bool use_shared_accounts = 3;            // Use shared accounts
  uint64 compute_unit_price = 4;           // Compute unit price
}

// Trading mode enum
enum TradingMode {
  PAPER = 0;
  LIVE = 1;
}

// Trade result message
message TradeResult {
  string request_id = 1;                   // Original request ID
  ExecutionStatus status = 2;              // Execution status
  string transaction_id = 3;               // Blockchain transaction ID
  string executed_amount_in = 4;           // Actual input amount
  string executed_amount_out = 5;          // Actual output amount
  double execution_price = 6;              // Execution price
  double slippage = 7;                     // Actual slippage percentage
  google.protobuf.Timestamp timestamp = 8; // Execution timestamp
  uint64 gas_used = 9;                     // Gas/compute units used
  uint64 priority_fee_paid = 10;           // Actual priority fee paid
  bool mev_protected = 11;                 // Was MEV protection applied
  ExecutionMetrics metrics = 12;           // Performance metrics
}

// Execution metrics
message ExecutionMetrics {
  uint64 latency_ms = 1;                   // Total execution latency
  uint64 quote_latency_ms = 2;             // Quote fetch latency
  uint64 simulation_latency_ms = 3;        // Simulation latency
  uint64 submission_latency_ms = 4;        // Transaction submission latency
  bool used_failover = 5;                  // Used failover Jupiter
}

// Execution status for streaming
message ExecutionStatus {
  string request_id = 1;                   // Request being tracked
  StatusCode status_code = 2;              // Current status
  string status_message = 3;               // Human-readable message
  double completion_percentage = 4;        // 0-100 completion
  google.protobuf.Timestamp timestamp = 5; // Status update time
  map<string, string> details = 6;         // Additional details
}

// Status codes
enum StatusCode {
  PENDING = 0;
  VALIDATING = 1;
  FETCHING_QUOTE = 2;
  SIMULATING = 3;
  SUBMITTING = 4;
  CONFIRMING = 5;
  COMPLETED = 6;
  FAILED = 7;
  CANCELLED = 8;
}

// History request
message HistoryRequest {
  google.protobuf.Timestamp start_time = 1;
  google.protobuf.Timestamp end_time = 2;
  int32 limit = 3;
  string auth_token = 4;
  repeated string token_filter = 5;        // Filter by tokens
  TradingMode mode_filter = 6;             // Filter by mode
}

// History response
message HistoryResponse {
  repeated TradeResult trades = 1;
  int32 total_count = 2;
  string next_cursor = 3;                  // Pagination cursor
}

// Position request
message PositionsRequest {
  string auth_token = 1;
  TradingMode mode = 2;
}

// Position response
message PositionsResponse {
  repeated Position positions = 1;
  double total_value = 2;
  double total_pnl = 3;
  google.protobuf.Timestamp timestamp = 4;
}

// Position message
message Position {
  string token = 1;
  string amount = 2;
  double cost_basis = 3;
  double current_price = 4;
  double unrealized_pnl = 5;
  double unrealized_pnl_percent = 6;
}

// Health check
message HealthRequest {}

message HealthResponse {
  bool healthy = 1;
  string version = 2;
  map<string, ComponentHealth> components = 3;
}

message ComponentHealth {
  bool healthy = 1;
  string status = 2;
  uint64 latency_ms = 3;
}
```

### 2. Service Implementation

```rust
use tonic::{transport::Server, Request, Response, Status, Streaming};
use trade_execution::trade_execution_service_server::{TradeExecutionService, TradeExecutionServiceServer};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use std::pin::Pin;
use futures::Stream;

pub struct TradeExecutionServiceImpl {
    paper_executor: Arc<PaperTradeExecutor>,
    live_executor: Arc<LiveTradeExecutor>,
    auth_service: Arc<AuthService>,
    metrics: Arc<MetricsCollector>,
    validator: Arc<RequestValidator>,
    logger: Arc<RequestLogger>,
    shutdown_manager: Arc<ShutdownManager>,
}

#[tonic::async_trait]
impl TradeExecutionService for TradeExecutionServiceImpl {
    async fn execute_trade(
        &self,
        request: Request<TradeRequest>,
    ) -> Result<Response<TradeResult>, Status> {
        let start_time = Instant::now();
        let trade_request = request.into_inner();
        
        // Register request for graceful shutdown
        self.shutdown_manager.register_request(trade_request.request_id.clone());
        
        // Log incoming request
        self.logger.log_request(&trade_request).await;
        self.metrics.record_request();
        
        // Validate authentication
        self.auth_service
            .validate_token(&trade_request.auth_token)
            .await
            .map_err(|_| Status::unauthenticated("Invalid authentication token"))?;
        
        // Validate request
        self.validator
            .validate_trade_request(&trade_request)
            .map_err(|e| Status::invalid_argument(format!("Validation failed: {}", e)))?;
        
        // Route to appropriate executor
        let result = match trade_request.mode() {
            TradingMode::Paper => {
                self.paper_executor
                    .execute(trade_request.clone())
                    .await
            }
            TradingMode::Live => {
                // Additional safety check for live trading
                if !self.auth_service.has_live_trading_permission(&trade_request.auth_token).await {
                    return Err(Status::permission_denied("Live trading not authorized"));
                }
                
                self.live_executor
                    .execute(trade_request.clone())
                    .await
            }
        };
        
        // Map execution result
        let trade_result = match result {
            Ok(exec_result) => {
                let latency_ms = start_time.elapsed().as_millis() as u64;
                self.metrics.record_latency(latency_ms as f64);
                
                TradeResult {
                    request_id: trade_request.request_id.clone(),
                    status: ExecutionStatus {
                        status_code: StatusCode::Completed as i32,
                        status_message: "Trade executed successfully".to_string(),
                        completion_percentage: 100.0,
                        timestamp: Some(Timestamp::from(SystemTime::now())),
                        details: HashMap::new(),
                    },
                    transaction_id: exec_result.signature.to_string(),
                    executed_amount_in: exec_result.amount_in.to_string(),
                    executed_amount_out: exec_result.amount_out.to_string(),
                    execution_price: exec_result.price,
                    slippage: exec_result.slippage,
                    timestamp: Some(Timestamp::from(SystemTime::now())),
                    gas_used: exec_result.compute_units_used,
                    priority_fee_paid: exec_result.priority_fee,
                    mev_protected: exec_result.mev_protected,
                    metrics: Some(ExecutionMetrics {
                        latency_ms,
                        quote_latency_ms: exec_result.quote_latency_ms,
                        simulation_latency_ms: exec_result.simulation_latency_ms,
                        submission_latency_ms: exec_result.submission_latency_ms,
                        used_failover: exec_result.used_failover,
                    }),
                }
            }
            Err(e) => {
                self.metrics.record_failure();
                return Err(map_execution_error(e));
            }
        };
        
        // Log response
        self.logger.log_response(&trade_request, &trade_result).await;
        
        // Complete request tracking
        self.shutdown_manager.complete_request(&trade_request.request_id);
        
        Ok(Response::new(trade_result))
    }
    
    type StreamExecutionStatusStream = Pin<Box<dyn Stream<Item = Result<ExecutionStatus, Status>> + Send + 'static>>;
    
    async fn stream_execution_status(
        &self,
        request: Request<TradeRequest>,
    ) -> Result<Response<Self::StreamExecutionStatusStream>, Status> {
        let trade_request = request.into_inner();
        
        // Validate authentication
        self.auth_service
            .validate_token(&trade_request.auth_token)
            .await
            .map_err(|_| Status::unauthenticated("Invalid authentication token"))?;
        
        // Create status update channel
        let (tx, rx) = mpsc::channel(32);
        let request_id = trade_request.request_id.clone();
        
        // Spawn execution task that sends status updates
        let executor = match trade_request.mode() {
            TradingMode::Paper => self.paper_executor.clone(),
            TradingMode::Live => self.live_executor.clone(),
        };
        
        tokio::spawn(async move {
            // Send initial status
            let _ = tx.send(ExecutionStatus {
                request_id: request_id.clone(),
                status_code: StatusCode::Pending as i32,
                status_message: "Trade request received".to_string(),
                completion_percentage: 0.0,
                timestamp: Some(Timestamp::from(SystemTime::now())),
                details: HashMap::new(),
            }).await;
            
            // Execute with status updates
            executor.execute_with_updates(trade_request, tx).await;
        });
        
        // Convert to streaming response
        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream) as Self::StreamExecutionStatusStream))
    }
    
    async fn get_trade_history(
        &self,
        request: Request<HistoryRequest>,
    ) -> Result<Response<HistoryResponse>, Status> {
        let history_request = request.into_inner();
        
        // Validate authentication
        self.auth_service
            .validate_token(&history_request.auth_token)
            .await
            .map_err(|_| Status::unauthenticated("Invalid authentication token"))?;
        
        // Query trade history
        let trades = self.logger
            .get_trade_history(
                history_request.start_time.map(|t| t.into()),
                history_request.end_time.map(|t| t.into()),
                history_request.limit as usize,
                history_request.token_filter,
                history_request.mode_filter,
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to retrieve history: {}", e)))?;
        
        Ok(Response::new(HistoryResponse {
            trades,
            total_count: trades.len() as i32,
            next_cursor: String::new(), // TODO: Implement pagination
        }))
    }
    
    async fn get_positions(
        &self,
        request: Request<PositionsRequest>,
    ) -> Result<Response<PositionsResponse>, Status> {
        let positions_request = request.into_inner();
        
        // Validate authentication
        self.auth_service
            .validate_token(&positions_request.auth_token)
            .await
            .map_err(|_| Status::unauthenticated("Invalid authentication token"))?;
        
        // Get positions based on mode
        let positions = match positions_request.mode() {
            TradingMode::Paper => {
                self.paper_executor
                    .get_positions()
                    .await
            }
            TradingMode::Live => {
                self.live_executor
                    .get_positions()
                    .await
            }
        }
        .map_err(|e| Status::internal(format!("Failed to get positions: {}", e)))?;
        
        // Calculate totals
        let total_value: f64 = positions.iter().map(|p| p.current_value()).sum();
        let total_pnl: f64 = positions.iter().map(|p| p.unrealized_pnl).sum();
        
        // Convert to proto format
        let proto_positions: Vec<Position> = positions
            .into_iter()
            .map(|p| Position {
                token: p.token,
                amount: p.amount.to_string(),
                cost_basis: p.cost_basis,
                current_price: p.current_price,
                unrealized_pnl: p.unrealized_pnl,
                unrealized_pnl_percent: p.unrealized_pnl_percent,
            })
            .collect();
        
        Ok(Response::new(PositionsResponse {
            positions: proto_positions,
            total_value,
            total_pnl,
            timestamp: Some(Timestamp::from(SystemTime::now())),
        }))
    }
    
    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        let mut components = HashMap::new();
        
        // Check paper executor health
        let paper_health = self.paper_executor.health_check().await;
        components.insert("paper_executor".to_string(), ComponentHealth {
            healthy: paper_health.is_healthy,
            status: paper_health.status,
            latency_ms: paper_health.latency_ms,
        });
        
        // Check live executor health
        let live_health = self.live_executor.health_check().await;
        components.insert("live_executor".to_string(), ComponentHealth {
            healthy: live_health.is_healthy,
            status: live_health.status,
            latency_ms: live_health.latency_ms,
        });
        
        // Check auth service health
        let auth_health = self.auth_service.health_check().await;
        components.insert("auth_service".to_string(), ComponentHealth {
            healthy: auth_health.is_healthy,
            status: auth_health.status,
            latency_ms: auth_health.latency_ms,
        });
        
        // Overall health
        let overall_healthy = components.values().all(|c| c.healthy);
        
        Ok(Response::new(HealthResponse {
            healthy: overall_healthy,
            version: env!("CARGO_PKG_VERSION").to_string(),
            components,
        }))
    }
}
```

### 3. Request Validation

```rust
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub struct RequestValidator {
    max_slippage: f64,
    min_amount: u64,
    max_amount: u64,
    supported_tokens: HashSet<String>,
}

impl RequestValidator {
    pub fn new(config: ValidatorConfig) -> Self {
        Self {
            max_slippage: config.max_slippage,
            min_amount: config.min_amount,
            max_amount: config.max_amount,
            supported_tokens: config.supported_tokens.into_iter().collect(),
        }
    }
    
    pub fn validate_trade_request(&self, request: &TradeRequest) -> Result<(), ValidationError> {
        // Validate request ID
        if request.request_id.is_empty() {
            return Err(ValidationError::MissingRequestId);
        }
        
        // Validate token addresses
        Pubkey::from_str(&request.token_in)
            .map_err(|_| ValidationError::InvalidTokenAddress("token_in".to_string()))?;
        
        Pubkey::from_str(&request.token_out)
            .map_err(|_| ValidationError::InvalidTokenAddress("token_out".to_string()))?;
        
        // Validate amount
        let amount = u64::from_str(&request.amount)
            .map_err(|_| ValidationError::InvalidAmount)?;
        
        if amount < self.min_amount {
            return Err(ValidationError::AmountTooSmall(self.min_amount));
        }
        
        if amount > self.max_amount {
            return Err(ValidationError::AmountTooLarge(self.max_amount));
        }
        
        // Validate slippage
        if request.slippage_tolerance < 0.0 || request.slippage_tolerance > self.max_slippage {
            return Err(ValidationError::InvalidSlippage(self.max_slippage));
        }
        
        // Validate supported tokens (optional)
        if !self.supported_tokens.is_empty() {
            if !self.supported_tokens.contains(&request.token_in) {
                return Err(ValidationError::UnsupportedToken(request.token_in.clone()));
            }
            if !self.supported_tokens.contains(&request.token_out) {
                return Err(ValidationError::UnsupportedToken(request.token_out.clone()));
            }
        }
        
        // Validate MEV parameters if provided
        if let Some(mev_params) = &request.mev_params {
            if mev_params.priority_fee > 1_000_000 {
                return Err(ValidationError::ExcessivePriorityFee);
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Missing request ID")]
    MissingRequestId,
    
    #[error("Invalid token address: {0}")]
    InvalidTokenAddress(String),
    
    #[error("Invalid amount format")]
    InvalidAmount,
    
    #[error("Amount too small, minimum: {0}")]
    AmountTooSmall(u64),
    
    #[error("Amount too large, maximum: {0}")]
    AmountTooLarge(u64),
    
    #[error("Invalid slippage tolerance, maximum: {0}")]
    InvalidSlippage(f64),
    
    #[error("Unsupported token: {0}")]
    UnsupportedToken(String),
    
    #[error("Excessive priority fee")]
    ExcessivePriorityFee,
}
```

### 4. Error Mapping

```rust
fn map_execution_error(error: ExecutionError) -> Status {
    match error {
        ExecutionError::InsufficientFunds { available, required } => {
            Status::failed_precondition(
                format!("Insufficient funds: available {}, required {}", available, required)
            )
        }
        ExecutionError::ExcessiveSlippage { expected, actual } => {
            Status::aborted(
                format!("Excessive slippage: expected {:.2}%, actual {:.2}%", expected * 100.0, actual * 100.0)
            )
        }
        ExecutionError::CircuitBreakerOpen => {
            Status::unavailable("Trading temporarily paused due to circuit breaker")
        }
        ExecutionError::RiskLimitExceeded(reason) => {
            Status::failed_precondition(format!("Risk limit exceeded: {}", reason))
        }
        ExecutionError::QuoteUnavailable => {
            Status::unavailable("Unable to fetch quote, please try again")
        }
        ExecutionError::TransactionFailed(reason) => {
            Status::internal(format!("Transaction failed: {}", reason))
        }
        ExecutionError::Timeout => {
            Status::deadline_exceeded("Trade execution timed out")
        }
        ExecutionError::ValidationFailed(reason) => {
            Status::invalid_argument(reason)
        }
        ExecutionError::AuthenticationFailed => {
            Status::unauthenticated("Authentication failed")
        }
        ExecutionError::PermissionDenied => {
            Status::permission_denied("Insufficient permissions")
        }
        ExecutionError::Internal(details) => {
            Status::internal(format!("Internal error: {}", details))
        }
    }
}
```

### 5. Request Logging

```rust
use questdb::QuestDbClient;

pub struct RequestLogger {
    questdb: Arc<QuestDbClient>,
}

impl RequestLogger {
    pub async fn log_request(&self, request: &TradeRequest) {
        let log_entry = json!({
            "timestamp": Utc::now(),
            "request_id": request.request_id,
            "token_in": request.token_in,
            "token_out": request.token_out,
            "amount": request.amount,
            "slippage_tolerance": request.slippage_tolerance,
            "mode": format!("{:?}", request.mode()),
            "mev_protection": request.enable_mev_protection,
        });
        
        if let Err(e) = self.questdb.insert("grpc_requests", log_entry).await {
            error!("Failed to log request: {}", e);
        }
    }
    
    pub async fn log_response(&self, request: &TradeRequest, result: &TradeResult) {
        let log_entry = json!({
            "timestamp": Utc::now(),
            "request_id": request.request_id,
            "status": result.status.status_code,
            "transaction_id": result.transaction_id,
            "executed_amount_in": result.executed_amount_in,
            "executed_amount_out": result.executed_amount_out,
            "execution_price": result.execution_price,
            "slippage": result.slippage,
            "gas_used": result.gas_used,
            "latency_ms": result.metrics.as_ref().map(|m| m.latency_ms),
        });
        
        if let Err(e) = self.questdb.insert("grpc_responses", log_entry).await {
            error!("Failed to log response: {}", e);
        }
    }
    
    pub async fn get_trade_history(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: usize,
        token_filter: Vec<String>,
        mode_filter: Option<i32>,
    ) -> Result<Vec<TradeResult>> {
        let mut query = String::from("SELECT * FROM grpc_responses WHERE 1=1");
        
        if let Some(start) = start_time {
            query.push_str(&format!(" AND timestamp >= '{}'", start));
        }
        
        if let Some(end) = end_time {
            query.push_str(&format!(" AND timestamp <= '{}'", end));
        }
        
        if !token_filter.is_empty() {
            let tokens = token_filter.iter()
                .map(|t| format!("'{}'", t))
                .collect::<Vec<_>>()
                .join(",");
            query.push_str(&format!(" AND (token_in IN ({}) OR token_out IN ({}))", tokens, tokens));
        }
        
        if let Some(mode) = mode_filter {
            query.push_str(&format!(" AND mode = {}", mode));
        }
        
        query.push_str(&format!(" ORDER BY timestamp DESC LIMIT {}", limit));
        
        self.questdb.query(&query).await
    }
}
```

### 6. Metrics Collection

```rust
use prometheus::{Counter, Histogram, Registry, Gauge};

pub struct MetricsCollector {
    request_counter: Counter,
    failure_counter: Counter,
    latency_histogram: Histogram,
    active_requests: Gauge,
    mode_counters: HashMap<String, Counter>,
}

impl MetricsCollector {
    pub fn new(registry: &Registry) -> Result<Self> {
        let request_counter = Counter::new("grpc_requests_total", "Total number of gRPC requests")?;
        let failure_counter = Counter::new("grpc_failures_total", "Total number of failed requests")?;
        let latency_histogram = Histogram::new("grpc_latency_ms", "Request latency in milliseconds")?;
        let active_requests = Gauge::new("grpc_active_requests", "Number of active requests")?;
        
        registry.register(Box::new(request_counter.clone()))?;
        registry.register(Box::new(failure_counter.clone()))?;
        registry.register(Box::new(latency_histogram.clone()))?;
        registry.register(Box::new(active_requests.clone()))?;
        
        let mut mode_counters = HashMap::new();
        for mode in &["paper", "live"] {
            let counter = Counter::new(
                &format!("grpc_requests_{}_total", mode),
                &format!("Total {} trading requests", mode)
            )?;
            registry.register(Box::new(counter.clone()))?;
            mode_counters.insert(mode.to_string(), counter);
        }
        
        Ok(Self {
            request_counter,
            failure_counter,
            latency_histogram,
            active_requests,
            mode_counters,
        })
    }
    
    pub fn record_request(&self) {
        self.request_counter.inc();
        self.active_requests.inc();
    }
    
    pub fn record_failure(&self) {
        self.failure_counter.inc();
        self.active_requests.dec();
    }
    
    pub fn record_latency(&self, latency_ms: f64) {
        self.latency_histogram.observe(latency_ms);
        self.active_requests.dec();
    }
    
    pub fn record_mode(&self, mode: &str) {
        if let Some(counter) = self.mode_counters.get(mode) {
            counter.inc();
        }
    }
}
```

### 7. Service Runner

```rust
use tonic::transport::Server;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

pub struct ServiceConfig {
    pub bind_address: String,
    pub paper_config: PaperTraderConfig,
    pub live_config: LiveTraderConfig,
    pub auth_config: AuthConfig,
    pub tls_config: Option<TlsConfig>,
    pub max_concurrent_requests: usize,
}

pub async fn run_service(config: ServiceConfig) -> Result<()> {
    let addr = config.bind_address.parse()?;
    
    // Initialize components
    let paper_executor = Arc::new(PaperTradeExecutor::new(config.paper_config).await?);
    let live_executor = Arc::new(LiveTradeExecutor::new(config.live_config).await?);
    let auth_service = Arc::new(AuthService::new(config.auth_config));
    let metrics = Arc::new(MetricsCollector::new(&prometheus::default_registry())?);
    let validator = Arc::new(RequestValidator::new(Default::default()));
    let logger = Arc::new(RequestLogger::new(questdb_client).await?);
    let shutdown_manager = Arc::new(ShutdownManager::new());
    
    let service = TradeExecutionServiceImpl {
        paper_executor,
        live_executor,
        auth_service,
        metrics,
        validator,
        logger,
        shutdown_manager: shutdown_manager.clone(),
    };
    
    info!("Starting gRPC Trade Execution Service on {}", addr);
    
    // Build server with middleware
    let server = Server::builder()
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_grpc())
                .buffer(config.max_concurrent_requests)
                .concurrency_limit(config.max_concurrent_requests)
                .into_inner()
        )
        .add_service(TradeExecutionServiceServer::new(service));
    
    // Apply TLS if configured
    let server = if let Some(tls) = config.tls_config {
        let cert = tokio::fs::read(tls.cert_path).await?;
        let key = tokio::fs::read(tls.key_path).await?;
        let identity = tonic::transport::Identity::from_pem(cert, key);
        
        server.tls_config(tonic::transport::ServerTlsConfig::new().identity(identity))?
    } else {
        server
    };
    
    // Run with graceful shutdown
    server
        .serve_with_shutdown(addr, shutdown_signal(shutdown_manager))
        .await?;
    
    Ok(())
}

async fn shutdown_signal(shutdown_manager: Arc<ShutdownManager>) {
    // Wait for CTRL+C
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    
    info!("Shutdown signal received, initiating graceful shutdown...");
    
    // Initiate graceful shutdown
    shutdown_manager.initiate_shutdown().await;
    
    info!("Graceful shutdown complete");
}
```

## Integration Points

### 1. Paper Trade Executor
- Routes paper trading requests to the paper trade executor
- Streams status updates during execution
- Retrieves virtual portfolio positions

### 2. Live Trade Executor
- Routes live trading requests with additional permission checks
- Applies MEV protection parameters
- Handles real transaction submission

### 3. Authentication Service
- Validates auth tokens for all requests
- Checks live trading permissions
- Maintains user session state

### 4. Monitoring Infrastructure
- Records all requests and responses
- Tracks performance metrics
- Provides health check data

## Performance Considerations

1. **Connection Pooling**: Reuse gRPC connections for efficiency
2. **Streaming**: Use server-side streaming for real-time updates
3. **Concurrency Limits**: Prevent overload with request limits
4. **Caching**: Cache authentication results briefly
5. **Batch Operations**: Support batch trade requests in future

## Security Considerations

1. **Authentication**: All requests require valid auth tokens
2. **Authorization**: Live trading requires additional permissions
3. **TLS**: Support TLS for encrypted communication
4. **Rate Limiting**: Prevent abuse with rate limits
5. **Audit Trail**: Log all requests and responses