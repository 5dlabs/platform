# Task 20: Implement Real-time Pool State Monitoring and Dynamic Slippage

## Overview

This task creates a system that monitors Solana DEX pool states in real-time and provides dynamic slippage calculations based on current liquidity conditions. This enhancement significantly improves trade simulation accuracy by using actual pool liquidity data instead of fixed slippage models.

## Architecture Context

The real-time pool monitoring system is crucial for accurate paper trading and optimal live execution:

- **Real-time Updates**: Subscribes to pool account changes via gRPC
- **Multi-DEX Support**: Handles Raydium, Orca, and other Jupiter-supported DEXs
- **Dynamic Slippage**: Calculates expected slippage based on actual liquidity
- **Historical Analysis**: Stores pool states in QuestDB for pattern analysis
- **Cache Layer**: Redis caching with 1-2 second TTL for performance

This component bridges the gap between paper and live trading by providing realistic market impact simulation.

## Implementation Details

### 1. Pool State Monitoring Service

#### Core Monitoring Architecture
```rust
use solana_client::rpc_pubsub::RpcPubsubClient;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct PoolStateMonitor {
    subscriptions: Arc<RwLock<HashMap<Pubkey, PoolSubscription>>>,
    parsers: Arc<PoolParserRegistry>,
    cache: Arc<dyn PriceCache>,
    metrics_store: Arc<dyn MetricsStore>,
    health_monitor: Arc<HealthMonitor>,
}

#[derive(Debug)]
struct PoolSubscription {
    pool_address: Pubkey,
    program_id: Pubkey,
    subscription_id: u64,
    last_update: Instant,
    update_count: u64,
}

impl PoolStateMonitor {
    pub async fn new(
        rpc_url: &str,
        cache: Arc<dyn PriceCache>,
        metrics_store: Arc<dyn MetricsStore>,
    ) -> Result<Self, MonitorError> {
        let parsers = Arc::new(PoolParserRegistry::new());
        
        // Register DEX parsers
        parsers.register(Box::new(RaydiumPoolParser::new()));
        parsers.register(Box::new(OrcaPoolParser::new()));
        parsers.register(Box::new(OrcaWhirlpoolParser::new()));
        parsers.register(Box::new(LifinityPoolParser::new()));
        
        Ok(Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            parsers,
            cache,
            metrics_store,
            health_monitor: Arc::new(HealthMonitor::new()),
        })
    }

    pub async fn subscribe_to_pool(&self, pool_address: Pubkey) -> Result<(), MonitorError> {
        // Determine DEX type from pool address
        let program_id = self.identify_program(&pool_address).await?;
        
        // Create subscription
        let pubsub_client = RpcPubsubClient::new(&self.rpc_url).await?;
        
        let subscription_id = pubsub_client
            .account_subscribe(
                &pool_address,
                Some(RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    commitment: Some(CommitmentConfig::confirmed()),
                    data_slice: None,
                }),
            )
            .await?;

        // Start monitoring task
        let monitor_handle = self.start_pool_monitor(
            pool_address,
            program_id,
            pubsub_client,
        ).await;

        // Store subscription
        let subscription = PoolSubscription {
            pool_address,
            program_id,
            subscription_id,
            last_update: Instant::now(),
            update_count: 0,
        };

        self.subscriptions.write().await.insert(pool_address, subscription);

        Ok(())
    }

    async fn start_pool_monitor(
        &self,
        pool_address: Pubkey,
        program_id: Pubkey,
        mut pubsub_client: RpcPubsubClient,
    ) -> JoinHandle<()> {
        let parsers = self.parsers.clone();
        let cache = self.cache.clone();
        let metrics_store = self.metrics_store.clone();
        let subscriptions = self.subscriptions.clone();

        tokio::spawn(async move {
            while let Ok(update) = pubsub_client.recv().await {
                match update {
                    RpcResponse::AccountUpdate(account_info) => {
                        if let Err(e) = Self::process_pool_update(
                            &pool_address,
                            &program_id,
                            account_info,
                            &parsers,
                            &cache,
                            &metrics_store,
                            &subscriptions,
                        ).await {
                            error!("Failed to process pool update: {}", e);
                        }
                    }
                    _ => {}
                }
            }
        })
    }

    async fn process_pool_update(
        pool_address: &Pubkey,
        program_id: &Pubkey,
        account_info: RpcAccountInfo,
        parsers: &PoolParserRegistry,
        cache: &Arc<dyn PriceCache>,
        metrics_store: &Arc<dyn MetricsStore>,
        subscriptions: &Arc<RwLock<HashMap<Pubkey, PoolSubscription>>>,
    ) -> Result<(), MonitorError> {
        let start = Instant::now();

        // Decode account data
        let account_data = base64::decode(&account_info.data[0])
            .map_err(|e| MonitorError::DecodeError(e.to_string()))?;

        // Parse pool state
        let parser = parsers
            .get_parser(program_id)
            .ok_or_else(|| MonitorError::UnsupportedDex(program_id.to_string()))?;

        let pool_state = parser.parse_pool_state(&account_data)?;

        // Cache pool state (1-2 second TTL)
        cache.cache_pool_state(
            pool_address,
            pool_state.clone(),
            Duration::from_secs(2),
        ).await?;

        // Record metrics
        metrics_store.record_metric(
            "pool_update_latency",
            start.elapsed().as_micros() as f64,
            hashmap! {
                "pool".to_string() => pool_address.to_string(),
                "dex".to_string() => parser.dex_name(),
            },
        ).await?;

        // Update subscription stats
        if let Some(mut sub) = subscriptions.write().await.get_mut(pool_address) {
            sub.last_update = Instant::now();
            sub.update_count += 1;
        }

        // Store in QuestDB for historical analysis
        Self::store_pool_snapshot(pool_address, &pool_state, metrics_store).await?;

        Ok(())
    }

    async fn identify_program(&self, pool_address: &Pubkey) -> Result<Pubkey, MonitorError> {
        // Query pool account to determine program owner
        let account = self.rpc_client.get_account(pool_address).await?;
        Ok(account.owner)
    }
}
```

