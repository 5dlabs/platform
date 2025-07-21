# Task 21: Implement Live Trader Binary with Command-Line Interface

## Overview

This task implements the main executable binary for the live trading system, providing a comprehensive command-line interface that mirrors the paper trader's functionality while adding live trading-specific features. The binary serves as the primary entry point for executing real trades on the Solana blockchain, integrating with all the previously built components including the Enhanced Wallet Manager, Live Trade Executor, Risk Management System, and monitoring infrastructure.

## Dependencies

This task depends on:
- Task 13: Risk Management System with Position Tracking - Provides risk validation and position monitoring
- Task 16: Enhanced Wallet Manager with Security Features - Handles secure key management and transaction signing
- Task 17: Live Trade Executor with MEV Protection - Executes actual blockchain transactions
- Task 18: Monitoring and Logging Infrastructure - Provides observability and audit trails

## Architecture Context

According to the architecture.md, the live trader binary is a critical component that:
- Shares the same interface as the paper trader for consistency
- Integrates with secure wallet management using encrypted keypairs
- Implements circuit breaker patterns for resilience
- Provides MEV protection through dynamic priority fees
- Maintains comprehensive audit trails for regulatory compliance

## Implementation Details

### 1. CLI Interface Structure

```rust
use clap::{App, Arg, SubCommand, ArgMatches};
use serde::{Deserialize, Serialize};
use trading_models::config::{LiveTraderConfig, TradingMode};
use wallet_manager::EnhancedWalletManager;
use live_trade_executor::LiveTradeExecutor;
use risk_management::RiskManager;
use monitoring::MetricsCollector;
use circuit_breaker::CircuitBreaker;

#[derive(Debug)]
struct LiveTraderApp {
    config: LiveTraderConfig,
    wallet_manager: Arc<EnhancedWalletManager>,
    trade_executor: Arc<LiveTradeExecutor>,
    risk_manager: Arc<RiskManager>,
    circuit_breaker: Arc<CircuitBreaker>,
    metrics_collector: Arc<MetricsCollector>,
}

fn build_cli() -> App<'static, 'static> {
    App::new("Solana Trading Bot - Live Trader")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Trading Team")
        .about("Live trading bot for Solana with Jupiter integration and MEV protection")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file (YAML/TOML)")
                .takes_value(true)
                .default_value("config.yaml"),
        )
        .arg(
            Arg::with_name("wallet")
                .short("w")
                .long("wallet")
                .value_name("WALLET_PATH")
                .help("Path to encrypted wallet file")
                .takes_value(true)
                .required_unless("dry-run")
                .env("SOLANA_WALLET_PATH"),
        )
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .value_name("MODE")
                .help("Trading mode: live or paper")
                .takes_value(true)
                .possible_values(&["live", "paper"])
                .default_value("paper"),
        )
        .arg(
            Arg::with_name("dry-run")
                .long("dry-run")
                .help("Simulate operations without executing transactions")
                .conflicts_with("mode"),
        )
        .arg(
            Arg::with_name("emergency-stop")
                .long("emergency-stop")
                .help("Trigger emergency stop for all trading operations")
                .conflicts_with_all(&["buy", "sell", "swap"]),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Increase verbosity (-v, -vv, -vvv)"),
        )
        .subcommand(
            SubCommand::with_name("buy")
                .about("Buy a token with base currency")
                .arg(Arg::with_name("token")
                    .required(true)
                    .help("Token symbol or address to buy"))
                .arg(Arg::with_name("amount")
                    .required(true)
                    .help("Amount of base currency to spend"))
                .arg(Arg::with_name("slippage")
                    .long("slippage")
                    .takes_value(true)
                    .default_value("100")
                    .help("Maximum slippage in basis points"))
                .arg(Arg::with_name("priority-fee")
                    .long("priority-fee")
                    .takes_value(true)
                    .help("Priority fee in lamports (auto if not specified)")),
        )
        .subcommand(
            SubCommand::with_name("sell")
                .about("Sell a token for base currency")
                .arg(Arg::with_name("token")
                    .required(true)
                    .help("Token symbol or address to sell"))
                .arg(Arg::with_name("amount")
                    .required(true)
                    .help("Amount of tokens to sell"))
                .arg(Arg::with_name("slippage")
                    .long("slippage")
                    .takes_value(true)
                    .default_value("100")
                    .help("Maximum slippage in basis points")),
        )
        .subcommand(
            SubCommand::with_name("swap")
                .about("Swap between two tokens")
                .arg(Arg::with_name("from")
                    .required(true)
                    .help("Source token symbol or address"))
                .arg(Arg::with_name("to")
                    .required(true)
                    .help("Destination token symbol or address"))
                .arg(Arg::with_name("amount")
                    .required(true)
                    .help("Amount of source token to swap")),
        )
        .subcommand(
            SubCommand::with_name("monitor")
                .about("Monitor positions and system health")
                .arg(Arg::with_name("interval")
                    .long("interval")
                    .takes_value(true)
                    .default_value("1000")
                    .help("Update interval in milliseconds")),
        )
        .subcommand(
            SubCommand::with_name("positions")
                .about("Show current positions and P&L"),
        )
        .subcommand(
            SubCommand::with_name("health")
                .about("Check system health and circuit breaker status"),
        )
}
```

