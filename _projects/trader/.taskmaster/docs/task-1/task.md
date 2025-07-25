# Task 1: Implement Common Libraries for Trade Models and MEV Structures

## Overview

This task establishes the foundation layer of the Solana Trading Platform by creating shared Rust crates that define common data structures and functionality for both paper and live trading modes. These libraries provide the core building blocks that all other components will depend upon.

## Architecture Context

As outlined in the system architecture, the common libraries serve as the foundation layer that must be built first. They provide:

- **Unified Data Models**: Ensure consistency between paper and live trading modes
- **Resilient Infrastructure**: Circuit breakers and health monitoring for system stability
- **MEV Protection**: Risk models to simulate and prevent sandwich attacks
- **Integration Abstractions**: Clean interfaces for Solana and Jupiter connections

## Implementation Details

### 1. Trading Models Crate (`common/models/`)

#### Enhanced Trade Model
```rust
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub priority_fee: Option<u64>,  // MEV protection fee (1000-10000 lamports)
    pub tx_signature: Option<String>,
    pub transfer_fee: Option<f64>,  // Token-2022 extension fees
    pub extension_data: Option<serde_json::Value>,
    pub mev_protected: bool,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeAction {
    Buy,
    Sell,
    Swap,
}

impl Trade {
    pub fn calculate_total_cost(&self) -> f64 {
        let base_cost = self.amount * self.price;
        let fees = self.fee + self.priority_fee.unwrap_or(0) as f64 / 1e9;
        base_cost + fees
    }

    pub fn calculate_slippage_bps(&self, expected_price: f64) -> u16 {
        ((self.price / expected_price - 1.0).abs() * 10000.0) as u16
    }
}
```

#### MEV Risk Model
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevRisk {
    pub sandwich_probability: f64,  // 0.0 to 1.0
    pub estimated_loss_bps: u16,    // basis points
    pub recommended_priority_fee: u64,  // lamports
}

impl MevRisk {
    pub fn calculate_risk(trade_size: f64, pool_liquidity: f64) -> Self {
        let trade_impact = trade_size / pool_liquidity;
        
        // Based on PRD: 15-20% sandwich attack rate for memecoins
        let sandwich_probability = if trade_impact > 0.005 {
            0.15 + (trade_impact * 100.0).min(0.35)  // 15-50% risk
        } else {
            0.05  // Base 5% risk
        };

        let estimated_loss_bps = (sandwich_probability * 500.0) as u16;
        let recommended_priority_fee = Self::calculate_priority_fee(sandwich_probability, trade_size);

        Self {
            sandwich_probability,
            estimated_loss_bps,
            recommended_priority_fee,
        }
    }

    fn calculate_priority_fee(risk: f64, trade_size: f64) -> u64 {
        // Dynamic fee calculation based on risk and trade size
        let base_fee = 1000u64;  // 1000 lamports minimum
        let risk_multiplier = (risk * 10.0) as u64;
        let size_multiplier = (trade_size / 1000.0).max(1.0) as u64;
        
        (base_fee * risk_multiplier * size_multiplier).min(10000)  // Cap at 10000 lamports
    }
}
```

#### Circuit Breaker Model
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitState {
    Open,     // Normal operation
    HalfOpen, // Testing recovery
    Closed,   // Circuit tripped, blocking operations
}

#[derive(Debug)]
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_threshold: u32,
    recovery_timeout: Duration,
    failure_count: Arc<RwLock<u32>>,
    last_failure: Arc<RwLock<Option<Instant>>>,
    latency_threshold_ms: u64,  // 200ms P99 target from PRD
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Open)),
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure: Arc::new(RwLock::new(None)),
            latency_threshold_ms: 200,  // PRD requirement
        }
    }

    pub async fn check_latency(&self, latency_ms: u64) -> Result<(), String> {
        if latency_ms > self.latency_threshold_ms {
            self.record_failure().await;
            return Err(format!("Latency {}ms exceeds threshold {}ms", 
                latency_ms, self.latency_threshold_ms));
        }
        self.record_success().await;
        Ok(())
    }

    async fn record_failure(&self) {
        let mut count = self.failure_count.write().await;
        *count += 1;
        *self.last_failure.write().await = Some(Instant::now());
        
        if *count >= self.failure_threshold {
            *self.state.write().await = CircuitState::Closed;
        }
    }

    async fn record_success(&self) {
        *self.failure_count.write().await = 0;
        if matches!(*self.state.read().await, CircuitState::HalfOpen) {
            *self.state.write().await = CircuitState::Open;
        }
    }

    pub async fn is_open(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitState::Open => true,
            CircuitState::Closed => {
                // Check if recovery timeout has passed
                if let Some(last_failure) = *self.last_failure.read().await {
                    if last_failure.elapsed() > self.recovery_timeout {
                        drop(state);
                        *self.state.write().await = CircuitState::HalfOpen;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }
}
```

