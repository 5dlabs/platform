# Task 25: Implement gRPC Testing Client for Trade Request Simulation

## Overview

This task implements a comprehensive testing client for the gRPC Trade Execution Service, enabling developers to simulate external trading decision services during development and integration testing. The client provides both interactive CLI capabilities and automated testing features, supporting individual trades, batch operations, performance testing, and error scenario validation.

## Dependencies

This task depends on:
- Task 1: Common Rust Libraries for Trading Models - Provides trade request structures
- Task 24: gRPC Trade Execution Service Interface - The service to test against

## Architecture Context

The testing client serves multiple purposes in the trading platform ecosystem:
- Development tool for testing the gRPC service during implementation
- Integration testing tool for validating end-to-end functionality
- Performance testing tool for load and latency measurements
- Debugging tool for troubleshooting service issues
- Demo tool for showcasing platform capabilities

## Implementation Details

### 1. CLI Application Structure

```rust
use clap::{App, Arg, ArgMatches, SubCommand};
use tokio::runtime::Runtime;
use trade_execution::trade_execution_service_client::TradeExecutionServiceClient;
use tonic::transport::Channel;
use std::time::{Duration, Instant};

fn main() {
    let app = build_cli();
    let matches = app.get_matches();
    
    // Initialize runtime
    let runtime = Runtime::new().expect("Failed to create runtime");
    
    // Process commands
    runtime.block_on(async {
        if let Err(e) = process_command(&matches).await {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    });
}

fn build_cli() -> App<'static, 'static> {
    App::new("Trade Execution Test Client")
        .version("1.0.0")
        .author("Trading Platform Team")
        .about("Testing client for gRPC Trade Execution Service")
        .arg(
            Arg::with_name("endpoint")
                .short("e")
                .long("endpoint")
                .value_name("URL")
                .help("gRPC service endpoint")
                .takes_value(true)
                .default_value("http://localhost:50051")
                .global(true)
        )
        .arg(
            Arg::with_name("auth-token")
                .short("a")
                .long("auth-token")
                .value_name("TOKEN")
                .help("Authentication token")
                .takes_value(true)
                .env("TRADE_AUTH_TOKEN")
                .global(true)
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Enable verbose output")
                .multiple(true)
                .global(true)
        )
        .subcommand(
            SubCommand::with_name("trade")
                .about("Execute a single trade")
                .arg(
                    Arg::with_name("token-in")
                        .long("token-in")
                        .value_name("TOKEN")
                        .help("Input token symbol or address")
                        .required(true)
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("token-out")
                        .long("token-out")
                        .value_name("TOKEN")
                        .help("Output token symbol or address")
                        .required(true)
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("amount")
                        .long("amount")
                        .value_name("AMOUNT")
                        .help("Amount to trade")
                        .required(true)
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("slippage")
                        .long("slippage")
                        .value_name("BPS")
                        .help("Slippage tolerance in basis points")
                        .takes_value(true)
                        .default_value("100")
                )
                .arg(
                    Arg::with_name("mode")
                        .long("mode")
                        .value_name("MODE")
                        .help("Trading mode: paper or live")
                        .takes_value(true)
                        .possible_values(&["paper", "live"])
                        .default_value("paper")
                )
                .arg(
                    Arg::with_name("priority-fee")
                        .long("priority-fee")
                        .value_name("LAMPORTS")
                        .help("Priority fee for MEV protection")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("stream")
                        .long("stream")
                        .help("Stream execution status updates")
                )
        )
        .subcommand(
            SubCommand::with_name("batch")
                .about("Execute batch trades from file")
                .arg(
                    Arg::with_name("file")
                        .short("f")
                        .long("file")
                        .value_name("FILE")
                        .help("JSON file containing trade requests")
                        .required(true)
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("concurrent")
                        .short("c")
                        .long("concurrent")
                        .value_name("N")
                        .help("Number of concurrent requests")
                        .takes_value(true)
                        .default_value("1")
                )
                .arg(
                    Arg::with_name("delay")
                        .short("d")
                        .long("delay")
                        .value_name("MS")
                        .help("Delay between requests in milliseconds")
                        .takes_value(true)
                        .default_value("0")
                )
        )
        .subcommand(
            SubCommand::with_name("template")
                .about("Execute trade from template")
                .arg(
                    Arg::with_name("name")
                        .short("n")
                        .long("name")
                        .value_name("NAME")
                        .help("Template name")
                        .required(true)
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .value_name("FILE")
                        .help("Configuration file path")
                        .takes_value(true)
                        .default_value("test-client.toml")
                )
        )
        .subcommand(
            SubCommand::with_name("perf")
                .about("Run performance test")
                .arg(
                    Arg::with_name("requests")
                        .short("n")
                        .long("requests")
                        .value_name("N")
                        .help("Total number of requests")
                        .takes_value(true)
                        .default_value("100")
                )
                .arg(
                    Arg::with_name("concurrent")
                        .short("c")
                        .long("concurrent")
                        .value_name("N")
                        .help("Concurrent requests")
                        .takes_value(true)
                        .default_value("10")
                )
                .arg(
                    Arg::with_name("duration")
                        .short("d")
                        .long("duration")
                        .value_name("SECONDS")
                        .help("Test duration in seconds")
                        .takes_value(true)
                        .conflicts_with("requests")
                )
                .arg(
                    Arg::with_name("rate")
                        .short("r")
                        .long("rate")
                        .value_name("RPS")
                        .help("Target requests per second")
                        .takes_value(true)
                )
        )
        .subcommand(
            SubCommand::with_name("history")
                .about("Query trade history")
                .arg(
                    Arg::with_name("start")
                        .long("start")
                        .value_name("TIMESTAMP")
                        .help("Start time (ISO 8601)")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("end")
                        .long("end")
                        .value_name("TIMESTAMP")
                        .help("End time (ISO 8601)")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("limit")
                        .long("limit")
                        .value_name("N")
                        .help("Maximum results")
                        .takes_value(true)
                        .default_value("100")
                )
        )
        .subcommand(
            SubCommand::with_name("positions")
                .about("Get current positions")
                .arg(
                    Arg::with_name("mode")
                        .long("mode")
                        .value_name("MODE")
                        .help("Trading mode: paper or live")
                        .takes_value(true)
                        .default_value("paper")
                )
        )
        .subcommand(
            SubCommand::with_name("health")
                .about("Check service health")
        )
}
```

