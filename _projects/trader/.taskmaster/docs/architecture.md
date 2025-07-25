# Solana Trading Platform - Application Implementation Plan

## System Architecture Overview

The trading platform consists of two primary services sharing common libraries:
- **Paper Trader**: Simulates trades with virtual funds for strategy testing
- **Live Trader**: Executes real trades with actual funds

Both services will be written in Rust and share identical interfaces to enable seamless switching between simulation and live trading. The platform leverages both self-hosted and public Jupiter instances for resilience.

## Core Components

### 1. Common Libraries (`common/`)

#### Trade Engine Core (`common/trade_engine/`)
```rust
// Enhanced trade execution with MEV protection
pub trait TradeExecutor {
    async fn execute_swap(&self, params: SwapParams) -> Result<SwapResult>;
    async fn get_quote(&self, params: QuoteParams) -> Result<Quote>;
    async fn simulate_mev_risk(&self, params: &SwapParams) -> Result<MevRisk>;
}

pub struct SwapParams {
    pub from_token: Pubkey,
    pub to_token: Pubkey,
    pub amount: u64,
    pub slippage_bps: u16,  // basis points
    pub base_currency: BaseCurrency,
    // MEV protection parameters (Jupiter v6)
    pub tip_lamports: Option<u64>,  // 1000-10000 typical
    pub wrap_and_unwrap_sol: bool,
    pub use_shared_accounts: bool,
    pub compute_unit_price_micro_lamports: Option<u64>,
}

pub struct MevRisk {
    pub sandwich_probability: f64,
    pub estimated_loss_bps: u16,
    pub recommended_priority_fee: u64,
}

pub struct SwapResult {
    pub executed_price: f64,
    pub amount_received: u64,
    pub fee: u64,
    pub slippage_actual: f64,
    pub latency_ms: u64,
    pub mev_protected: bool,
}
```

#### Data Models (`common/models/`)
```rust
pub enum BaseCurrency {
    SOL,
    USDC,
}

pub struct Trade {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub action: TradeAction,
    pub base_token: String,
    pub quote_token: String,
    pub amount: f64,
    pub price: f64,
    pub fee: f64,
    pub slippage: f64,
    pub priority_fee: Option<u64>,
    pub tx_signature: Option<String>,
    // Token-2022 extension data
    pub transfer_fee: Option<f64>,
    pub extension_data: Option<serde_json::Value>,
}

pub struct Position {
    pub token: String,
    pub amount: f64,
    pub cost_basis: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_percent: f64,
}

pub struct OrderRule {
    pub id: Uuid,
    pub position_token: String,
    pub rule_type: OrderRuleType,
    pub trigger_price: f64,
    pub amount: f64,
}

pub enum OrderRuleType {
    StopLoss,
    TakeProfit,
}
```

#### Solana Integration (`common/solana/`)
```rust
pub struct SolanaClient {
    grpc_client: GrpcClient,
    latency_tracker: LatencyTracker,
    health_monitor: HealthMonitor,
}

impl SolanaClient {
    pub async fn get_account_data(&self, pubkey: &Pubkey) -> Result<Account> {
        self.latency_tracker.track("get_account", async {
            self.grpc_client.get_account(pubkey).await
        }).await
    }

    pub async fn send_transaction(&self, tx: &Transaction) -> Result<Signature> {
        // Check node health before sending
        if !self.health_monitor.is_healthy().await {
            return Err(anyhow!("Node unhealthy, circuit breaker triggered"));
        }

        self.latency_tracker.track("send_transaction", async {
            self.grpc_client.send_transaction(tx).await
        }).await
    }

    pub async fn get_priority_fee(&self) -> Result<u64> {
        // Dynamic fee calculation based on network congestion
        let recent_fees = self.grpc_client.get_recent_priority_fees().await?;
        Ok(calculate_optimal_fee(recent_fees))
    }
}

pub struct HealthMonitor {
    latency_threshold_ms: u64,  // 200ms P99 target
    error_rate_threshold: f64,
    circuit_breaker: CircuitBreaker,
}
```

