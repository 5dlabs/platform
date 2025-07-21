# Task 4: Develop Jupiter Failover Client

## Overview

This task implements a resilient Jupiter V6 API client with automatic failover between self-hosted and public instances. The client ensures continuous trade routing availability while prioritizing the low-latency self-hosted instance when available. This is crucial for maintaining the platform's performance targets even during infrastructure issues.

## Architecture Context

The Jupiter failover client is part of the resilient infrastructure layer, providing:

- **Dual-Instance Support**: Primary self-hosted instance for <200ms latency, public fallback for reliability
- **Automatic Failover**: Seamless switching on timeout or error conditions
- **Health Monitoring**: Background health checks to restore primary service
- **MEV Protection**: Integration of protection parameters in all swap requests

This component directly supports the PRD's requirement for 99.5% uptime through redundant routing infrastructure.

## Implementation Details

### 1. Core Jupiter Client Structure

```rust
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use anyhow::{Result, anyhow};

#[derive(Clone)]
pub struct JupiterFailoverClient {
    self_hosted_url: String,
    public_url: String,
    self_hosted_client: Client,
    public_client: Client,
    self_hosted_healthy: Arc<AtomicBool>,
    circuit_breaker: Arc<CircuitBreaker>,
    metrics_collector: Arc<MetricsCollector>,
}

impl JupiterFailoverClient {
    pub fn new(self_hosted_url: &str, public_url: &str) -> Result<Self> {
        // Configure clients with appropriate timeouts
        let self_hosted_client = ClientBuilder::new()
            .timeout(Duration::from_millis(200))  // PRD requirement
            .connect_timeout(Duration::from_millis(50))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build()?;

        let public_client = ClientBuilder::new()
            .timeout(Duration::from_millis(500))  // More lenient for public
            .connect_timeout(Duration::from_millis(100))
            .build()?;

        let client = Self {
            self_hosted_url: self_hosted_url.to_string(),
            public_url: public_url.to_string(),  // lite-api.jup.ag/v6
            self_hosted_client,
            public_client,
            self_hosted_healthy: Arc::new(AtomicBool::new(true)),
            circuit_breaker: Arc::new(CircuitBreaker::new()),
            metrics_collector: Arc::new(MetricsCollector::new()),
        };

        // Start background health checker
        client.start_health_monitor();

        Ok(client)
    }

    fn start_health_monitor(&self) {
        let self_hosted_url = self.self_hosted_url.clone();
        let self_hosted_client = self.self_hosted_client.clone();
        let self_hosted_healthy = self.self_hosted_healthy.clone();
        let circuit_breaker = self.circuit_breaker.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Only check if currently marked unhealthy
                if !self_hosted_healthy.load(Ordering::Relaxed) {
                    let health_check = timeout(
                        Duration::from_millis(200),
                        self_hosted_client.get(&format!("{}/health", self_hosted_url))
                            .send()
                    ).await;

                    match health_check {
                        Ok(Ok(response)) if response.status().is_success() => {
                            self_hosted_healthy.store(true, Ordering::Relaxed);
                            circuit_breaker.reset().await;
                            tracing::info!("Self-hosted Jupiter instance recovered");
                        }
                        _ => {
                            tracing::debug!("Self-hosted Jupiter still unhealthy");
                        }
                    }
                }
            }
        });
    }
}
```

### 2. Quote Implementation with Failover

