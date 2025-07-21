# Task 14: Implement Correlation Analysis for Paper vs Live Trading

## Overview
This task implements sophisticated analysis tools to compare paper trading results with small live trades, validating simulation accuracy. The system calculates correlation metrics, identifies divergence patterns, and provides recommendations for improving the paper trading model to achieve the target 85-90% correlation specified in the PRD.

## Architecture Context
According to the architecture.md and PRD, correlation analysis is critical for:
- Validating paper trading accuracy before scaling strategies
- Identifying simulation weaknesses and model improvements
- Achieving 85-90% correlation target for strategy viability
- Providing data-driven recommendations for model refinement

## Implementation Requirements

### 1. Data Collection System

Implement parallel tracking of paper and live trades:

```rust
use statistical::{mean, standard_deviation, correlation};

pub struct CorrelationAnalyzer {
    quest_db: Arc<QuestDbClient>,
    postgres: Arc<PostgresClient>,
    analysis_cache: Arc<Mutex<AnalysisCache>>,
}

pub struct TradePair {
    paper_trade: Trade,
    live_trade: Trade,
    matched_at: DateTime<Utc>,
    match_confidence: f64,
}

pub struct AnalysisCache {
    recent_pairs: VecDeque<TradePair>,
    correlation_history: Vec<CorrelationMetrics>,
    divergence_patterns: HashMap<String, DivergencePattern>,
}

impl CorrelationAnalyzer {
    pub async fn collect_trade_pairs(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<TradePair>> {
        // Fetch paper trades
        let paper_trades = self.quest_db.get_trades(TradeFilter {
            start_time: Some(start_time),
            end_time: Some(end_time),
            mode: Some("paper".to_string()),
            ..Default::default()
        }).await?;
        
        // Fetch live trades
        let live_trades = self.quest_db.get_trades(TradeFilter {
            start_time: Some(start_time),
            end_time: Some(end_time),
            mode: Some("live".to_string()),
            ..Default::default()
        }).await?;
        
        // Match trades with intelligent pairing
        let matched_pairs = self.match_trades_intelligently(paper_trades, live_trades)?;
        
        // Store in cache for quick access
        let mut cache = self.analysis_cache.lock().await;
        cache.recent_pairs.extend(matched_pairs.clone());
        
        // Limit cache size
        while cache.recent_pairs.len() > 10000 {
            cache.recent_pairs.pop_front();
        }
        
        Ok(matched_pairs)
    }
    
    fn match_trades_intelligently(
        &self,
        paper_trades: Vec<Trade>,
        live_trades: Vec<Trade>,
    ) -> Result<Vec<TradePair>> {
        let mut pairs = Vec::new();
        let mut used_live_trades = HashSet::new();
        
        for paper_trade in paper_trades {
            // Find best matching live trade
            let best_match = live_trades.iter()
                .enumerate()
                .filter(|(idx, _)| !used_live_trades.contains(idx))
                .filter_map(|(idx, live_trade)| {
                    let score = self.calculate_match_score(&paper_trade, live_trade);
                    if score > 0.8 { // 80% confidence threshold
                        Some((idx, live_trade, score))
                    } else {
                        None
                    }
                })
                .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
            
            if let Some((idx, live_trade, confidence)) = best_match {
                used_live_trades.insert(idx);
                pairs.push(TradePair {
                    paper_trade: paper_trade.clone(),
                    live_trade: live_trade.clone(),
                    matched_at: Utc::now(),
                    match_confidence: confidence,
                });
            }
        }
        
        Ok(pairs)
    }
    
    fn calculate_match_score(&self, paper: &Trade, live: &Trade) -> f64 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;
        
        // Time proximity (weight: 0.3)
        let time_diff = (paper.timestamp - live.timestamp).num_seconds().abs() as f64;
        let time_score = (-time_diff / 60.0).exp(); // Exponential decay per minute
        score += time_score * 0.3;
        weight_sum += 0.3;
        
        // Token pair match (weight: 0.3)
        if paper.base_token == live.base_token && paper.quote_token == live.quote_token {
            score += 0.3;
        }
        weight_sum += 0.3;
        
        // Trade size similarity (weight: 0.2)
        let size_ratio = (paper.base_amount / live.base_amount).min(live.base_amount / paper.base_amount);
        score += size_ratio * 0.2;
        weight_sum += 0.2;
        
        // Trade direction match (weight: 0.2)
        if paper.action == live.action {
            score += 0.2;
        }
        weight_sum += 0.2;
        
        score / weight_sum
    }
}
```

