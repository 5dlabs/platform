# Task 14: Implement Correlation Analysis for Paper vs Live Trading - Autonomous Prompt

You are implementing a sophisticated correlation analysis system that compares paper trading results with live trading to validate simulation accuracy. The goal is to achieve 85-90% correlation between paper and live trading results as specified in the PRD.

## Context
- Paper trading must accurately simulate live trading to be useful for strategy validation
- The PRD requires 85-90% correlation for a strategy to be considered viable
- Small live trades are used as ground truth for comparison
- Analysis must identify specific areas where simulation diverges from reality

## Your Task

Implement a comprehensive correlation analysis system with the following components:

### 1. Trade Matching System
Create intelligent trade pairing between paper and live trades:

**Matching Criteria**:
- Time proximity (trades within same time window)
- Token pair identity (same base/quote tokens)
- Trade size similarity (within 20% of each other)
- Trade direction match (buy/sell/swap)

**Match Scoring**:
```rust
// Calculate match confidence score (0.0 to 1.0)
- Time weight: 30% (exponential decay per minute)
- Token match: 30% (exact match required)
- Size similarity: 20% (ratio-based scoring)
- Direction match: 20% (binary match)
```

### 2. Statistical Analysis
Implement comprehensive correlation metrics:

**Price Correlation** (40% weight):
- Pearson correlation coefficient of execution prices
- Compare actual execution prices between paper and live
- Account for market movements between executions

**Slippage Accuracy** (30% weight):
- How accurately paper trading predicts actual slippage
- Relative error tolerance: ±20%
- Consider both positive and negative slippage

**MEV Prediction Accuracy** (20% weight):
- F1 score for MEV impact prediction
- True positive: Correctly predicted MEV attack
- False positive: Predicted MEV but didn't occur
- False negative: Missed MEV attack

**Timing Correlation** (10% weight):
- Execution latency comparison
- Order processing time similarity
- Network delay simulation accuracy

### 3. Divergence Pattern Analysis
Identify conditions where simulation accuracy decreases:

**Market Conditions to Analyze**:
- High volatility periods (>5% price swings)
- Low liquidity situations (small pool sizes)
- Large trade sizes (>$10,000 equivalent)
- Network congestion (high gas prices)
- Time of day patterns (US vs Asia hours)

**Pattern Detection**:
```rust
// For each condition:
1. Group trades by condition
2. Calculate correlation within group
3. Compare to overall correlation
4. If divergence >15%, flag as pattern
5. Generate specific recommendations
```

### 4. Reporting System
Generate actionable reports with:

**Correlation Metrics**:
- Overall correlation score with confidence interval
- Breakdown by component (price, slippage, MEV, timing)
- Sample size and statistical significance
- Trend analysis over time

**Validation Result**:
- PASSED: ≥85% correlation
- FAILED: <85% correlation
- INSUFFICIENT: <30 trade pairs

**Recommendations**:
- Prioritized list of improvements
- Expected correlation gain for each
- Specific parameter adjustments
- Implementation complexity

### 5. Data Structures
```rust
pub struct CorrelationMetrics {
    price_correlation: f64,
    slippage_accuracy: f64,
    mev_prediction_accuracy: f64,
    timing_correlation: f64,
    overall_correlation: f64,
    sample_size: usize,
    confidence_interval: (f64, f64),
}

pub struct DivergencePattern {
    condition: MarketCondition,
    divergence_rate: f64,
    sample_count: usize,
    recommendations: Vec<String>,
}

pub struct CorrelationReport {
    metrics: CorrelationMetrics,
    divergence_patterns: Vec<DivergencePattern>,
    recommendations: Vec<Recommendation>,
    validation_result: ValidationResult,
}
```

## Technical Requirements

1. **Statistical Accuracy**: Use proper statistical methods (Pearson correlation, bootstrap confidence intervals)
2. **Performance**: Analysis should complete within 5 seconds for 1000 trade pairs
3. **Data Quality**: Handle missing data, outliers, and mismatched trades gracefully
4. **Caching**: Cache recent analysis results to avoid recomputation
5. **Visualization**: Support data export for external visualization tools

## Success Criteria

Your implementation will be considered complete when:
1. Trade matching achieves >90% accuracy
2. Correlation calculations are statistically sound
3. Divergence patterns are correctly identified
4. Reports provide actionable recommendations
5. 85% correlation threshold is properly evaluated
6. Analysis completes within performance targets
7. Edge cases are handled gracefully

## Example Usage

```rust
// Initialize analyzer
let analyzer = CorrelationAnalyzer::new(quest_db, postgres).await?;

// Run analysis for last 24 hours
let report = analyzer.generate_report(Duration::hours(24)).await?;

// Check validation result
match report.validation_result {
    ValidationResult::Passed { correlation } => {
        println!("✅ Strategy validated with {:.2}% correlation", correlation * 100.0);
    }
    ValidationResult::Failed { correlation, .. } => {
        println!("❌ Strategy failed with only {:.2}% correlation", correlation * 100.0);
        
        // Show top recommendations
        for rec in report.recommendations.iter().take(3) {
            println!("- {}: +{:.2}% expected improvement", 
                rec.description, 
                rec.expected_improvement * 100.0
            );
        }
    }
    _ => {}
}

// Analyze specific patterns
let patterns = analyzer.analyze_divergence_patterns(&trade_pairs).await?;
for pattern in patterns {
    println!("Pattern: {:?} causes {:.2}% divergence", 
        pattern.condition, 
        pattern.divergence_rate * 100.0
    );
}
```

## Important Considerations

1. **Sample Size**: Require minimum 30 trade pairs for statistical validity
2. **Time Windows**: Match trades within reasonable time windows (5 minutes)
3. **Outlier Handling**: Use robust statistics to handle extreme values
4. **Continuous Improvement**: Store historical correlations to track improvement
5. **Real-time Monitoring**: Support live correlation tracking during trading

## Expected Outputs

**Console Output Example**:
```
=== Correlation Analysis Report ===
Overall Correlation: 87.3% [85.1%, 89.5%]
✅ VALIDATION PASSED

Component Breakdown:
- Price Correlation: 92.1%
- Slippage Accuracy: 84.5%
- MEV Prediction: 78.9%
- Timing Correlation: 89.2%

Top Recommendations:
1. Improve MEV sandwich attack detection threshold
   Expected improvement: +2.8%
2. Adjust slippage model for high volatility periods
   Expected improvement: +1.9%
```