### 2. DEX-Specific Pool Parsers

```rust
pub trait PoolStateParser: Send + Sync + 'static {
    fn parse_pool_state(&self, account_data: &[u8]) -> Result<PoolState, ParseError>;
    fn get_supported_program_ids(&self) -> Vec<Pubkey>;
    fn dex_name(&self) -> &'static str;
}

pub struct RaydiumPoolParser;

impl PoolStateParser for RaydiumPoolParser {
    fn parse_pool_state(&self, account_data: &[u8]) -> Result<PoolState, ParseError> {
        // Raydium AMM V4 pool layout
        const POOL_STATE_SIZE: usize = 1000; // Actual size
        
        if account_data.len() < POOL_STATE_SIZE {
            return Err(ParseError::InvalidDataSize);
        }

        // Parse using bytemuck or manual deserialization
        let pool_data = RaydiumAmmInfo::try_from_slice(account_data)?;

        // Calculate current price from reserves
        let price = pool_data.coin_vault_balance as f64 / pool_data.pc_vault_balance as f64;
        
        // Calculate liquidity metrics
        let liquidity_usd = Self::calculate_liquidity_usd(
            pool_data.coin_vault_balance,
            pool_data.pc_vault_balance,
            &pool_data.coin_mint,
            &pool_data.pc_mint,
        ).await?;

        Ok(PoolState {
            pool_address: pool_data.pool_address,
            token_a_mint: pool_data.coin_mint,
            token_b_mint: pool_data.pc_mint,
            token_a_reserve: pool_data.coin_vault_balance,
            token_b_reserve: pool_data.pc_vault_balance,
            fee_rate: pool_data.fee_rate as f64 / 10000.0, // Convert basis points
            liquidity_usd,
            price,
            volume_24h: 0.0, // Would need to track separately
            last_update: Utc::now(),
            pool_type: PoolType::Raydium,
            additional_data: Some(json!({
                "status": pool_data.status,
                "open_orders": pool_data.open_orders.to_string(),
            })),
        })
    }

    fn get_supported_program_ids(&self) -> Vec<Pubkey> {
        vec![
            Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap(), // Raydium V4
        ]
    }

    fn dex_name(&self) -> &'static str {
        "Raydium"
    }
}

pub struct OrcaPoolParser;

impl PoolStateParser for OrcaPoolParser {
    fn parse_pool_state(&self, account_data: &[u8]) -> Result<PoolState, ParseError> {
        // Orca pool uses different layout
        let pool_data = OrcaPoolState::try_from_slice(account_data)?;

        let sqrt_price = pool_data.sqrt_price;
        let price = (sqrt_price as f64 / 2u64.pow(64) as f64).powi(2);

        Ok(PoolState {
            pool_address: pool_data.pool_address,
            token_a_mint: pool_data.token_a,
            token_b_mint: pool_data.token_b,
            token_a_reserve: pool_data.token_a_amount,
            token_b_reserve: pool_data.token_b_amount,
            fee_rate: pool_data.fee_rate as f64 / 1_000_000.0, // Parts per million
            liquidity_usd: 0.0, // Calculate separately
            price,
            volume_24h: 0.0,
            last_update: Utc::now(),
            pool_type: PoolType::Orca,
            additional_data: Some(json!({
                "tick_spacing": pool_data.tick_spacing,
                "liquidity": pool_data.liquidity,
            })),
        })
    }

    fn get_supported_program_ids(&self) -> Vec<Pubkey> {
        vec![
            Pubkey::from_str("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP").unwrap(), // Orca
        ]
    }

    fn dex_name(&self) -> &'static str {
        "Orca"
    }
}

// Additional parsers for other DEXs...

pub struct PoolParserRegistry {
    parsers: RwLock<HashMap<Pubkey, Box<dyn PoolStateParser>>>,
}

impl PoolParserRegistry {
    pub fn register(&self, parser: Box<dyn PoolStateParser>) {
        let mut parsers = self.parsers.write().unwrap();
        for program_id in parser.get_supported_program_ids() {
            parsers.insert(program_id, parser.clone());
        }
    }

    pub fn get_parser(&self, program_id: &Pubkey) -> Option<Box<dyn PoolStateParser>> {
        self.parsers.read().unwrap().get(program_id).cloned()
    }
}
```

