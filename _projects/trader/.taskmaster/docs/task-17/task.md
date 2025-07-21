# Task 17: Implement Live Trade Executor with Real Transaction Execution

## Overview

This task creates a robust live trading execution engine that can execute real transactions on Solana based on external TradeRequest objects received via gRPC. The executor integrates comprehensive risk management, circuit breaker protection, and transaction confirmation monitoring to ensure safe and reliable trade execution in production.

## Architecture Context

The Live Trade Executor is the core component of the live trading service, responsible for:

- **External Trade Reception**: Accepting pre-validated trade requests from external strategy services via gRPC
- **Transaction Construction**: Building Solana transactions for Jupiter V6 swaps with MEV protection
- **Risk Validation**: Performing technical execution checks before committing funds
- **Circuit Breaker Integration**: Automatically halting trades during system issues
- **Confirmation Monitoring**: Tracking transaction status through to finalization

As outlined in the architecture, this component assumes trade requests have already been validated for business logic and portfolio risk by external services.

## Implementation Details

### 1. Core Transaction Execution System

#### Transaction Builder for Jupiter V6
```rust
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    transaction::Transaction,
    signature::Signature,
};
use jupiter_sdk::JupiterSwapApiClient;

pub struct TransactionBuilder {
    jupiter_client: Arc<JupiterSwapApiClient>,
    wallet_manager: Arc<WalletManager>,
    priority_fee_calculator: Arc<PriorityFeeCalculator>,
}

impl TransactionBuilder {
    pub async fn build_swap_transaction(
        &self,
        request: &TradeRequest,
    ) -> Result<Transaction, ExecutorError> {
        // Calculate optimal priority fee based on network congestion
        let priority_fee = self.priority_fee_calculator
            .calculate_fee(request.urgency_level)
            .await?;

        // Build Jupiter V6 swap parameters with MEV protection
        let swap_params = SwapRequest {
            user_public_key: self.wallet_manager.get_pubkey().await?.to_string(),
            input_mint: request.input_token.to_string(),
            output_mint: request.output_token.to_string(),
            amount: request.amount_lamports,
            slippage_bps: request.max_slippage_bps,
            // MEV protection parameters
            priority_fee_lamports: Some(priority_fee),
            wrap_and_unwrap_sol: true,
            use_shared_accounts: true,
            fee_account: request.fee_account.map(|a| a.to_string()),
            tracking_account: request.tracking_account.map(|a| a.to_string()),
            compute_unit_price_micro_lamports: Some(priority_fee * 1000),
            as_legacy_transaction: false, // Use versioned transactions
        };

        // Get swap instructions from Jupiter
        let swap_response = self.jupiter_client
            .get_swap_instructions(&swap_params)
            .await
            .map_err(|e| ExecutorError::JupiterError(e.to_string()))?;

        // Build transaction with additional instructions
        let mut instructions = vec![];

        // Add compute budget instruction
        instructions.push(
            ComputeBudgetInstruction::set_compute_unit_price(priority_fee * 1000)
        );

        // Add setup instructions if any
        instructions.extend(swap_response.setup_instructions);

        // Add main swap instruction
        instructions.push(swap_response.swap_instruction);

        // Add cleanup instructions if any
        instructions.extend(swap_response.cleanup_instructions);

        // Create transaction
        let recent_blockhash = self.get_recent_blockhash().await?;
        let message = Message::new_with_blockhash(
            &instructions,
            Some(&self.wallet_manager.get_pubkey().await?),
            &recent_blockhash,
        );

        Ok(Transaction::new_unsigned(message))
    }

    async fn get_recent_blockhash(&self) -> Result<Hash, ExecutorError> {
        // Get recent blockhash with retry logic
        retry_with_backoff(|| async {
            self.solana_client
                .get_latest_blockhash()
                .await
                .map_err(|e| ExecutorError::SolanaError(e.to_string()))
        }, 3, Duration::from_millis(100))
        .await
    }
}
```