```rust
#[derive(Debug, Serialize)]
pub struct QuoteRequest {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    pub amount: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "onlyDirectRoutes")]
    pub only_direct_routes: bool,
    #[serde(rename = "asLegacyTransaction")]
    pub as_legacy_transaction: bool,
}

#[derive(Debug, Deserialize)]
pub struct Quote {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    pub other_amount_threshold: String,
    #[serde(rename = "swapMode")]
    pub swap_mode: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: String,
    #[serde(rename = "routePlan")]
    pub route_plan: Vec<RoutePlanStep>,
}

#[derive(Debug, Deserialize)]
pub struct RoutePlanStep {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    pub percent: u8,
}

#[derive(Debug, Deserialize)]
pub struct SwapInfo {
    #[serde(rename = "ammKey")]
    pub amm_key: String,
    pub label: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "feeAmount")]
    pub fee_amount: String,
    #[serde(rename = "feeMint")]
    pub fee_mint: String,
}

impl JupiterFailoverClient {
    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote> {
        let start = std::time::Instant::now();
        
        // Try self-hosted first if healthy
        if self.self_hosted_healthy.load(Ordering::Relaxed) 
            && self.circuit_breaker.is_open().await {
            
            let self_hosted_result = self.try_self_hosted_quote(request).await;
            
            match self_hosted_result {
                Ok(quote) => {
                    let latency = start.elapsed();
                    self.metrics_collector.record_quote_latency("self_hosted", latency);
                    self.circuit_breaker.record_success().await;
                    return Ok(quote);
                }
                Err(e) => {
                    tracing::warn!("Self-hosted quote failed: {}", e);
                    self.self_hosted_healthy.store(false, Ordering::Relaxed);
                    self.circuit_breaker.record_failure().await;
                }
            }
        }

        // Fallback to public
        self.try_public_quote(request).await.map(|quote| {
            let latency = start.elapsed();
            self.metrics_collector.record_quote_latency("public", latency);
            quote
        })
    }

    async fn try_self_hosted_quote(&self, request: &QuoteRequest) -> Result<Quote> {
        let url = format!("{}/quote", self.self_hosted_url);
        
        let response = self.self_hosted_client
            .get(&url)
            .query(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Quote request failed: {}", response.status()));
        }

        response.json::<Quote>().await
            .map_err(|e| anyhow!("Failed to parse quote response: {}", e))
    }

    async fn try_public_quote(&self, request: &QuoteRequest) -> Result<Quote> {
        let url = format!("{}/quote", self.public_url);
        
        let response = self.public_client
            .get(&url)
            .query(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Public quote request failed: {}", response.status()));
        }

        response.json::<Quote>().await
            .map_err(|e| anyhow!("Failed to parse public quote response: {}", e))
    }
}
```

### 3. Swap Implementation with MEV Protection

```rust
#[derive(Debug, Serialize)]
pub struct SwapRequest {
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
    #[serde(rename = "wrapAndUnwrapSol")]
    pub wrap_and_unwrap_sol: bool,
    #[serde(rename = "useSharedAccounts")]
    pub use_shared_accounts: bool,
    #[serde(rename = "feeAccount")]
    pub fee_account: Option<String>,
    #[serde(rename = "trackingAccount")]
    pub tracking_account: Option<String>,
    #[serde(rename = "computeUnitPriceMicroLamports")]
    pub compute_unit_price_micro_lamports: Option<u64>,  // MEV protection
    #[serde(rename = "prioritizationFeeLamports")]
    pub prioritization_fee_lamports: Option<u64>,  // 1000-10000 as per PRD
    #[serde(rename = "asLegacyTransaction")]
    pub as_legacy_transaction: bool,
    #[serde(rename = "useTokenLedger")]
    pub use_token_ledger: bool,
    #[serde(rename = "destinationTokenAccount")]
    pub destination_token_account: Option<String>,
    #[serde(rename = "dynamicComputeUnitLimit")]
    pub dynamic_compute_unit_limit: bool,
    #[serde(rename = "skipUserAccountsRpcCalls")]
    pub skip_user_accounts_rpc_calls: bool,
    // Quote data
    #[serde(rename = "quoteResponse")]
    pub quote_response: Quote,
}

#[derive(Debug, Deserialize)]
pub struct SwapResponse {
    #[serde(rename = "swapTransaction")]
    pub swap_transaction: String,  // Base64 encoded transaction
    #[serde(rename = "lastValidBlockHeight")]
    pub last_valid_block_height: u64,
    #[serde(rename = "prioritizationFeeLamports")]
    pub prioritization_fee_lamports: u64,
}

impl JupiterFailoverClient {
    pub async fn get_swap_transaction(
        &self,
        quote: Quote,
        user_public_key: &str,
        priority_fee: Option<u64>,
    ) -> Result<SwapResponse> {
        let request = SwapRequest {
            user_public_key: user_public_key.to_string(),
            wrap_and_unwrap_sol: true,  // MEV protection
            use_shared_accounts: true,  // MEV protection
            fee_account: None,
            tracking_account: None,
            compute_unit_price_micro_lamports: priority_fee.map(|f| f * 1000),
            prioritization_fee_lamports: priority_fee,
            as_legacy_transaction: false,
            use_token_ledger: false,
            destination_token_account: None,
            dynamic_compute_unit_limit: true,
            skip_user_accounts_rpc_calls: false,
            quote_response: quote,
        };

        // Use same failover logic as quotes
        if self.self_hosted_healthy.load(Ordering::Relaxed) 
            && self.circuit_breaker.is_open().await {
            
            match self.try_self_hosted_swap(&request).await {
                Ok(swap) => {
                    self.circuit_breaker.record_success().await;
                    return Ok(swap);
                }
                Err(e) => {
                    tracing::warn!("Self-hosted swap failed: {}", e);
                    self.self_hosted_healthy.store(false, Ordering::Relaxed);
                    self.circuit_breaker.record_failure().await;
                }
            }
        }

        self.try_public_swap(&request).await
    }

    async fn try_self_hosted_swap(&self, request: &SwapRequest) -> Result<SwapResponse> {
        let url = format!("{}/swap", self.self_hosted_url);
        
        let response = self.self_hosted_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Swap request failed: {}", error_text));
        }

        response.json::<SwapResponse>().await
            .map_err(|e| anyhow!("Failed to parse swap response: {}", e))
    }

    async fn try_public_swap(&self, request: &SwapRequest) -> Result<SwapResponse> {
        let url = format!("{}/swap", self.public_url);
        
        let response = self.public_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Public swap request failed: {}", error_text));
        }

        response.json::<SwapResponse>().await
            .map_err(|e| anyhow!("Failed to parse public swap response: {}", e))
    }
}
```

