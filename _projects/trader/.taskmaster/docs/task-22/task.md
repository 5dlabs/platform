# Task 22: Implement Comprehensive Testing Framework for All Components

## Overview

This task establishes a comprehensive testing framework that ensures the reliability, performance, and accuracy of the entire trading system. The framework covers unit testing, integration testing, performance benchmarking, and correlation validation to verify that the system meets its critical 85-90% paper-to-live trading correlation target. This testing infrastructure is essential for maintaining code quality and preventing regressions as the system evolves.

## Dependencies

This task depends on:
- Task 1: Common Rust Libraries for Trading Models - Provides core structures to test
- Task 2: PostgreSQL Database Schema and Migration System - Database layer to test
- Task 3: QuestDB Time-Series Database Integration - Time-series functionality to validate
- Task 5: Paper Trader Core Trading Engine - Paper trading logic to test
- Task 6: Virtual Portfolio Manager for Paper Trading - Portfolio calculations to verify
- Task 15: Paper Trader Binary with TUI - Complete paper trader to test
- Task 19: Paper Trader Comprehensive Testing Suite - Existing tests to expand upon

## Architecture Context

The testing framework aligns with the architecture.md specifications by:
- Validating performance targets (Redis <1ms, QuestDB 100ms batches, circuit breaker 200ms P99)
- Ensuring MEV simulation accuracy (15-20% attack probability for memecoins)
- Verifying failover mechanisms (<250ms Jupiter fallback)
- Confirming paper-to-live correlation meets the 85-90% target

## Implementation Details

### 1. Unit Testing Framework

```rust
// tests/common/mod.rs - Shared testing utilities
use std::sync::Arc;
use tokio::sync::RwLock;
use mockall::automock;

/// Test fixture for common test data
pub struct TestFixture {
    pub tokens: TokenFixtures,
    pub trades: TradeFixtures,
    pub config: TestConfig,
}

impl TestFixture {
    pub fn new() -> Self {
        Self {
            tokens: TokenFixtures::default(),
            trades: TradeFixtures::default(),
            config: TestConfig::test_defaults(),
        }
    }
}

/// Standard tokens used in tests
pub struct TokenFixtures {
    pub sol: TokenInfo,
    pub usdc: TokenInfo,
    pub bonk: TokenInfo,
}

impl Default for TokenFixtures {
    fn default() -> Self {
        Self {
            sol: TokenInfo {
                symbol: "SOL".to_string(),
                address: Pubkey::new_unique(),
                decimals: 9,
                has_transfer_fee: false,
            },
            usdc: TokenInfo {
                symbol: "USDC".to_string(),
                address: Pubkey::new_unique(),
                decimals: 6,
                has_transfer_fee: false,
            },
            bonk: TokenInfo {
                symbol: "BONK".to_string(),
                address: Pubkey::new_unique(),
                decimals: 5,
                has_transfer_fee: false,
            },
        }
    }
}

/// Trade data generators for testing
pub struct TradeFixtures;

impl TradeFixtures {
    pub fn generate_trades(count: usize) -> Vec<Trade> {
        (0..count)
            .map(|i| Trade {
                id: Uuid::new_v4(),
                timestamp: Utc::now() - Duration::minutes(i as i64),
                action: if i % 2 == 0 { TradeAction::Buy } else { TradeAction::Sell },
                base_token: "SOL".to_string(),
                quote_token: "USDC".to_string(),
                amount: 10.0 + (i as f64),
                price: 100.0 + (i as f64 * 0.1),
                fee: 0.025,
                slippage: 0.001 * (i % 5) as f64,
                priority_fee: Some(1000 + (i * 100) as u64),
                tx_signature: Some(format!("sig_{}", i)),
                transfer_fee: None,
                extension_data: None,
            })
            .collect()
    }
    
    pub fn random_trade() -> Trade {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        Trade {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: if rng.gen_bool(0.5) { TradeAction::Buy } else { TradeAction::Sell },
            base_token: "SOL".to_string(),
            quote_token: "USDC".to_string(),
            amount: rng.gen_range(1.0..100.0),
            price: rng.gen_range(50.0..150.0),
            fee: rng.gen_range(0.001..0.01),
            slippage: rng.gen_range(-0.01..0.01),
            priority_fee: Some(rng.gen_range(1000..10000)),
            tx_signature: Some(format!("sig_{}", Uuid::new_v4())),
            transfer_fee: if rng.gen_bool(0.1) { Some(rng.gen_range(0.01..0.05)) } else { None },
            extension_data: None,
        }
    }
}

/// Property-based testing helpers
pub mod prop_helpers {
    use proptest::prelude::*;
    
    pub fn arb_token_symbol() -> impl Strategy<Value = String> {
        prop::string::string_regex("[A-Z]{3,6}").unwrap()
    }
    
    pub fn arb_price() -> impl Strategy<Value = f64> {
        (0.001f64..10000.0).prop_map(|p| (p * 1000.0).round() / 1000.0)
    }
    
    pub fn arb_amount() -> impl Strategy<Value = f64> {
        (0.1f64..1000.0).prop_map(|a| (a * 100.0).round() / 100.0)
    }
    
    pub fn arb_slippage_bps() -> impl Strategy<Value = u16> {
        1u16..500u16
    }
    
    pub fn arb_trade() -> impl Strategy<Value = Trade> {
        (
            arb_token_symbol(),
            arb_token_symbol(),
            arb_amount(),
            arb_price(),
            prop::bool::ANY,
        ).prop_map(|(base, quote, amount, price, is_buy)| {
            Trade {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                action: if is_buy { TradeAction::Buy } else { TradeAction::Sell },
                base_token: base,
                quote_token: quote,
                amount,
                price,
                fee: amount * 0.0025,
                slippage: 0.001,
                priority_fee: Some(1000),
                tx_signature: None,
                transfer_fee: None,
                extension_data: None,
            }
        })
    }
}

/// Assertion helpers for common test scenarios
pub mod assertions {
    use approx::assert_relative_eq;
    
    pub fn assert_price_within_tolerance(actual: f64, expected: f64, tolerance_percent: f64) {
        let tolerance = expected * (tolerance_percent / 100.0);
        assert!(
            (actual - expected).abs() <= tolerance,
            "Price {} not within {}% of expected {}",
            actual, tolerance_percent, expected
        );
    }
    
    pub fn assert_slippage_reasonable(actual_slippage: f64, max_slippage_bps: u16) {
        let max_slippage = max_slippage_bps as f64 / 10000.0;
        assert!(
            actual_slippage <= max_slippage,
            "Slippage {} exceeds maximum {} bps",
            actual_slippage * 10000.0, max_slippage_bps
        );
    }
    
    pub fn assert_correlation(values1: &[f64], values2: &[f64], min_correlation: f64) {
        let correlation = calculate_correlation(values1, values2);
        assert!(
            correlation >= min_correlation,
            "Correlation {} below minimum threshold {}",
            correlation, min_correlation
        );
    }
}
```