#### Priority Fee Calculation
```rust
pub struct PriorityFeeCalculator {
    solana_client: Arc<SolanaClient>,
    fee_history: Arc<RwLock<VecDeque<u64>>>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl PriorityFeeCalculator {
    pub async fn calculate_fee(&self, urgency: UrgencyLevel) -> Result<u64, ExecutorError> {
        // Check circuit breaker
        if !self.circuit_breaker.is_open().await {
            return Ok(1000); // Minimum fee if circuit is closed
        }

        // Get recent priority fees
        let recent_fees = self.solana_client
            .get_recent_prioritization_fees(&[])
            .await?;

        if recent_fees.is_empty() {
            return Ok(self.get_default_fee(urgency));
        }

        // Calculate percentile based on urgency
        let percentile = match urgency {
            UrgencyLevel::Low => 50,    // 50th percentile
            UrgencyLevel::Normal => 75, // 75th percentile
            UrgencyLevel::High => 90,   // 90th percentile
            UrgencyLevel::Critical => 95, // 95th percentile
        };

        let mut fees: Vec<u64> = recent_fees.iter()
            .map(|f| f.prioritization_fee)
            .collect();
        fees.sort_unstable();

        let index = (fees.len() * percentile / 100).min(fees.len() - 1);
        let calculated_fee = fees[index];

        // Clamp to PRD specified range (1000-10000 lamports)
        Ok(calculated_fee.max(1000).min(10000))
    }

    fn get_default_fee(&self, urgency: UrgencyLevel) -> u64 {
        match urgency {
            UrgencyLevel::Low => 1000,
            UrgencyLevel::Normal => 2500,
            UrgencyLevel::High => 5000,
            UrgencyLevel::Critical => 10000,
        }
    }
}
```

### 2. gRPC Interface Implementation

```rust
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug)]
pub struct TradeExecutorService {
    executor: Arc<LiveTradeExecutor>,
    request_validator: Arc<RequestValidator>,
    metrics_collector: Arc<MetricsCollector>,
}

#[tonic::async_trait]
impl trade_executor::TradeExecutor for TradeExecutorService {
    async fn execute_trade_request(
        &self,
        request: Request<TradeRequest>,
    ) -> Result<Response<TradeResult>, Status> {
        let start = Instant::now();
        let trade_request = request.into_inner();
        
        // Log received request
        info!(
            "Received trade request: id={}, token_pair={}/{}, amount={}",
            trade_request.request_id,
            trade_request.input_token,
            trade_request.output_token,
            trade_request.amount_lamports
        );

        // Validate request format
        if let Err(e) = self.request_validator.validate(&trade_request).await {
            self.metrics_collector.record_validation_failure(&trade_request);
            return Err(Status::invalid_argument(format!("Invalid request: {}", e)));
        }

        // Execute trade
        match self.executor.execute_trade(trade_request.clone()).await {
            Ok(execution_result) => {
                let trade_result = TradeResult {
                    request_id: trade_request.request_id,
                    status: TradeStatus::Success as i32,
                    transaction_signature: Some(execution_result.signature.to_string()),
                    executed_amount: execution_result.amount_received,
                    executed_price: execution_result.price,
                    slippage_bps: execution_result.slippage_bps,
                    fees_paid: execution_result.total_fees,
                    priority_fee: execution_result.priority_fee,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    error_message: None,
                    mev_protected: execution_result.mev_protected,
                    confirmation_time_ms: execution_result.confirmation_time_ms,
                };

                self.metrics_collector.record_successful_trade(&trade_result);
                Ok(Response::new(trade_result))
            }
            Err(e) => {
                let trade_result = TradeResult {
                    request_id: trade_request.request_id,
                    status: TradeStatus::Failed as i32,
                    transaction_signature: None,
                    executed_amount: 0,
                    executed_price: 0.0,
                    slippage_bps: 0,
                    fees_paid: 0,
                    priority_fee: 0,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    error_message: Some(e.to_string()),
                    mev_protected: false,
                    confirmation_time_ms: 0,
                };

                self.metrics_collector.record_failed_trade(&trade_result);
                
                match e {
                    ExecutorError::InsufficientBalance => {
                        Err(Status::failed_precondition(e.to_string()))
                    }
                    ExecutorError::CircuitBreakerOpen => {
                        Err(Status::unavailable(e.to_string()))
                    }
                    _ => Err(Status::internal(e.to_string()))
                }
            }
        }
    }

    async fn get_executor_status(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<ExecutorStatus>, Status> {
        let status = self.executor.get_status().await;
        Ok(Response::new(status))
    }
}

// Request validation
pub struct RequestValidator {
    supported_tokens: Arc<RwLock<HashSet<Pubkey>>>,
    min_trade_size: u64,
    max_trade_size: u64,
}

impl RequestValidator {
    pub async fn validate(&self, request: &TradeRequest) -> Result<(), ValidationError> {
        // Check request ID format
        if request.request_id.is_empty() {
            return Err(ValidationError::MissingRequestId);
        }

        // Validate tokens
        let input_token = Pubkey::from_str(&request.input_token)
            .map_err(|_| ValidationError::InvalidToken("input_token"))?;
        let output_token = Pubkey::from_str(&request.output_token)
            .map_err(|_| ValidationError::InvalidToken("output_token"))?;

        let supported = self.supported_tokens.read().await;
        if !supported.contains(&input_token) {
            return Err(ValidationError::UnsupportedToken(input_token));
        }
        if !supported.contains(&output_token) {
            return Err(ValidationError::UnsupportedToken(output_token));
        }

        // Validate amount
        if request.amount_lamports < self.min_trade_size {
            return Err(ValidationError::AmountTooSmall);
        }
        if request.amount_lamports > self.max_trade_size {
            return Err(ValidationError::AmountTooLarge);
        }

        // Validate slippage
        if request.max_slippage_bps > 1000 { // Max 10%
            return Err(ValidationError::SlippageTooHigh);
        }

        Ok(())
    }
}
```