### 4. Token Support and Validation

```rust
pub struct TokenRegistry {
    tokens: HashMap<String, TokenInfo>,
}

impl TokenRegistry {
    pub fn new() -> Self {
        let mut tokens = HashMap::new();
        
        // MVP tokens as per PRD
        tokens.insert("SOL".to_string(), TokenInfo {
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            decimals: 9,
            has_transfer_fee: false,
        });
        
        tokens.insert("USDC".to_string(), TokenInfo {
            symbol: "USDC".to_string(),
            mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            decimals: 6,
            has_transfer_fee: false,
        });
        
        tokens.insert("BONK".to_string(), TokenInfo {
            symbol: "BONK".to_string(),
            mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
            decimals: 5,
            has_transfer_fee: false,
        });
        
        tokens.insert("JitoSOL".to_string(), TokenInfo {
            symbol: "JitoSOL".to_string(),
            mint: "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn".to_string(),
            decimals: 9,
            has_transfer_fee: false,
        });
        
        tokens.insert("RAY".to_string(), TokenInfo {
            symbol: "RAY".to_string(),
            mint: "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(),
            decimals: 6,
            has_transfer_fee: false,
        });

        Self { tokens }
    }

    pub fn get_token_info(&self, symbol: &str) -> Option<&TokenInfo> {
        self.tokens.get(symbol)
    }

    pub fn validate_token_pair(&self, input: &str, output: &str) -> Result<()> {
        if !self.tokens.contains_key(input) {
            return Err(anyhow!("Invalid input token: {}", input));
        }
        if !self.tokens.contains_key(output) {
            return Err(anyhow!("Invalid output token: {}", output));
        }
        if input == output {
            return Err(anyhow!("Input and output tokens cannot be the same"));
        }
        Ok(())
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_failover_on_timeout() {
        let self_hosted = mockito::mock("GET", "/quote")
            .with_status(200)
            .with_body(r#"{"error": "timeout"}"#)
            .delay(Duration::from_millis(300))  // Exceed 200ms timeout
            .create();

        let public = mockito::mock("GET", "/quote")
            .with_status(200)
            .with_body(include_str!("test_fixtures/quote_response.json"))
            .create();

        let client = JupiterFailoverClient::new(
            &mockito::server_url(),
            &mockito::server_url()
        ).unwrap();

        let request = QuoteRequest {
            input_mint: "SOL".to_string(),
            output_mint: "USDC".to_string(),
            amount: "1000000000".to_string(),
            slippage_bps: 50,
            only_direct_routes: false,
            as_legacy_transaction: false,
        };

        let quote = client.get_quote(&request).await.unwrap();
        assert!(!quote.out_amount.is_empty());
        
        // Verify self-hosted marked unhealthy
        assert!(!client.self_hosted_healthy.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_mev_protection_parameters() {
        let mock = mockito::mock("POST", "/swap")
            .match_body(mockito::Matcher::Json(serde_json::json!({
                "wrapAndUnwrapSol": true,
                "useSharedAccounts": true,
                "prioritizationFeeLamports": 5000
            })))
            .with_status(200)
            .with_body(include_str!("test_fixtures/swap_response.json"))
            .create();

        let client = JupiterFailoverClient::new(
            &mockito::server_url(),
            "https://lite-api.jup.ag/v6"
        ).unwrap();

        let quote = create_test_quote();
        let swap = client.get_swap_transaction(
            quote,
            "test_pubkey",
            Some(5000)  // MEV protection fee
        ).await.unwrap();

        assert_eq!(swap.prioritization_fee_lamports, 5000);
    }

    #[tokio::test]
    async fn test_health_recovery() {
        let client = JupiterFailoverClient::new(
            "http://localhost:8080",
            "https://lite-api.jup.ag/v6"
        ).unwrap();

        // Mark unhealthy
        client.self_hosted_healthy.store(false, Ordering::Relaxed);
        
        // Simulate health check success
        let _health_mock = mockito::mock("GET", "/health")
            .with_status(200)
            .create();

        // Wait for health check
        tokio::time::sleep(Duration::from_secs(31)).await;
        
        // Should be healthy again
        assert!(client.self_hosted_healthy.load(Ordering::Relaxed));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_real_jupiter_integration() {
    let client = JupiterFailoverClient::new(
        "http://localhost:8080",  // Assumes local Jupiter
        "https://lite-api.jup.ag/v6"
    ).unwrap();

    // Test with real tokens
    let request = QuoteRequest {
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: "1000000".to_string(),  // 0.001 SOL
        slippage_bps: 50,
        only_direct_routes: false,
        as_legacy_transaction: false,
    };

    let quote = client.get_quote(&request).await;
    assert!(quote.is_ok());
}

#[tokio::test]
async fn test_latency_comparison() {
    let client = JupiterFailoverClient::new(
        "http://localhost:8080",
        "https://lite-api.jup.ag/v6"
    ).unwrap();

    let request = create_test_quote_request();
    
    // Measure self-hosted latency
    let start = std::time::Instant::now();
    let _ = client.get_quote(&request).await;
    let self_hosted_latency = start.elapsed();
    
    // Force public
    client.self_hosted_healthy.store(false, Ordering::Relaxed);
    
    let start = std::time::Instant::now();
    let _ = client.get_quote(&request).await;
    let public_latency = start.elapsed();
    
    println!("Self-hosted: {:?}, Public: {:?}", self_hosted_latency, public_latency);
    
    // Self-hosted should be faster
    assert!(self_hosted_latency < public_latency);
}
```

## Dependencies

- **Task 1**: Uses Circuit Breaker from common libraries

## Integration Points

- **Paper Trader**: Uses client for price quotes and simulated swaps
- **Live Trader**: Uses identical interface for real trades
- **MEV Protection**: Integrates priority fee parameters
- **Monitoring**: Exports latency metrics per endpoint

## Performance Considerations

- Self-hosted timeout: 200ms (strict)
- Public timeout: 500ms (more lenient)
- Connection pooling for reuse
- Background health checks every 30 seconds
- Request coalescing for duplicate quotes

## Security Considerations

- No private keys in client
- TLS for all connections
- Request validation before sending
- Rate limiting awareness
- Error messages sanitized

## Future Enhancements

- Request caching for identical quotes
- WebSocket support for streaming quotes
- Multiple self-hosted instances
- Geographic failover
- Compression for bandwidth optimization