### 2. Mock Implementations for External Dependencies

```rust
// tests/mocks/solana_client.rs
use mockall::automock;
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::Transaction};

#[automock]
pub trait SolanaClient: Send + Sync + 'static {
    async fn get_account(&self, pubkey: &Pubkey) -> Result<Account, ClientError>;
    async fn submit_transaction(&self, tx: &Transaction) -> Result<Signature, ClientError>;
    async fn get_recent_blockhash(&self) -> Result<Hash, ClientError>;
    async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64, ClientError>;
    async fn get_transaction(&self, signature: &Signature) -> Result<TransactionStatus, ClientError>;
}

pub struct TestSolanaClient {
    accounts: Arc<RwLock<HashMap<Pubkey, Account>>>,
    transactions: Arc<RwLock<Vec<(Transaction, Signature)>>>,
    latency_ms: Arc<RwLock<u64>>,
    error_rate: Arc<RwLock<f64>>,
}

impl TestSolanaClient {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(Vec::new())),
            latency_ms: Arc::new(RwLock::new(50)),
            error_rate: Arc::new(RwLock::new(0.0)),
        }
    }
    
    pub async fn set_account(&self, pubkey: Pubkey, account: Account) {
        self.accounts.write().await.insert(pubkey, account);
    }
    
    pub async fn set_latency(&self, latency_ms: u64) {
        *self.latency_ms.write().await = latency_ms;
    }
    
    pub async fn set_error_rate(&self, rate: f64) {
        *self.error_rate.write().await = rate;
    }
    
    async fn maybe_fail(&self) -> Result<(), ClientError> {
        let error_rate = *self.error_rate.read().await;
        if rand::random::<f64>() < error_rate {
            return Err(ClientError::NetworkError("Simulated failure".to_string()));
        }
        Ok(())
    }
}

#[async_trait]
impl SolanaClient for TestSolanaClient {
    async fn get_account(&self, pubkey: &Pubkey) -> Result<Account, ClientError> {
        // Simulate latency
        let latency = *self.latency_ms.read().await;
        tokio::time::sleep(Duration::from_millis(latency)).await;
        
        self.maybe_fail().await?;
        
        self.accounts
            .read()
            .await
            .get(pubkey)
            .cloned()
            .ok_or_else(|| ClientError::AccountNotFound)
    }
    
    async fn submit_transaction(&self, tx: &Transaction) -> Result<Signature, ClientError> {
        let latency = *self.latency_ms.read().await;
        tokio::time::sleep(Duration::from_millis(latency)).await;
        
        self.maybe_fail().await?;
        
        let signature = Signature::new_unique();
        self.transactions.write().await.push((tx.clone(), signature));
        Ok(signature)
    }
}

// tests/mocks/jupiter_client.rs
pub struct MockJupiterClient {
    quotes: Arc<RwLock<HashMap<(String, String), Quote>>>,
    failover_behavior: Arc<RwLock<FailoverBehavior>>,
}

#[derive(Clone)]
pub enum FailoverBehavior {
    AlwaysSucceed,
    AlwaysFail,
    FailFirstNTimes(usize),
    RandomFailure(f64),
}

impl MockJupiterClient {
    pub fn new() -> Self {
        Self {
            quotes: Arc::new(RwLock::new(HashMap::new())),
            failover_behavior: Arc::new(RwLock::new(FailoverBehavior::AlwaysSucceed)),
        }
    }
    
    pub async fn set_quote(&self, from: &str, to: &str, quote: Quote) {
        self.quotes.write().await.insert((from.to_string(), to.to_string()), quote);
    }
    
    pub async fn set_failover_behavior(&self, behavior: FailoverBehavior) {
        *self.failover_behavior.write().await = behavior;
    }
}
```

