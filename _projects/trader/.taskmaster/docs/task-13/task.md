# Task 13: Implement Paper Trader Binary with CLI Interface

## Overview
This task implements the main paper trader binary with a comprehensive command-line interface for configuration and operation. The binary serves as the entry point for the paper trading system, orchestrating all components including database connections, monitoring services, and the TUI.

## Architecture Context
According to the architecture.md, the paper trader binary:
- Initializes all system components in the correct order
- Manages configuration from files and CLI arguments
- Launches background services (monitoring, price feeds, batch writers)
- Starts the TUI for user interaction
- Handles graceful shutdown of all services

## Implementation Requirements

### 1. CLI Interface with Clap

Define comprehensive command-line arguments:

```rust
use clap::{App, Arg, ArgMatches};
use serde::{Deserialize, Serialize};

fn build_cli() -> App<'static, 'static> {
    App::new("Solana Paper Trader")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Trading Platform Team")
        .about("Paper trading for Solana with MEV simulation and real-time monitoring")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Path to configuration file (YAML/TOML)")
            .takes_value(true)
            .default_value("config.yaml"))
        .arg(Arg::with_name("sol")
            .long("initial-sol")
            .value_name("AMOUNT")
            .help("Initial SOL allocation for paper trading")
            .takes_value(true)
            .validator(validate_positive_decimal))
        .arg(Arg::with_name("usdc")
            .long("initial-usdc")
            .value_name("AMOUNT")
            .help("Initial USDC allocation for paper trading")
            .takes_value(true)
            .validator(validate_positive_decimal))
        .arg(Arg::with_name("slippage")
            .long("slippage")
            .value_name("PERCENT")
            .help("Default slippage tolerance (0.5-2.0%)")
            .takes_value(true)
            .validator(validate_slippage_range))
        .arg(Arg::with_name("mev-protection")
            .long("mev-protection")
            .help("Enable MEV protection (default: true)")
            .takes_value(true)
            .possible_values(&["true", "false", "auto"])
            .default_value("true"))
        .arg(Arg::with_name("tokens")
            .long("tokens")
            .value_name("LIST")
            .help("Comma-separated list of tokens to monitor")
            .takes_value(true)
            .default_value("SOL,USDC,BONK,JitoSOL,RAY"))
        .arg(Arg::with_name("log-level")
            .short("l")
            .long("log-level")
            .value_name("LEVEL")
            .help("Logging verbosity")
            .takes_value(true)
            .possible_values(&["error", "warn", "info", "debug", "trace"])
            .default_value("info"))
        .arg(Arg::with_name("no-tui")
            .long("no-tui")
            .help("Run in headless mode without TUI"))
        .arg(Arg::with_name("metrics-port")
            .long("metrics-port")
            .value_name("PORT")
            .help("Port for Prometheus metrics endpoint")
            .takes_value(true)
            .default_value("9090"))
}

fn validate_positive_decimal(v: String) -> Result<(), String> {
    match v.parse::<Decimal>() {
        Ok(d) if d > Decimal::zero() => Ok(()),
        Ok(_) => Err("Value must be positive".to_string()),
        Err(_) => Err("Invalid decimal number".to_string()),
    }
}

fn validate_slippage_range(v: String) -> Result<(), String> {
    match v.parse::<f64>() {
        Ok(s) if s >= 0.5 && s <= 2.0 => Ok(()),
        Ok(_) => Err("Slippage must be between 0.5% and 2.0%".to_string()),
        Err(_) => Err("Invalid percentage".to_string()),
    }
}
```

### 2. Configuration System