#### Database Abstraction (`common/db/`)
```rust
pub trait TradeStore {
    async fn insert_trade(&self, trade: &Trade) -> Result<()>;
    async fn get_trades(&self, filter: TradeFilter) -> Result<Vec<Trade>>;
    async fn get_positions(&self) -> Result<Vec<Position>>;
}

pub trait MetricsStore {
    async fn record_latency(&self, operation: &str, latency_ms: u64) -> Result<()>;
    async fn record_slippage(&self, expected: f64, actual: f64) -> Result<()>;
    async fn record_mev_event(&self, risk: &MevRisk, avoided: bool) -> Result<()>;
}

// Redis cache layer
pub trait PriceCache {
    async fn get_price(&self, token: &str) -> Result<Option<f64>>;
    async fn set_price(&self, token: &str, price: f64, ttl_secs: u64) -> Result<()>;
    async fn get_pool_state(&self, pool: &Pubkey) -> Result<Option<PoolState>>;
}
```

### 2. Paper Trader Service (`paper_trader/`)

#### Virtual Portfolio Manager
```rust
pub struct VirtualPortfolio {
    balances: HashMap<String, f64>,
    trades: Vec<Trade>,
    cost_basis: HashMap<String, f64>,
}

impl VirtualPortfolio {
    pub fn new(initial_sol: f64, initial_usdc: f64) -> Self {
        let mut balances = HashMap::new();
        balances.insert("SOL".to_string(), initial_sol);
        balances.insert("USDC".to_string(), initial_usdc);

        Self {
            balances,
            trades: Vec::new(),
            cost_basis: HashMap::new(),
        }
    }

    pub fn execute_trade(&mut self, trade: Trade) -> Result<()> {
        // Update balances accounting for Token-2022 transfer fees
        let effective_amount = if let Some(transfer_fee) = trade.transfer_fee {
            trade.amount * (1.0 - transfer_fee / 100.0)
        } else {
            trade.amount
        };

        // Update balances
        // Track cost basis
        // Calculate P&L
        Ok(())
    }

    pub fn get_position(&self, token: &str) -> Option<Position> {
        // Return current position with P&L
    }
}
```

#### Price Feed Subscriber with Redis
```rust
pub struct PriceFeed {
    pool_subscribers: HashMap<String, PoolSubscriber>,
    redis_cache: Arc<RedisClient>,
    stream_consumer: StreamConsumer,
}

impl PriceFeed {
    pub async fn subscribe_to_pool(&mut self, pool_address: Pubkey) -> Result<()> {
        // Subscribe to pool account updates via gRPC
        let subscriber = PoolSubscriber::new(pool_address, self.redis_cache.clone());
        subscriber.start_subscription().await?;
        self.pool_subscribers.insert(pool_address.to_string(), subscriber);
        Ok(())
    }

    pub async fn get_price(&self, token: &str) -> Result<f64> {
        // Check Redis cache first (TTL: 1-2 seconds)
        if let Some(price) = self.redis_cache.get_price(token).await? {
            return Ok(price);
        }

        // Fallback to fresh query
        self.fetch_fresh_price(token).await
    }

    pub async fn start_event_stream(&self) -> Result<()> {
        // Use Redis Streams with XREAD blocking
        loop {
            let events = self.stream_consumer
                .read_blocking("price-updates", 100)  // 100ms timeout
                .await?;

            for event in events {
                self.process_price_event(event).await?;
            }
        }
    }
}
```