### 2. Statistical Analysis Tools

Implement comprehensive correlation calculations:

```rust
pub struct CorrelationMetrics {
    price_correlation: f64,         // Pearson correlation of execution prices
    slippage_accuracy: f64,         // How well we predict slippage
    mev_prediction_accuracy: f64,   // MEV simulation accuracy
    timing_correlation: f64,        // Execution timing similarity
    overall_correlation: f64,       // Weighted average (target: 0.85-0.90)
    sample_size: usize,
    confidence_interval: (f64, f64),
}

impl CorrelationAnalyzer {
    pub async fn calculate_correlation_metrics(
        &self,
        pairs: &[TradePair],
    ) -> Result<CorrelationMetrics> {
        if pairs.len() < 30 {
            return Err(Error::InsufficientData(
                "Need at least 30 trade pairs for reliable correlation analysis".into()
            ));
        }
        
        // Extract data vectors
        let (paper_prices, live_prices): (Vec<f64>, Vec<f64>) = pairs.iter()
            .map(|p| (p.paper_trade.executed_price, p.live_trade.executed_price))
            .unzip();
        
        let (paper_slippage, live_slippage): (Vec<f64>, Vec<f64>) = pairs.iter()
            .map(|p| (p.paper_trade.actual_slippage, p.live_trade.actual_slippage))
            .unzip();
        
        // Calculate correlations
        let price_correlation = pearson_correlation(&paper_prices, &live_prices)?;
        let slippage_accuracy = self.calculate_slippage_accuracy(pairs)?;
        let mev_prediction_accuracy = self.calculate_mev_accuracy(pairs)?;
        let timing_correlation = self.calculate_timing_correlation(pairs)?;
        
        // Calculate overall correlation with PRD-specified weights
        let overall_correlation = self.calculate_weighted_correlation(
            price_correlation,
            slippage_accuracy,
            mev_prediction_accuracy,
            timing_correlation,
        );
        
        // Calculate confidence interval using bootstrap
        let confidence_interval = self.bootstrap_confidence_interval(pairs, 0.95)?;
        
        Ok(CorrelationMetrics {
            price_correlation,
            slippage_accuracy,
            mev_prediction_accuracy,
            timing_correlation,
            overall_correlation,
            sample_size: pairs.len(),
            confidence_interval,
        })
    }
    
    fn calculate_slippage_accuracy(&self, pairs: &[TradePair]) -> Result<f64> {
        let mut accurate_predictions = 0;
        let mut total_error = 0.0;
        
        for pair in pairs {
            let paper_slippage = pair.paper_trade.actual_slippage;
            let live_slippage = pair.live_trade.actual_slippage;
            let error = (paper_slippage - live_slippage).abs();
            
            // Consider prediction accurate if within 20% relative error
            if paper_slippage != 0.0 {
                let relative_error = error / paper_slippage.abs();
                if relative_error <= 0.2 {
                    accurate_predictions += 1;
                }
            }
            
            total_error += error;
        }
        
        // Combine accuracy rate and mean absolute error
        let accuracy_rate = accurate_predictions as f64 / pairs.len() as f64;
        let mean_error = total_error / pairs.len() as f64;
        let error_score = (-mean_error * 100.0).exp(); // Exponential penalty for error
        
        Ok((accuracy_rate + error_score) / 2.0)
    }
    
    fn calculate_mev_accuracy(&self, pairs: &[TradePair]) -> Result<f64> {
        let mut true_positives = 0;  // Correctly predicted MEV
        let mut true_negatives = 0;  // Correctly predicted no MEV
        let mut false_positives = 0; // Predicted MEV but didn't happen
        let mut false_negatives = 0; // Didn't predict MEV but it happened
        
        for pair in pairs {
            let paper_mev = pair.paper_trade.mev_status == "impacted";
            let live_mev = pair.live_trade.mev_status == "impacted";
            
            match (paper_mev, live_mev) {
                (true, true) => true_positives += 1,
                (false, false) => true_negatives += 1,
                (true, false) => false_positives += 1,
                (false, true) => false_negatives += 1,
            }
        }
        
        // Calculate F1 score for balanced accuracy
        let precision = if true_positives + false_positives > 0 {
            true_positives as f64 / (true_positives + false_positives) as f64
        } else {
            0.0
        };
        
        let recall = if true_positives + false_negatives > 0 {
            true_positives as f64 / (true_positives + false_negatives) as f64
        } else {
            0.0
        };
        
        if precision + recall > 0.0 {
            Ok(2.0 * (precision * recall) / (precision + recall))
        } else {
            Ok(0.0)
        }
    }
    
    fn calculate_weighted_correlation(
        &self,
        price_corr: f64,
        slippage_acc: f64,
        mev_acc: f64,
        timing_corr: f64,
    ) -> f64 {
        // Weights based on PRD priorities
        let weights = [
            (price_corr, 0.4),      // Price accuracy most important
            (slippage_acc, 0.3),    // Slippage prediction critical
            (mev_acc, 0.2),         // MEV simulation important
            (timing_corr, 0.1),     // Timing less critical
        ];
        
        let weighted_sum: f64 = weights.iter()
            .map(|(value, weight)| value * weight)
            .sum();
        
        weighted_sum.max(0.0).min(1.0)
    }
}
```