Implement layered configuration with proper precedence:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub trading: TradingConfig,
    pub portfolio: PortfolioConfig,
    pub mev: MevConfig,
    pub database: DatabaseConfig,
    pub jupiter: JupiterConfig,
    pub monitoring: MonitoringConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradingConfig {
    pub slippage_tolerance: Decimal,
    pub monitored_tokens: Vec<String>,
    pub order_monitoring_interval_ms: u64,  // 100ms default
    pub max_position_size: Option<Decimal>,
    pub max_daily_loss: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioConfig {
    pub initial_allocation: HashMap<String, Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MevConfig {
    pub protection_enabled: bool,
    pub default_priority_fee: u64,
    pub sandwich_simulation: bool,
    pub max_priority_fee: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub questdb_url: String,
    pub postgres_url: String,
    pub redis_url: String,
    pub connection_pool_size: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JupiterConfig {
    pub self_hosted_url: String,
    pub public_url: String,
    pub timeout_ms: u64,
    pub use_self_hosted: bool,
}

impl Config {
    pub fn load(config_path: &str) -> Result<Self> {
        // Load from file
        let config_str = std::fs::read_to_string(config_path)?;
        let mut config: Config = match config_path.ends_with(".yaml") || config_path.ends_with(".yml") {
            true => serde_yaml::from_str(&config_str)?,
            false => toml::from_str(&config_str)?,
        };
        
        // Apply environment variable overrides
        config.apply_env_overrides()?;
        
        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }
    
    pub fn apply_cli_overrides(&mut self, matches: &ArgMatches) -> Result<()> {
        // Override initial SOL allocation
        if let Some(sol) = matches.value_of("sol") {
            let amount = sol.parse::<Decimal>()?;
            self.portfolio.initial_allocation.insert("SOL".to_string(), amount);
        }
        
        // Override initial USDC allocation
        if let Some(usdc) = matches.value_of("usdc") {
            let amount = usdc.parse::<Decimal>()?;
            self.portfolio.initial_allocation.insert("USDC".to_string(), amount);
        }
        
        // Override slippage tolerance
        if let Some(slippage) = matches.value_of("slippage") {
            self.trading.slippage_tolerance = Decimal::from_str(slippage)?;
        }
        
        // Override MEV protection
        if let Some(mev) = matches.value_of("mev-protection") {
            self.mev.protection_enabled = mev.parse()?;
        }
        
        // Override monitored tokens
        if let Some(tokens) = matches.value_of("tokens") {
            self.trading.monitored_tokens = tokens.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
        
        Ok(())
    }
    
    fn validate(&self) -> Result<()> {
        // Validate slippage range
        if self.trading.slippage_tolerance < Decimal::from_str("0.5")? ||
           self.trading.slippage_tolerance > Decimal::from_str("2.0")? {
            return Err(anyhow!("Slippage tolerance must be between 0.5% and 2.0%"));
        }
        
        // Validate initial allocation
        if self.portfolio.initial_allocation.is_empty() {
            return Err(anyhow!("Initial portfolio allocation cannot be empty"));
        }
        
        // Validate database URLs
        if self.database.questdb_url.is_empty() {
            return Err(anyhow!("QuestDB URL is required"));
        }
        
        // Validate monitoring interval
        if self.trading.order_monitoring_interval_ms < 50 ||
           self.trading.order_monitoring_interval_ms > 1000 {
            return Err(anyhow!("Order monitoring interval must be between 50ms and 1000ms"));
        }
        
        Ok(())
    }
}
```

### 3. Main Application Flow

Implement the main entry point with proper initialization:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let matches = build_cli().get_matches();
    
    // Initialize logging
    init_logging(matches.value_of("log-level").unwrap())?;
    
    // Load and merge configuration
    let config_path = matches.value_of("config").unwrap();
    let mut config = Config::load(config_path)?;
    config.apply_cli_overrides(&matches)?;
    
    info!("Starting Solana Paper Trader v{}", env!("CARGO_PKG_VERSION"));
    info!("Configuration loaded from: {}", config_path);
    
    // Initialize shutdown handler
    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
    let shutdown_rx_clone = shutdown_tx.subscribe();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        info!("Shutdown signal received");
        let _ = shutdown_tx.send(());
    });
    
    // Initialize all components
    let components = initialize_components(&config).await?;
    
    // Start background services
    let service_handles = start_background_services(&components, &config).await?;
    
    // Start metrics server if not in headless mode
    if !matches.is_present("no-tui") {
        let metrics_port: u16 = matches.value_of("metrics-port").unwrap().parse()?;
        start_metrics_server(metrics_port, &components).await?;
    }
    
    // Run main application
    if matches.is_present("no-tui") {
        run_headless(components, shutdown_rx_clone).await?;
    } else {
        run_with_tui(components, shutdown_rx_clone).await?;
    }
    
    // Graceful shutdown
    info!("Shutting down services...");
    shutdown_services(service_handles).await?;
    
    info!("Paper trader stopped successfully");
    Ok(())
}

struct Components {
    questdb: Arc<QuestDbClient>,
    postgres: Arc<PostgresClient>,
    redis_pool: Pool<RedisConnectionManager>,
    price_cache: Arc<PriceCache>,
    jupiter_client: Arc<JupiterClientWithFailover>,
    portfolio: Arc<RwLock<VirtualPortfolio>>,
    trade_executor: Arc<PaperTradeExecutor>,
    order_monitor: Arc<OrderMonitor>,
    mev_simulator: Arc<MevSimulator>,
    system_health: Arc<SystemHealth>,
}

async fn initialize_components(config: &Config) -> Result<Components> {
    info!("Initializing database connections...");
    
    // Initialize QuestDB with retry
    let questdb = retry_with_backoff(|| {
        QuestDbClient::new(QuestDbConfig {
            connection_string: config.database.questdb_url.clone(),
            max_pool_size: config.database.connection_pool_size,
            batch_interval_ms: 100,
            max_batch_size: 1000,
            retention_days: 30,
        })
    }, 5, Duration::from_secs(2)).await?;
    
    // Initialize PostgreSQL
    let postgres = PostgresClient::new(&config.database.postgres_url).await?;
    
    // Initialize Redis
    let redis_pool = create_redis_pool(&config.database.redis_url).await?;
    
    info!("Initializing trading components...");
    
    // Create price cache
    let price_cache = Arc::new(PriceCache::new(
        redis_pool.clone(),
        Duration::from_secs(1),
    ));
    
    // Create Jupiter client with failover
    let jupiter_client = Arc::new(JupiterClientWithFailover::new(
        &config.jupiter.self_hosted_url,
        &config.jupiter.public_url,
        Duration::from_millis(config.jupiter.timeout_ms),
    ).await?);
    
    // Create virtual portfolio
    let portfolio = Arc::new(RwLock::new(
        VirtualPortfolio::new(config.portfolio.initial_allocation.clone())
    ));
    
    // Create MEV simulator
    let mev_simulator = Arc::new(MevSimulator::new(
        redis_pool.clone(),
        config.mev.sandwich_simulation,
    ));
    
    // Create trade executor
    let trade_executor = Arc::new(PaperTradeExecutor::new(
        portfolio.clone(),
        price_cache.clone(),
        mev_simulator.clone(),
        jupiter_client.clone(),
        Arc::new(questdb.clone()),
        SlippageConfig::Fixed(config.trading.slippage_tolerance),
    ));
    
    // Create order monitor
    let order_monitor = Arc::new(OrderMonitor::new(
        portfolio.clone(),
        price_cache.clone(),
        trade_executor.clone(),
    ));
    
    // Create system health monitor
    let system_health = Arc::new(SystemHealth::new(
        redis_pool.clone(),
        config.monitoring.latency_threshold_ms,
    ));
    
    Ok(Components {
        questdb: Arc::new(questdb),
        postgres: Arc::new(postgres),
        redis_pool,
        price_cache,
        jupiter_client,
        portfolio,
        trade_executor,
        order_monitor,
        mev_simulator,
        system_health,
    })
}

async fn start_background_services(
    components: &Components,
    config: &Config,
) -> Result<Vec<JoinHandle<()>>> {
    let mut handles = Vec::new();
    
    info!("Starting background services...");
    
    // Start price updater
    handles.push(
        components.price_cache
            .start_price_updater(config.trading.monitored_tokens.clone())
            .await
    );
    
    // Start order monitoring
    handles.push(
        components.order_monitor
            .start_monitoring()
            .await
    );
    
    // Start QuestDB batch writer
    handles.push(
        components.questdb
            .start_batch_writer()
            .await
    );
    
    // Start retention manager
    handles.push(
        components.questdb
            .start_retention_manager()
            .await
    );
    
    // Start health monitoring
    handles.push(
        components.system_health
            .start_monitoring()
            .await
    );
    
    info!("All background services started successfully");
    
    Ok(handles)
}
```

### 4. TUI Integration

Launch the TUI with proper event handling:

```rust
async fn run_with_tui(
    components: Components,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    // Create event stream for TUI
    let event_stream = Arc::new(EventStream::new(
        components.redis_pool.clone(),
        "ui-events",
        10000,
    ));
    
    // Create TUI instance
    let mut tui = TradingTui::new(
        components.portfolio.clone(),
        event_stream.clone(),
        components.trade_executor.clone(),
        components.order_monitor.clone(),
        components.system_health.clone(),
    )?;
    
    // Run TUI with shutdown handling
    tokio::select! {
        result = tui.run() => {
            result?;
        }
        _ = shutdown_rx.recv() => {
            info!("Shutdown signal received in TUI");
        }
    }
    
    Ok(())
}
```

### 5. Graceful Shutdown

Implement proper cleanup on exit:

```rust
async fn shutdown_services(handles: Vec<JoinHandle<()>>) -> Result<()> {
    info!("Stopping background services...");
    
    // Cancel all background tasks
    for handle in handles {
        handle.abort();
    }
    
    // Give services time to cleanup
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    info!("All services stopped");
    Ok(())
}

fn init_logging(level: &str) -> Result<()> {
    let log_level = match level {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };
    
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .format_timestamp_millis()
        .init();
    
    Ok(())
}
```

## Error Handling

Comprehensive error handling throughout:

```rust
async fn retry_with_backoff<F, Fut, T>(
    mut f: F,
    max_attempts: u32,
    initial_delay: Duration,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut delay = initial_delay;
    
    for attempt in 1..=max_attempts {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_attempts => {
                warn!("Attempt {} failed: {}. Retrying in {:?}...", attempt, e, delay);
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
    
    unreachable!()
}
```

## Testing Strategy

1. **Unit Tests**: Test configuration loading and validation
2. **Integration Tests**: Verify component initialization
3. **CLI Tests**: Test all command-line argument combinations
4. **Shutdown Tests**: Verify graceful shutdown behavior
5. **Error Tests**: Test failure scenarios and recovery

## Dependencies
- Task 9: Paper Trade Executor
- Task 10: Terminal User Interface
- Task 11: Order Monitoring System
- Task 12: QuestDB Integration