### 3. Risk Management Integration

```rust
pub struct PreTradeValidator {
    balance_checker: Arc<BalanceChecker>,
    circuit_breaker: Arc<CircuitBreaker>,
    token_whitelist: Arc<TokenWhitelist>,
    trade_limiter: Arc<TradeLimiter>,
}

impl PreTradeValidator {
    pub async fn validate_trade(
        &self,
        request: &TradeRequest,
        wallet_pubkey: &Pubkey,
    ) -> Result<ValidationResult, RiskError> {
        // Check circuit breaker first
        if !self.circuit_breaker.is_open().await {
            return Err(RiskError::CircuitBreakerOpen);
        }

        // Verify sufficient balance
        let balance = self.balance_checker
            .get_token_balance(wallet_pubkey, &request.input_token)
            .await?;

        let required_balance = request.amount_lamports + 
            self.estimate_fees(request).await?;

        if balance < required_balance {
            return Err(RiskError::InsufficientBalance {
                available: balance,
                required: required_balance,
            });
        }

        // Check token whitelist
        if !self.token_whitelist.is_approved(&request.input_token).await? ||
           !self.token_whitelist.is_approved(&request.output_token).await? {
            return Err(RiskError::TokenNotWhitelisted);
        }

        // Verify trade size limits
        self.trade_limiter.check_limits(request).await?;

        Ok(ValidationResult {
            approved: true,
            estimated_fees: self.estimate_fees(request).await?,
            warnings: vec![],
        })
    }

    async fn estimate_fees(&self, request: &TradeRequest) -> Result<u64, RiskError> {
        // Base transaction fee
        let mut total_fees = 5000u64; // 0.000005 SOL

        // Priority fee based on urgency
        let priority_fee = match request.urgency_level() {
            UrgencyLevel::Low => 1000,
            UrgencyLevel::Normal => 2500,
            UrgencyLevel::High => 5000,
            UrgencyLevel::Critical => 10000,
        };
        total_fees += priority_fee;

        // Jupiter platform fee (if any)
        let platform_fee = (request.amount_lamports as f64 * 0.0025) as u64; // 0.25%
        total_fees += platform_fee;

        Ok(total_fees)
    }
}
```

### 4. Transaction Execution and Monitoring