### 3. Divergence Pattern Analysis

Identify conditions where simulation accuracy decreases:

```rust
pub struct DivergencePattern {
    condition: MarketCondition,
    divergence_rate: f64,
    sample_count: usize,
    recommendations: Vec<String>,
}

pub enum MarketCondition {
    HighVolatility { threshold: f64 },
    LowLiquidity { pool_size: f64 },
    LargeTradeSize { percentile: f64 },
    NetworkCongestion { avg_priority_fee: u64 },
    TimeOfDay { hour: u8 },
}

impl CorrelationAnalyzer {
    pub async fn analyze_divergence_patterns(
        &self,
        pairs: &[TradePair],
    ) -> Result<Vec<DivergencePattern>> {
        let mut patterns = Vec::new();
        
        // Analyze by volatility
        let volatility_pattern = self.analyze_volatility_impact(pairs).await?;
        if volatility_pattern.divergence_rate > 0.15 {
            patterns.push(volatility_pattern);
        }
        
        // Analyze by trade size
        let size_pattern = self.analyze_trade_size_impact(pairs).await?;
        if size_pattern.divergence_rate > 0.15 {
            patterns.push(size_pattern);
        }
        
        // Analyze by time of day
        let time_pattern = self.analyze_time_of_day_impact(pairs).await?;
        if time_pattern.divergence_rate > 0.15 {
            patterns.push(time_pattern);
        }
        
        // Analyze by network congestion
        let congestion_pattern = self.analyze_congestion_impact(pairs).await?;
        if congestion_pattern.divergence_rate > 0.15 {
            patterns.push(congestion_pattern);
        }
        
        Ok(patterns)
    }
    
    async fn analyze_volatility_impact(&self, pairs: &[TradePair]) -> Result<DivergencePattern> {
        // Group trades by volatility level
        let mut high_volatility_pairs = Vec::new();
        let mut low_volatility_pairs = Vec::new();
        
        for pair in pairs {
            let volatility = self.calculate_price_volatility(&pair.paper_trade).await?;
            if volatility > 0.05 { // 5% threshold
                high_volatility_pairs.push(pair);
            } else {
                low_volatility_pairs.push(pair);
            }
        }
        
        let high_vol_corr = self.calculate_correlation_metrics(&high_volatility_pairs).await?;
        let low_vol_corr = self.calculate_correlation_metrics(&low_volatility_pairs).await?;
        
        let divergence_rate = (low_vol_corr.overall_correlation - high_vol_corr.overall_correlation).abs();
        
        let mut recommendations = Vec::new();
        if high_vol_corr.overall_correlation < 0.85 {
            recommendations.push(
                "Increase slippage model sensitivity during high volatility periods".to_string()
            );
            recommendations.push(
                "Consider dynamic MEV risk adjustment based on volatility".to_string()
            );
        }
        
        Ok(DivergencePattern {
            condition: MarketCondition::HighVolatility { threshold: 0.05 },
            divergence_rate,
            sample_count: high_volatility_pairs.len(),
            recommendations,
        })
    }
}
```

