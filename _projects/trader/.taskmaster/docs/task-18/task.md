# Task 18: Implement Risk Management System with Pre-trade Validation

## Overview

This task creates a focused risk management system that validates technical execution aspects of trades before execution. The system ensures safe transaction processing for pre-approved trade requests from external services by checking execution parameters, system health, and technical constraints.

## Architecture Context

The Risk Management System serves as a critical safety layer in the trading platform:

- **Technical Focus**: Validates execution parameters rather than portfolio risk (handled externally)
- **Real-time Validation**: Performs checks immediately before trade execution
- **Circuit Breaker Integration**: Connects to system health monitoring
- **Configurable Thresholds**: Allows dynamic adjustment of risk parameters
- **Audit Trail**: Logs all risk decisions for compliance

As specified in the architecture, this component assumes trade requests have already passed business logic validation and focuses purely on technical execution safety.

## Implementation Details

### 1. Core Risk Management Components

#### Risk Manager Structure
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RiskManager {
    config: Arc<RwLock<RiskConfig>>,
    execution_validator: Arc<ExecutionValidator>,
    system_health_checker: Arc<SystemHealthChecker>,
    circuit_breaker: Arc<CircuitBreaker>,
    metrics_collector: Arc<MetricsCollector>,
    audit_logger: Arc<AuditLogger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    // Execution limits
    pub max_slippage_bps: u16,           // Default: 500 (5%)
    pub min_confirmation_time_ms: u64,    // Default: 100ms
    pub max_confirmation_time_ms: u64,    // Default: 30000ms (30s)
    
    // System health thresholds
    pub max_node_latency_ms: u64,        // Default: 200ms (from PRD)
    pub min_node_health_score: f64,      // Default: 0.8
    pub max_error_rate: f64,             // Default: 0.05 (5%)
    
    // MEV protection
    pub mev_protection_required: bool,    // Default: true
    pub min_priority_fee: u64,           // Default: 1000 lamports
    pub max_priority_fee: u64,           // Default: 10000 lamports
    
    // Circuit breaker triggers
    pub max_consecutive_failures: u32,    // Default: 5
    pub failure_window_secs: u64,        // Default: 60
    pub recovery_timeout_secs: u64,      // Default: 300
}

impl RiskManager {
    pub async fn new(config: RiskConfig) -> Result<Self, RiskError> {
        let config = Arc::new(RwLock::new(config));
        
        Ok(Self {
            config: config.clone(),
            execution_validator: Arc::new(ExecutionValidator::new(config.clone())),
            system_health_checker: Arc::new(SystemHealthChecker::new()),
            circuit_breaker: Arc::new(CircuitBreaker::new(config.clone())),
            metrics_collector: Arc::new(MetricsCollector::new()),
            audit_logger: Arc::new(AuditLogger::new()),
        })
    }

    pub async fn validate_trade_execution(
        &self,
        trade_request: &TradeRequest,
    ) -> Result<ValidationResult, RiskError> {
        let start = Instant::now();
        
        // Create validation context
        let context = ValidationContext {
            request_id: trade_request.request_id.clone(),
            timestamp: Utc::now(),
            source: trade_request.source_service.clone(),
        };

        // Run all validation checks
        let validations = vec![
            self.check_circuit_breaker_status(&context).await,
            self.validate_execution_parameters(trade_request, &context).await,
            self.check_system_health(&context).await,
            self.validate_mev_protection(trade_request, &context).await,
        ];

        // Aggregate results
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        
        for result in validations {
            match result {
                Ok(Some(warning)) => warnings.push(warning),
                Ok(None) => {},
                Err(e) => errors.push(e),
            }
        }

        // Record validation metrics
        self.metrics_collector.record_validation_time(
            start.elapsed().as_micros() as u64
        );

        // Log validation result
        self.audit_logger.log_validation_result(
            &context,
            &warnings,
            &errors,
        ).await;

        if !errors.is_empty() {
            return Err(RiskError::ValidationFailed { errors });
        }

        Ok(ValidationResult {
            approved: true,
            warnings,
            validation_time_us: start.elapsed().as_micros() as u64,
            risk_score: self.calculate_risk_score(trade_request).await?,
        })
    }

