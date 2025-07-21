# Task 9: Implement Paper Trade Executor with Configurable Models

## Overview

This task implements the core paper trade execution system that processes external trade requests, applies configurable slippage models, integrates MEV simulation, and updates the virtual portfolio. The executor serves as the primary interface for strategy testing, ensuring execution behavior matches live trading while maintaining the flexibility to adjust simulation parameters.

## Architecture Context

According to `architecture.md`, the Paper Trade Executor is a critical component that:
- Implements the `TradeExecutor` trait for compatibility with live trading
- Processes trades with <100ms latency (excluding simulation)
- Integrates with Jupiter for price discovery via failover client
- Uses Redis for sub-millisecond price caching
- Records all trades in QuestDB with 100ms batch writes
- Maintains identical interfaces to live trading for seamless switching

## Dependencies

- **Task 4**: Jupiter Integration with Failover (for price discovery)
- **Task 7**: Virtual Portfolio Manager (for balance updates)
- **Task 8**: MEV Risk Simulation (for realistic price impact)

## Implementation Details

### 1. Core Trade Executor Structure

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use tracing::{info, debug, warn, error};

pub struct PaperTradeExecutor {
    portfolio: Arc<RwLock<VirtualPortfolio>>,
    price_cache: Arc<PriceCache>,
    mev_simulator: Arc<Mutex<MevSimulator>>,
    jupiter_client: Arc<JupiterClientWithFailover>,
    quest_db: Arc<QuestDbClient>,
    slippage_config: SlippageConfig,
    execution_config: ExecutionConfig,
    metrics_collector: Arc<MetricsCollector>,
}

#[derive(Clone, Debug)]
pub enum SlippageConfig {
    Fixed {
        percentage: Decimal,  // 0.005 to 0.02 (0.5% to 2%)
    },
    Dynamic {
        base_percentage: Decimal,
        size_impact_factor: Decimal,
        volatility_multiplier: Decimal,
    },
}

#[derive(Clone, Debug)]
pub struct ExecutionConfig {
    pub enable_mev_simulation: bool,
    pub default_priority_fee: u64,
    pub max_execution_time_ms: u64,
    pub price_cache_ttl_ms: u64,
}

// External trade request structure
#[derive(Clone, Debug, Deserialize)]
pub struct TradeRequest {
    pub request_id: Uuid,
    pub action: TradeAction,
    pub base_token: String,
    pub quote_token: String,
    pub amount: Decimal,
    pub is_base_input: bool,  // true if amount is in base token
    pub slippage_tolerance_bps: Option<u16>,  // Override default slippage
    pub priority_fee: u64,
    pub simulate_mev: bool,
    pub max_price_impact_bps: Option<u16>,
    pub metadata: Option<serde_json::Value>,
}

// Comprehensive trade result
#[derive(Clone, Debug, Serialize)]
pub struct TradeResult {
    pub request_id: Uuid,
    pub trade: Trade,
    pub portfolio_state: PortfolioSummary,
    pub execution_metrics: ExecutionMetrics,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExecutionMetrics {
    pub expected_price: Decimal,
    pub execution_price: Decimal,
    pub price_impact_bps: i64,
    pub slippage_bps: i64,
    pub execution_time_ms: u64,
    pub mev_simulation_applied: bool,
    pub cache_hit: bool,
}
```

### 2. Trade Execution Implementation

```rust
impl PaperTradeExecutor {
    pub fn new(
        portfolio: Arc<RwLock<VirtualPortfolio>>,
        price_cache: Arc<PriceCache>,
        mev_simulator: Arc<Mutex<MevSimulator>>,
        jupiter_client: Arc<JupiterClientWithFailover>,
        quest_db: Arc<QuestDbClient>,
        slippage_config: SlippageConfig,
    ) -> Self {
        let execution_config = ExecutionConfig {
            enable_mev_simulation: true,
            default_priority_fee: 1000,
            max_execution_time_ms: 100,
            price_cache_ttl_ms: 2000,  // 2 second cache TTL
        };
        
        Self {
            portfolio,
            price_cache,
            mev_simulator,
            jupiter_client,
            quest_db,
            slippage_config,
            execution_config,
            metrics_collector: Arc::new(MetricsCollector::new()),
        }
    }
    