### 4. Reporting and Recommendations

Generate comprehensive analysis reports:

```rust
pub struct CorrelationReport {
    metrics: CorrelationMetrics,
    divergence_patterns: Vec<DivergencePattern>,
    recommendations: Vec<Recommendation>,
    validation_result: ValidationResult,
}

pub struct Recommendation {
    priority: Priority,
    category: RecommendationCategory,
    description: String,
    expected_improvement: f64,
}

pub enum RecommendationCategory {
    SlippageModel,
    MevSimulation,
    PriceOracle,
    ExecutionTiming,
    MarketDepth,
}

pub enum ValidationResult {
    Passed { correlation: f64 },
    Failed { correlation: f64, minimum_required: f64 },
    Insufficient { reason: String },
}

impl CorrelationAnalyzer {
    pub async fn generate_report(
        &self,
        duration: Duration,
    ) -> Result<CorrelationReport> {
        let end_time = Utc::now();
        let start_time = end_time - duration;
        
        // Collect and analyze data
        let pairs = self.collect_trade_pairs(start_time, end_time).await?;
        let metrics = self.calculate_correlation_metrics(&pairs).await?;
        let patterns = self.analyze_divergence_patterns(&pairs).await?;
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&metrics, &patterns)?;
        
        // Determine validation result (85% threshold from PRD)
        let validation_result = if metrics.sample_size < 30 {
            ValidationResult::Insufficient {
                reason: format!("Only {} trade pairs available, need at least 30", metrics.sample_size)
            }
        } else if metrics.overall_correlation >= 0.85 {
            ValidationResult::Passed {
                correlation: metrics.overall_correlation,
            }
        } else {
            ValidationResult::Failed {
                correlation: metrics.overall_correlation,
                minimum_required: 0.85,
            }
        };
        
        Ok(CorrelationReport {
            metrics,
            divergence_patterns: patterns,
            recommendations,
            validation_result,
        })
    }
    
    fn generate_recommendations(
        &self,
        metrics: &CorrelationMetrics,
        patterns: &[DivergencePattern],
    ) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();
        
        // Price correlation recommendations
        if metrics.price_correlation < 0.9 {
            recommendations.push(Recommendation {
                priority: Priority::High,
                category: RecommendationCategory::PriceOracle,
                description: "Improve price feed latency to reduce execution price divergence".to_string(),
                expected_improvement: (0.9 - metrics.price_correlation) * 0.4, // 40% weight
            });
        }
        
        // Slippage model recommendations
        if metrics.slippage_accuracy < 0.8 {
            recommendations.push(Recommendation {
                priority: Priority::High,
                category: RecommendationCategory::SlippageModel,
                description: "Refine slippage model with recent trade data, consider pool depth impact".to_string(),
                expected_improvement: (0.8 - metrics.slippage_accuracy) * 0.3, // 30% weight
            });
        }
        
        // MEV simulation recommendations
        if metrics.mev_prediction_accuracy < 0.7 {
            recommendations.push(Recommendation {
                priority: Priority::Medium,
                category: RecommendationCategory::MevSimulation,
                description: "Update MEV risk parameters based on observed sandwich attack patterns".to_string(),
                expected_improvement: (0.7 - metrics.mev_prediction_accuracy) * 0.2, // 20% weight
            });
        }
        
        // Pattern-based recommendations
        for pattern in patterns {
            recommendations.extend(pattern.recommendations.iter().map(|rec| {
                Recommendation {
                    priority: if pattern.divergence_rate > 0.25 { Priority::High } else { Priority::Medium },
                    category: RecommendationCategory::MarketDepth,
                    description: rec.clone(),
                    expected_improvement: pattern.divergence_rate * 0.1,
                }
            }));
        }
        
        // Sort by expected improvement
        recommendations.sort_by(|a, b| {
            b.expected_improvement.partial_cmp(&a.expected_improvement).unwrap()
        });
        
        Ok(recommendations)
    }
    
    pub fn format_report(&self, report: &CorrelationReport) -> String {
        let mut output = String::new();
        
        output.push_str("=== Solana Paper Trading Correlation Analysis Report ===\n\n");
        
        // Summary
        output.push_str(&format!("Analysis Period: Last {} trades\n", report.metrics.sample_size));
        output.push_str(&format!("Overall Correlation: {:.2}%\n", report.metrics.overall_correlation * 100.0));
        output.push_str(&format!("Confidence Interval: [{:.2}%, {:.2}%]\n\n", 
            report.metrics.confidence_interval.0 * 100.0,
            report.metrics.confidence_interval.1 * 100.0
        ));
        
        // Validation Result
        match &report.validation_result {
            ValidationResult::Passed { correlation } => {
                output.push_str(&format!("✅ VALIDATION PASSED: Correlation {:.2}% exceeds 85% threshold\n\n", correlation * 100.0));
            }
            ValidationResult::Failed { correlation, minimum_required } => {
                output.push_str(&format!("❌ VALIDATION FAILED: Correlation {:.2}% below {:.0}% threshold\n\n", 
                    correlation * 100.0, minimum_required * 100.0));
            }
            ValidationResult::Insufficient { reason } => {
                output.push_str(&format!("⚠️  INSUFFICIENT DATA: {}\n\n", reason));
            }
        }
        
        // Detailed Metrics
        output.push_str("Detailed Correlation Metrics:\n");
        output.push_str(&format!("  • Price Correlation: {:.2}%\n", report.metrics.price_correlation * 100.0));
        output.push_str(&format!("  • Slippage Accuracy: {:.2}%\n", report.metrics.slippage_accuracy * 100.0));
        output.push_str(&format!("  • MEV Prediction: {:.2}%\n", report.metrics.mev_prediction_accuracy * 100.0));
        output.push_str(&format!("  • Timing Correlation: {:.2}%\n\n", report.metrics.timing_correlation * 100.0));
        
        // Divergence Patterns
        if !report.divergence_patterns.is_empty() {
            output.push_str("Identified Divergence Patterns:\n");
            for pattern in &report.divergence_patterns {
                output.push_str(&format!("  • {:?}: {:.2}% divergence ({} samples)\n",
                    pattern.condition,
                    pattern.divergence_rate * 100.0,
                    pattern.sample_count
                ));
            }
            output.push_str("\n");
        }
        
        // Recommendations
        output.push_str("Recommendations for Improvement:\n");
        for (i, rec) in report.recommendations.iter().take(5).enumerate() {
            output.push_str(&format!("{}. [{:?}] {}\n   Expected improvement: +{:.2}%\n",
                i + 1,
                rec.priority,
                rec.description,
                rec.expected_improvement * 100.0
            ));
        }
        
        output
    }
}
```

## Error Handling

```rust
pub enum CorrelationError {
    InsufficientData(String),
    MatchingFailed(String),
    CalculationError(String),
    DatabaseError(String),
}

impl CorrelationAnalyzer {
    async fn handle_analysis_error(&self, error: CorrelationError) -> Result<()> {
        match error {
            CorrelationError::InsufficientData(msg) => {
                warn!("Insufficient data for correlation analysis: {}", msg);
                // Schedule retry with longer time window
            }
            CorrelationError::MatchingFailed(msg) => {
                error!("Trade matching failed: {}", msg);
                // Alert user to check trade execution
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Testing Strategy

1. **Unit Tests**: Test correlation calculations with known datasets
2. **Integration Tests**: Verify database queries and trade matching
3. **Statistical Tests**: Validate correlation algorithms against standard libraries
4. **Performance Tests**: Ensure analysis completes within reasonable time
5. **Edge Case Tests**: Handle insufficient data, extreme correlations

## Dependencies
- Task 9: Paper Trade Executor (source of paper trades)
- Task 12: QuestDB Integration (trade data storage)