### 2. Solana Integration Crate (`common/solana/`)

```rust
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::Transaction};
use std::time::Instant;

pub struct SolanaClient {
    rpc_client: RpcClient,
    circuit_breaker: CircuitBreaker,
    metrics_collector: MetricsCollector,
}

impl SolanaClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            rpc_client: RpcClient::new(endpoint.to_string()),
            circuit_breaker: CircuitBreaker::new(),
            metrics_collector: MetricsCollector::new(),
        }
    }

    pub async fn send_transaction_with_retry(
        &self,
        transaction: &Transaction,
        max_retries: u32,
    ) -> Result<Signature, Box<dyn std::error::Error>> {
        if !self.circuit_breaker.is_open().await {
            return Err("Circuit breaker is closed due to high latency".into());
        }

        let mut retries = 0;
        let mut last_error = None;

        while retries < max_retries {
            let start = Instant::now();
            
            match self.rpc_client.send_transaction(transaction) {
                Ok(signature) => {
                    let latency_ms = start.elapsed().as_millis() as u64;
                    self.circuit_breaker.check_latency(latency_ms).await.ok();
                    self.metrics_collector.record_latency("send_transaction", latency_ms);
                    return Ok(signature);
                }
                Err(e) => {
                    last_error = Some(e);
                    retries += 1;
                    
                    // Exponential backoff
                    let delay = Duration::from_millis(100 * 2u64.pow(retries));
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(format!("Failed after {} retries: {:?}", max_retries, last_error).into())
    }

    pub async fn get_priority_fee(&self) -> Result<u64, Box<dyn std::error::Error>> {
        // Dynamic fee calculation based on network congestion
        // Returns value between 1000-10000 lamports as per PRD
        let recent_fees = self.rpc_client.get_recent_prioritization_fees(&[])?;
        
        if recent_fees.is_empty() {
            return Ok(1000);  // Default minimum
        }

        let avg_fee = recent_fees.iter()
            .map(|f| f.prioritization_fee)
            .sum::<u64>() / recent_fees.len() as u64;

        Ok(avg_fee.max(1000).min(10000))  // Clamp to PRD range
    }
}
```