### 3. Integration Testing Framework

```rust
// tests/integration/mod.rs
use testcontainers::{clients::Cli, images::postgres::Postgres, Container};
use redis::aio::ConnectionManager;

pub struct TestEnvironment {
    docker: Cli,
    postgres: Container<'static, Postgres>,
    redis: Container<'static, GenericImage>,
    questdb: Container<'static, GenericImage>,
    postgres_url: String,
    redis_url: String,
    questdb_url: String,
}

impl TestEnvironment {
    pub async fn new() -> Result<Self> {
        let docker = Cli::default();
        
        // Start PostgreSQL
        let postgres = docker.run(Postgres::default());
        let postgres_port = postgres.get_host_port(5432);
        let postgres_url = format!("postgresql://postgres:postgres@localhost:{}/test", postgres_port);
        
        // Start Redis
        let redis_image = GenericImage::new("redis", "7-alpine")
            .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"));
        let redis = docker.run(redis_image);
        let redis_port = redis.get_host_port(6379);
        let redis_url = format!("redis://localhost:{}", redis_port);
        
        // Start QuestDB
        let questdb_image = GenericImage::new("questdb/questdb", "latest")
            .with_exposed_port(9000)
            .with_exposed_port(8812)
            .with_wait_for(WaitFor::message_on_stdout("server started"));
        let questdb = docker.run(questdb_image);
        let questdb_port = questdb.get_host_port(8812);
        let questdb_url = format!("postgresql://admin:quest@localhost:{}/qdb", questdb_port);
        
        let env = Self {
            docker,
            postgres,
            redis,
            questdb,
            postgres_url,
            redis_url,
            questdb_url,
        };
        
        // Run migrations
        env.run_migrations().await?;
        
        Ok(env)
    }
    
    async fn run_migrations(&self) -> Result<()> {
        // Run PostgreSQL migrations
        let mut conn = PgConnection::connect(&self.postgres_url).await?;
        sqlx::migrate!("./migrations/postgres").run(&mut conn).await?;
        
        // Initialize QuestDB tables
        let questdb_conn = PgConnection::connect(&self.questdb_url).await?;
        sqlx::raw_sql(include_str!("../../migrations/questdb/init.sql"))
            .execute(&questdb_conn)
            .await?;
        
        Ok(())
    }
    
    pub async fn get_postgres_pool(&self) -> Result<PgPool> {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.postgres_url)
            .await
            .context("Failed to create PostgreSQL pool")
    }
    
    pub async fn get_redis_conn(&self) -> Result<ConnectionManager> {
        let client = redis::Client::open(self.redis_url.as_str())?;
        ConnectionManager::new(client).await
            .context("Failed to create Redis connection")
    }
    
    pub async fn get_questdb_pool(&self) -> Result<PgPool> {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.questdb_url)
            .await
            .context("Failed to create QuestDB pool")
    }
}

// Integration test example
#[tokio::test]
async fn test_full_trade_flow() {
    let env = TestEnvironment::new().await.unwrap();
    let postgres = env.get_postgres_pool().await.unwrap();
    let redis = env.get_redis_conn().await.unwrap();
    let questdb = env.get_questdb_pool().await.unwrap();
    
    // Initialize components
    let config = TradingConfig::test_defaults();
    let risk_manager = RiskManager::new(postgres.clone(), config.risk_parameters).await.unwrap();
    let price_cache = PriceCache::new(redis).await.unwrap();
    let metrics_store = MetricsStore::new(questdb).await.unwrap();
    
    // Set up test data
    price_cache.set_price("SOL", 100.0, 60).await.unwrap();
    price_cache.set_price("USDC", 1.0, 60).await.unwrap();
    
    // Execute trade
    let trade = Trade {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        action: TradeAction::Buy,
        base_token: "USDC".to_string(),
        quote_token: "SOL".to_string(),
        amount: 1000.0,
        price: 100.0,
        fee: 2.5,
        slippage: 0.001,
        priority_fee: Some(5000),
        tx_signature: Some("test_sig".to_string()),
        transfer_fee: None,
        extension_data: None,
    };
    
    // Validate risk
    risk_manager.validate_trade(&trade).await.unwrap();
    
    // Store metrics
    metrics_store.record_trade(&trade).await.unwrap();
    
    // Verify storage
    let stored_trades = metrics_store.get_trades(
        Utc::now() - Duration::minutes(1),
        Utc::now()
    ).await.unwrap();
    
    assert_eq!(stored_trades.len(), 1);
    assert_eq!(stored_trades[0].id, trade.id);
}
```