    async fn calculate_risk_score(&self, request: &TradeRequest) -> Result<f64, RiskError> {
        let config = self.config.read().await;
        
        // Simple risk scoring based on execution parameters
        let mut score = 1.0;
        
        // Slippage risk
        let slippage_ratio = request.max_slippage_bps as f64 / config.max_slippage_bps as f64;
        score -= slippage_ratio * 0.3;
        
        // Urgency risk
        match request.urgency_level() {
            UrgencyLevel::Critical => score -= 0.2,
            UrgencyLevel::High => score -= 0.1,
            _ => {},
        }
        
        // System health impact
        let health_score = self.system_health_checker.get_health_score().await?;
        score *= health_score;
        
        Ok(score.max(0.0).min(1.0))
    }
}
```

### 2. Execution Parameter Validation

```rust
pub struct ExecutionValidator {
    config: Arc<RwLock<RiskConfig>>,
    metrics: Arc<MetricsCollector>,
}

impl ExecutionValidator {
    pub async fn validate_parameters(
        &self,
        request: &TradeRequest,
        context: &ValidationContext,
    ) -> Result<Option<String>, RiskError> {
        let config = self.config.read().await;
        let mut warnings = Vec::new();

        // Validate slippage tolerance
        if request.max_slippage_bps > config.max_slippage_bps {
            return Err(RiskError::SlippageExceedsLimit {
                requested: request.max_slippage_bps,
                limit: config.max_slippage_bps,
            });
        }

        if request.max_slippage_bps > config.max_slippage_bps * 80 / 100 {
            warnings.push(format!(
                "Slippage tolerance {}bps is close to limit {}bps",
                request.max_slippage_bps,
                config.max_slippage_bps
            ));
        }

        // Validate MEV protection
        if config.mev_protection_required && !request.mev_protection_enabled {
            return Err(RiskError::MevProtectionRequired);
        }

        // Check priority fee range
        if let Some(priority_fee) = request.suggested_priority_fee {
            if priority_fee < config.min_priority_fee {
                warnings.push(format!(
                    "Priority fee {} below recommended minimum {}",
                    priority_fee,
                    config.min_priority_fee
                ));
            }
            if priority_fee > config.max_priority_fee {
                return Err(RiskError::PriorityFeeTooHigh {
                    requested: priority_fee,
                    limit: config.max_priority_fee,
                });
            }
        }

        // Validate execution timeouts
        if let Some(timeout) = request.execution_timeout_ms {
            if timeout < config.min_confirmation_time_ms {
                return Err(RiskError::TimeoutTooShort {
                    requested: timeout,
                    minimum: config.min_confirmation_time_ms,
                });
            }
            if timeout > config.max_confirmation_time_ms {
                warnings.push(format!(
                    "Execution timeout {}ms exceeds recommended maximum",
                    timeout
                ));
            }
        }

        Ok(if warnings.is_empty() {
            None
        } else {
            Some(warnings.join("; "))
        })
    }
}
```

### 3. System Health Monitoring

```rust
pub struct SystemHealthChecker {
    node_monitor: Arc<NodeHealthMonitor>,
    service_monitors: Arc<RwLock<HashMap<String, ServiceMonitor>>>,
    metrics_store: Arc<dyn MetricsStore>,
}