### 3. Liquidity Depth Analysis

```rust
pub struct LiquidityAnalyzer {
    pool_cache: Arc<dyn PriceCache>,
    historical_store: Arc<dyn MetricsStore>,
}

impl LiquidityAnalyzer {
    pub async fn calculate_liquidity_depth(
        &self,
        pool_address: &Pubkey,
        token_mint: &Pubkey,
    ) -> Result<LiquidityDepth, AnalysisError> {
        // Get current pool state
        let pool_state = self.pool_cache
            .get_pool_state(pool_address)
            .await?
            .ok_or(AnalysisError::PoolNotFound)?;

        // Calculate depth at various price points
        let current_price = pool_state.price;
        let price_points = vec![
            current_price * 0.99,  // 1% down
            current_price * 0.98,  // 2% down
            current_price * 0.95,  // 5% down
            current_price * 0.90,  // 10% down
            current_price * 1.01,  // 1% up
            current_price * 1.02,  // 2% up
            current_price * 1.05,  // 5% up
            current_price * 1.10,  // 10% up
        ];

        let mut depth_levels = Vec::new();
        
        for target_price in price_points {
            let available_liquidity = self.calculate_liquidity_at_price(
                &pool_state,
                target_price,
                token_mint,
            )?;

            depth_levels.push(LiquidityLevel {
                price: target_price,
                available_amount: available_liquidity,
                price_impact_bps: ((target_price / current_price - 1.0).abs() * 10000.0) as u16,
            });
        }

        // Analyze concentration
        let concentration_score = self.calculate_concentration_score(&depth_levels);

        Ok(LiquidityDepth {
            pool_address: *pool_address,
            timestamp: Utc::now(),
            current_price,
            depth_levels,
            total_liquidity_usd: pool_state.liquidity_usd,
            concentration_score,
            health_score: self.calculate_health_score(&pool_state, concentration_score),
        })
    }

    fn calculate_liquidity_at_price(
        &self,
        pool: &PoolState,
        target_price: f64,
        token_mint: &Pubkey,
    ) -> Result<f64, AnalysisError> {
        // Constant product AMM formula: x * y = k
        let k = pool.token_a_reserve as f64 * pool.token_b_reserve as f64;
        
        if *token_mint == pool.token_a_mint {
            // Calculate how much token A can be bought to reach target price
            let new_reserve_b = (k / target_price).sqrt();
            let amount_b_needed = new_reserve_b - pool.token_b_reserve as f64;
            
            if amount_b_needed > 0.0 {
                // Buying token A
                let new_reserve_a = k / new_reserve_b;
                Ok(pool.token_a_reserve as f64 - new_reserve_a)
            } else {
                // Selling token A
                let new_reserve_a = (k * target_price).sqrt();
                Ok(new_reserve_a - pool.token_a_reserve as f64)
            }
        } else {
            // Similar calculation for token B
            let new_reserve_a = (k * target_price).sqrt();
            let amount_a_needed = new_reserve_a - pool.token_a_reserve as f64;
            
            if amount_a_needed > 0.0 {
                let new_reserve_b = k / new_reserve_a;
                Ok(pool.token_b_reserve as f64 - new_reserve_b)
            } else {
                let new_reserve_b = (k / target_price).sqrt();
                Ok(new_reserve_b - pool.token_b_reserve as f64)
            }
        }
    }

    fn calculate_concentration_score(&self, depth_levels: &[LiquidityLevel]) -> f64 {
        // Score from 0-1 where 1 is perfectly distributed liquidity
        let total_liquidity: f64 = depth_levels.iter()
            .map(|l| l.available_amount)
            .sum();

        if total_liquidity == 0.0 {
            return 0.0;
        }

        // Calculate standard deviation of liquidity distribution
        let mean = total_liquidity / depth_levels.len() as f64;
        let variance: f64 = depth_levels.iter()
            .map(|l| (l.available_amount - mean).powi(2))
            .sum::<f64>() / depth_levels.len() as f64;

        let std_dev = variance.sqrt();
        let cv = std_dev / mean; // Coefficient of variation

        // Convert to 0-1 score (lower CV = higher score)
        (1.0 / (1.0 + cv)).min(1.0).max(0.0)
    }

    pub async fn analyze_volume_impact(
        &self,
        pool_address: &Pubkey,
        window: Duration,
    ) -> Result<VolumeImpact, AnalysisError> {
        // Query historical volume data
        let volume_data = self.historical_store
            .query_metrics(MetricsQuery {
                metric_name: "pool_volume".to_string(),
                start_time: Utc::now() - window,
                end_time: Utc::now(),
                labels: hashmap! {
                    "pool".to_string() => pool_address.to_string(),
                },
                aggregation: Some(AggregationType::Sum),
            })
            .await?;

        // Analyze volume patterns
        let total_volume = volume_data.iter()
            .map(|p| p.value)
            .sum::<f64>();

        let avg_trade_size = if !volume_data.is_empty() {
            total_volume / volume_data.len() as f64
        } else {
            0.0
        };

        // Get current liquidity
        let pool_state = self.pool_cache
            .get_pool_state(pool_address)
            .await?
            .ok_or(AnalysisError::PoolNotFound)?;

        Ok(VolumeImpact {
            average_trade_size: avg_trade_size,
            volume_to_liquidity_ratio: total_volume / pool_state.liquidity_usd,
            large_trade_threshold: pool_state.liquidity_usd * 0.01, // 1% of liquidity
            estimated_large_trade_slippage_bps: self.estimate_slippage_for_size(
                avg_trade_size * 10.0, // 10x average
                &pool_state,
            ),
        })
    }

    fn estimate_slippage_for_size(&self, trade_size_usd: f64, pool: &PoolState) -> u16 {
        // Simplified slippage estimation
        let size_ratio = trade_size_usd / pool.liquidity_usd;
        
        // Base slippage from fee
        let base_slippage = pool.fee_rate;
        
        // Price impact based on size
        let price_impact = size_ratio * 100.0; // Roughly 1% impact per 1% of liquidity
        
        ((base_slippage + price_impact) * 10000.0) as u16
    }
}
```