    pub async fn execute_trade_request(&self, request: TradeRequest) -> Result<TradeResult> {
        let start_time = Instant::now();
        let mut warnings = Vec::new();
        
        info!(
            request_id = %request.request_id,
            action = ?request.action,
            base_token = %request.base_token,
            quote_token = %request.quote_token,
            amount = %request.amount,
            "Received trade request"
        );
        
        // Validate request format
        self.validate_trade_request(&request)?;
        
        // Get current prices with cache optimization
        let (base_price, quote_price, cache_hit) = self.get_token_prices(
            &request.base_token,
            &request.quote_token
        ).await?;
        
        debug!(
            base_price = %base_price,
            quote_price = %quote_price,
            cache_hit = cache_hit,
            "Retrieved token prices"
        );
        
        // Calculate expected execution price
        let expected_price = self.calculate_expected_price(
            &request,
            base_price,
            quote_price
        ).await?;
        
        // Apply slippage model
        let slippage_adjusted_price = self.apply_slippage_model(
            &request,
            expected_price,
            &mut warnings
        )?;
        
        debug!(
            expected_price = %expected_price,
            slippage_adjusted_price = %slippage_adjusted_price,
            "Applied slippage model"
        );
        
        // Apply MEV simulation if enabled
        let (final_price, mev_status) = if request.simulate_mev && self.execution_config.enable_mev_simulation {
            self.apply_mev_simulation(
                &request,
                slippage_adjusted_price,
                base_price,
                &mut warnings
            ).await?
        } else {
            (slippage_adjusted_price, MevStatus::NotSimulated)
        };
        
        // Check price impact limits
        let price_impact_bps = self.calculate_price_impact_bps(expected_price, final_price);
        if let Some(max_impact) = request.max_price_impact_bps {
            if price_impact_bps.abs() > max_impact as i64 {
                return Err(anyhow!(
                    "Price impact {} bps exceeds maximum allowed {} bps",
                    price_impact_bps,
                    max_impact
                ));
            }
        }
        
        // Calculate trade amounts
        let (base_amount, quote_amount) = self.calculate_trade_amounts(
            &request,
            final_price
        )?;
        
        // Create trade record
        let trade = Trade {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: request.action.clone(),
            base_token: TokenInfo {
                symbol: request.base_token.clone(),
                address: self.get_token_address(&request.base_token).await?,
                decimals: 9,  // Default for Solana
            },
            quote_token: TokenInfo {
                symbol: request.quote_token.clone(),
                address: self.get_token_address(&request.quote_token).await?,
                decimals: 9,
            },
            base_amount,
            quote_amount,
            executed_price: final_price,
            transaction_fee: 5000,  // Standard Solana transaction fee
            priority_fee: request.priority_fee,
            expected_slippage: self.get_configured_slippage(),
            actual_slippage: self.calculate_actual_slippage(expected_price, final_price),
            mev_status,
            transfer_fees: self.calculate_transfer_fees(&request).await?,
            tx_signature: None,  // No actual transaction in paper trading
            execution_time_ms: 0,  // Will be updated
            metadata: request.metadata.clone(),
        };
        
        // Update portfolio atomically
        let portfolio_update_start = Instant::now();
        let portfolio_state = self.update_portfolio(&trade).await?;
        let portfolio_update_time = portfolio_update_start.elapsed().as_millis() as u64;
        
        info!(
            trade_id = %trade.id,
            portfolio_update_ms = portfolio_update_time,
            "Portfolio updated successfully"
        );
        
        // Record trade in QuestDB (async, don't wait)
        let trade_clone = trade.clone();
        let quest_db = self.quest_db.clone();
        tokio::spawn(async move {
            if let Err(e) = quest_db.record_trade(&trade_clone).await {
                error!("Failed to record trade in QuestDB: {}", e);
            }
        });
        
        // Calculate execution metrics
        let total_execution_time = start_time.elapsed().as_millis() as u64;
        let execution_metrics = ExecutionMetrics {
            expected_price,
            execution_price: final_price,
            price_impact_bps,
            slippage_bps: ((final_price - expected_price) / expected_price * Decimal::from(10000))
                .round()
                .to_i64()
                .unwrap_or(0),
            execution_time_ms: total_execution_time,
            mev_simulation_applied: request.simulate_mev && mev_status != MevStatus::NotSimulated,
            cache_hit,
        };
        
        // Record metrics
        self.metrics_collector.record_execution(
            &request.base_token,
            &execution_metrics
        ).await;
        
        // Check execution time warning
        if total_execution_time > self.execution_config.max_execution_time_ms {
            warnings.push(format!(
                "Execution time {}ms exceeded target {}ms",
                total_execution_time,
                self.execution_config.max_execution_time_ms
            ));
        }
        
        info!(
            request_id = %request.request_id,
            execution_time_ms = total_execution_time,
            final_price = %final_price,
            "Trade execution completed"
        );
        
        Ok(TradeResult {
            request_id: request.request_id,
            trade,
            portfolio_state,
            execution_metrics,
            warnings,
        })
    }
}
```

### 3. Price Discovery and Caching

```rust
impl PaperTradeExecutor {
    async fn get_token_prices(
        &self,
        base_token: &str,
        quote_token: &str,
    ) -> Result<(Decimal, Decimal, bool)> {
        let mut cache_hit = true;
        
        // Try cache first for both tokens
        let base_price = match self.price_cache.get_price(base_token).await? {
            Some(price) => Decimal::from_f64(price).unwrap(),
            None => {
                cache_hit = false;
                self.fetch_fresh_price(base_token).await?
            }
        };
        
        let quote_price = match self.price_cache.get_price(quote_token).await? {
            Some(price) => Decimal::from_f64(price).unwrap(),
            None => {
                cache_hit = false;
                self.fetch_fresh_price(quote_token).await?
            }
        };
        
        Ok((base_price, quote_price, cache_hit))
    }
    