#### Enhanced Trade Simulator
```rust
pub struct PaperTrader {
    portfolio: Arc<Mutex<VirtualPortfolio>>,
    price_feed: Arc<PriceFeed>,
    jupiter_client: JupiterClientWithFailover,
    slippage_model: SlippageModel,
    mev_simulator: MevSimulator,
}

impl TradeExecutor for PaperTrader {
    async fn execute_swap(&self, params: SwapParams) -> Result<SwapResult> {
        let start = Instant::now();

        // Simulate MEV risk
        let mev_risk = self.simulate_mev_risk(&params).await?;

        // Get quote from Jupiter (with failover)
        let quote = self.jupiter_client.get_quote(&params).await?;

        // Apply enhanced slippage model
        let simulated_price = self.slippage_model.apply(
            quote.price,
            params.amount,
            params.slippage_bps,
            &mev_risk,
        );

        // Update virtual portfolio
        let trade = Trade {
            timestamp: Utc::now(),
            action: TradeAction::Swap,
            base_token: params.from_token.to_string(),
            quote_token: params.to_token.to_string(),
            amount: params.amount as f64,
            price: simulated_price,
            fee: quote.fee as f64,
            slippage: simulated_price / quote.price - 1.0,
            priority_fee: params.tip_lamports,
            tx_signature: None,
            transfer_fee: None,  // Check token extensions
            extension_data: None,
            ..Default::default()
        };

        self.portfolio.lock().await.execute_trade(trade.clone())?;

        // Record to database
        self.db.insert_trade(&trade).await?;

        Ok(SwapResult {
            executed_price: simulated_price,
            amount_received: (params.amount as f64 * simulated_price) as u64,
            fee: quote.fee,
            slippage_actual: simulated_price / quote.price - 1.0,
            latency_ms: start.elapsed().as_millis() as u64,
            mev_protected: params.tip_lamports.is_some(),
        })
    }

    async fn simulate_mev_risk(&self, params: &SwapParams) -> Result<MevRisk> {
        self.mev_simulator.analyze(params).await
    }
}
```

#### MVP Slippage Model
```rust
pub enum SlippageModel {
    // MVP: Simple fixed percentage
    Fixed { percent: f64 },
    // Future: Dynamic with MEV consideration
    Dynamic {
        base_percent: f64,
        size_impact: f64,
        pool_metrics: PoolMetrics,
    },
}

impl SlippageModel {
    pub fn apply(&self, base_price: f64, amount: f64, max_slippage_bps: u16, mev_risk: &MevRisk) -> f64 {
        match self {
            SlippageModel::Fixed { percent } => {
                // MVP implementation: fixed percentage + MEV adjustment
                let mev_adjustment = mev_risk.estimated_loss_bps as f64 / 10000.0;
                base_price * (1.0 - (percent / 100.0 + mev_adjustment))
            }
            SlippageModel::Dynamic { .. } => {
                // Future: incorporate pool depth, recent volume, etc.
                unimplemented!("Dynamic slippage for Phase 2")
            }
        }
    }
}
```

#### MEV Simulator
```rust
pub struct MevSimulator {
    redis_cache: Arc<RedisClient>,
}

impl MevSimulator {
    pub async fn analyze(&self, params: &SwapParams) -> Result<MevRisk> {
        // Get pool state from cache
        let pool_state = self.redis_cache
            .get_pool_state(&params.from_token)
            .await?
            .ok_or_else(|| anyhow!("Pool state not found"))?;

        // Calculate sandwich attack probability
        let trade_impact = params.amount as f64 / pool_state.liquidity;
        let sandwich_probability = if trade_impact > 0.005 {  // >0.5% of pool
            0.15 + (trade_impact * 100.0).min(0.35)  // 15-50% risk
        } else {
            0.05  // Base 5% risk
        };

        // Estimate potential loss
        let estimated_loss_bps = (sandwich_probability * 500.0) as u16;  // Up to 5%

        // Recommend priority fee
        let recommended_priority_fee = calculate_priority_fee(
            sandwich_probability,
            params.amount,
        );

        Ok(MevRisk {
            sandwich_probability,
            estimated_loss_bps,
            recommended_priority_fee,
        })
    }
}
```

### 3. Live Trader Service (`live_trader/`)

#### Enhanced Wallet Manager
```rust
use ring::aead;
use secrecy::{ExposeSecret, Secret};

pub struct WalletManager {
    keypair: Secret<Keypair>,
    recent_blockhash: Arc<RwLock<Hash>>,
    rotation_schedule: RotationSchedule,
}

impl WalletManager {
    pub fn new(encrypted_keypair: &[u8], key: &[u8]) -> Result<Self> {
        // Decrypt keypair using ring
        let keypair_bytes = decrypt_keypair(encrypted_keypair, key)?;
        let keypair = Keypair::from_bytes(&keypair_bytes)?;

        // Zero out decrypted bytes
        keypair_bytes.zeroize();

        Ok(Self {
            keypair: Secret::new(keypair),
            recent_blockhash: Arc::new(RwLock::new(Hash::default())),
            rotation_schedule: RotationSchedule::daily(),
        })
    }

    pub async fn sign_transaction(&self, tx: &mut Transaction) -> Result<()> {
        let blockhash = self.recent_blockhash.read().await.clone();
        tx.message.recent_blockhash = blockhash;
        tx.sign(&[self.keypair.expose_secret()], blockhash);
        Ok(())
    }
}
```