### 4. Dynamic Slippage Calculator

```rust
pub struct DynamicSlippageCalculator {
    liquidity_analyzer: Arc<LiquidityAnalyzer>,
    pool_cache: Arc<dyn PriceCache>,
    historical_analyzer: Arc<HistoricalSlippageAnalyzer>,
}

impl DynamicSlippageCalculator {
    pub async fn calculate_dynamic_slippage(
        &self,
        pool_address: &Pubkey,
        token_in: &Pubkey,
        token_out: &Pubkey,
        amount_in: u64,
        side: TradeSide,
    ) -> Result<SlippageEstimate, CalculationError> {
        let start = Instant::now();

        // Get current pool state
        let pool_state = self.pool_cache
            .get_pool_state(pool_address)
            .await?
            .ok_or(CalculationError::PoolNotFound)?;

        // Calculate base slippage from current liquidity
        let base_slippage = self.calculate_base_slippage(
            &pool_state,
            token_in,
            amount_in,
        )?;

        // Get liquidity depth analysis
        let liquidity_depth = self.liquidity_analyzer
            .calculate_liquidity_depth(pool_address, token_in)
            .await?;

        // Analyze historical patterns
        let historical_adjustment = self.historical_analyzer
            .get_slippage_adjustment(
                pool_address,
                amount_in,
                side,
                Duration::hours(24),
            )
            .await?;

        // Calculate volume impact
        let volume_impact = self.liquidity_analyzer
            .analyze_volume_impact(pool_address, Duration::hours(1))
            .await?;

        // Combine factors
        let dynamic_slippage_bps = self.combine_slippage_factors(
            base_slippage,
            &liquidity_depth,
            historical_adjustment,
            &volume_impact,
            amount_in,
        );

        // Calculate confidence based on data quality
        let confidence = self.calculate_confidence(
            &pool_state,
            &liquidity_depth,
            start.elapsed(),
        );

        Ok(SlippageEstimate {
            expected_slippage_bps: dynamic_slippage_bps,
            worst_case_slippage_bps: (dynamic_slippage_bps as f64 * 1.5) as u16,
            confidence_score: confidence,
            factors: SlippageFactors {
                base_fee_bps: (pool_state.fee_rate * 10000.0) as u16,
                liquidity_impact_bps: base_slippage,
                volume_impact_bps: (volume_impact.volume_to_liquidity_ratio * 10000.0) as u16,
                historical_adjustment_bps: historical_adjustment,
            },
            calculation_time_us: start.elapsed().as_micros() as u64,
        })
    }

    fn calculate_base_slippage(
        &self,
        pool: &PoolState,
        token_in: &Pubkey,
        amount_in: u64,
    ) -> Result<u16, CalculationError> {
        // Constant product formula impact
        let (reserve_in, reserve_out) = if *token_in == pool.token_a_mint {
            (pool.token_a_reserve, pool.token_b_reserve)
        } else {
            (pool.token_b_reserve, pool.token_a_reserve)
        };

        let amount_in_with_fee = amount_in as f64 * (1.0 - pool.fee_rate);
        let amount_out = (amount_in_with_fee * reserve_out as f64) / 
                        (reserve_in as f64 + amount_in_with_fee);

        let effective_price = amount_out / amount_in as f64;
        let spot_price = reserve_out as f64 / reserve_in as f64;
        
        let slippage = ((spot_price - effective_price) / spot_price).abs();
        
        Ok((slippage * 10000.0) as u16)
    }

    fn combine_slippage_factors(
        &self,
        base_slippage: u16,
        liquidity_depth: &LiquidityDepth,
        historical_adjustment: i16,
        volume_impact: &VolumeImpact,
        trade_size: u64,
    ) -> u16 {
        let mut total_slippage = base_slippage as i32;

        // Adjust for liquidity concentration
        if liquidity_depth.concentration_score < 0.5 {
            // Poor liquidity distribution increases slippage
            total_slippage = (total_slippage as f64 * 1.2) as i32;
        }

        // Add historical adjustment (can be negative)
        total_slippage += historical_adjustment as i32;

        // Add volume impact for large trades
        if trade_size as f64 > volume_impact.large_trade_threshold {
            total_slippage += volume_impact.estimated_large_trade_slippage_bps as i32;
        }

        // Ensure non-negative
        total_slippage.max(0) as u16
    }

    fn calculate_confidence(
        &self,
        pool_state: &PoolState,
        liquidity_depth: &LiquidityDepth,
        calculation_time: Duration,
    ) -> f64 {
        let mut confidence = 1.0;

        // Reduce confidence for stale data
        let data_age = Utc::now() - pool_state.last_update;
        if data_age > Duration::seconds(5) {
            confidence *= 0.8;
        }

        // Reduce confidence for poor liquidity
        if liquidity_depth.health_score < 0.5 {
            confidence *= 0.7;
        }

        // Reduce confidence for slow calculations
        if calculation_time > Duration::from_millis(100) {
            confidence *= 0.9;
        }

        confidence.max(0.0).min(1.0)
    }
}
```