### 4. Performance Benchmarking Suite

```rust
// benches/performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;

fn bench_redis_cache(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let redis_client = rt.block_on(create_redis_client()).unwrap();
    let cache = PriceCache::new(redis_client);
    
    // Populate cache
    rt.block_on(async {
        for i in 0..100 {
            cache.set_price(&format!("TOKEN_{}", i), 100.0 + i as f64, 10).await.unwrap();
        }
    });
    
    c.bench_function("redis_cache_read", |b| {
        b.to_async(&rt).iter(|| async {
            let price = cache.get_price("TOKEN_50").await.unwrap();
            black_box(price);
        });
    });
    
    c.bench_function("redis_cache_write", |b| {
        b.to_async(&rt).iter(|| async {
            cache.set_price("BENCH_TOKEN", 123.45, 5).await.unwrap();
        });
    });
}

fn bench_questdb_batch_writes(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let questdb = rt.block_on(create_questdb_connection()).unwrap();
    let metrics_store = MetricsStore::new(questdb);
    
    let mut group = c.benchmark_group("questdb_writes");
    
    for batch_size in [10, 50, 100, 500].iter() {
        let trades: Vec<Trade> = (0..*batch_size)
            .map(|_| TradeFixtures::random_trade())
            .collect();
        
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    metrics_store.batch_insert_trades(&trades).await.unwrap();
                });
            },
        );
    }
    
    group.finish();
}

fn bench_circuit_breaker_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        latency_threshold_ms: 200,
        error_rate_threshold: 0.05,
        failure_threshold: 5,
        recovery_timeout: Duration::from_secs(60),
        half_open_max_requests: 3,
    });
    
    c.bench_function("circuit_breaker_healthy_check", |b| {
        b.to_async(&rt).iter(|| async {
            let is_healthy = circuit_breaker.is_healthy().await;
            black_box(is_healthy);
        });
    });
    
    c.bench_function("circuit_breaker_record_success", |b| {
        b.to_async(&rt).iter(|| async {
            circuit_breaker.record_success(50).await;
        });
    });
}

fn bench_jupiter_failover(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Set up mock Jupiter clients
    let primary = MockJupiterClient::new();
    let fallback = MockJupiterClient::new();
    
    rt.block_on(async {
        primary.set_failover_behavior(FailoverBehavior::AlwaysFail).await;
        fallback.set_quote("SOL", "USDC", Quote {
            price: 100.0,
            out_amount: 100_000_000,
            fee: 250_000,
        }).await;
    });
    
    let jupiter_client = JupiterClientWithFailover::new(primary, fallback);
    
    c.bench_function("jupiter_failover_time", |b| {
        b.to_async(&rt).iter(|| async {
            let quote = jupiter_client.get_quote(&SwapParams {
                from_token: "SOL".to_string(),
                to_token: "USDC".to_string(),
                amount: 1_000_000_000,
                slippage_bps: 100,
            }).await.unwrap();
            black_box(quote);
        });
    });
}

criterion_group!(
    benches,
    bench_redis_cache,
    bench_questdb_batch_writes,
    bench_circuit_breaker_latency,
    bench_jupiter_failover
);
criterion_main!(benches);
```

### 5. Correlation Testing Framework