```rust
pub struct LiveTradeExecutor {
    transaction_builder: Arc<TransactionBuilder>,
    wallet_manager: Arc<WalletManager>,
    solana_client: Arc<SolanaClient>,
    pre_trade_validator: Arc<PreTradeValidator>,
    confirmation_monitor: Arc<ConfirmationMonitor>,
    trade_recorder: Arc<TradeRecorder>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl LiveTradeExecutor {
    pub async fn execute_trade(
        &self,
        request: TradeRequest,
    ) -> Result<ExecutionResult, ExecutorError> {
        let start = Instant::now();

        // Pre-trade validation
        let wallet_pubkey = self.wallet_manager.get_pubkey().await?;
        self.pre_trade_validator
            .validate_trade(&request, &wallet_pubkey)
            .await
            .map_err(|e| ExecutorError::ValidationError(e.to_string()))?;

        // Simulate transaction first
        let simulation_result = self.simulate_trade(&request).await?;
        if !simulation_result.is_successful {
            return Err(ExecutorError::SimulationFailed(simulation_result.error));
        }

        // Build transaction
        let mut transaction = self.transaction_builder
            .build_swap_transaction(&request)
            .await?;

        // Sign transaction
        let signed_tx = self.wallet_manager
            .sign_transaction(transaction)
            .await
            .map_err(|e| ExecutorError::SigningError(e.to_string()))?;

        // Send transaction with retry
        let signature = self.send_with_retry(&signed_tx, 3).await?;

        // Monitor confirmation
        let confirmation = self.confirmation_monitor
            .wait_for_confirmation(&signature, Duration::from_secs(30))
            .await?;

        // Record trade in QuestDB
        let execution_result = ExecutionResult {
            signature,
            amount_received: confirmation.amount_received,
            price: confirmation.executed_price,
            slippage_bps: self.calculate_slippage(&request, confirmation.executed_price),
            total_fees: confirmation.total_fees,
            priority_fee: confirmation.priority_fee,
            mev_protected: true,
            confirmation_time_ms: confirmation.confirmation_time_ms,
            execution_time_ms: start.elapsed().as_millis() as u64,
        };

        self.trade_recorder.record_execution(&request, &execution_result).await?;

        Ok(execution_result)
    }

    async fn simulate_trade(&self, request: &TradeRequest) -> Result<SimulationResult, ExecutorError> {
        let transaction = self.transaction_builder
            .build_swap_transaction(request)
            .await?;

        let config = RpcSimulateTransactionConfig {
            sig_verify: false,
            replace_recent_blockhash: true,
            commitment: Some(CommitmentConfig::processed()),
            accounts: Some(RpcSimulateTransactionAccountsConfig {
                encoding: Some(UiAccountEncoding::Base64),
                addresses: vec![],
            }),
        };

        let result = self.solana_client
            .simulate_transaction_with_config(&transaction, config)
            .await
            .map_err(|e| ExecutorError::SimulationError(e.to_string()))?;

        Ok(SimulationResult {
            is_successful: result.value.err.is_none(),
            error: result.value.err.map(|e| e.to_string()).unwrap_or_default(),
            units_consumed: result.value.units_consumed.unwrap_or(0),
            logs: result.value.logs.unwrap_or_default(),
        })
    }

    async fn send_with_retry(
        &self,
        transaction: &Transaction,
        max_retries: u32,
    ) -> Result<Signature, ExecutorError> {
        let mut last_error = None;
        
        for attempt in 0..max_retries {
            match self.solana_client.send_transaction(transaction).await {
                Ok(signature) => return Ok(signature),
                Err(e) => {
                    last_error = Some(e);
                    
                    // Don't retry on certain errors
                    if Self::is_permanent_error(&last_error) {
                        break;
                    }

                    // Exponential backoff
                    let delay = Duration::from_millis(100 * 2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(ExecutorError::SendTransactionError(
            last_error.map(|e| e.to_string()).unwrap_or_default()
        ))
    }

    fn is_permanent_error(error: &Option<ClientError>) -> bool {
        if let Some(err) = error {
            // Check for permanent errors like insufficient funds, invalid transaction, etc.
            err.to_string().contains("insufficient") ||
            err.to_string().contains("invalid") ||
            err.to_string().contains("account not found")
        } else {
            false
        }
    }

    fn calculate_slippage(&self, request: &TradeRequest, executed_price: f64) -> u16 {
        let expected_price = request.expected_price;
        let slippage = ((executed_price - expected_price) / expected_price).abs();
        (slippage * 10000.0) as u16 // Convert to basis points
    }
}
```