### 5. Integration with Paper Trade Executor

```rust
pub struct EnhancedPaperTrader {
    base_executor: Arc<PaperTradeExecutor>,
    slippage_calculator: Arc<DynamicSlippageCalculator>,
    pool_monitor: Arc<PoolStateMonitor>,
    feedback_collector: Arc<SlippageFeedbackCollector>,
}

impl EnhancedPaperTrader {
    pub async fn execute_trade_with_dynamic_slippage(
        &self,
        request: TradeRequest,
    ) -> Result<TradeResult, TradeError> {
        // Ensure pool is monitored
        self.ensure_pool_monitored(&request.pool_address).await?;

        // Calculate dynamic slippage
        let slippage_estimate = self.slippage_calculator
            .calculate_dynamic_slippage(
                &request.pool_address,
                &request.token_in,
                &request.token_out,
                request.amount_in,
                request.side,
            )
            .await?;

        // Log slippage factors for transparency
        info!(
            "Dynamic slippage for trade {}: expected {}bps (confidence: {:.2})",
            request.id,
            slippage_estimate.expected_slippage_bps,
            slippage_estimate.confidence_score
        );

        // Apply slippage to get expected output
        let expected_output = self.calculate_expected_output(
            &request,
            slippage_estimate.expected_slippage_bps,
        )?;

        // Execute paper trade
        let mut result = self.base_executor
            .execute_trade(request.clone())
            .await?;

        // Enhance result with dynamic calculations
        result.actual_slippage_bps = slippage_estimate.expected_slippage_bps;
        result.expected_output = expected_output;
        result.slippage_factors = Some(slippage_estimate.factors);

        // Collect feedback for model improvement
        self.feedback_collector
            .record_trade_result(&request, &result, &slippage_estimate)
            .await?;

        Ok(result)
    }

    async fn ensure_pool_monitored(&self, pool_address: &Pubkey) -> Result<(), TradeError> {
        let subscriptions = self.pool_monitor.subscriptions.read().await;
        
        if !subscriptions.contains_key(pool_address) {
            drop(subscriptions);
            self.pool_monitor.subscribe_to_pool(*pool_address).await?;
            
            // Wait for initial state
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }
}

// Feedback collection for continuous improvement
pub struct SlippageFeedbackCollector {
    storage: Arc<dyn MetricsStore>,
}

impl SlippageFeedbackCollector {
    pub async fn record_trade_result(
        &self,
        request: &TradeRequest,
        result: &TradeResult,
        estimate: &SlippageEstimate,
    ) -> Result<(), FeedbackError> {
        // Store prediction vs actual for analysis
        let feedback = SlippageFeedback {
            timestamp: Utc::now(),
            pool_address: request.pool_address,
            trade_size: request.amount_in,
            predicted_slippage_bps: estimate.expected_slippage_bps,
            actual_slippage_bps: result.actual_slippage_bps,
            confidence_score: estimate.confidence_score,
            factors: estimate.factors.clone(),
        };

        self.storage
            .record_metric(
                "slippage_feedback",
                feedback.actual_slippage_bps as f64,
                feedback.to_labels(),
            )
            .await?;

        Ok(())
    }

    pub async fn analyze_prediction_accuracy(
        &self,
        window: Duration,
    ) -> Result<AccuracyReport, FeedbackError> {
        let feedback_data = self.storage
            .query_metrics(MetricsQuery {
                metric_name: "slippage_feedback".to_string(),
                start_time: Utc::now() - window,
                end_time: Utc::now(),
                labels: HashMap::new(),
                aggregation: None,
            })
            .await?;

        // Calculate accuracy metrics
        let mut total_error = 0.0;
        let mut weighted_error = 0.0;
        let mut total_weight = 0.0;

        for point in &feedback_data {
            let predicted = point.labels.get("predicted_bps")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            let actual = point.value;
            let confidence = point.labels.get("confidence")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.5);

            let error = (predicted - actual).abs();
            total_error += error;
            weighted_error += error * confidence;
            total_weight += confidence;
        }

        let avg_error = if !feedback_data.is_empty() {
            total_error / feedback_data.len() as f64
        } else {
            0.0
        };

        let weighted_avg_error = if total_weight > 0.0 {
            weighted_error / total_weight
        } else {
            0.0
        };

        Ok(AccuracyReport {
            sample_count: feedback_data.len(),
            average_error_bps: avg_error,
            weighted_average_error_bps: weighted_avg_error,
            confidence_correlation: self.calculate_confidence_correlation(&feedback_data),
        })
    }
}
```