```rust
// tests/correlation/mod.rs
use statistical::{mean, standard_deviation, correlation};

pub struct CorrelationTester {
    paper_trades: Vec<TradeResult>,
    live_trades: Vec<TradeResult>,
}

#[derive(Clone, Debug)]
pub struct TradeResult {
    pub timestamp: DateTime<Utc>,
    pub token_pair: (String, String),
    pub expected_price: f64,
    pub executed_price: f64,
    pub slippage: f64,
    pub latency_ms: u64,
}

impl CorrelationTester {
    pub fn new() -> Self {
        Self {
            paper_trades: Vec::new(),
            live_trades: Vec::new(),
        }
    }
    
    pub fn add_paper_trade(&mut self, result: TradeResult) {
        self.paper_trades.push(result);
    }
    
    pub fn add_live_trade(&mut self, result: TradeResult) {
        self.live_trades.push(result);
    }
    
    pub fn calculate_price_correlation(&self) -> Result<f64> {
        if self.paper_trades.len() != self.live_trades.len() {
            return Err(anyhow!("Mismatched trade counts"));
        }
        
        let paper_prices: Vec<f64> = self.paper_trades
            .iter()
            .map(|t| t.executed_price)
            .collect();
            
        let live_prices: Vec<f64> = self.live_trades
            .iter()
            .map(|t| t.executed_price)
            .collect();
        
        Ok(correlation(&paper_prices, &live_prices))
    }
    
    pub fn calculate_slippage_correlation(&self) -> Result<f64> {
        let paper_slippage: Vec<f64> = self.paper_trades
            .iter()
            .map(|t| t.slippage)
            .collect();
            
        let live_slippage: Vec<f64> = self.live_trades
            .iter()
            .map(|t| t.slippage)
            .collect();
        
        Ok(correlation(&paper_slippage, &live_slippage))
    }
    
    pub fn generate_correlation_report(&self) -> CorrelationReport {
        let price_correlation = self.calculate_price_correlation().unwrap_or(0.0);
        let slippage_correlation = self.calculate_slippage_correlation().unwrap_or(0.0);
        
        let price_variance = self.calculate_price_variance();
        let latency_comparison = self.compare_latencies();
        
        CorrelationReport {
            price_correlation,
            slippage_correlation,
            price_variance,
            latency_comparison,
            sample_size: self.paper_trades.len(),
            meets_target: price_correlation >= 0.85 && price_correlation <= 0.90,
        }
    }
    
    fn calculate_price_variance(&self) -> f64 {
        let variances: Vec<f64> = self.paper_trades
            .iter()
            .zip(self.live_trades.iter())
            .map(|(paper, live)| {
                ((paper.executed_price - live.executed_price) / live.executed_price).abs()
            })
            .collect();
        
        mean(&variances)
    }
    
    fn compare_latencies(&self) -> LatencyComparison {
        let paper_latencies: Vec<f64> = self.paper_trades
            .iter()
            .map(|t| t.latency_ms as f64)
            .collect();
            
        let live_latencies: Vec<f64> = self.live_trades
            .iter()
            .map(|t| t.latency_ms as f64)
            .collect();
        
        LatencyComparison {
            paper_mean: mean(&paper_latencies),
            paper_p99: percentile(&paper_latencies, 99.0),
            live_mean: mean(&live_latencies),
            live_p99: percentile(&live_latencies, 99.0),
        }
    }
}

#[derive(Debug)]
pub struct CorrelationReport {
    pub price_correlation: f64,
    pub slippage_correlation: f64,
    pub price_variance: f64,
    pub latency_comparison: LatencyComparison,
    pub sample_size: usize,
    pub meets_target: bool,
}

// Correlation test scenarios
#[tokio::test]
async fn test_correlation_under_normal_conditions() {
    let mut tester = CorrelationTester::new();
    
    // Run 100 matched trades
    for i in 0..100 {
        let base_price = 100.0 + (i as f64 * 0.1);
        
        // Paper trade with simulated slippage
        tester.add_paper_trade(TradeResult {
            timestamp: Utc::now(),
            token_pair: ("SOL".to_string(), "USDC".to_string()),
            expected_price: base_price,
            executed_price: base_price * (1.0 - 0.001), // 0.1% slippage
            slippage: 0.001,
            latency_ms: 50 + (i % 20),
        });
        
        // Live trade with slightly different slippage
        tester.add_live_trade(TradeResult {
            timestamp: Utc::now(),
            token_pair: ("SOL".to_string(), "USDC".to_string()),
            expected_price: base_price,
            executed_price: base_price * (1.0 - 0.0012), // 0.12% slippage
            slippage: 0.0012,
            latency_ms: 60 + (i % 30),
        });
    }
    
    let report = tester.generate_correlation_report();
    println!("Correlation Report: {:?}", report);
    
    assert!(report.meets_target, "Correlation {} not in 85-90% range", report.price_correlation);
    assert!(report.price_variance < 0.002, "Price variance too high");
}
```

### 6. MEV Simulation Validation