### 5. Transaction Confirmation Monitoring

```rust
pub struct ConfirmationMonitor {
    solana_client: Arc<SolanaClient>,
    metrics_collector: Arc<MetricsCollector>,
}

impl ConfirmationMonitor {
    pub async fn wait_for_confirmation(
        &self,
        signature: &Signature,
        timeout: Duration,
    ) -> Result<ConfirmationDetails, ExecutorError> {
        let start = Instant::now();
        let deadline = start + timeout;

        loop {
            // Check if timeout exceeded
            if Instant::now() > deadline {
                return Err(ExecutorError::ConfirmationTimeout);
            }

            // Get transaction status
            match self.get_transaction_details(signature).await {
                Ok(Some(details)) => {
                    if details.is_finalized {
                        self.metrics_collector.record_confirmation_time(
                            start.elapsed().as_millis() as u64
                        );
                        return Ok(details);
                    }
                }
                Ok(None) => {
                    // Transaction not found yet, continue polling
                }
                Err(e) => {
                    warn!("Error checking transaction status: {}", e);
                }
            }

            // Poll interval with exponential backoff
            let elapsed = start.elapsed().as_secs();
            let poll_interval = if elapsed < 5 {
                Duration::from_millis(100) // Fast polling initially
            } else if elapsed < 15 {
                Duration::from_millis(500) // Medium polling
            } else {
                Duration::from_secs(1) // Slow polling
            };

            tokio::time::sleep(poll_interval).await;
        }
    }

    async fn get_transaction_details(
        &self,
        signature: &Signature,
    ) -> Result<Option<ConfirmationDetails>, ExecutorError> {
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: Some(CommitmentConfig::finalized()),
            max_supported_transaction_version: Some(0),
        };

        match self.solana_client
            .get_transaction_with_config(signature, config)
            .await
        {
            Ok(response) => {
                let details = self.parse_transaction_details(response)?;
                Ok(Some(details))
            }
            Err(e) if e.to_string().contains("not found") => Ok(None),
            Err(e) => Err(ExecutorError::RpcError(e.to_string())),
        }
    }

    fn parse_transaction_details(
        &self,
        response: EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<ConfirmationDetails, ExecutorError> {
        let meta = response.transaction.meta
            .ok_or_else(|| ExecutorError::ParseError("Missing transaction meta".to_string()))?;

        // Extract execution details from logs and balance changes
        // This is simplified - real implementation would parse Jupiter swap events
        let details = ConfirmationDetails {
            is_finalized: response.slot.is_some(),
            slot: response.slot.unwrap_or(0),
            amount_received: 0, // Parse from logs
            executed_price: 0.0, // Calculate from amounts
            total_fees: meta.fee,
            priority_fee: 0, // Extract from compute budget
            confirmation_time_ms: 0, // Set by caller
            error: meta.err.map(|e| format!("{:?}", e)),
        };

        Ok(details)
    }
}
```

### 6. Trade Recording in QuestDB