    async fn fetch_fresh_price(&self, token: &str) -> Result<Decimal> {
        // Get price from Jupiter with failover
        let price = self.jupiter_client
            .get_price(token, "USDC", Decimal::new(1000000, 6))
            .await?;
        
        // Cache the price
        self.price_cache
            .set_price(token, price.to_f64().unwrap(), self.execution_config.price_cache_ttl_ms / 1000)
            .await?;
        
        Ok(price)
    }
    
    async fn calculate_expected_price(
        &self,
        request: &TradeRequest,
        base_price: Decimal,
        quote_price: Decimal,
    ) -> Result<Decimal> {
        match request.action {
            TradeAction::Buy => Ok(quote_price / base_price),
            TradeAction::Sell => Ok(base_price / quote_price),
            TradeAction::Swap => {
                // For swaps, get route-specific price from Jupiter
                let params = SwapParams {
                    from_token: request.base_token.clone(),
                    to_token: request.quote_token.clone(),
                    amount: request.amount,
                    slippage_bps: request.slippage_tolerance_bps.unwrap_or(50),
                };
                
                let quote = self.jupiter_client.get_quote(&params).await?;
                Ok(quote.price)
            }
        }
    }
}
```

### 4. Slippage Model Implementation

```rust
impl PaperTradeExecutor {
    fn apply_slippage_model(
        &self,
        request: &TradeRequest,
        expected_price: Decimal,
        warnings: &mut Vec<String>,
    ) -> Result<Decimal> {
        let slippage = match &self.slippage_config {
            SlippageConfig::Fixed { percentage } => {
                // Use request override if provided
                if let Some(tolerance_bps) = request.slippage_tolerance_bps {
                    Decimal::new(tolerance_bps as i64, 4)
                } else {
                    *percentage
                }
            }
            SlippageConfig::Dynamic { base_percentage, size_impact_factor, volatility_multiplier } => {
                // Calculate dynamic slippage based on trade size and market conditions
                let base = *base_percentage;
                
                // Size impact: larger trades have more slippage
                let size_usd = request.amount * expected_price;
                let size_impact = size_usd / Decimal::new(10000, 0) * size_impact_factor;
                
                // Volatility impact (would use real volatility data in production)
                let volatility_impact = base * volatility_multiplier;
                
                let total_slippage = base + size_impact + volatility_impact;
                
                // Warn if dynamic slippage exceeds 5%
                if total_slippage > Decimal::new(5, 2) {
                    warnings.push(format!(
                        "High dynamic slippage calculated: {}%",
                        total_slippage * Decimal::from(100)
                    ));
                }
                
                total_slippage
            }
        };
        
        // Apply slippage based on trade direction
        let slippage_factor = match request.action {
            TradeAction::Buy => Decimal::ONE + slippage,
            TradeAction::Sell => Decimal::ONE - slippage,
            TradeAction::Swap => {
                if request.is_base_input {
                    Decimal::ONE - slippage  // Selling base for quote
                } else {
                    Decimal::ONE + slippage  // Buying base with quote
                }
            }
        };
        
        Ok(expected_price * slippage_factor)
    }
    