```rust
// tests/mev/mod.rs
pub struct MevSimulationValidator {
    historical_data: Vec<MevEvent>,
    simulator: MevSimulator,
}

#[derive(Clone, Debug)]
pub struct MevEvent {
    pub timestamp: DateTime<Utc>,
    pub token_pair: (String, String),
    pub trade_size: f64,
    pub pool_liquidity: f64,
    pub was_sandwiched: bool,
    pub loss_bps: Option<u16>,
}

impl MevSimulationValidator {
    pub async fn validate_sandwich_detection(&self) -> ValidationResult {
        let mut true_positives = 0;
        let mut false_positives = 0;
        let mut true_negatives = 0;
        let mut false_negatives = 0;
        
        for event in &self.historical_data {
            let params = SwapParams {
                from_token: event.token_pair.0.clone(),
                to_token: event.token_pair.1.clone(),
                amount: (event.trade_size * 1e9) as u64,
                slippage_bps: 100,
                base_currency: BaseCurrency::SOL,
                tip_lamports: None,
                wrap_and_unwrap_sol: true,
                use_shared_accounts: true,
                compute_unit_price_micro_lamports: None,
            };
            
            let risk = self.simulator.analyze(&params, event.pool_liquidity).await.unwrap();
            let predicted_attack = risk.sandwich_probability > 0.15;
            
            match (predicted_attack, event.was_sandwiched) {
                (true, true) => true_positives += 1,
                (true, false) => false_positives += 1,
                (false, false) => true_negatives += 1,
                (false, true) => false_negatives += 1,
            }
        }
        
        let precision = true_positives as f64 / (true_positives + false_positives) as f64;
        let recall = true_positives as f64 / (true_positives + false_negatives) as f64;
        let f1_score = 2.0 * (precision * recall) / (precision + recall);
        
        ValidationResult {
            precision,
            recall,
            f1_score,
            total_samples: self.historical_data.len(),
        }
    }
    
    pub async fn validate_loss_estimation(&self) -> LossValidation {
        let mut loss_errors = Vec::new();
        
        for event in &self.historical_data.iter().filter(|e| e.was_sandwiched) {
            let params = SwapParams {
                from_token: event.token_pair.0.clone(),
                to_token: event.token_pair.1.clone(),
                amount: (event.trade_size * 1e9) as u64,
                slippage_bps: 100,
                base_currency: BaseCurrency::SOL,
                tip_lamports: None,
                wrap_and_unwrap_sol: true,
                use_shared_accounts: true,
                compute_unit_price_micro_lamports: None,
            };
            
            let risk = self.simulator.analyze(&params, event.pool_liquidity).await.unwrap();
            
            if let Some(actual_loss) = event.loss_bps {
                let error = (risk.estimated_loss_bps as i32 - actual_loss as i32).abs() as f64;
                loss_errors.push(error);
            }
        }
        
        LossValidation {
            mean_absolute_error: mean(&loss_errors),
            max_error: loss_errors.iter().cloned().fold(0.0, f64::max),
            within_tolerance: mean(&loss_errors) < 50.0, // 50 bps tolerance
        }
    }
}

#[tokio::test]
async fn test_mev_protection_effectiveness() {
    let trades_without_protection = vec![
        execute_trade_without_mev_protection("BONK", 1000.0).await,
        execute_trade_without_mev_protection("BONK", 5000.0).await,
        execute_trade_without_mev_protection("BONK", 10000.0).await,
    ];
    
    let trades_with_protection = vec![
        execute_trade_with_mev_protection("BONK", 1000.0, 5000).await,
        execute_trade_with_mev_protection("BONK", 5000.0, 7500).await,
        execute_trade_with_mev_protection("BONK", 10000.0, 10000).await,
    ];
    
    // Calculate sandwich attack rates
    let unprotected_sandwiched = trades_without_protection
        .iter()
        .filter(|t| t.was_sandwiched)
        .count();
        
    let protected_sandwiched = trades_with_protection
        .iter()
        .filter(|t| t.was_sandwiched)
        .count();
    
    let protection_effectiveness = 1.0 - (protected_sandwiched as f64 / unprotected_sandwiched as f64);
    
    println!("MEV Protection Effectiveness: {:.2}%", protection_effectiveness * 100.0);
    assert!(protection_effectiveness > 0.5, "MEV protection should reduce attacks by >50%");
}
```

### 7. Load Testing Framework