```rust
pub struct TradeRecorder {
    questdb_pool: Arc<QuestDbPool>,
    metrics_collector: Arc<MetricsCollector>,
}

impl TradeRecorder {
    pub async fn record_execution(
        &self,
        request: &TradeRequest,
        result: &ExecutionResult,
    ) -> Result<(), RecorderError> {
        let query = r#"
            INSERT INTO trades (
                timestamp, request_id, trader_id, mode, action,
                base_token, quote_token, amount, price, slippage,
                fee, priority_fee, transfer_fee, tx_signature,
                latency_ms, mev_protected, source_service,
                execution_time_ms, confirmation_time_ms
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19
            )
        "#;

        let timestamp = Utc::now();
        let action = if request.input_token == "SOL" { "sell" } else { "buy" };

        self.questdb_pool
            .execute(query)
            .bind(timestamp)
            .bind(&request.request_id)
            .bind(&request.source_service)
            .bind("live")
            .bind(action)
            .bind(&request.input_token)
            .bind(&request.output_token)
            .bind(request.amount_lamports as f64 / 1e9)
            .bind(result.price)
            .bind(result.slippage_bps as f64 / 10000.0)
            .bind(result.total_fees as f64 / 1e9)
            .bind(result.priority_fee)
            .bind(0.0) // transfer_fee - TODO: extract from transaction
            .bind(result.signature.to_string())
            .bind(result.execution_time_ms as i32)
            .bind(result.mev_protected)
            .bind(&request.source_service)
            .bind(result.execution_time_ms)
            .bind(result.confirmation_time_ms)
            .await
            .map_err(|e| RecorderError::DatabaseError(e.to_string()))?;

        // Record metrics
        self.metrics_collector.record_trade_execution(result);

        Ok(())
    }

    pub async fn record_failed_attempt(
        &self,
        request: &TradeRequest,
        error: &ExecutorError,
    ) -> Result<(), RecorderError> {
        let query = r#"
            INSERT INTO failed_trades (
                timestamp, request_id, trader_id, error_type,
                error_message, base_token, quote_token, amount
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#;

        self.questdb_pool
            .execute(query)
            .bind(Utc::now())
            .bind(&request.request_id)
            .bind(&request.source_service)
            .bind(error.error_type())
            .bind(error.to_string())
            .bind(&request.input_token)
            .bind(&request.output_token)
            .bind(request.amount_lamports as f64 / 1e9)
            .await
            .map_err(|e| RecorderError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
```

### 7. Error Handling and Recovery

```rust
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,
    
    #[error("Jupiter API error: {0}")]
    JupiterError(String),
    
    #[error("Solana RPC error: {0}")]
    SolanaError(String),
    
    #[error("Transaction signing error: {0}")]
    SigningError(String),
    
    #[error("Simulation failed: {0}")]
    SimulationFailed(String),
    
    #[error("Send transaction error: {0}")]
    SendTransactionError(String),
    
    #[error("Confirmation timeout")]
    ConfirmationTimeout,
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl ExecutorError {
    pub fn error_type(&self) -> &'static str {
        match self {
            Self::ValidationError(_) => "validation",
            Self::InsufficientBalance => "insufficient_balance",
            Self::CircuitBreakerOpen => "circuit_breaker",
            Self::JupiterError(_) => "jupiter_error",
            Self::SolanaError(_) => "solana_error",
            Self::SigningError(_) => "signing_error",
            Self::SimulationFailed(_) => "simulation_failed",
            Self::SendTransactionError(_) => "send_failed",
            Self::ConfirmationTimeout => "confirmation_timeout",
            Self::ParseError(_) => "parse_error",
            Self::DatabaseError(_) => "database_error",
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::SolanaError(_) | 
            Self::SendTransactionError(_) | 
            Self::ConfirmationTimeout |
            Self::JupiterError(_)
        )
    }
}

// Retry logic helper
pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_attempts: u32,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> BoxFuture<'static, Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;
    
    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == max_attempts {
                    return Err(e);
                }
                
                warn!("Attempt {} failed: {}. Retrying in {:?}", attempt, e, delay);
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
    
    unreachable!()
}
```

### 8. Audit Trail and Compliance