    fn get_configured_slippage(&self) -> Decimal {
        match &self.slippage_config {
            SlippageConfig::Fixed { percentage } => *percentage,
            SlippageConfig::Dynamic { base_percentage, .. } => *base_percentage,
        }
    }
}
```

### 5. MEV Simulation Integration

```rust
impl PaperTradeExecutor {
    async fn apply_mev_simulation(
        &self,
        request: &TradeRequest,
        current_price: Decimal,
        base_price: Decimal,
        warnings: &mut Vec<String>,
    ) -> Result<(Decimal, MevStatus)> {
        // Calculate trade size in USD
        let trade_size_usd = if request.is_base_input {
            request.amount * base_price
        } else {
            request.amount
        };
        
        debug!(
            trade_size_usd = %trade_size_usd,
            token = %request.base_token,
            "Running MEV simulation"
        );
        
        // Get MEV risk assessment
        let mev_assessment = self.mev_simulator
            .lock()
            .await
            .simulate_mev(&request.base_token, trade_size_usd, Utc::now())
            .await?;
        
        // Check if attack occurs and protection is insufficient
        if mev_assessment.is_attacked && request.priority_fee < mev_assessment.recommended_fee {
            let impact_factor = Decimal::new(mev_assessment.estimated_loss_bps as i64, 4);
            
            // Apply MEV impact to price
            let mev_price = match request.action {
                TradeAction::Buy => current_price * (Decimal::ONE + impact_factor),
                TradeAction::Sell => current_price * (Decimal::ONE - impact_factor),
                TradeAction::Swap => {
                    if request.is_base_input {
                        current_price * (Decimal::ONE - impact_factor)
                    } else {
                        current_price * (Decimal::ONE + impact_factor)
                    }
                }
            };
            
            warnings.push(format!(
                "MEV attack simulated: {} bps impact. Recommended priority fee: {} lamports",
                mev_assessment.estimated_loss_bps,
                mev_assessment.recommended_fee
            ));
            
            warn!(
                attack_type = ?mev_assessment.attack_type,
                impact_bps = mev_assessment.estimated_loss_bps,
                "MEV attack simulated"
            );
            
            Ok((mev_price, MevStatus::AtRisk))
        } else {
            if request.priority_fee >= mev_assessment.recommended_fee {
                debug!("Trade protected with sufficient priority fee");
                Ok((current_price, MevStatus::Protected))
            } else {
                debug!("No MEV attack simulated for this trade");
                Ok((current_price, MevStatus::NoAttack))
            }
        }
    }
}
```

### 6. Portfolio Update and Trade Recording

```rust
impl PaperTradeExecutor {
    async fn update_portfolio(&self, trade: &Trade) -> Result<PortfolioSummary> {
        let mut portfolio = self.portfolio.write().await;
        
        // Execute trade in virtual portfolio
        portfolio.execute_trade(trade.clone()).await?;
        
        // Get updated portfolio state
        Ok(portfolio.get_summary())
    }
    
    fn calculate_trade_amounts(
        &self,
        request: &TradeRequest,
        execution_price: Decimal,
    ) -> Result<(Decimal, Decimal)> {
        let (base_amount, quote_amount) = if request.is_base_input {
            (request.amount, request.amount * execution_price)
        } else {
            (request.amount / execution_price, request.amount)
        };
        
        Ok((base_amount, quote_amount))
    }
    