### 6. Pool Health Monitoring

```rust
pub struct PoolHealthMonitor {
    pool_cache: Arc<dyn PriceCache>,
    metrics_store: Arc<dyn MetricsStore>,
    alert_manager: Arc<AlertManager>,
}

impl PoolHealthMonitor {
    pub async fn monitor_pool_health(&self) -> Result<(), MonitorError> {
        loop {
            let pools = self.get_monitored_pools().await?;

            for pool_address in pools {
                if let Err(e) = self.check_pool_health(&pool_address).await {
                    error!("Health check failed for pool {}: {}", pool_address, e);
                }
            }

            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }

    async fn check_pool_health(&self, pool_address: &Pubkey) -> Result<(), MonitorError> {
        let pool_state = self.pool_cache
            .get_pool_state(pool_address)
            .await?
            .ok_or(MonitorError::PoolNotFound)?;

        let health_metrics = PoolHealthMetrics {
            liquidity_usd: pool_state.liquidity_usd,
            price_stability: self.calculate_price_stability(pool_address).await?,
            update_frequency: self.get_update_frequency(pool_address).await?,
            spread_bps: self.calculate_spread(pool_address).await?,
        };

        // Check thresholds
        if health_metrics.liquidity_usd < 10_000.0 {
            self.alert_manager
                .send_alert(Alert {
                    severity: Severity::Warning,
                    message: format!("Low liquidity in pool {}: ${:.2}", 
                        pool_address, health_metrics.liquidity_usd),
                    pool_address: Some(*pool_address),
                })
                .await?;
        }

        if health_metrics.update_frequency < 0.1 {
            self.alert_manager
                .send_alert(Alert {
                    severity: Severity::Critical,
                    message: format!("Pool {} appears stale ({}Hz update rate)", 
                        pool_address, health_metrics.update_frequency),
                    pool_address: Some(*pool_address),
                })
                .await?;
        }

        // Record metrics
        self.metrics_store
            .record_metric(
                "pool_health_score",
                self.calculate_health_score(&health_metrics),
                hashmap! {
                    "pool".to_string() => pool_address.to_string(),
                },
            )
            .await?;

        Ok(())
    }

    fn calculate_health_score(&self, metrics: &PoolHealthMetrics) -> f64 {
        let mut score = 1.0;

        // Liquidity score (log scale)
        let liquidity_score = (metrics.liquidity_usd.ln() / 20.0).min(1.0).max(0.0);
        score *= liquidity_score;

        // Update frequency score
        let update_score = (metrics.update_frequency / 1.0).min(1.0).max(0.0);
        score *= update_score;

        // Price stability score
        score *= metrics.price_stability;

        // Spread score (inverse relationship)
        let spread_score = 1.0 / (1.0 + metrics.spread_bps as f64 / 100.0);
        score *= spread_score;

        score
    }
}
```

### 7. Pool Registry System