#### Live Trade Executor with Resilience
```rust
pub struct LiveTrader {
    wallet: Arc<WalletManager>,
    solana_client: Arc<SolanaClient>,
    jupiter_client: JupiterClientWithFailover,
    risk_manager: RiskManager,
    circuit_breaker: CircuitBreaker,
}

impl TradeExecutor for LiveTrader {
    async fn execute_swap(&self, params: SwapParams) -> Result<SwapResult> {
        let start = Instant::now();

        // Check circuit breaker
        if !self.circuit_breaker.is_open() {
            return Err(anyhow!("Trading paused due to system issues"));
        }

        // Check risk limits
        self.risk_manager.check_trade(&params)?;

        // Calculate optimal priority fee
        let priority_fee = self.solana_client.get_priority_fee().await?;
        let mut params = params;
        params.tip_lamports = Some(priority_fee);

        // Get swap transaction from Jupiter with MEV protection
        let swap_response = self.jupiter_client
            .get_swap_transaction(&params)
            .await?;

        // Deserialize and sign transaction
        let mut tx = bincode::deserialize::<Transaction>(
            &base64::decode(&swap_response.swap_transaction)?
        )?;

        self.wallet.sign_transaction(&mut tx).await?;

        // Send via gRPC with retry
        let signature = retry_with_backoff(|| {
            self.solana_client.send_transaction(&tx)
        }, 3, Duration::from_millis(100)).await?;

        // Monitor confirmation
        let confirmation = self.wait_for_confirmation(&signature).await?;

        // Record trade
        let trade = Trade {
            timestamp: Utc::now(),
            tx_signature: Some(signature.to_string()),
            priority_fee: params.tip_lamports,
            // ... other fields from confirmation
        };

        self.db.insert_trade(&trade).await?;

        Ok(SwapResult {
            latency_ms: start.elapsed().as_millis() as u64,
            mev_protected: true,
            // ... other fields
        })
    }
}
```

#### Jupiter Client with Failover
```rust
pub struct JupiterClientWithFailover {
    self_hosted_client: JupiterClient,
    public_client: JupiterClient,
    circuit_breaker: CircuitBreaker,
}

impl JupiterClientWithFailover {
    pub async fn get_quote(&self, params: &SwapParams) -> Result<Quote> {
        // Try self-hosted first
        match timeout(Duration::from_millis(200),
            self.self_hosted_client.get_quote(params)
        ).await {
            Ok(Ok(quote)) => return Ok(quote),
            _ => {
                // Fallback to public
                self.circuit_breaker.record_failure();
            }
        }

        // Use public Jupiter
        self.public_client.get_quote(params).await
    }
}
```

### 4. Enhanced Terminal User Interface

#### TUI with Real-time Updates via Redis Streams
```rust
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline},
    layout::{Layout, Constraint, Direction},
    Terminal,
};

pub struct TradingUI {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    app_state: Arc<RwLock<AppState>>,
    event_stream: RedisStreamReader,
}

impl TradingUI {
    pub async fn run(&mut self) -> Result<()> {
        // Start event stream consumer
        let state_clone = self.app_state.clone();
        tokio::spawn(async move {
            let mut reader = RedisStreamReader::new("ui-events");
            loop {
                if let Ok(events) = reader.read_blocking(50).await {
                    for event in events {
                        state_clone.write().await.process_event(event);
                    }
                }
            }
        });

        loop {
            self.terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),   // Header
                        Constraint::Length(4),   // Portfolio
                        Constraint::Min(10),     // Positions
                        Constraint::Length(5),   // P&L Chart
                        Constraint::Length(10),  // Trades
                        Constraint::Length(3),   // Commands
                    ])
                    .split(f.size());

                self.render_header(f, chunks[0]);
                self.render_portfolio(f, chunks[1]);
                self.render_positions(f, chunks[2]);
                self.render_pnl_chart(f, chunks[3]);
                self.render_trades(f, chunks[4]);
                self.render_commands(f, chunks[5]);
            })?;

            // Handle keyboard input
            if let Ok(true) = crossterm::event::poll(Duration::from_millis(50)) {
                self.handle_input().await?;
            }
        }
    }

    fn render_pnl_chart(&self, f: &mut Frame, area: Rect) {
        let app_state = self.app_state.blocking_read();
        let pnl_data: Vec<u64> = app_state.pnl_history
            .iter()
            .map(|&pnl| ((pnl + 100.0) * 100.0) as u64)
            .collect();

        let sparkline = Sparkline::default()
            .block(Block::default().title("P&L %").borders(Borders::ALL))
            .data(&pnl_data)
            .style(Style::default().fg(Color::Green));

        f.render_widget(sparkline, area);
    }
}
```