```rust
// tests/load/mod.rs
use tokio::task::JoinSet;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct LoadTester {
    config: LoadTestConfig,
    metrics: Arc<LoadTestMetrics>,
}

#[derive(Clone)]
pub struct LoadTestConfig {
    pub concurrent_users: usize,
    pub requests_per_user: usize,
    pub ramp_up_seconds: u64,
    pub test_duration_seconds: u64,
}

#[derive(Default)]
pub struct LoadTestMetrics {
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub total_latency_ms: AtomicU64,
}

impl LoadTester {
    pub async fn run_trade_executor_load_test(&self) -> LoadTestReport {
        let start_time = Instant::now();
        let mut tasks = JoinSet::new();
        
        // Ramp up users gradually
        for user_id in 0..self.config.concurrent_users {
            let delay = (user_id as u64 * self.config.ramp_up_seconds) / self.config.concurrent_users as u64;
            let executor = create_test_trade_executor().await;
            let metrics = self.metrics.clone();
            let requests_per_user = self.config.requests_per_user;
            
            tasks.spawn(async move {
                tokio::time::sleep(Duration::from_secs(delay)).await;
                
                for _ in 0..requests_per_user {
                    let start = Instant::now();
                    
                    match executor.execute_swap(create_random_swap_params()).await {
                        Ok(_) => {
                            metrics.successful_requests.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(_) => {
                            metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    
                    let latency = start.elapsed().as_millis() as u64;
                    metrics.total_latency_ms.fetch_add(latency, Ordering::Relaxed);
                    metrics.total_requests.fetch_add(1, Ordering::Relaxed);
                    
                    // Add some randomness to avoid thundering herd
                    tokio::time::sleep(Duration::from_millis(rand::random::<u64>() % 100)).await;
                }
            });
        }
        
        // Wait for all tasks to complete
        while let Some(_) = tasks.join_next().await {}
        
        let duration = start_time.elapsed();
        self.generate_report(duration)
    }
    
    pub async fn run_circuit_breaker_stress_test(&self) -> CircuitBreakerStressReport {
        let circuit_breaker = Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
            latency_threshold_ms: 200,
            error_rate_threshold: 0.05,
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(10),
            half_open_max_requests: 3,
        }));
        
        let mut tasks = JoinSet::new();
        
        // Spawn tasks that will cause failures
        for i in 0..50 {
            let cb = circuit_breaker.clone();
            tasks.spawn(async move {
                // Simulate varying latencies
                let latency = if i % 10 == 0 { 300 } else { 50 };
                cb.record_success(latency).await;
                
                // Simulate some failures
                if i % 7 == 0 {
                    cb.record_failure().await;
                }
            });
        }
        
        while let Some(_) = tasks.join_next().await {}
        
        // Check circuit breaker behavior
        let status = circuit_breaker.get_status().await;
        
        CircuitBreakerStressReport {
            final_state: status.state,
            total_failures: status.failure_count,
            opened_count: status.times_opened,
            correctly_triggered: status.state == CircuitState::Open && status.current_latency_ms > 200,
        }
    }
    
    fn generate_report(&self, duration: Duration) -> LoadTestReport {
        let total = self.metrics.total_requests.load(Ordering::Relaxed);
        let successful = self.metrics.successful_requests.load(Ordering::Relaxed);
        let failed = self.metrics.failed_requests.load(Ordering::Relaxed);
        let total_latency = self.metrics.total_latency_ms.load(Ordering::Relaxed);
        
        LoadTestReport {
            duration,
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            requests_per_second: total as f64 / duration.as_secs_f64(),
            average_latency_ms: total_latency as f64 / total as f64,
            success_rate: successful as f64 / total as f64,
        }
    }
}

#[tokio::test]
async fn test_system_under_load() {
    let load_tester = LoadTester::new(LoadTestConfig {
        concurrent_users: 100,
        requests_per_user: 50,
        ramp_up_seconds: 10,
        test_duration_seconds: 60,
    });
    
    let report = load_tester.run_trade_executor_load_test().await;
    
    println!("Load Test Report:");
    println!("  Total Requests: {}", report.total_requests);
    println!("  Success Rate: {:.2}%", report.success_rate * 100.0);
    println!("  Average Latency: {:.2}ms", report.average_latency_ms);
    println!("  Requests/Second: {:.2}", report.requests_per_second);
    
    // Assertions
    assert!(report.success_rate > 0.95, "Success rate should be >95%");
    assert!(report.average_latency_ms < 200.0, "Average latency should be <200ms");
}
```

### 8. Test Data Generation Utilities