impl SystemHealthChecker {
    pub async fn check_system_health(
        &self,
        context: &ValidationContext,
    ) -> Result<SystemHealthStatus, RiskError> {
        let mut health_status = SystemHealthStatus {
            overall_health: HealthLevel::Healthy,
            node_health: self.check_node_health().await?,
            service_health: self.check_service_health().await?,
            recent_error_rate: self.calculate_error_rate().await?,
            warnings: Vec::new(),
        };

        // Check node latency (PRD requirement: 200ms P99)
        let node_latency = self.node_monitor.get_p99_latency().await?;
        if node_latency > 200 {
            health_status.overall_health = HealthLevel::Degraded;
            health_status.warnings.push(format!(
                "Node P99 latency {}ms exceeds 200ms threshold",
                node_latency
            ));
        }

        // Check error rates
        if health_status.recent_error_rate > 0.05 {
            health_status.overall_health = HealthLevel::Unhealthy;
            return Err(RiskError::SystemUnhealthy {
                reason: format!("Error rate {:.1}% exceeds threshold", 
                    health_status.recent_error_rate * 100.0),
            });
        }

        Ok(health_status)
    }

    async fn check_node_health(&self) -> Result<NodeHealthStatus, RiskError> {
        let metrics = self.node_monitor.get_current_metrics().await?;
        
        Ok(NodeHealthStatus {
            latency_ms: metrics.p99_latency_ms,
            error_rate: metrics.error_rate,
            last_block_time: metrics.last_block_time,
            connection_status: metrics.connection_status,
            health_score: self.calculate_node_health_score(&metrics),
        })
    }

    async fn check_service_health(&self) -> Result<ServiceHealthStatus, RiskError> {
        let monitors = self.service_monitors.read().await;
        let mut service_statuses = HashMap::new();

        for (name, monitor) in monitors.iter() {
            let status = monitor.get_status().await?;
            service_statuses.insert(name.clone(), status);
        }

        // Check critical services
        let jupiter_health = service_statuses.get("jupiter")
            .ok_or(RiskError::ServiceNotMonitored("jupiter".to_string()))?;
        
        if !jupiter_health.is_healthy {
            return Err(RiskError::CriticalServiceDown("jupiter".to_string()));
        }

        Ok(ServiceHealthStatus {
            services: service_statuses,
            all_healthy: service_statuses.values().all(|s| s.is_healthy),
        })
    }

    async fn calculate_error_rate(&self) -> Result<f64, RiskError> {
        let window = Duration::minutes(5);
        let metrics = self.metrics_store
            .get_error_metrics(window)
            .await?;

        let total_requests = metrics.total_requests as f64;
        if total_requests == 0.0 {
            return Ok(0.0);
        }

        Ok(metrics.failed_requests as f64 / total_requests)
    }

    fn calculate_node_health_score(&self, metrics: &NodeMetrics) -> f64 {
        let mut score = 1.0;

        // Latency impact (200ms target)
        if metrics.p99_latency_ms > 200 {
            let excess = (metrics.p99_latency_ms - 200) as f64;
            score -= (excess / 1000.0).min(0.5); // Max 50% penalty
        }

        // Error rate impact
        score -= metrics.error_rate.min(0.3); // Max 30% penalty

        // Connection status impact
        if !metrics.connection_status.is_connected {
            score *= 0.5;
        }

        score.max(0.0)
    }
}
```

### 4. Circuit Breaker Integration

```rust
pub struct CircuitBreakerManager {
    breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    config: Arc<RwLock<RiskConfig>>,
    event_bus: Arc<EventBus>,
}

impl CircuitBreakerManager {
    pub async fn check_circuit_breakers(&self) -> Result<CircuitStatus, RiskError> {
        let breakers = self.breakers.read().await;
        let mut status = CircuitStatus {
            all_open: true,
            closed_breakers: Vec::new(),
            warnings: Vec::new(),
        };

        for (name, breaker) in breakers.iter() {
            let breaker_state = breaker.get_state().await;
            
            match breaker_state {
                CircuitState::Closed => {
                    status.all_open = false;
                    status.closed_breakers.push(name.clone());
                }
                CircuitState::HalfOpen => {
                    status.warnings.push(format!(
                        "Circuit breaker '{}' is in half-open state",
                        name
                    ));
                }
                CircuitState::Open => {}
            }
        }

        if !status.all_open {
            return Err(RiskError::CircuitBreakerClosed {
                breakers: status.closed_breakers,
            });
        }

        Ok(status)
    }