    async fn calculate_transfer_fees(&self, request: &TradeRequest) -> Result<Option<Decimal>> {
        // Check if token has transfer fees (Token-2022)
        // This would query token metadata in production
        if request.base_token.contains("TRANSFER_FEE") {
            Ok(Some(Decimal::new(25, 4)))  // 0.25% example fee
        } else {
            Ok(None)
        }
    }
}
```

### 7. Request Validation

```rust
impl PaperTradeExecutor {
    fn validate_trade_request(&self, request: &TradeRequest) -> Result<()> {
        // Format validation only, not business logic
        if request.amount <= Decimal::ZERO {
            return Err(anyhow!("Trade amount must be positive"));
        }
        
        if request.base_token.is_empty() || request.quote_token.is_empty() {
            return Err(anyhow!("Token symbols cannot be empty"));
        }
        
        if request.base_token == request.quote_token {
            return Err(anyhow!("Base and quote tokens must be different"));
        }
        
        if request.priority_fee > 1_000_000 {  // Sanity check
            return Err(anyhow!("Priority fee exceeds reasonable limit"));
        }
        
        if let Some(slippage_bps) = request.slippage_tolerance_bps {
            if slippage_bps > 5000 {  // 50% max
                return Err(anyhow!("Slippage tolerance exceeds 50%"));
            }
        }
        
        Ok(())
    }
}
```

### 8. Metrics Collection

```rust
pub struct MetricsCollector {
    execution_times: Arc<RwLock<Vec<u64>>>,
    slippage_stats: Arc<RwLock<HashMap<String, Vec<i64>>>>,
    cache_hit_rate: Arc<RwLock<(u64, u64)>>,  // (hits, total)
}

impl MetricsCollector {
    pub async fn record_execution(&self, token: &str, metrics: &ExecutionMetrics) {
        // Record execution time
        self.execution_times.write().await.push(metrics.execution_time_ms);
        
        // Record slippage by token
        self.slippage_stats
            .write()
            .await
            .entry(token.to_string())
            .or_default()
            .push(metrics.slippage_bps);
        
        // Update cache hit rate
        let mut cache_stats = self.cache_hit_rate.write().await;
        cache_stats.1 += 1;  // Total
        if metrics.cache_hit {
            cache_stats.0 += 1;  // Hits
        }
    }
    
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let execution_times = self.execution_times.read().await;
        let cache_stats = self.cache_hit_rate.read().await;
        
        PerformanceSummary {
            avg_execution_time_ms: execution_times.iter().sum::<u64>() / execution_times.len() as u64,
            p99_execution_time_ms: self.calculate_percentile(&execution_times, 0.99),
            cache_hit_rate: cache_stats.0 as f64 / cache_stats.1 as f64,
            total_executions: execution_times.len(),
        }
    }
}
```

## Configuration Options

The Paper Trade Executor supports extensive configuration:

1. **Slippage Models**:
   - Fixed percentage (0.5-2% for MVP)
   - Dynamic model with size and volatility factors

2. **Execution Parameters**:
   - MEV simulation enable/disable
   - Default priority fees
   - Maximum execution time limits
   - Price cache TTL

3. **Risk Controls**:
   - Maximum price impact limits
   - Slippage tolerance overrides
   - Trade size validation

## Integration Points

1. **gRPC Interface**: Accepts external `TradeRequest` objects
2. **Virtual Portfolio**: Updates balances and tracks P&L
3. **MEV Simulator**: Applies realistic market impact
4. **Jupiter Client**: Provides accurate price discovery
5. **Redis Cache**: Enables <1ms price lookups
6. **QuestDB**: Records comprehensive trade history

## Performance Characteristics

- Trade execution: <100ms target (excluding MEV simulation)
- Price cache lookups: <1ms
- Portfolio updates: <10ms
- Total end-to-end: <100ms with all features enabled
- Supports 1000+ concurrent executions

## Future Enhancements

1. Advanced slippage models using real liquidity data
2. Multi-route execution simulation
3. Gas optimization strategies
4. Cross-DEX arbitrage detection
5. Machine learning-based price prediction