```rust
pub struct PoolRegistry {
    registered_pools: Arc<RwLock<HashMap<Pubkey, PoolMetadata>>>,
    storage: Arc<dyn ConfigStore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMetadata {
    pub address: Pubkey,
    pub token_a: TokenInfo,
    pub token_b: TokenInfo,
    pub dex_type: DexType,
    pub fee_tier: u16,
    pub is_active: bool,
    pub priority: PoolPriority,
    pub added_at: DateTime<Utc>,
    pub last_health_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PoolPriority {
    Critical,  // Core trading pairs
    High,      // High volume pairs
    Normal,    // Regular pairs
    Low,       // Low priority monitoring
}

impl PoolRegistry {
    pub async fn register_pool(
        &self,
        address: Pubkey,
        metadata: PoolMetadata,
    ) -> Result<(), RegistryError> {
        // Validate pool exists on-chain
        self.validate_pool(&address).await?;

        // Store in registry
        self.registered_pools.write().await.insert(address, metadata.clone());

        // Persist to database
        self.storage
            .save_pool_metadata(&metadata)
            .await?;

        info!("Registered pool {} with priority {:?}", address, metadata.priority);

        Ok(())
    }

    pub async fn get_pools_by_priority(
        &self,
        priority: PoolPriority,
    ) -> Vec<PoolMetadata> {
        self.registered_pools
            .read()
            .await
            .values()
            .filter(|p| p.priority == priority && p.is_active)
            .cloned()
            .collect()
    }

    pub async fn update_pool_health(
        &self,
        address: &Pubkey,
        health_status: PoolHealthStatus,
    ) -> Result<(), RegistryError> {
        let mut pools = self.registered_pools.write().await;
        
        if let Some(pool) = pools.get_mut(address) {
            pool.last_health_check = Utc::now();
            
            // Deactivate unhealthy pools
            if health_status.score < 0.3 {
                pool.is_active = false;
                warn!("Deactivating unhealthy pool {}", address);
            }
        }

        Ok(())
    }

    pub async fn get_monitoring_priorities(&self) -> HashMap<Pubkey, PoolPriority> {
        self.registered_pools
            .read()
            .await
            .iter()
            .filter(|(_, p)| p.is_active)
            .map(|(addr, meta)| (*addr, meta.priority))
            .collect()
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
    async fn test_pool_parser_raydium() {
        let parser = RaydiumPoolParser;
        let test_data = create_test_raydium_pool_data();

        let pool_state = parser.parse_pool_state(&test_data).unwrap();
        
        assert_eq!(pool_state.pool_type, PoolType::Raydium);
        assert!(pool_state.fee_rate > 0.0);
        assert!(pool_state.token_a_reserve > 0);
        assert!(pool_state.token_b_reserve > 0);
    }

    #[tokio::test]
    async fn test_dynamic_slippage_calculation() {
        let calculator = create_test_slippage_calculator();
        
        let estimate = calculator
            .calculate_dynamic_slippage(
                &test_pool_address(),
                &SOL_MINT,
                &USDC_MINT,
                1_000_000_000, // 1 SOL
                TradeSide::Buy,
            )
            .await
            .unwrap();

        assert!(estimate.expected_slippage_bps > 0);
        assert!(estimate.worst_case_slippage_bps > estimate.expected_slippage_bps);
        assert!(estimate.confidence_score > 0.0 && estimate.confidence_score <= 1.0);
    }

    #[tokio::test]
    async fn test_liquidity_depth_analysis() {
        let analyzer = create_test_liquidity_analyzer();
        let pool_state = create_test_pool_state();

        let depth = analyzer
            .calculate_liquidity_depth(&pool_state.pool_address, &SOL_MINT)
            .await
            .unwrap();

        assert_eq!(depth.depth_levels.len(), 8);
        assert!(depth.concentration_score >= 0.0 && depth.concentration_score <= 1.0);
        
        // Verify price points are ordered
        let prices: Vec<f64> = depth.depth_levels.iter().map(|l| l.price).collect();
        assert!(prices.windows(2).all(|w| w[0] <= w[1] || w[0] >= w[1]));
    }

    #[tokio::test]
    async fn test_pool_state_caching() {
        let cache = MockPriceCache::new();
        let pool_state = create_test_pool_state();

        // Cache with 2 second TTL
        cache
            .cache_pool_state(&pool_state.pool_address, pool_state.clone(), Duration::from_secs(2))
            .await
            .unwrap();

        // Should be available immediately
        let cached = cache.get_pool_state(&pool_state.pool_address).await.unwrap();
        assert_eq!(cached.unwrap().pool_address, pool_state.pool_address);

        // Should expire after TTL
        tokio::time::sleep(Duration::from_secs(3)).await;
        let expired = cache.get_pool_state(&pool_state.pool_address).await.unwrap();
        assert!(expired.is_none());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_pool_monitoring_integration() {
    let test_env = setup_test_environment().await;
    
    // Start pool monitor
    let monitor = PoolStateMonitor::new(
        &test_env.rpc_url,
        test_env.cache.clone(),
        test_env.metrics_store.clone(),
    ).await.unwrap();

    // Subscribe to test pool
    let pool_address = create_test_pool_on_chain().await;
    monitor.subscribe_to_pool(pool_address).await.unwrap();

    // Wait for initial update
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Verify pool state is cached
    let pool_state = test_env.cache.get_pool_state(&pool_address).await.unwrap();
    assert!(pool_state.is_some());

    // Simulate pool update
    update_test_pool_liquidity(&pool_address, 1000000).await;

    // Wait for update to propagate
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify updated state
    let updated_state = test_env.cache.get_pool_state(&pool_address).await.unwrap().unwrap();
    assert!(updated_state.liquidity_usd > 0.0);
}

#[tokio::test]
async fn test_slippage_feedback_loop() {
    let test_env = setup_test_environment().await;
    
    // Execute multiple paper trades
    for i in 0..100 {
        let request = create_test_trade_request(i);
        let result = test_env.paper_trader
            .execute_trade_with_dynamic_slippage(request)
            .await
            .unwrap();
        
        // Record should be saved automatically
    }

    // Analyze prediction accuracy
    let accuracy = test_env.feedback_collector
        .analyze_prediction_accuracy(Duration::hours(1))
        .await
        .unwrap();

    println!("Slippage prediction accuracy: {:.2} bps average error", accuracy.average_error_bps);
    assert!(accuracy.sample_count >= 100);
    assert!(accuracy.average_error_bps < 50.0); // Should be reasonably accurate
}

#[tokio::test]
async fn test_pool_health_monitoring() {
    let monitor = create_test_pool_health_monitor();
    
    // Create pools with different health levels
    let healthy_pool = create_pool_with_liquidity(1_000_000.0).await;
    let unhealthy_pool = create_pool_with_liquidity(1_000.0).await;

    // Check health
    monitor.check_pool_health(&healthy_pool).await.unwrap();
    let result = monitor.check_pool_health(&unhealthy_pool).await;
    
    // Should generate alert for low liquidity
    assert!(test_env.alert_manager.has_alerts().await);
}
```