    pub async fn register_technical_failure(&self, failure_type: TechnicalFailure) {
        let breakers = self.breakers.read().await;
        
        match failure_type {
            TechnicalFailure::NodeTimeout => {
                if let Some(breaker) = breakers.get("node_health") {
                    breaker.record_failure().await;
                }
            }
            TechnicalFailure::JupiterError => {
                if let Some(breaker) = breakers.get("jupiter") {
                    breaker.record_failure().await;
                }
            }
            TechnicalFailure::TransactionFailure => {
                if let Some(breaker) = breakers.get("transaction") {
                    breaker.record_failure().await;
                }
            }
        }

        // Emit event for monitoring
        self.event_bus.emit(Event::TechnicalFailure(failure_type)).await;
    }

    pub async fn create_circuit_breaker(&self, name: &str) -> Arc<CircuitBreaker> {
        let config = self.config.read().await;
        
        let breaker = Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: config.max_consecutive_failures,
            failure_window: Duration::seconds(config.failure_window_secs as i64),
            recovery_timeout: Duration::seconds(config.recovery_timeout_secs as i64),
            half_open_requests: 3,
        }));

        self.breakers.write().await.insert(name.to_string(), breaker.clone());
        breaker
    }
}
```

### 5. Risk Metrics and Monitoring

```rust
pub struct RiskMetricsCollector {
    metrics_store: Arc<dyn MetricsStore>,
    prometheus_registry: Arc<Registry>,
    validation_histogram: Histogram,
    risk_score_gauge: Gauge,
    violation_counter: Counter,
}

impl RiskMetricsCollector {
    pub async fn record_validation(&self, result: &ValidationResult) {
        // Record validation time
        self.validation_histogram
            .observe(result.validation_time_us as f64 / 1000.0); // Convert to ms

        // Update risk score gauge
        self.risk_score_gauge.set(result.risk_score);

        // Store in time-series database
        let metric = ValidationMetric {
            timestamp: Utc::now(),
            validation_time_us: result.validation_time_us,
            risk_score: result.risk_score,
            warnings_count: result.warnings.len() as u32,
            approved: result.approved,
        };

        self.metrics_store.record_validation_metric(metric).await.ok();
    }

    pub async fn record_violation(&self, violation_type: &str, details: &str) {
        self.violation_counter
            .with_label_values(&[violation_type])
            .inc();

        let violation = RiskViolation {
            timestamp: Utc::now(),
            violation_type: violation_type.to_string(),
            details: details.to_string(),
            severity: self.classify_severity(violation_type),
        };

        self.metrics_store.record_violation(violation).await.ok();
    }

    pub async fn get_risk_dashboard_data(&self) -> Result<RiskDashboard, RiskError> {
        let window = Duration::hours(1);
        
        Ok(RiskDashboard {
            current_risk_score: self.risk_score_gauge.get(),
            validations_per_minute: self.calculate_validation_rate().await?,
            recent_violations: self.get_recent_violations(window).await?,
            system_health: self.get_system_health_summary().await?,
            circuit_breaker_status: self.get_circuit_breaker_summary().await?,
        })
    }

    async fn calculate_validation_rate(&self) -> Result<f64, RiskError> {
        let window = Duration::minutes(1);
        let count = self.metrics_store
            .count_validations(window)
            .await?;
        
        Ok(count as f64)
    }

    fn classify_severity(&self, violation_type: &str) -> Severity {
        match violation_type {
            "circuit_breaker_closed" | "system_unhealthy" => Severity::Critical,
            "slippage_exceeded" | "node_timeout" => Severity::High,
            "mev_protection_disabled" => Severity::Medium,
            _ => Severity::Low,
        }
    }
}
```

### 6. Risk Override Capabilities

```rust
pub struct RiskOverrideManager {
    overrides: Arc<RwLock<HashMap<String, Override>>>,
    audit_logger: Arc<AuditLogger>,
    permissions: Arc<PermissionManager>,
}