### 2. Configuration Management

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LiveTraderConfig {
    pub mode: TradingMode,
    pub wallet: WalletConfig,
    pub risk_parameters: RiskParameters,
    pub mev_protection: MevProtectionConfig,
    pub jupiter: JupiterConfig,
    pub monitoring: MonitoringConfig,
    pub circuit_breaker: CircuitBreakerConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WalletConfig {
    pub encryption_key_env: String,
    pub rotation_schedule: RotationSchedule,
    pub backup_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MevProtectionConfig {
    pub min_priority_fee: u64,
    pub max_priority_fee: u64,
    pub dynamic_fee_enabled: bool,
    pub wrap_sol_enabled: bool,
}

impl LiveTraderConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .context("Failed to read configuration file")?;
        
        let config: Self = if path.as_ref().extension() == Some("yaml".as_ref()) {
            serde_yaml::from_str(&content)?
        } else if path.as_ref().extension() == Some("toml".as_ref()) {
            toml::from_str(&content)?
        } else {
            return Err(anyhow!("Unsupported configuration format"));
        };
        
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<()> {
        // Validate risk parameters
        if self.risk_parameters.max_position_size <= 0.0 {
            return Err(anyhow!("Invalid max position size"));
        }
        
        if self.risk_parameters.daily_loss_limit <= 0.0 {
            return Err(anyhow!("Invalid daily loss limit"));
        }
        
        // Validate MEV protection
        if self.mev_protection.min_priority_fee > self.mev_protection.max_priority_fee {
            return Err(anyhow!("Invalid priority fee range"));
        }
        
        // Validate circuit breaker thresholds
        if self.circuit_breaker.latency_threshold_ms < 100 {
            return Err(anyhow!("Latency threshold too low (min: 100ms)"));
        }
        
        Ok(())
    }
    
    pub fn merge_with_env(&mut self) -> Result<()> {
        // Override with environment variables
        if let Ok(mode) = env::var("TRADING_MODE") {
            self.mode = TradingMode::from_str(&mode)?;
        }
        
        if let Ok(max_pos) = env::var("MAX_POSITION_SIZE") {
            self.risk_parameters.max_position_size = max_pos.parse()?;
        }
        
        // Additional environment overrides...
        Ok(())
    }
}
```

### 3. Main Application Logic

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Parse CLI arguments
    let matches = build_cli().get_matches();
    
    // Setup signal handlers for graceful shutdown
    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
    setup_signal_handlers(shutdown_tx.clone())?;
    
    // Load and validate configuration
    let mut config = LiveTraderConfig::from_file(
        matches.value_of("config").unwrap_or("config.yaml")
    )?;
    config.merge_with_env()?;
    
    // Initialize application
    let app = LiveTraderApp::new(config, &matches).await?;
    
    // Handle emergency stop
    if matches.is_present("emergency-stop") {
        return app.emergency_stop().await;
    }
    
    // Process subcommands
    match matches.subcommand() {
        ("buy", Some(sub_matches)) => {
            app.execute_buy(sub_matches, shutdown_rx).await?;
        }
        ("sell", Some(sub_matches)) => {
            app.execute_sell(sub_matches, shutdown_rx).await?;
        }
        ("swap", Some(sub_matches)) => {
            app.execute_swap(sub_matches, shutdown_rx).await?;
        }
        ("monitor", Some(sub_matches)) => {
            app.run_monitor(sub_matches, shutdown_rx).await?;
        }
        ("positions", _) => {
            app.show_positions().await?;
        }
        ("health", _) => {
            app.check_health().await?;
        }
        _ => {
            // No subcommand, show help
            build_cli().print_help()?;
        }
    }
    
    Ok(())
}

impl LiveTraderApp {
    async fn new(config: LiveTraderConfig, matches: &ArgMatches<'_>) -> Result<Self> {
        // Verify trading mode
        let mode = if matches.is_present("dry-run") {
            TradingMode::DryRun
        } else {
            TradingMode::from_str(matches.value_of("mode").unwrap_or("paper"))?
        };
        
        // Show warning for live mode
        if mode == TradingMode::Live {
            println!("⚠️  WARNING: Running in LIVE TRADING mode!");
            println!("Real funds will be used. Press Ctrl+C within 5 seconds to cancel...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        
        // Initialize wallet manager
        let wallet_path = matches.value_of("wallet")
            .ok_or_else(|| anyhow!("Wallet path required for live trading"))?;
        
        let encryption_key = env::var(&config.wallet.encryption_key_env)
            .context("Failed to load wallet encryption key from environment")?;
        
        let wallet_manager = Arc::new(
            EnhancedWalletManager::new(
                wallet_path,
                encryption_key.as_bytes(),
                config.wallet.rotation_schedule.clone(),
            ).await?
        );
        
        // Initialize risk manager
        let risk_manager = Arc::new(
            RiskManager::new(config.risk_parameters.clone()).await?
        );
        
        // Initialize circuit breaker
        let circuit_breaker = Arc::new(
            CircuitBreaker::new(config.circuit_breaker.clone())
        );
        
        // Initialize metrics collector
        let metrics_collector = Arc::new(
            MetricsCollector::new(&config.monitoring).await?
        );
        
        // Initialize trade executor
        let trade_executor = Arc::new(
            LiveTradeExecutor::new(
                wallet_manager.clone(),
                risk_manager.clone(),
                circuit_breaker.clone(),
                metrics_collector.clone(),
                config.clone(),
            ).await?
        );
        
        Ok(Self {
            config,
            wallet_manager,
            trade_executor,
            risk_manager,
            circuit_breaker,
            metrics_collector,
        })
    }
}
```

### 4. Trade Execution Commands

```rust
impl LiveTraderApp {
    async fn execute_buy(
        &self,
        matches: &ArgMatches<'_>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<()> {
        let token = matches.value_of("token").unwrap();
        let amount: f64 = matches.value_of("amount").unwrap().parse()?;
        let slippage_bps: u16 = matches.value_of("slippage").unwrap().parse()?;
        
        // Parse optional priority fee
        let priority_fee = matches.value_of("priority-fee")
            .map(|v| v.parse::<u64>())
            .transpose()?;
        
        // Pre-trade validation
        println!("🔍 Validating trade parameters...");
        
        // Check circuit breaker
        if !self.circuit_breaker.is_healthy().await {
            return Err(anyhow!("❌ Circuit breaker is open - trading paused"));
        }
        
        // Risk validation
        self.risk_manager.validate_trade(
            token,
            amount,
            TradeDirection::Buy,
        ).await?;
        
        // Get quote
        println!("📊 Getting quote from Jupiter...");
        let quote = self.trade_executor.get_quote(
            &self.config.risk_parameters.base_currency,
            token,
            amount,
            slippage_bps,
        ).await?;
        
        // Display trade details
        println!("\n📋 Trade Details:");
        println!("  Token: {}", token);
        println!("  Amount: {} {}", amount, self.config.risk_parameters.base_currency);
        println!("  Expected Price: {:.6}", quote.price);
        println!("  Slippage: {}%", slippage_bps as f64 / 100.0);
        println!("  MEV Protection: {}", if priority_fee.is_some() { "Custom" } else { "Dynamic" });
        println!("  Estimated Output: {:.6} {}", quote.out_amount, token);
        
        // Confirmation prompt for live mode
        if self.config.mode == TradingMode::Live {
            print!("\n⚠️  Confirm LIVE trade? [y/N]: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("❌ Trade cancelled");
                return Ok(());
            }
        }
        
        // Execute trade with progress updates
        println!("\n🚀 Executing trade...");
        
        let trade_future = self.trade_executor.execute_buy(
            token,
            amount,
            slippage_bps,
            priority_fee,
        );
        
        // Execute with shutdown capability
        let result = tokio::select! {
            result = trade_future => result?,
            _ = shutdown_rx.recv() => {
                println!("\n⚠️  Shutdown signal received, cancelling trade...");
                return Ok(());
            }
        };
        
        // Display results
        println!("\n✅ Trade executed successfully!");
        println!("  Transaction: {}", result.signature);
        println!("  Executed Price: {:.6}", result.executed_price);
        println!("  Received: {:.6} {}", result.amount_received, token);
        println!("  Slippage: {:.2}%", result.slippage_actual * 100.0);
        println!("  MEV Protected: {}", result.mev_protected);
        println!("  Latency: {}ms", result.latency_ms);
        
        // Update position tracking
        self.risk_manager.update_position(
            token,
            result.amount_received,
            result.executed_price,
        ).await?;
        
        Ok(())
    }
    
    async fn execute_sell(
        &self,
        matches: &ArgMatches<'_>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<()> {
        // Similar implementation for sell command
        // ...
    }
    
    async fn execute_swap(
        &self,
        matches: &ArgMatches<'_>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<()> {
        // Implementation for generic token swap
        // ...
    }
}
```

### 5. Monitoring and Health Commands

```rust
impl LiveTraderApp {
    async fn run_monitor(
        &self,
        matches: &ArgMatches<'_>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<()> {
        let interval_ms: u64 = matches.value_of("interval").unwrap().parse()?;
        let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));
        
        println!("📊 Starting position monitor (Ctrl+C to stop)...\n");
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Clear screen for update
                    print!("\x1B[2J\x1B[1;1H");
                    
                    // Display header
                    println!("🏦 Solana Trading Bot - Live Monitor");
                    println!("⏰ {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
                    println!("{}", "─".repeat(60));
                    
                    // System health
                    let health = self.circuit_breaker.get_status().await;
                    println!("\n🔧 System Health:");
                    println!("  Circuit Breaker: {}", health.state);
                    println!("  Node Latency: {}ms", health.current_latency_ms);
                    println!("  Error Rate: {:.2}%", health.error_rate * 100.0);
                    
                    // Positions
                    let positions = self.risk_manager.get_all_positions().await?;
                    println!("\n💼 Current Positions:");
                    
                    if positions.is_empty() {
                        println!("  No open positions");
                    } else {
                        for position in positions {
                            let pnl_color = if position.unrealized_pnl >= 0.0 { "🟢" } else { "🔴" };
                            println!("  {} {}: {:.6} @ {:.6} | P&L: {} {:.2} ({:.2}%)",
                                pnl_color,
                                position.token,
                                position.amount,
                                position.cost_basis,
                                pnl_color,
                                position.unrealized_pnl,
                                position.unrealized_pnl_percent * 100.0
                            );
                        }
                    }
                    
                    // Risk metrics
                    let risk_status = self.risk_manager.get_risk_status().await?;
                    println!("\n📈 Risk Metrics:");
                    println!("  Daily P&L: ${:.2}", risk_status.daily_pnl);
                    println!("  Daily Limit: ${:.2}", self.config.risk_parameters.daily_loss_limit);
                    println!("  Total Exposure: ${:.2}", risk_status.total_exposure);
                    println!("  Available Capital: ${:.2}", risk_status.available_capital);
                }
                _ = shutdown_rx.recv() => {
                    println!("\n👋 Monitor stopped");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn show_positions(&self) -> Result<()> {
        let positions = self.risk_manager.get_all_positions().await?;
        
        if positions.is_empty() {
            println!("No open positions");
            return Ok(());
        }
        
        // Display positions in table format
        println!("{:<10} {:>12} {:>12} {:>12} {:>12} {:>8}",
            "Token", "Amount", "Cost Basis", "Current", "P&L", "P&L %");
        println!("{}", "─".repeat(70));
        
        for position in positions {
            println!("{:<10} {:>12.6} {:>12.6} {:>12.6} {:>12.2} {:>7.2}%",
                position.token,
                position.amount,
                position.cost_basis,
                position.current_price,
                position.unrealized_pnl,
                position.unrealized_pnl_percent * 100.0
            );
        }
        
        Ok(())
    }
    
    async fn check_health(&self) -> Result<()> {
        println!("🏥 System Health Check\n");
        
        // Circuit breaker status
        let cb_status = self.circuit_breaker.get_status().await;
        println!("Circuit Breaker:");
        println!("  State: {}", cb_status.state);
        println!("  Failures: {}/{}", cb_status.failure_count, cb_status.failure_threshold);
        println!("  Last Failure: {}", cb_status.last_failure.map_or("None".to_string(), |t| t.to_string()));
        
        // Node health
        println!("\nNode Health:");
        println!("  Latency: {}ms (threshold: {}ms)", 
            cb_status.current_latency_ms,
            self.config.circuit_breaker.latency_threshold_ms
        );
        println!("  Error Rate: {:.2}% (threshold: {:.2}%)",
            cb_status.error_rate * 100.0,
            self.config.circuit_breaker.error_rate_threshold * 100.0
        );
        
        // Wallet status
        println!("\nWallet Status:");
        let wallet_info = self.wallet_manager.get_status().await?;
        println!("  Address: {}", wallet_info.address);
        println!("  Balance: {} SOL", wallet_info.sol_balance);
        println!("  Last Rotation: {}", wallet_info.last_rotation);
        
        // Risk limits
        println!("\nRisk Limits:");
        let risk_status = self.risk_manager.get_risk_status().await?;
        println!("  Daily P&L: ${:.2} / ${:.2}",
            risk_status.daily_pnl,
            self.config.risk_parameters.daily_loss_limit
        );
        println!("  Positions: {} / {}",
            risk_status.open_positions,
            self.config.risk_parameters.max_positions
        );
        
        Ok(())
    }
}
```

### 6. Safety Features Implementation

```rust
impl LiveTraderApp {
    async fn emergency_stop(&self) -> Result<()> {
        println!("🚨 EMERGENCY STOP INITIATED");
        
        // Pause all trading immediately
        self.circuit_breaker.trigger_emergency_stop().await;
        println!("✅ Circuit breaker opened - all trading paused");
        
        // Cancel any pending orders
        let cancelled = self.trade_executor.cancel_all_pending().await?;
        println!("✅ Cancelled {} pending orders", cancelled);
        
        // Report current positions
        println!("\n📊 Current Positions:");
        let positions = self.risk_manager.get_all_positions().await?;
        
        for position in positions {
            println!("  {}: {:.6} @ {:.6} (P&L: {:.2})",
                position.token,
                position.amount,
                position.cost_basis,
                position.unrealized_pnl
            );
        }
        
        // Save state to file
        let emergency_state = EmergencyState {
            timestamp: Utc::now(),
            positions,
            reason: "Manual emergency stop triggered".to_string(),
        };
        
        let state_file = format!("emergency_stop_{}.json", Utc::now().timestamp());
        fs::write(&state_file, serde_json::to_string_pretty(&emergency_state)?)?;
        println!("\n✅ State saved to: {}", state_file);
        
        println!("\n⚠️  Trading halted. Review positions and restart when ready.");
        
        Ok(())
    }
}

fn setup_signal_handlers(shutdown_tx: broadcast::Sender<()>) -> Result<()> {
    // Handle Ctrl+C
    ctrlc::set_handler(move || {
        println!("\n📛 Shutdown signal received...");
        let _ = shutdown_tx.send(());
    })?;
    
    Ok(())
}

// Audit logging with sensitive data redaction
impl LiveTraderApp {
    fn log_trade_audit(&self, action: &str, details: &serde_json::Value) {
        let mut redacted = details.clone();
        
        // Redact sensitive fields
        if let Some(obj) = redacted.as_object_mut() {
            if obj.contains_key("wallet_key") {
                obj.insert("wallet_key".to_string(), json!("[REDACTED]"));
            }
            if obj.contains_key("api_key") {
                obj.insert("api_key".to_string(), json!("[REDACTED]"));
            }
        }
        
        info!("AUDIT: {} - {}", action, redacted);
    }
}
```

## Example Configuration File

```yaml
# config.yaml - Live Trader Configuration
mode: paper  # Options: paper, live
wallet:
  encryption_key_env: WALLET_ENCRYPTION_KEY
  rotation_schedule: daily
  backup_path: ./backups/wallet

risk_parameters:
  base_currency: SOL
  max_position_size: 100.0
  max_positions: 10
  daily_loss_limit: 500.0
  max_slippage_bps: 200

mev_protection:
  min_priority_fee: 1000
  max_priority_fee: 10000
  dynamic_fee_enabled: true
  wrap_sol_enabled: true

jupiter:
  self_hosted_url: "http://localhost:8080"
  public_url: "https://lite-api.jup.ag/v6"
  timeout_ms: 200
  use_failover: true

monitoring:
  metrics_endpoint: "http://localhost:9090"
  log_level: info
  enable_audit_log: true

circuit_breaker:
  latency_threshold_ms: 200
  error_rate_threshold: 0.05
  failure_threshold: 5
  recovery_timeout_secs: 60
  half_open_max_requests: 3
```

## Integration Points

### 1. Wallet Manager Integration
- Secure loading of encrypted wallet keys
- Transaction signing with automatic nonce management
- Key rotation scheduling

### 2. Risk Management Integration
- Pre-trade validation against position limits
- Real-time P&L tracking
- Daily loss limit enforcement

### 3. Trade Executor Integration
- Quote fetching with Jupiter failover
- MEV-protected transaction submission
- Confirmation monitoring

### 4. Monitoring Integration
- Metrics collection for all operations
- Audit trail generation
- Performance tracking

## Security Considerations

1. **Wallet Security**:
   - Encrypted wallet storage
   - Environment variable for decryption keys
   - No plaintext keys in configuration

2. **Audit Trail**:
   - All trades logged with timestamps
   - Sensitive data automatically redacted
   - Immutable audit log storage

3. **Access Control**:
   - Confirmation prompts for live mode
   - Emergency stop capability
   - Dry-run mode for testing

## Error Handling

The CLI implements comprehensive error handling:
- Clear error messages with context
- Suggestions for resolution
- Graceful degradation when possible
- Emergency stop on critical failures

## Performance Considerations

- Async command execution for responsiveness
- Connection pooling for database access
- Efficient position tracking updates
- Minimal overhead in critical path