## Database Schema

### QuestDB Tables (Enhanced)

```sql
CREATE TABLE trades (
    timestamp TIMESTAMP,
    trader_id SYMBOL,
    mode SYMBOL,  -- 'paper' or 'live'
    action SYMBOL,  -- 'buy' or 'sell'
    base_token SYMBOL,
    quote_token SYMBOL,
    amount DOUBLE,
    price DOUBLE,
    slippage DOUBLE,
    fee DOUBLE,
    priority_fee LONG,  -- MEV protection fee
    transfer_fee DOUBLE,  -- Token-2022 fees
    tx_signature STRING,
    latency_ms INT,
    mev_protected BOOLEAN
) timestamp(timestamp) PARTITION BY DAY;

CREATE TABLE positions (
    timestamp TIMESTAMP,
    trader_id SYMBOL,
    token SYMBOL,
    amount DOUBLE,
    cost_basis DOUBLE,
    current_price DOUBLE,
    unrealized_pnl DOUBLE
) timestamp(timestamp) PARTITION BY HOUR;

CREATE TABLE metrics (
    timestamp TIMESTAMP,
    metric_name SYMBOL,
    value DOUBLE,
    labels STRING  -- JSON
) timestamp(timestamp) PARTITION BY DAY;

CREATE TABLE mev_events (
    timestamp TIMESTAMP,
    trader_id SYMBOL,
    sandwich_probability DOUBLE,
    estimated_loss_bps INT,
    avoided BOOLEAN,
    priority_fee LONG
) timestamp(timestamp) PARTITION BY DAY;
```

### PostgreSQL Tables

```sql
CREATE TABLE trader_config (
    id UUID PRIMARY KEY,
    name VARCHAR(255),
    mode VARCHAR(20),
    max_position_size DECIMAL,
    max_daily_loss DECIMAL,
    max_slippage_bps INTEGER,
    mev_protection_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

CREATE TABLE order_rules (
    id UUID PRIMARY KEY,
    trader_id UUID REFERENCES trader_config(id),
    position_token VARCHAR(50),
    rule_type VARCHAR(20),
    trigger_price DECIMAL,
    amount DECIMAL,
    created_at TIMESTAMP,
    triggered_at TIMESTAMP
);

CREATE TABLE tokens (
    address VARCHAR(50) PRIMARY KEY,
    symbol VARCHAR(20),
    name VARCHAR(255),
    decimals INTEGER,
    has_transfer_fee BOOLEAN DEFAULT false,
    transfer_fee_bps INTEGER,
    extensions JSONB,  -- Token-2022 extension data
    last_updated TIMESTAMP
);

-- Resilience tracking
CREATE TABLE node_health (
    timestamp TIMESTAMP PRIMARY KEY,
    latency_ms INTEGER,
    error_rate DECIMAL,
    circuit_breaker_status VARCHAR(20),
    last_failure TIMESTAMP
);
```

### Redis Schema