### 2. Client Implementation

```rust
use trade_execution::{
    TradeRequest, TradeResult, ExecutionStatus, HistoryRequest, HistoryResponse,
    PositionsRequest, PositionsResponse, HealthRequest, HealthResponse,
    TradingMode, MevProtectionParams,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use std::collections::HashMap;

pub struct TestClient {
    client: TradeExecutionServiceClient<Channel>,
    endpoint: String,
    auth_token: Option<String>,
    metrics_collector: MetricsCollector,
}

impl TestClient {
    pub async fn new(endpoint: &str, auth_token: Option<String>) -> Result<Self> {
        let channel = Channel::from_shared(endpoint.to_string())?
            .timeout(Duration::from_secs(5))
            .rate_limit(100, Duration::from_secs(1))
            .concurrency_limit(50)
            .connect()
            .await?;
        
        let client = TradeExecutionServiceClient::new(channel);
        
        Ok(Self {
            client,
            endpoint: endpoint.to_string(),
            auth_token,
            metrics_collector: MetricsCollector::new(),
        })
    }
    
    pub async fn execute_trade(
        &mut self,
        params: TradeParams,
        stream_updates: bool,
    ) -> Result<TradeExecutionResult> {
        let start_time = Instant::now();
        
        // Build request
        let request = self.build_trade_request(params)?;
        
        if stream_updates {
            // Execute with streaming
            self.execute_with_streaming(request).await
        } else {
            // Execute unary call
            self.execute_unary(request).await
        }
    }
    
    async fn execute_unary(&mut self, request: TradeRequest) -> Result<TradeExecutionResult> {
        let start_time = Instant::now();
        
        let response = self.client
            .execute_trade(tonic::Request::new(request))
            .await?;
        
        let result = response.into_inner();
        let latency = start_time.elapsed();
        
        self.metrics_collector.record_trade_result(&result, latency);
        
        Ok(TradeExecutionResult {
            request_id: result.request_id,
            success: result.status.as_ref()
                .map(|s| s.status_code == StatusCode::Completed as i32)
                .unwrap_or(false),
            transaction_id: result.transaction_id,
            execution_price: result.execution_price,
            slippage: result.slippage,
            latency,
            details: result,
        })
    }
    
    async fn execute_with_streaming(
        &mut self,
        request: TradeRequest,
    ) -> Result<TradeExecutionResult> {
        let start_time = Instant::now();
        let request_id = request.request_id.clone();
        
        // Start streaming
        let mut stream = self.client
            .stream_execution_status(tonic::Request::new(request))
            .await?
            .into_inner();
        
        let mut updates = Vec::new();
        let mut final_result = None;
        
        // Collect updates
        while let Some(status) = stream.message().await? {
            println!("Status Update: {} - {} ({}%)",
                status.status_code,
                status.status_message,
                status.completion_percentage
            );
            
            updates.push(status.clone());
            
            if status.status_code == StatusCode::Completed as i32 
                || status.status_code == StatusCode::Failed as i32 {
                final_result = Some(status);
                break;
            }
        }
        
        let latency = start_time.elapsed();
        
        Ok(TradeExecutionResult {
            request_id,
            success: final_result.as_ref()
                .map(|s| s.status_code == StatusCode::Completed as i32)
                .unwrap_or(false),
            transaction_id: String::new(), // Would be in final status details
            execution_price: 0.0,
            slippage: 0.0,
            latency,
            details: TradeResult::default(), // Simplified for streaming
        })
    }
    
    fn build_trade_request(&self, params: TradeParams) -> Result<TradeRequest> {
        // Map token symbols to addresses
        let token_in_address = self.resolve_token_address(&params.token_in)?;
        let token_out_address = self.resolve_token_address(&params.token_out)?;
        
        // Convert amount to lamports/smallest unit
        let amount_str = self.convert_amount_to_string(
            params.amount,
            &params.token_in
        )?;
        
        // Build MEV protection params if specified
        let mev_params = params.priority_fee.map(|fee| MevProtectionParams {
            priority_fee: fee,
            wrap_sol: true,
            use_shared_accounts: true,
            compute_unit_price: 0, // Auto
        });
        
        Ok(TradeRequest {
            request_id: params.request_id.unwrap_or_else(|| {
                format!("test-{}", uuid::Uuid::new_v4())
            }),
            token_in: token_in_address,
            token_out: token_out_address,
            amount: amount_str,
            is_exact_in: true,
            slippage_tolerance: params.slippage_bps as f64 / 10000.0,
            enable_mev_protection: mev_params.is_some(),
            mode: params.mode as i32,
            auth_token: self.auth_token.clone().unwrap_or_default(),
            metadata: HashMap::new(),
            mev_params,
        })
    }
    
    fn resolve_token_address(&self, token: &str) -> Result<String> {
        // Map common symbols to addresses
        match token.to_uppercase().as_str() {
            "SOL" => Ok("So11111111111111111111111111111111111111112".to_string()),
            "USDC" => Ok("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
            "BONK" => Ok("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string()),
            "JITOSOL" => Ok("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn".to_string()),
            "RAY" => Ok("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string()),
            _ => {
                // Assume it's already an address
                if token.len() >= 32 && token.len() <= 44 {
                    Ok(token.to_string())
                } else {
                    Err(anyhow!("Unknown token symbol: {}", token))
                }
            }
        }
    }
    
    fn convert_amount_to_string(&self, amount: f64, token: &str) -> Result<String> {
        let decimals = self.get_token_decimals(token)?;
        let multiplier = 10u64.pow(decimals);
        let amount_units = (amount * multiplier as f64) as u64;
        Ok(amount_units.to_string())
    }
    
    fn get_token_decimals(&self, token: &str) -> Result<u32> {
        match token.to_uppercase().as_str() {
            "SOL" | "JITOSOL" => Ok(9),
            "USDC" => Ok(6),
            "BONK" => Ok(5),
            "RAY" => Ok(6),
            _ => Ok(9), // Default to 9 decimals
        }
    }
}

#[derive(Debug)]
pub struct TradeParams {
    pub token_in: String,
    pub token_out: String,
    pub amount: f64,
    pub slippage_bps: u32,
    pub mode: TradingMode,
    pub priority_fee: Option<u64>,
    pub request_id: Option<String>,
}

#[derive(Debug)]
pub struct TradeExecutionResult {
    pub request_id: String,
    pub success: bool,
    pub transaction_id: String,
    pub execution_price: f64,
    pub slippage: f64,
    pub latency: Duration,
    pub details: TradeResult,
}
```

