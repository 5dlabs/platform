# Task 14: Implement Correlation Analysis for Paper vs Live Trading - Acceptance Criteria

## Functional Requirements

### 1. Trade Matching
- [ ] Matches paper trades with corresponding live trades
- [ ] Match confidence score calculated correctly
- [ ] Time proximity weighted appropriately (30%)
- [ ] Token pair matching enforced (30%)
- [ ] Trade size similarity calculated (20%)
- [ ] Direction matching verified (20%)
- [ ] Unmatched trades handled gracefully
- [ ] Match threshold configurable (default 80%)

### 2. Correlation Calculations
- [ ] Price correlation uses Pearson coefficient
- [ ] Slippage accuracy measures relative error
- [ ] MEV prediction uses F1 score
- [ ] Timing correlation accounts for latency
- [ ] Overall correlation weighted correctly:
  - [ ] Price: 40% weight
  - [ ] Slippage: 30% weight
  - [ ] MEV: 20% weight
  - [ ] Timing: 10% weight
- [ ] Confidence intervals calculated via bootstrap

### 3. Statistical Validity
- [ ] Minimum 30 trade pairs required
- [ ] Sample size reported in results
- [ ] Confidence interval at 95% level
- [ ] Outliers detected and handled
- [ ] Statistical significance tested
- [ ] Correlation bounds [0.0, 1.0]

### 4. Divergence Analysis
- [ ] High volatility periods identified (>5% swings)
- [ ] Low liquidity conditions detected
- [ ] Large trade impacts analyzed
- [ ] Network congestion patterns found
- [ ] Time-of-day effects identified
- [ ] Divergence threshold 15% triggers pattern

### 5. Report Generation
- [ ] Overall correlation score displayed
- [ ] Component breakdown provided
- [ ] Validation result clear:
  - [ ] PASSED if ≥85%
  - [ ] FAILED if <85%
  - [ ] INSUFFICIENT if <30 pairs
- [ ] Recommendations prioritized by impact
- [ ] Expected improvements quantified
- [ ] Actionable suggestions provided

### 6. Data Collection
- [ ] Fetches paper trades from QuestDB
- [ ] Fetches live trades from QuestDB
- [ ] Time range filtering works correctly
- [ ] Mode filtering (paper/live) accurate
- [ ] Handles missing data gracefully
- [ ] Caches recent analysis results

## Performance Requirements

### 1. Analysis Speed
- [ ] Completes within 5 seconds for 1000 pairs
- [ ] Trade matching scales linearly
- [ ] Correlation calculations optimized
- [ ] Database queries use indexes
- [ ] Parallel processing where applicable

### 2. Memory Usage
- [ ] Memory usage <500MB for 10,000 trades
- [ ] Old cache entries evicted
- [ ] No memory leaks over time
- [ ] Efficient data structures used

### 3. Accuracy
- [ ] Trade matching >90% accurate
- [ ] Correlation calculations match numpy/scipy
- [ ] Statistical tests validated
- [ ] No numerical instability

## Reliability Requirements

### 1. Error Handling
- [ ] Insufficient data returns clear error
- [ ] Database failures handled gracefully
- [ ] Invalid trades filtered out
- [ ] Calculation errors logged
- [ ] Partial results not returned

### 2. Data Quality
- [ ] Validates trade data integrity
- [ ] Handles time zone differences
- [ ] Detects duplicate trades
- [ ] Filters test trades
- [ ] Manages data gaps

### 3. Consistency
- [ ] Same data produces same results
- [ ] Correlation metrics reproducible
- [ ] Reports format consistent
- [ ] Recommendations deterministic

## Integration Requirements

### 1. Database Integration
- [ ] Queries QuestDB efficiently
- [ ] Uses proper time ranges
- [ ] Handles connection failures
- [ ] Respects query timeouts

### 2. Reporting Integration
- [ ] Reports exportable to JSON
- [ ] Console output well-formatted
- [ ] Integrates with monitoring
- [ ] Supports automated analysis

### 3. Configuration
- [ ] Correlation thresholds configurable
- [ ] Time windows adjustable
- [ ] Weights customizable
- [ ] Output formats selectable