#[derive(Clone, Debug)]
pub struct Override {
    id: String,
    override_type: OverrideType,
    created_by: String,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    reason: String,
    conditions: OverrideConditions,
}

#[derive(Clone, Debug)]
pub enum OverrideType {
    SlippageLimit { new_limit_bps: u16 },
    CircuitBreaker { breaker_name: String },
    SystemHealth,
    AllChecks,
}

impl RiskOverrideManager {
    pub async fn create_override(
        &self,
        request: OverrideRequest,
        auth_token: &str,
    ) -> Result<String, RiskError> {
        // Verify permissions
        let user = self.permissions
            .verify_override_permission(auth_token)
            .await?;

        // Validate override parameters
        self.validate_override_request(&request)?;

        // Create override
        let override_id = Uuid::new_v4().to_string();
        let override_entry = Override {
            id: override_id.clone(),
            override_type: request.override_type,
            created_by: user.username,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(request.duration_minutes as i64),
            reason: request.reason,
            conditions: request.conditions,
        };

        // Store override
        self.overrides.write().await.insert(
            override_id.clone(),
            override_entry.clone(),
        );

        // Audit log
        self.audit_logger.log_override_created(&override_entry).await;

        Ok(override_id)
    }

    pub async fn check_overrides(&self, context: &ValidationContext) -> Vec<Override> {
        let overrides = self.overrides.read().await;
        let now = Utc::now();

        overrides.values()
            .filter(|o| {
                o.expires_at > now && 
                self.matches_conditions(&o.conditions, context)
            })
            .cloned()
            .collect()
    }

    pub async fn cleanup_expired_overrides(&self) {
        let mut overrides = self.overrides.write().await;
        let now = Utc::now();
        
        overrides.retain(|_, override| {
            if override.expires_at <= now {
                // Log expiration
                tokio::spawn({
                    let logger = self.audit_logger.clone();
                    let override_clone = override.clone();
                    async move {
                        logger.log_override_expired(&override_clone).await;
                    }
                });
                false
            } else {
                true
            }
        });
    }