```rust
pub struct AuditLogger {
    db_pool: Arc<PgPool>,
}

impl AuditLogger {
    pub async fn log_trade_request(&self, request: &TradeRequest) {
        let query = r#"
            INSERT INTO trade_audit_log (
                timestamp, event_type, request_id, source_service,
                input_token, output_token, amount, request_data
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#;

        let _ = sqlx::query(query)
            .bind(Utc::now())
            .bind("trade_request_received")
            .bind(&request.request_id)
            .bind(&request.source_service)
            .bind(&request.input_token)
            .bind(&request.output_token)
            .bind(request.amount_lamports as i64)
            .bind(serde_json::to_value(request).unwrap_or_default())
            .execute(&*self.db_pool)
            .await;
    }

    pub async fn log_execution_step(
        &self,
        request_id: &str,
        step: &str,
        details: serde_json::Value,
    ) {
        let query = r#"
            INSERT INTO trade_audit_log (
                timestamp, event_type, request_id, step_name, step_details
            ) VALUES ($1, $2, $3, $4, $5)
        "#;

        let _ = sqlx::query(query)
            .bind(Utc::now())
            .bind("execution_step")
            .bind(request_id)
            .bind(step)
            .bind(details)
            .execute(&*self.db_pool)
            .await;
    }

    pub async fn log_trade_result(
        &self,
        request: &TradeRequest,
        result: Result<&ExecutionResult, &ExecutorError>,
    ) {
        let (event_type, details) = match result {
            Ok(res) => ("trade_success", serde_json::to_value(res).unwrap_or_default()),
            Err(err) => ("trade_failed", json!({
                "error": err.to_string(),
                "error_type": err.error_type(),
                "is_retryable": err.is_retryable()
            })),
        };

        let query = r#"
            INSERT INTO trade_audit_log (
                timestamp, event_type, request_id, result_data
            ) VALUES ($1, $2, $3, $4)
        "#;

        let _ = sqlx::query(query)
            .bind(Utc::now())
            .bind(event_type)
            .bind(&request.request_id)
            .bind(details)
            .execute(&*self.db_pool)
            .await;
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
    async fn test_priority_fee_calculation() {
        let calculator = create_test_calculator();

        // Test different urgency levels
        let low_fee = calculator.calculate_fee(UrgencyLevel::Low).await.unwrap();
        let high_fee = calculator.calculate_fee(UrgencyLevel::High).await.unwrap();

        assert!(low_fee >= 1000);
        assert!(high_fee > low_fee);
        assert!(high_fee <= 10000);
    }

    #[tokio::test]
    async fn test_request_validation() {
        let validator = create_test_validator();

        // Valid request
        let valid_request = TradeRequest {
            request_id: "test-123".to_string(),
            input_token: "So11111111111111111111111111111111111111112".to_string(),
            output_token: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            amount_lamports: 1_000_000_000,
            max_slippage_bps: 50,
            ..Default::default()
        };
        assert!(validator.validate(&valid_request).await.is_ok());

        // Invalid token
        let invalid_request = TradeRequest {
            input_token: "invalid".to_string(),
            ..valid_request.clone()
        };
        assert!(matches!(
            validator.validate(&invalid_request).await,
            Err(ValidationError::InvalidToken(_))
        ));
    }

    #[tokio::test]
    async fn test_transaction_simulation() {
        let executor = create_test_executor();
        let request = create_test_trade_request();

        let simulation = executor.simulate_trade(&request).await.unwrap();
        assert!(simulation.is_successful);
        assert!(simulation.units_consumed > 0);
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let mut attempt_count = 0;
        
        let result = retry_with_backoff(|| {
            attempt_count += 1;
            Box::pin(async move {
                if attempt_count < 3 {
                    Err("Temporary error")
                } else {
                    Ok("Success")
                }
            })
        }, 5, Duration::from_millis(10))
        .await;

        assert_eq!(result.unwrap(), "Success");
        assert_eq!(attempt_count, 3);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_grpc_trade_execution() {
    let service = create_test_service().await;

    // Start gRPC server
    let addr = "127.0.0.1:50051".parse().unwrap();
    let server = Server::builder()
        .add_service(TradeExecutorServer::new(service))
        .serve(addr);

    tokio::spawn(server);

    // Create client
    let mut client = TradeExecutorClient::connect("http://127.0.0.1:50051")
        .await
        .unwrap();

    // Send trade request
    let request = tonic::Request::new(TradeRequest {
        request_id: "test-123".to_string(),
        source_service: "test_client".to_string(),
        input_token: SOL_MINT.to_string(),
        output_token: USDC_MINT.to_string(),
        amount_lamports: 100_000_000, // 0.1 SOL
        max_slippage_bps: 50,
        urgency_level: UrgencyLevel::Normal as i32,
        ..Default::default()
    });

    let response = client.execute_trade_request(request).await.unwrap();
    let result = response.into_inner();

    assert_eq!(result.status, TradeStatus::Success as i32);
    assert!(result.transaction_signature.is_some());
    assert!(result.executed_amount > 0);
}

#[tokio::test]
async fn test_circuit_breaker_integration() {
    let executor = create_test_executor();
    let circuit_breaker = executor.circuit_breaker.clone();

    // Close circuit breaker
    circuit_breaker.trip().await;

    // Trade should fail
    let request = create_test_trade_request();
    let result = executor.execute_trade(request).await;

    assert!(matches!(result, Err(ExecutorError::CircuitBreakerOpen)));

    // Reset circuit breaker
    circuit_breaker.reset().await;

    // Trade should work now
    let request = create_test_trade_request();
    let result = executor.execute_trade(request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_confirmation_monitoring() {
    let monitor = create_test_confirmation_monitor();
    let client = create_test_solana_client();

    // Send a test transaction
    let tx = create_test_transaction();
    let signature = client.send_transaction(&tx).await.unwrap();

    // Monitor confirmation
    let confirmation = monitor
        .wait_for_confirmation(&signature, Duration::from_secs(30))
        .await
        .unwrap();

    assert!(confirmation.is_finalized);
    assert!(confirmation.slot > 0);
    assert!(confirmation.error.is_none());
}
```