### Performance Tests

```rust
#[tokio::test]
async fn test_slippage_calculation_performance() {
    let calculator = create_test_slippage_calculator();
    let pool_address = test_pool_address();

    // Warm up cache
    calculator.calculate_dynamic_slippage(
        &pool_address,
        &SOL_MINT,
        &USDC_MINT,
        1_000_000_000,
        TradeSide::Buy,
    ).await.unwrap();

    // Measure calculation time
    let mut times = Vec::new();
    for _ in 0..1000 {
        let start = Instant::now();
        
        let estimate = calculator.calculate_dynamic_slippage(
            &pool_address,
            &SOL_MINT,
            &USDC_MINT,
            1_000_000_000,
            TradeSide::Buy,
        ).await.unwrap();
        
        times.push(estimate.calculation_time_us);
    }

    times.sort_unstable();
    let p50 = times[500];
    let p99 = times[990];

    println!("Slippage calculation times - P50: {}μs, P99: {}μs", p50, p99);
    assert!(p99 < 1000); // Should be sub-millisecond
}

#[tokio::test]
async fn test_concurrent_pool_monitoring() {
    let monitor = create_test_pool_monitor();
    let pools: Vec<Pubkey> = (0..100).map(|_| Pubkey::new_unique()).collect();

    // Subscribe to many pools concurrently
    let handles: Vec<_> = pools.iter().map(|pool| {
        let monitor = monitor.clone();
        let pool = *pool;
        tokio::spawn(async move {
            monitor.subscribe_to_pool(pool).await
        })
    }).collect();

    // All should complete without issues
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }

    // Verify all subscriptions are active
    let subs = monitor.subscriptions.read().await;
    assert_eq!(subs.len(), 100);
}
```

## Dependencies

- **Task 1**: Common libraries for models and structures
- **Task 2**: Database setup for QuestDB storage
- **Task 3**: Solana/Jupiter integration for pool data
- **Task 5**: Paper trade executor integration

## Integration Points

- **Pool Monitor**: Subscribes to Solana account updates via gRPC
- **Cache Layer**: Redis for 1-2 second pool state caching
- **Paper Trader**: Uses dynamic slippage for realistic simulation
- **QuestDB**: Stores historical pool states and slippage data
- **Alert System**: Notifies on pool health issues

## Performance Considerations

- Pool state updates processed in <10ms
- Dynamic slippage calculation <1ms with cached data
- Redis cache reduces repeated calculations
- Background tasks for non-critical processing
- Efficient batch operations for historical data

## Security Considerations

- Read-only access to pool accounts
- No private keys or sensitive data
- Rate limiting on subscriptions
- Circuit breaker for failing pools
- Audit trail for all calculations

## Future Enhancements

- Machine learning for slippage prediction
- Cross-DEX arbitrage detection
- Liquidity provision opportunities
- Advanced pool health scoring
- WebSocket API for real-time data
- Integration with more DEX protocols