### 3. Template System

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTemplate {
    pub name: String,
    pub description: Option<String>,
    pub token_in: String,
    pub token_out: String,
    pub amount: f64,
    pub slippage_bps: u32,
    pub priority_fee_mode: PriorityFeeMode,
    pub trading_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PriorityFeeMode {
    Low,
    Medium,
    High,
    Custom(u64),
}

impl PriorityFeeMode {
    pub fn to_lamports(&self) -> u64 {
        match self {
            PriorityFeeMode::Low => 1_000,
            PriorityFeeMode::Medium => 5_000,
            PriorityFeeMode::High => 10_000,
            PriorityFeeMode::Custom(fee) => *fee,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    pub connection: ConnectionConfig,
    pub default_trade: DefaultTradeConfig,
    pub templates: Vec<RequestTemplate>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectionConfig {
    pub endpoint: String,
    pub timeout_ms: u64,
    pub retry_attempts: u32,
}

#[derive(Debug, Deserialize)]
pub struct DefaultTradeConfig {
    pub token_in: String,
    pub token_out: String,
    pub amount: f64,
    pub slippage_bps: u32,
    pub priority_fee_mode: String,
}

impl ClientConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn get_template(&self, name: &str) -> Option<&RequestTemplate> {
        self.templates.iter().find(|t| t.name == name)
    }
}

pub async fn execute_template(
    client: &mut TestClient,
    template: &RequestTemplate,
) -> Result<TradeExecutionResult> {
    let params = TradeParams {
        token_in: template.token_in.clone(),
        token_out: template.token_out.clone(),
        amount: template.amount,
        slippage_bps: template.slippage_bps,
        mode: match template.trading_mode.as_str() {
            "live" => TradingMode::Live,
            _ => TradingMode::Paper,
        },
        priority_fee: Some(template.priority_fee_mode.to_lamports()),
        request_id: None,
    };
    
    client.execute_trade(params, false).await
}
```

### 4. Batch Processing

```rust
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug, Deserialize)]
pub struct BatchRequest {
    pub trades: Vec<TradeParams>,
}

pub async fn execute_batch(
    client: Arc<TestClient>,
    batch_file: &str,
    concurrent_limit: usize,
    delay_ms: u64,
) -> Result<BatchResults> {
    // Load batch requests
    let content = fs::read_to_string(batch_file)?;
    let batch: BatchRequest = serde_json::from_str(&content)?;
    
    let total_requests = batch.trades.len();
    let semaphore = Arc::new(Semaphore::new(concurrent_limit));
    let results = Arc::new(Mutex::new(Vec::new()));
    
    println!("Executing {} trades with concurrency limit {}", total_requests, concurrent_limit);
    
    let start_time = Instant::now();
    
    // Execute trades concurrently with rate limiting
    let futures = batch.trades.into_iter().enumerate().map(|(index, params)| {
        let client = client.clone();
        let semaphore = semaphore.clone();
        let results = results.clone();
        
        async move {
            // Acquire semaphore permit
            let _permit = semaphore.acquire().await.unwrap();
            
            // Add delay if specified
            if delay_ms > 0 && index > 0 {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
            
            // Execute trade
            let mut client_mut = client.as_ref().clone();
            match client_mut.execute_trade(params, false).await {
                Ok(result) => {
                    println!("Trade {} completed: {} ({}ms)",
                        index + 1,
                        if result.success { "SUCCESS" } else { "FAILED" },
                        result.latency.as_millis()
                    );
                    results.lock().unwrap().push(result);
                }
                Err(e) => {
                    eprintln!("Trade {} failed: {}", index + 1, e);
                }
            }
        }
    });
    
    // Wait for all trades to complete
    stream::iter(futures)
        .buffer_unordered(concurrent_limit)
        .collect::<Vec<_>>()
        .await;
    
    let elapsed = start_time.elapsed();
    let results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    
    // Calculate statistics
    let successful = results.iter().filter(|r| r.success).count();
    let failed = total_requests - successful;
    let avg_latency = if !results.is_empty() {
        results.iter().map(|r| r.latency.as_millis()).sum::<u128>() / results.len() as u128
    } else {
        0
    };
    
    Ok(BatchResults {
        total_requests,
        successful,
        failed,
        total_time: elapsed,
        average_latency_ms: avg_latency as u64,
        throughput: total_requests as f64 / elapsed.as_secs_f64(),
        results,
    })
}

#[derive(Debug)]
pub struct BatchResults {
    pub total_requests: usize,
    pub successful: usize,
    pub failed: usize,
    pub total_time: Duration,
    pub average_latency_ms: u64,
    pub throughput: f64,
    pub results: Vec<TradeExecutionResult>,
}
```

### 5. Performance Testing

```rust
use hdrhistogram::Histogram;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct PerformanceTester {
    client: Arc<TestClient>,
    metrics: Arc<PerformanceMetrics>,
}

#[derive(Default)]
pub struct PerformanceMetrics {
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    total_latency_ns: AtomicU64,
    latency_histogram: Mutex<Histogram<u64>>,
}

impl PerformanceTester {
    pub fn new(client: TestClient) -> Self {
        Self {
            client: Arc::new(client),
            metrics: Arc::new(PerformanceMetrics::default()),
        }
    }
    
    pub async fn run_test(
        &self,
        test_config: PerformanceTestConfig,
    ) -> Result<PerformanceReport> {
        let start_time = Instant::now();
        let semaphore = Arc::new(Semaphore::new(test_config.concurrent_requests));
        
        println!("Starting performance test:");
        println!("  Target requests: {}", test_config.total_requests);
        println!("  Concurrency: {}", test_config.concurrent_requests);
        if let Some(rate) = test_config.target_rps {
            println!("  Target RPS: {}", rate);
        }
        
        // Create request generator
        let request_generator = RequestGenerator::new(test_config.request_pattern);
        
        // Generate and execute requests
        let futures = (0..test_config.total_requests).map(|i| {
            let client = self.client.clone();
            let metrics = self.metrics.clone();
            let semaphore = semaphore.clone();
            let params = request_generator.generate_request(i);
            
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                // Rate limiting
                if let Some(target_rps) = test_config.target_rps {
                    let expected_time = Duration::from_secs_f64(i as f64 / target_rps);
                    let elapsed = start_time.elapsed();
                    if expected_time > elapsed {
                        tokio::time::sleep(expected_time - elapsed).await;
                    }
                }
                
                // Execute request
                let request_start = Instant::now();
                let mut client_mut = client.as_ref().clone();
                
                match client_mut.execute_trade(params, false).await {
                    Ok(result) => {
                        let latency = request_start.elapsed();
                        
                        metrics.total_requests.fetch_add(1, Ordering::Relaxed);
                        if result.success {
                            metrics.successful_requests.fetch_add(1, Ordering::Relaxed);
                        } else {
                            metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
                        }
                        
                        metrics.total_latency_ns.fetch_add(latency.as_nanos() as u64, Ordering::Relaxed);
                        
                        let mut hist = metrics.latency_histogram.lock().unwrap();
                        hist.record(latency.as_micros() as u64).unwrap();
                    }
                    Err(_) => {
                        metrics.total_requests.fetch_add(1, Ordering::Relaxed);
                        metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });
        
        // Execute all requests
        stream::iter(futures)
            .buffer_unordered(test_config.concurrent_requests)
            .collect::<Vec<_>>()
            .await;
        
        let elapsed = start_time.elapsed();
        
        // Generate report
        self.generate_report(elapsed)
    }
    
    fn generate_report(&self, elapsed: Duration) -> Result<PerformanceReport> {
        let total = self.metrics.total_requests.load(Ordering::Relaxed);
        let successful = self.metrics.successful_requests.load(Ordering::Relaxed);
        let failed = self.metrics.failed_requests.load(Ordering::Relaxed);
        let total_latency = self.metrics.total_latency_ns.load(Ordering::Relaxed);
        
        let hist = self.metrics.latency_histogram.lock().unwrap();
        
        Ok(PerformanceReport {
            duration: elapsed,
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            requests_per_second: total as f64 / elapsed.as_secs_f64(),
            average_latency_ms: (total_latency / total) / 1_000_000,
            p50_latency_ms: hist.value_at_percentile(50.0) / 1_000,
            p90_latency_ms: hist.value_at_percentile(90.0) / 1_000,
            p95_latency_ms: hist.value_at_percentile(95.0) / 1_000,
            p99_latency_ms: hist.value_at_percentile(99.0) / 1_000,
            p999_latency_ms: hist.value_at_percentile(99.9) / 1_000,
            max_latency_ms: hist.max() / 1_000,
        })
    }
}

#[derive(Debug)]
pub struct PerformanceTestConfig {
    pub total_requests: usize,
    pub concurrent_requests: usize,
    pub target_rps: Option<f64>,
    pub request_pattern: RequestPattern,
}

#[derive(Debug, Clone)]
pub enum RequestPattern {
    Fixed(TradeParams),
    RandomPairs { tokens: Vec<String> },
    Weighted { patterns: Vec<(TradeParams, f64)> },
}

#[derive(Debug)]
pub struct PerformanceReport {
    pub duration: Duration,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub requests_per_second: f64,
    pub average_latency_ms: u64,
    pub p50_latency_ms: u64,
    pub p90_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub p99_latency_ms: u64,
    pub p999_latency_ms: u64,
    pub max_latency_ms: u64,
}

impl PerformanceReport {
    pub fn print_summary(&self) {
        println!("\n=== Performance Test Results ===");
        println!("Duration: {:?}", self.duration);
        println!("Total Requests: {}", self.total_requests);
        println!("Successful: {} ({:.2}%)", 
            self.successful_requests,
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        );
        println!("Failed: {}", self.failed_requests);
        println!("Throughput: {:.2} req/s", self.requests_per_second);
        println!("\nLatency Statistics:");
        println!("  Average: {}ms", self.average_latency_ms);
        println!("  P50: {}ms", self.p50_latency_ms);
        println!("  P90: {}ms", self.p90_latency_ms);
        println!("  P95: {}ms", self.p95_latency_ms);
        println!("  P99: {}ms", self.p99_latency_ms);
        println!("  P99.9: {}ms", self.p999_latency_ms);
        println!("  Max: {}ms", self.max_latency_ms);
    }
}
```

### 6. Error Scenario Testing

```rust
pub struct ErrorScenarioTester {
    client: TestClient,
}

impl ErrorScenarioTester {
    pub async fn run_error_scenarios(&mut self) -> Result<ErrorTestReport> {
        let mut results = Vec::new();
        
        // Test invalid token address
        results.push(self.test_invalid_token().await);
        
        // Test excessive slippage
        results.push(self.test_excessive_slippage().await);
        
        // Test invalid amount
        results.push(self.test_invalid_amount().await);
        
        // Test missing auth token
        results.push(self.test_missing_auth().await);
        
        // Test timeout handling
        results.push(self.test_timeout().await);
        
        Ok(ErrorTestReport { results })
    }
    
    async fn test_invalid_token(&mut self) -> ErrorTestResult {
        let params = TradeParams {
            token_in: "INVALID_TOKEN".to_string(),
            token_out: "SOL".to_string(),
            amount: 10.0,
            slippage_bps: 100,
            mode: TradingMode::Paper,
            priority_fee: None,
            request_id: Some("error-test-invalid-token".to_string()),
        };
        
        match self.client.execute_trade(params, false).await {
            Ok(_) => ErrorTestResult {
                scenario: "Invalid Token".to_string(),
                expected_error: true,
                got_error: false,
                error_type: None,
                passed: false,
            },
            Err(e) => ErrorTestResult {
                scenario: "Invalid Token".to_string(),
                expected_error: true,
                got_error: true,
                error_type: Some(format!("{:?}", e)),
                passed: true,
            },
        }
    }
    
    async fn test_excessive_slippage(&mut self) -> ErrorTestResult {
        let params = TradeParams {
            token_in: "USDC".to_string(),
            token_out: "SOL".to_string(),
            amount: 10.0,
            slippage_bps: 10000, // 100% slippage
            mode: TradingMode::Paper,
            priority_fee: None,
            request_id: Some("error-test-excessive-slippage".to_string()),
        };
        
        match self.client.execute_trade(params, false).await {
            Ok(_) => ErrorTestResult {
                scenario: "Excessive Slippage".to_string(),
                expected_error: true,
                got_error: false,
                error_type: None,
                passed: false,
            },
            Err(e) => ErrorTestResult {
                scenario: "Excessive Slippage".to_string(),
                expected_error: true,
                got_error: true,
                error_type: Some(format!("{:?}", e)),
                passed: true,
            },
        }
    }
    
    // Additional error test implementations...
}

#[derive(Debug)]
pub struct ErrorTestResult {
    pub scenario: String,
    pub expected_error: bool,
    pub got_error: bool,
    pub error_type: Option<String>,
    pub passed: bool,
}

#[derive(Debug)]
pub struct ErrorTestReport {
    pub results: Vec<ErrorTestResult>,
}

impl ErrorTestReport {
    pub fn print_summary(&self) {
        println!("\n=== Error Scenario Test Results ===");
        
        let passed = self.results.iter().filter(|r| r.passed).count();
        let total = self.results.len();
        
        println!("Total Scenarios: {}", total);
        println!("Passed: {} ({:.2}%)", passed, (passed as f64 / total as f64) * 100.0);
        println!("\nDetails:");
        
        for result in &self.results {
            println!("  {} - {}: {}", 
                if result.passed { "✓" } else { "✗" },
                result.scenario,
                if result.passed { "PASSED" } else { "FAILED" }
            );
            if !result.passed {
                println!("    Expected error: {}, Got error: {}",
                    result.expected_error,
                    result.got_error
                );
            }
            if let Some(error) = &result.error_type {
                println!("    Error: {}", error);
            }
        }
    }
}
```

### 7. Main Command Processing

```rust
async fn process_command(matches: &ArgMatches<'_>) -> Result<()> {
    let endpoint = matches.value_of("endpoint").unwrap();
    let auth_token = matches.value_of("auth-token").map(|s| s.to_string());
    let verbose = matches.occurrences_of("verbose") > 0;
    
    // Initialize logging
    if verbose {
        env_logger::init_from_env(
            env_logger::Env::new().default_filter_or("debug")
        );
    } else {
        env_logger::init_from_env(
            env_logger::Env::new().default_filter_or("info")
        );
    }
    
    // Create client
    let mut client = TestClient::new(endpoint, auth_token).await?;
    
    match matches.subcommand() {
        ("trade", Some(sub_matches)) => {
            handle_trade_command(&mut client, sub_matches).await?
        }
        ("batch", Some(sub_matches)) => {
            handle_batch_command(&client, sub_matches).await?
        }
        ("template", Some(sub_matches)) => {
            handle_template_command(&mut client, sub_matches).await?
        }
        ("perf", Some(sub_matches)) => {
            handle_perf_command(&client, sub_matches).await?
        }
        ("history", Some(sub_matches)) => {
            handle_history_command(&mut client, sub_matches).await?
        }
        ("positions", Some(sub_matches)) => {
            handle_positions_command(&mut client, sub_matches).await?
        }
        ("health", _) => {
            handle_health_command(&mut client).await?
        }
        _ => {
            eprintln!("No subcommand specified");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

async fn handle_trade_command(
    client: &mut TestClient,
    matches: &ArgMatches<'_>,
) -> Result<()> {
    let params = TradeParams {
        token_in: matches.value_of("token-in").unwrap().to_string(),
        token_out: matches.value_of("token-out").unwrap().to_string(),
        amount: matches.value_of("amount").unwrap().parse()?,
        slippage_bps: matches.value_of("slippage").unwrap().parse()?,
        mode: match matches.value_of("mode").unwrap() {
            "live" => TradingMode::Live,
            _ => TradingMode::Paper,
        },
        priority_fee: matches.value_of("priority-fee")
            .map(|v| v.parse())
            .transpose()?,
        request_id: None,
    };
    
    let stream_updates = matches.is_present("stream");
    
    println!("Executing trade: {} {} -> {}",
        params.amount, params.token_in, params.token_out
    );
    
    let result = client.execute_trade(params, stream_updates).await?;
    
    println!("\n=== Trade Result ===");
    println!("Request ID: {}", result.request_id);
    println!("Status: {}", if result.success { "SUCCESS" } else { "FAILED" });
    if !result.transaction_id.is_empty() {
        println!("Transaction: {}", result.transaction_id);
    }
    println!("Execution Price: {:.6}", result.execution_price);
    println!("Slippage: {:.2}%", result.slippage * 100.0);
    println!("Latency: {}ms", result.latency.as_millis());
    
    Ok(())
}

// Additional command handlers...
```

## Example Configuration File

```toml
# test-client.toml

[connection]
endpoint = "http://localhost:50051"
timeout_ms = 5000
retry_attempts = 3

[default_trade]
token_in = "USDC"
token_out = "SOL"
amount = 10.0
slippage_bps = 50
priority_fee_mode = "medium"

[[templates]]
name = "small_sol_buy"
description = "Small SOL purchase for testing"
token_in = "USDC"
token_out = "SOL"
amount = 5.0
slippage_bps = 30
priority_fee_mode = "low"
trading_mode = "paper"

[[templates]]
name = "bonk_scalp"
description = "BONK scalping trade"
token_in = "USDC"
token_out = "BONK"
amount = 20.0
slippage_bps = 100
priority_fee_mode = "high"
trading_mode = "paper"

[[templates]]
name = "large_sol_buy"
description = "Large SOL purchase with MEV protection"
token_in = "USDC"
token_out = "SOL"
amount = 1000.0
slippage_bps = 50
priority_fee_mode = { custom = 15000 }
trading_mode = "paper"
```

## Example Batch File

```json
{
  "trades": [
    {
      "token_in": "USDC",
      "token_out": "SOL",
      "amount": 10.0,
      "slippage_bps": 50,
      "mode": "Paper",
      "priority_fee": 5000
    },
    {
      "token_in": "SOL",
      "token_out": "BONK",
      "amount": 0.1,
      "slippage_bps": 100,
      "mode": "Paper",
      "priority_fee": 10000
    },
    {
      "token_in": "USDC",
      "token_out": "RAY",
      "amount": 25.0,
      "slippage_bps": 75,
      "mode": "Paper",
      "priority_fee": 7500
    }
  ]
}
```

## Usage Examples

```bash
# Execute a single trade
./test-client trade --token-in USDC --token-out SOL --amount 10 --slippage 50

# Execute with streaming updates
./test-client trade --token-in USDC --token-out SOL --amount 10 --stream

# Execute from template
./test-client template --name small_sol_buy

# Run batch trades
./test-client batch --file trades.json --concurrent 5 --delay 100

# Run performance test
./test-client perf --requests 1000 --concurrent 50 --rate 100

# Check service health
./test-client health

# Query trade history
./test-client history --start "2024-01-01T00:00:00Z" --limit 50
```

## Key Features

1. **Comprehensive CLI**: Full-featured command-line interface for all testing scenarios
2. **Template System**: Reusable trade configurations for common scenarios
3. **Batch Processing**: Execute multiple trades with concurrency control
4. **Performance Testing**: Measure latency, throughput, and system limits
5. **Error Testing**: Validate error handling with negative test cases
6. **Real-time Monitoring**: Stream execution updates for visibility
7. **Metrics Collection**: Detailed performance metrics and statistics