```yaml
# Price cache (TTL: 1-2 seconds)
price:{token_symbol}: float

# Pool state cache (TTL: 2 seconds)
pool:{pool_address}:
  liquidity: float
  reserve_a: float
  reserve_b: float
  volume_24h: float

# Event streams (MAXLEN: 10000)
stream:price-updates: JSON events
stream:trade-events: JSON events
stream:ui-events: JSON events

# Circuit breaker state
circuit:{service_name}:
  status: open|closed|half-open
  failures: int
  last_check: timestamp
```

## MVP Token Selection

For strategy validation, the MVP will focus on:
1. **SOL** - Base currency with high liquidity
2. **USDC** - Stable base for P&L calculation
3. **BONK** - High-volume memecoin for volatility testing
4. **JitoSOL** - Liquid staking token for different dynamics
5. **RAY** - Established DeFi token with stable liquidity

This set provides diverse liquidity profiles and slippage patterns for meaningful strategy validation.

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mev_risk_calculation() {
        let simulator = MevSimulator::new();
        let params = SwapParams {
            amount: 1_000_000_000,  // 1 SOL
            // ... other fields
        };

        let risk = simulator.analyze(&params).await.unwrap();
        assert!(risk.sandwich_probability < 0.5);
        assert!(risk.recommended_priority_fee > 1000);
    }

    #[tokio::test]
    async fn test_paper_live_accuracy() {
        let paper_trader = setup_paper_trader().await;
        let live_trader = setup_live_trader().await;

        // Execute identical trades
        let params = create_test_swap_params();
        let paper_result = paper_trader.execute_swap(params.clone()).await.unwrap();
        let live_result = live_trader.execute_swap(params).await.unwrap();

        // Check 85-90% accuracy threshold
        let price_variance = (paper_result.executed_price - live_result.executed_price).abs()
            / live_result.executed_price;
        assert!(price_variance < 0.15);  // Within 15%
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_failover_behavior() {
    let jupiter_client = JupiterClientWithFailover::new(
        "http://self-hosted:8080",
        "https://lite-api.jup.ag/v6"
    );

    // Simulate self-hosted failure
    mock_server_timeout("http://self-hosted:8080");

    let start = Instant::now();
    let quote = jupiter_client.get_quote(&test_params()).await.unwrap();

    // Should fallback within 250ms
    assert!(start.elapsed().as_millis() < 250);
    assert!(quote.price > 0.0);
}
```

### Performance Benchmarks
```rust
#[tokio::test]
async fn benchmark_with_redis_cache() {
    let trader = setup_paper_trader_with_redis().await;
    let iterations = 1000;

    let start = Instant::now();
    for _ in 0..iterations {
        trader.get_price("SOL").await.unwrap();
    }
    let duration = start.elapsed();

    let avg_latency = duration.as_micros() / iterations;
    println!("Average price fetch with Redis: {}μs", avg_latency);
    assert!(avg_latency < 1000);  // Sub-1ms target
}
```

## Resilience and Monitoring

### Circuit Breaker Implementation
```rust
pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: Arc<RwLock<CircuitState>>,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>
    {
        let state = self.state.read().await;
        match *state {
            CircuitState::Open => Err(anyhow!("Circuit breaker is open")),
            CircuitState::Closed => {
                match f.await {
                    Ok(result) => {
                        self.record_success().await;
                        Ok(result)
                    }
                    Err(e) => {
                        self.record_failure().await;
                        Err(e)
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Test with single request
                match f.await {
                    Ok(result) => {
                        self.close().await;
                        Ok(result)
                    }
                    Err(e) => {
                        self.open().await;
                        Err(e)
                    }
                }
            }
        }
    }
}
```

### Node Health Monitoring
```rust
pub struct NodeHealthMonitor {
    metrics_store: Arc<dyn MetricsStore>,
    alert_threshold_ms: u64,  // 200ms P99
}

impl NodeHealthMonitor {
    pub async fn check_health(&self) -> HealthStatus {
        let latency = self.measure_latency().await;
        let error_rate = self.calculate_error_rate().await;

        if latency > self.alert_threshold_ms {
            self.trigger_alert("High latency detected").await;
            return HealthStatus::Degraded;
        }

        if error_rate > 0.05 {  // 5% error rate
            self.trigger_alert("High error rate").await;
            return HealthStatus::Unhealthy;
        }

        HealthStatus::Healthy
    }
}
```