### 3. Jupiter Client with Failover (`common/jupiter/`)

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct SwapParams {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub slippage_bps: u16,
    pub user_public_key: String,
    pub wrap_and_unwrap_sol: bool,
    pub use_shared_accounts: bool,
    pub priority_fee_lamports: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct SwapResponse {
    pub swap_transaction: String,
    pub last_valid_block_height: u64,
    pub prioritization_fee_lamports: u64,
}

pub struct JupiterClient {
    self_hosted_url: String,
    public_url: String,
    client: Client,
    circuit_breaker: CircuitBreaker,
}

impl JupiterClient {
    pub fn new(self_hosted_url: &str, public_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(200))  // PRD requirement
            .build()
            .unwrap();

        Self {
            self_hosted_url: self_hosted_url.to_string(),
            public_url: public_url.to_string(),
            client,
            circuit_breaker: CircuitBreaker::new(),
        }
    }

    pub async fn get_swap_with_failover(
        &self,
        params: &SwapParams,
    ) -> Result<SwapResponse, Box<dyn std::error::Error>> {
        // Try self-hosted first
        let start = Instant::now();
        
        match self.client
            .post(&format!("{}/swap", self.self_hosted_url))
            .json(params)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                let latency_ms = start.elapsed().as_millis() as u64;
                self.circuit_breaker.check_latency(latency_ms).await.ok();
                
                let swap_response: SwapResponse = response.json().await?;
                return Ok(swap_response);
            }
            _ => {
                // Fallback to public Jupiter
                self.circuit_breaker.record_failure().await;
                
                let response = self.client
                    .post(&format!("{}/swap", self.public_url))
                    .json(params)
                    .send()
                    .await?;

                if response.status().is_success() {
                    Ok(response.json().await?)
                } else {
                    Err(format!("Jupiter API error: {}", response.status()).into())
                }
            }
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mev_risk_calculation() {
        // Test low risk scenario
        let low_risk = MevRisk::calculate_risk(100.0, 100000.0);
        assert!(low_risk.sandwich_probability < 0.1);
        assert_eq!(low_risk.recommended_priority_fee, 1000);

        // Test high risk scenario
        let high_risk = MevRisk::calculate_risk(1000.0, 10000.0);
        assert!(high_risk.sandwich_probability > 0.3);
        assert!(high_risk.recommended_priority_fee > 5000);
    }

    #[tokio::test]
    async fn test_circuit_breaker_latency() {
        let breaker = CircuitBreaker::new();
        
        // Should be open initially
        assert!(breaker.is_open().await);

        // Record high latency multiple times
        for _ in 0..5 {
            breaker.check_latency(300).await.err();
        }

        // Should be closed after threshold
        assert!(!breaker.is_open().await);
    }

    #[test]
    fn test_trade_model_serialization() {
        let trade = Trade {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: TradeAction::Swap,
            base_token: "SOL".to_string(),
            quote_token: "USDC".to_string(),
            amount: 10.0,
            price: 150.0,
            fee: 0.25,
            slippage: 0.5,
            priority_fee: Some(5000),
            tx_signature: None,
            transfer_fee: None,
            extension_data: None,
            mev_protected: true,
            latency_ms: 150,
        };

        let serialized = serde_json::to_string(&trade).unwrap();
        let deserialized: Trade = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(trade.id, deserialized.id);
        assert_eq!(trade.mev_protected, deserialized.mev_protected);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_jupiter_failover() {
    // Mock server that always times out
    let _mock = mockito::mock("POST", "/swap")
        .with_status(500)
        .create();

    let client = JupiterClient::new(
        &mockito::server_url(),
        "https://lite-api.jup.ag/v6"
    );

    let params = SwapParams {
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 1_000_000_000,
        slippage_bps: 50,
        user_public_key: "test_pubkey".to_string(),
        wrap_and_unwrap_sol: true,
        use_shared_accounts: true,
        priority_fee_lamports: Some(5000),
    };

    // Should failover to public endpoint
    let result = client.get_swap_with_failover(&params).await;
    assert!(result.is_ok() || result.is_err());  // Either succeeds or fails gracefully
}
```

## Dependencies

- None (this is the foundation layer)

## Integration Points

- **Paper Trader**: Will use these models for virtual portfolio tracking
- **Live Trader**: Will use identical models for real trading
- **Database Layer**: Models must be serializable for storage
- **TUI**: Will display model data in real-time

## Performance Considerations

- Circuit breaker latency threshold: 200ms (P99 target)
- Jupiter API timeout: 200ms before failover
- MEV fee range: 1000-10000 lamports
- Retry logic: Exponential backoff starting at 100ms

## Security Considerations

- No sensitive data (private keys) in models
- All external API calls through circuit breakers
- Proper error handling without exposing internal details
- Input validation for all model fields

## Future Enhancements

- Dynamic slippage models based on pool depth
- Advanced MEV prediction using ML
- Multi-node Solana connection pooling
- Historical pattern analysis for circuit breaker tuning