```rust
// tests/generators/mod.rs
use rand::distributions::{Distribution, Normal};
use rand::seq::SliceRandom;

pub struct TestDataGenerator {
    rng: StdRng,
}

impl TestDataGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
    
    pub fn generate_price_series(&mut self, initial: f64, count: usize, volatility: f64) -> Vec<f64> {
        let mut prices = vec![initial];
        let normal = Normal::new(0.0, volatility);
        
        for _ in 1..count {
            let change = normal.sample(&mut self.rng);
            let new_price = prices.last().unwrap() * (1.0 + change);
            prices.push(new_price.max(0.001)); // Ensure positive
        }
        
        prices
    }
    
    pub fn generate_order_book(&mut self, mid_price: f64, depth: usize) -> OrderBook {
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        
        for i in 0..depth {
            let spread = 0.001 * (i + 1) as f64;
            let size = self.rng.gen_range(100.0..10000.0);
            
            bids.push(Order {
                price: mid_price * (1.0 - spread),
                size,
            });
            
            asks.push(Order {
                price: mid_price * (1.0 + spread),
                size,
            });
        }
        
        OrderBook { bids, asks, mid_price }
    }
    
    pub fn generate_mev_scenarios(&mut self, count: usize) -> Vec<MevScenario> {
        let token_pairs = vec![
            ("SOL", "USDC"),
            ("BONK", "USDC"),
            ("RAY", "USDC"),
            ("JitoSOL", "SOL"),
        ];
        
        (0..count).map(|_| {
            let (from, to) = token_pairs.choose(&mut self.rng).unwrap();
            let trade_size = self.rng.gen_range(100.0..50000.0);
            let pool_liquidity = self.rng.gen_range(100000.0..10000000.0);
            let is_memecoin = from == &"BONK";
            
            // Higher sandwich probability for memecoins and large trades
            let size_impact = trade_size / pool_liquidity;
            let base_probability = if is_memecoin { 0.15 } else { 0.05 };
            let sandwich_probability = (base_probability + size_impact * 2.0).min(0.5);
            
            MevScenario {
                token_pair: (from.to_string(), to.to_string()),
                trade_size,
                pool_liquidity,
                sandwich_probability,
                is_memecoin,
            }
        }).collect()
    }
}

// Test cleanup utilities
pub struct TestCleanup;

impl TestCleanup {
    pub async fn reset_databases(postgres: &PgPool, questdb: &PgPool) -> Result<()> {
        // Clear PostgreSQL tables
        sqlx::query("TRUNCATE TABLE trades, positions, order_rules CASCADE")
            .execute(postgres)
            .await?;
        
        // Clear QuestDB tables (drop and recreate for time-series)
        sqlx::query("DROP TABLE IF EXISTS trades")
            .execute(questdb)
            .await?;
            
        sqlx::query(include_str!("../../migrations/questdb/trades.sql"))
            .execute(questdb)
            .await?;
        
        Ok(())
    }
    
    pub async fn clear_redis(redis: &mut ConnectionManager) -> Result<()> {
        redis::cmd("FLUSHDB")
            .query_async(redis)
            .await
            .context("Failed to flush Redis")?;
        Ok(())
    }
}
```

### 9. CI/CD Pipeline Configuration

```yaml
# .github/workflows/test.yml
name: Comprehensive Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    
    - name: Run unit tests
      run: cargo test --lib --bins
    
    - name: Run doc tests
      run: cargo test --doc

  integration-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
      
      redis:
        image: redis:7-alpine
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379
      
      questdb:
        image: questdb/questdb:latest
        ports:
          - 9000:9000
          - 8812:8812
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true
    
    - name: Run integration tests
      run: cargo test --test '*' -- --test-threads=1
      env:
        DATABASE_URL: postgresql://postgres:postgres@localhost/test
        REDIS_URL: redis://localhost:6379
        QUESTDB_URL: postgresql://admin:quest@localhost:8812/qdb

  benchmarks:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true
    
    - name: Run benchmarks
      run: cargo bench --no-fail-fast | tee benchmark_results.txt
    
    - name: Upload benchmark results
      uses: actions/upload-artifact@v3
      with:
        name: benchmark-results
        path: benchmark_results.txt
    
    - name: Check performance regression
      run: |
        # Parse benchmark results and check against thresholds
        python scripts/check_performance_regression.py benchmark_results.txt

  coverage:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true
    
    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin
    
    - name: Generate coverage
      run: cargo tarpaulin --out Xml --all-features --workspace --timeout 300
    
    - name: Upload to codecov.io
      uses: codecov/codecov-action@v3
      with:
        token: ${{ secrets.CODECOV_TOKEN }}
        fail_ci_if_error: true
```

## Test Organization

The test suite is organized as follows:

```
tests/
├── common/           # Shared test utilities
│   ├── mod.rs       # Test fixtures and helpers
│   └── assertions.rs # Custom assertion functions
├── unit/            # Unit tests for individual components
├── integration/     # Integration tests with real services
├── correlation/     # Paper vs live correlation tests
├── mev/            # MEV simulation validation
├── load/           # Load and stress tests
└── generators/     # Test data generation

benches/            # Performance benchmarks
├── redis.rs        # Redis cache benchmarks
├── questdb.rs      # QuestDB write benchmarks
├── circuit.rs      # Circuit breaker benchmarks
└── jupiter.rs      # Jupiter failover benchmarks
```

## Key Testing Patterns

1. **Mock External Services**: All external dependencies have mock implementations for isolated testing
2. **Property-Based Testing**: Complex logic uses property testing to find edge cases
3. **Performance Validation**: Benchmarks ensure performance targets are met
4. **Correlation Testing**: Validates paper-to-live trading accuracy
5. **Load Testing**: Ensures system stability under high load
6. **CI/CD Integration**: Automated testing on every commit

## Success Metrics

The testing framework validates:
- Redis cache reads complete in <1ms
- QuestDB handles 100ms batch intervals
- Circuit breaker respects 200ms P99 threshold
- Jupiter failover occurs within 250ms
- Paper-to-live correlation maintains 85-90% accuracy
- MEV simulation correctly identifies 80% of attacks
- System maintains >95% success rate under load