### End-to-End Tests

```rust
#[tokio::test]
async fn test_full_trade_flow() {
    let test_env = setup_test_environment().await;

    // Create trade request
    let request = TradeRequest {
        request_id: Uuid::new_v4().to_string(),
        source_service: "e2e_test".to_string(),
        input_token: SOL_MINT.to_string(),
        output_token: USDC_MINT.to_string(),
        amount_lamports: 1_000_000_000, // 1 SOL
        max_slippage_bps: 100, // 1%
        urgency_level: UrgencyLevel::Normal as i32,
        expected_price: 150.0,
        ..Default::default()
    };

    // Execute via gRPC
    let response = test_env.client
        .execute_trade_request(tonic::Request::new(request.clone()))
        .await
        .unwrap();

    let result = response.into_inner();

    // Verify result
    assert_eq!(result.request_id, request.request_id);
    assert_eq!(result.status, TradeStatus::Success as i32);
    assert!(result.transaction_signature.is_some());
    assert!(result.mev_protected);

    // Verify in database
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    let trade = test_env.db
        .get_trade_by_request_id(&request.request_id)
        .await
        .unwrap();

    assert_eq!(trade.tx_signature, result.transaction_signature);
    assert_eq!(trade.mode, "live");
    assert!(trade.mev_protected);

    // Verify audit log
    let audit_logs = test_env.db
        .get_audit_logs_for_request(&request.request_id)
        .await
        .unwrap();

    assert!(audit_logs.iter().any(|log| log.event_type == "trade_request_received"));
    assert!(audit_logs.iter().any(|log| log.event_type == "trade_success"));
}
```

## Dependencies

- **Task 1**: Common libraries for trade models and MEV structures
- **Task 3**: Database setup for QuestDB integration
- **Task 6**: gRPC service setup for external communication
- **Task 16**: Wallet manager for secure transaction signing

## Integration Points

- **External Services**: Receives TradeRequest objects via gRPC
- **Jupiter API**: Constructs swap transactions with MEV protection
- **Solana Network**: Executes and monitors transactions
- **QuestDB**: Records all trade executions and metrics
- **Circuit Breaker**: Integrates with system health monitoring
- **Risk Manager**: Validates technical execution parameters

## Performance Considerations

- **Transaction Building**: <50ms including Jupiter API call
- **Pre-trade Validation**: <10ms for all checks
- **Transaction Signing**: <10ms via wallet manager
- **Total Execution**: <100ms to transaction broadcast
- **Confirmation Monitoring**: Asynchronous with timeout

## Security Considerations

- All trade requests validated for format and limits
- Transaction simulation before execution
- Secure signing via wallet manager
- Comprehensive audit trail for compliance
- Circuit breaker prevents execution during issues
- MEV protection on all transactions

## Future Enhancements

- Multi-signature support for large trades
- Advanced MEV protection strategies
- Direct mempool submission for priority
- Cross-chain bridge integration
- Advanced order types (TWAP, VWAP)
- ML-based slippage prediction