    fn validate_override_request(&self, request: &OverrideRequest) -> Result<(), RiskError> {
        // Maximum override duration: 24 hours
        if request.duration_minutes > 1440 {
            return Err(RiskError::InvalidOverride(
                "Override duration cannot exceed 24 hours".to_string()
            ));
        }

        // Validate reason is provided
        if request.reason.trim().is_empty() {
            return Err(RiskError::InvalidOverride(
                "Override reason must be provided".to_string()
            ));
        }

        // Validate override type specific constraints
        match &request.override_type {
            OverrideType::SlippageLimit { new_limit_bps } => {
                if *new_limit_bps > 2000 { // Max 20%
                    return Err(RiskError::InvalidOverride(
                        "Slippage override cannot exceed 20%".to_string()
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }
}
```

### 7. Integration with Trading Systems

```rust
pub struct TradingSystemIntegration {
    risk_manager: Arc<RiskManager>,
    paper_trader: Option<Arc<dyn TradeExecutor>>,
    live_trader: Option<Arc<dyn TradeExecutor>>,
}

impl TradingSystemIntegration {
    pub async fn validate_and_execute(
        &self,
        request: TradeRequest,
        mode: TradingMode,
    ) -> Result<TradeResult, TradingError> {
        // Perform risk validation
        let validation_result = self.risk_manager
            .validate_trade_execution(&request)
            .await
            .map_err(|e| TradingError::RiskValidationFailed(e))?;

        // Log warnings if any
        for warning in &validation_result.warnings {
            warn!("Risk warning for trade {}: {}", request.request_id, warning);
        }

        // Select appropriate executor
        let executor = match mode {
            TradingMode::Paper => self.paper_trader.as_ref()
                .ok_or(TradingError::ExecutorNotAvailable("paper"))?,
            TradingMode::Live => self.live_trader.as_ref()
                .ok_or(TradingError::ExecutorNotAvailable("live"))?,
        };

        // Execute trade with risk context
        let mut enriched_request = request.clone();
        enriched_request.risk_score = Some(validation_result.risk_score);
        enriched_request.risk_warnings = validation_result.warnings;

        executor.execute_trade(enriched_request).await
    }

    pub fn create_mock_risk_manager() -> Arc<RiskManager> {
        Arc::new(MockRiskManager::new())
    }
}

// Mock implementation for testing
pub struct MockRiskManager;

impl MockRiskManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn validate_trade_execution(
        &self,
        _request: &TradeRequest,
    ) -> Result<ValidationResult, RiskError> {
        Ok(ValidationResult {
            approved: true,
            warnings: vec![],
            validation_time_us: 100,
            risk_score: 0.95,
        })
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
    async fn test_slippage_validation() {
        let config = RiskConfig {
            max_slippage_bps: 100, // 1%
            ..Default::default()
        };
        
        let validator = ExecutionValidator::new(Arc::new(RwLock::new(config)));
        
        // Valid slippage
        let request = TradeRequest {
            max_slippage_bps: 50,
            ..create_test_request()
        };
        assert!(validator.validate_parameters(&request, &test_context()).await.is_ok());
        
        // Excessive slippage
        let request = TradeRequest {
            max_slippage_bps: 200,
            ..create_test_request()
        };
        assert!(matches!(
            validator.validate_parameters(&request, &test_context()).await,
            Err(RiskError::SlippageExceedsLimit { .. })
        ));
    }

    #[tokio::test]
    async fn test_circuit_breaker_trigger() {
        let manager = CircuitBreakerManager::new(test_config());
        let breaker = manager.create_circuit_breaker("test").await;
        
        // Record failures
        for _ in 0..5 {
            manager.register_technical_failure(TechnicalFailure::NodeTimeout).await;
        }
        
        // Check circuit is closed
        let status = manager.check_circuit_breakers().await;
        assert!(matches!(status, Err(RiskError::CircuitBreakerClosed { .. })));
    }

    #[tokio::test]
    async fn test_risk_score_calculation() {
        let risk_manager = create_test_risk_manager().await;
        
        // Low risk trade
        let request = TradeRequest {
            max_slippage_bps: 50,
            urgency_level: UrgencyLevel::Normal as i32,
            ..create_test_request()
        };
        let result = risk_manager.validate_trade_execution(&request).await.unwrap();
        assert!(result.risk_score > 0.8);
        
        // High risk trade
        let request = TradeRequest {
            max_slippage_bps: 400,
            urgency_level: UrgencyLevel::Critical as i32,
            ..create_test_request()
        };
        let result = risk_manager.validate_trade_execution(&request).await.unwrap();
        assert!(result.risk_score < 0.5);
    }

    #[tokio::test]
    async fn test_override_functionality() {
        let override_manager = create_test_override_manager();
        
        // Create override
        let request = OverrideRequest {
            override_type: OverrideType::SlippageLimit { new_limit_bps: 1000 },
            duration_minutes: 60,
            reason: "Market volatility spike".to_string(),
            conditions: OverrideConditions::default(),
        };
        
        let override_id = override_manager
            .create_override(request, "valid_token")
            .await
            .unwrap();
        
        // Check override exists
        let overrides = override_manager.check_overrides(&test_context()).await;
        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides[0].id, override_id);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_risk_validation_with_unhealthy_system() {
    let mut test_env = setup_test_environment().await;
    
    // Simulate high node latency
    test_env.node_monitor.set_latency(300).await;
    
    let risk_manager = RiskManager::new(default_config()).await.unwrap();
    let request = create_test_trade_request();
    
    let result = risk_manager.validate_trade_execution(&request).await;
    assert!(matches!(result, Err(RiskError::SystemUnhealthy { .. })));
}

#[tokio::test]
async fn test_risk_manager_with_trading_systems() {
    let integration = setup_trading_integration().await;
    
    // Test with paper trader
    let request = create_test_trade_request();
    let result = integration
        .validate_and_execute(request.clone(), TradingMode::Paper)
        .await;
    assert!(result.is_ok());
    
    // Test with circuit breaker closed
    integration.risk_manager.circuit_breaker.trip().await;
    let result = integration
        .validate_and_execute(request, TradingMode::Paper)
        .await;
    assert!(matches!(result, Err(TradingError::RiskValidationFailed(_))));
}

#[tokio::test]
async fn test_metric_collection() {
    let risk_manager = create_test_risk_manager().await;
    let collector = risk_manager.metrics_collector.clone();
    
    // Execute multiple validations
    for i in 0..10 {
        let mut request = create_test_trade_request();
        request.max_slippage_bps = (i * 10) as u16;
        
        let _ = risk_manager.validate_trade_execution(&request).await;
    }
    
    // Check metrics
    let dashboard = collector.get_risk_dashboard_data().await.unwrap();
    assert!(dashboard.validations_per_minute > 0.0);
    assert!(dashboard.current_risk_score >= 0.0 && dashboard.current_risk_score <= 1.0);
}
```

### Performance Tests

```rust
#[tokio::test]
async fn test_validation_performance() {
    let risk_manager = create_test_risk_manager().await;
    let request = create_test_trade_request();
    
    // Warm up
    for _ in 0..10 {
        let _ = risk_manager.validate_trade_execution(&request).await;
    }
    
    // Measure validation time
    let mut times = Vec::new();
    for _ in 0..1000 {
        let start = Instant::now();
        let result = risk_manager.validate_trade_execution(&request).await.unwrap();
        times.push(result.validation_time_us);
    }
    
    // Calculate percentiles
    times.sort_unstable();
    let p50 = times[500];
    let p99 = times[990];
    
    println!("Validation times - P50: {}μs, P99: {}μs", p50, p99);
    
    // Should be very fast (target: <1ms P99)
    assert!(p99 < 1000);
}

#[tokio::test]
async fn test_concurrent_validations() {
    let risk_manager = Arc::new(create_test_risk_manager().await);
    let mut handles = vec![];
    
    // Spawn 100 concurrent validations
    for i in 0..100 {
        let manager = risk_manager.clone();
        let handle = tokio::spawn(async move {
            let mut request = create_test_trade_request();
            request.request_id = format!("concurrent-{}", i);
            manager.validate_trade_execution(&request).await
        });
        handles.push(handle);
    }
    
    // All should complete successfully
    let results = futures::future::join_all(handles).await;
    for result in results {
        assert!(result.unwrap().is_ok());
    }
}
```

## Dependencies

- **Task 1**: Common libraries for circuit breaker and models
- **Task 6**: Integration points for health monitoring
- **Task 13**: Database for configuration persistence
- **Task 17**: Integration with live trade executor

## Integration Points

- **Trade Executors**: Both paper and live traders integrate risk validation
- **Circuit Breaker**: Centralized system health management
- **Monitoring System**: Exports metrics for dashboards
- **Configuration Service**: Dynamic threshold updates
- **Audit System**: Comprehensive logging of all decisions

## Performance Considerations

- Validation completes in <1ms for P99
- Minimal memory allocation during validation
- Efficient concurrent access to shared state
- Caching of health metrics to reduce queries
- Asynchronous metric recording

## Security Considerations

- Override permissions require authentication
- All risk decisions logged for audit
- Sensitive thresholds encrypted at rest
- Rate limiting on override creation
- Tamper-evident audit trail

## Future Enhancements

- Machine learning for dynamic risk scoring
- Predictive circuit breaker triggers
- Integration with external risk systems
- Advanced correlation analysis
- Real-time risk dashboard with WebSocket
- Automated threshold optimization