## Test Scenarios

### 1. Basic Correlation Test
```rust
#[tokio::test]
async fn test_basic_correlation() {
    // Create 50 matched trade pairs
    // Paper prices: 100 ± 0.5%
    // Live prices: 100 ± 0.6%
    // Expected correlation: >0.95
}
```

### 2. Divergence Pattern Test
```rust
#[tokio::test]
async fn test_volatility_divergence() {
    // Create trades in high/low volatility
    // High volatility correlation: 0.75
    // Low volatility correlation: 0.90
    // Should detect pattern
}
```

### 3. MEV Prediction Test
```rust
#[tokio::test]
async fn test_mev_accuracy() {
    // Create trades with MEV impacts
    // True positives: 8
    // False positives: 2
    // False negatives: 1
    // Expected F1 score: ~0.84
}
```

### 4. Insufficient Data Test
```rust
#[tokio::test]
async fn test_insufficient_data() {
    // Provide only 20 trade pairs
    // Should return INSUFFICIENT
    // Clear error message
}
```

### 5. Report Generation Test
```rust
#[tokio::test]
async fn test_report_format() {
    // Generate full report
    // Verify all sections present
    // Check recommendation ordering
    // Validate percentages
}
```

### 6. Performance Benchmark
```rust
#[tokio::test]
async fn test_analysis_performance() {
    // Create 5000 trade pairs
    // Run full analysis
    // Verify <5 second completion
    // Check memory usage
}
```

## Edge Cases

### 1. Perfect Correlation
- [ ] Handles 100% correlation correctly
- [ ] Confidence interval narrow
- [ ] No false recommendations

### 2. Zero Correlation
- [ ] Handles 0% correlation appropriately
- [ ] Generates maximum recommendations
- [ ] Clear failure indication

### 3. Missing Prices
- [ ] Skips trades with null prices
- [ ] Adjusts sample size accordingly
- [ ] Warns about data quality

### 4. Time Gaps
- [ ] Handles market closures
- [ ] Manages maintenance windows
- [ ] Adjusts matching windows

### 5. Extreme Values
- [ ] Filters unrealistic prices
- [ ] Handles flash crashes
- [ ] Manages data errors

## Validation Scenarios

### 1. Strategy Approval
- [ ] 87% correlation → PASSED
- [ ] Clear success message
- [ ] Minor recommendations only

### 2. Strategy Rejection
- [ ] 75% correlation → FAILED
- [ ] Detailed failure analysis
- [ ] Prioritized improvements

### 3. Borderline Case
- [ ] 84.5% correlation → FAILED
- [ ] Explains near-miss
- [ ] Quick-win recommendations

## Output Requirements

### 1. Console Output
- [ ] Well-formatted ASCII report
- [ ] Color coding for pass/fail
- [ ] Progress indicators
- [ ] Clear section headers

### 2. JSON Export
- [ ] Complete metrics included
- [ ] Machine-readable format
- [ ] Timestamp included
- [ ] Version information

### 3. Metrics Export
- [ ] Prometheus format supported
- [ ] Time series data
- [ ] Correlation trends
- [ ] Alert thresholds

## Documentation Requirements

### 1. API Documentation
- [ ] All public methods documented
- [ ] Parameter descriptions clear
- [ ] Return types specified
- [ ] Examples provided

### 2. Statistical Methods
- [ ] Correlation formulas explained
- [ ] Weight justification provided
- [ ] Confidence interval method
- [ ] Bootstrap approach detailed

### 3. Interpretation Guide
- [ ] Correlation ranges explained
- [ ] Recommendation impact described
- [ ] Common patterns documented
- [ ] Troubleshooting section

## Acceptance Sign-off

The implementation is considered complete when:
1. All functional requirements pass
2. Statistical accuracy verified
3. Performance targets met
4. Reports provide actionable insights
5. 85% threshold properly enforced
6. Edge cases handled correctly
7. Documentation comprehensive

### Key Success Metrics
- Trade matching accuracy: >90%
- Analysis completion time: <5 seconds
- Statistical validity: p<0.05
- Report clarity: User-tested
- Recommendation accuracy: Validated