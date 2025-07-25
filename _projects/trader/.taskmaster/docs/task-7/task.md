# Task 7: Develop Virtual Portfolio for Paper Trading

## Overview

This task implements a comprehensive virtual portfolio system for paper trading that accurately simulates real trading conditions. The system tracks balances, positions, and P&L calculations while accounting for fees, slippage, and Token-2022 extensions. This enables traders to validate strategies with high confidence before deploying real capital.

## Architecture Context

The virtual portfolio is a core component of the paper trading system:

- **Realistic Simulation**: Mirrors live trading with identical fee structures and slippage models
- **Real-time Updates**: Uses Redis-cached prices for live P&L calculations
- **Token-2022 Support**: Handles transfer fees and other token extensions
- **MEV Impact Tracking**: Records simulated MEV losses for strategy validation

This component directly supports the PRD's goal of achieving 85-90% accuracy between paper and live trading results.

## Implementation Details

### 1. Virtual Wallet Implementation

```rust
use rust_decimal::Decimal;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub symbol: String,
    pub mint: String,
    pub amount: Decimal,
    pub decimals: u8,
    pub value_usd: Option<Decimal>,
    pub has_transfer_fee: bool,
    pub transfer_fee_bps: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct VirtualWallet {
    balances: Arc<RwLock<HashMap<String, TokenBalance>>>,
    transaction_log: Arc<RwLock<Vec<WalletTransaction>>>,
    total_fees_paid: Arc<RwLock<HashMap<String, Decimal>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WalletTransaction {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub transaction_type: TransactionType,
    pub token: String,
    pub amount: Decimal,
    pub fee: Option<Decimal>,
    pub balance_after: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Trade,
    Fee,
    TransferFee,  // Token-2022 fees
}

impl VirtualWallet {
    pub fn new(initial_balances: HashMap<String, Decimal>) -> Self {
        let mut balances = HashMap::new();
        
        // Initialize with MVP tokens
        for (symbol, amount) in initial_balances {
            let token_info = Self::get_token_info(&symbol);
            balances.insert(symbol.clone(), TokenBalance {
                symbol: symbol.clone(),
                mint: token_info.mint,
                amount,
                decimals: token_info.decimals,
                value_usd: None,
                has_transfer_fee: token_info.has_transfer_fee,
                transfer_fee_bps: token_info.transfer_fee_bps,
            });
        }
        
        Self {
            balances: Arc::new(RwLock::new(balances)),
            transaction_log: Arc::new(RwLock::new(Vec::new())),
            total_fees_paid: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn update_balance(
        &self,
        token: &str,
        amount_change: Decimal,
        transaction_type: TransactionType,
    ) -> Result<Decimal, PortfolioError> {
        let mut balances = self.balances.write().await;
        
        let balance = balances.get_mut(token)
            .ok_or_else(|| PortfolioError::TokenNotFound(token.to_string()))?;
        
        // Check for sufficient balance on withdrawals
        if amount_change < Decimal::ZERO && balance.amount + amount_change < Decimal::ZERO {
            return Err(PortfolioError::InsufficientBalance {
                token: token.to_string(),
                available: balance.amount,
                required: -amount_change,
            });
        }
        
        // Apply Token-2022 transfer fee if applicable
        let effective_amount = if balance.has_transfer_fee && amount_change > Decimal::ZERO {
            let fee_bps = balance.transfer_fee_bps.unwrap_or(0);
            let fee = amount_change * Decimal::from(fee_bps) / Decimal::from(10000);
            
            // Record transfer fee
            let mut fees = self.total_fees_paid.write().await;
            *fees.entry(token.to_string()).or_insert(Decimal::ZERO) += fee;
            
            // Log transfer fee transaction
            self.transaction_log.write().await.push(WalletTransaction {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                transaction_type: TransactionType::TransferFee,
                token: token.to_string(),
                amount: -fee,
                fee: Some(fee),
                balance_after: balance.amount + amount_change - fee,
            });
            
            amount_change - fee
        } else {
            amount_change
        };
        
        // Update balance
        balance.amount += effective_amount;
        let new_balance = balance.amount;
        
        // Log transaction
        self.transaction_log.write().await.push(WalletTransaction {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            transaction_type,
            token: token.to_string(),
            amount: effective_amount,
            fee: None,
            balance_after: new_balance,
        });
        
        Ok(new_balance)
    }

    pub async fn get_balance(&self, token: &str) -> Result<Decimal, PortfolioError> {
        let balances = self.balances.read().await;
        balances.get(token)
            .map(|b| b.amount)
            .ok_or_else(|| PortfolioError::TokenNotFound(token.to_string()))
    }

    pub async fn get_all_balances(&self) -> HashMap<String, TokenBalance> {
        self.balances.read().await.clone()
    }

    pub async fn update_usd_values(&self, price_cache: &PriceCache) -> Result<Decimal, PortfolioError> {
        let mut balances = self.balances.write().await;
        let mut total_value = Decimal::ZERO;
        
        for balance in balances.values_mut() {
            if let Ok(Some(price_data)) = price_cache.get_price(&balance.symbol).await {
                let value = balance.amount * price_data.price;
                balance.value_usd = Some(value);
                total_value += value;
            }
        }
        
        Ok(total_value)
    }

    fn get_token_info(symbol: &str) -> TokenInfo {
        // MVP token information
        match symbol {
            "SOL" => TokenInfo {
                mint: "So11111111111111111111111111111111111111112".to_string(),
                decimals: 9,
                has_transfer_fee: false,
                transfer_fee_bps: None,
            },
            "USDC" => TokenInfo {
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                decimals: 6,
                has_transfer_fee: false,
                transfer_fee_bps: None,
            },
            "BONK" => TokenInfo {
                mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
                decimals: 5,
                has_transfer_fee: false,
                transfer_fee_bps: None,
            },
            "JitoSOL" => TokenInfo {
                mint: "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn".to_string(),
                decimals: 9,
                has_transfer_fee: false,
                transfer_fee_bps: None,
            },
            "RAY" => TokenInfo {
                mint: "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(),
                decimals: 6,
                has_transfer_fee: false,
                transfer_fee_bps: None,
            },
            _ => panic!("Unknown token: {}", symbol),
        }
    }
}

struct TokenInfo {
    mint: String,
    decimals: u8,
    has_transfer_fee: bool,
    transfer_fee_bps: Option<u16>,
}
```

### 2. Position Tracking System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub token: String,
    pub amount: Decimal,
    pub cost_basis: Decimal,  // Average price per token
    pub total_cost: Decimal,  // Total cost including fees
    pub entry_timestamp: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub realized_pnl: Decimal,
    pub trades: Vec<PositionTrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionTrade {
    pub trade_id: String,
    pub timestamp: DateTime<Utc>,
    pub action: TradeAction,
    pub amount: Decimal,
    pub price: Decimal,
    pub fee: Decimal,
    pub mev_loss: Option<Decimal>,
}

pub struct PositionManager {
    positions: Arc<RwLock<HashMap<String, Position>>>,
    closed_positions: Arc<RwLock<Vec<ClosedPosition>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClosedPosition {
    pub token: String,
    pub entry_timestamp: DateTime<Utc>,
    pub exit_timestamp: DateTime<Utc>,
    pub total_amount: Decimal,
    pub entry_price: Decimal,
    pub exit_price: Decimal,
    pub realized_pnl: Decimal,
    pub total_fees: Decimal,
    pub holding_period: Duration,
}

impl PositionManager {
    pub fn new() -> Self {
        Self {
            positions: Arc::new(RwLock::new(HashMap::new())),
            closed_positions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn update_position(
        &self,
        trade: &Trade,
    ) -> Result<(), PortfolioError> {
        let mut positions = self.positions.write().await;
        
        match trade.action {
            TradeAction::Buy | TradeAction::Swap => {
                let position = positions.entry(trade.base_token.clone()).or_insert_with(|| {
                    Position {
                        token: trade.base_token.clone(),
                        amount: Decimal::ZERO,
                        cost_basis: Decimal::ZERO,
                        total_cost: Decimal::ZERO,
                        entry_timestamp: trade.timestamp,
                        last_update: trade.timestamp,
                        realized_pnl: Decimal::ZERO,
                        trades: Vec::new(),
                    }
                });
                
                // Update weighted average cost basis
                let old_total_cost = position.amount * position.cost_basis;
                let new_cost = trade.amount * trade.price + trade.fee;
                let new_total_cost = old_total_cost + new_cost;
                let new_amount = position.amount + trade.amount;
                
                position.amount = new_amount;
                position.cost_basis = if new_amount > Decimal::ZERO {
                    new_total_cost / new_amount
                } else {
                    Decimal::ZERO
                };
                position.total_cost = new_total_cost;
                position.last_update = trade.timestamp;
                
                // Record trade
                position.trades.push(PositionTrade {
                    trade_id: trade.id.to_string(),
                    timestamp: trade.timestamp,
                    action: trade.action.clone(),
                    amount: trade.amount,
                    price: trade.price,
                    fee: trade.fee,
                    mev_loss: if trade.mev_protected {
                        None
                    } else {
                        Some(trade.amount * trade.price * Decimal::from_str("0.002").unwrap()) // 0.2% MEV loss estimate
                    },
                });
            }
            TradeAction::Sell => {
                let position = positions.get_mut(&trade.base_token)
                    .ok_or_else(|| PortfolioError::NoPosition(trade.base_token.clone()))?;
                
                if trade.amount > position.amount {
                    return Err(PortfolioError::InsufficientPosition {
                        token: trade.base_token.clone(),
                        available: position.amount,
                        required: trade.amount,
                    });
                }
                
                // Calculate realized P&L for this trade
                let sale_value = trade.amount * trade.price - trade.fee;
                let cost_basis_for_sold = trade.amount * position.cost_basis;
                let realized_pnl = sale_value - cost_basis_for_sold;
                
                position.amount -= trade.amount;
                position.realized_pnl += realized_pnl;
                position.last_update = trade.timestamp;
                
                // Record trade
                position.trades.push(PositionTrade {
                    trade_id: trade.id.to_string(),
                    timestamp: trade.timestamp,
                    action: trade.action.clone(),
                    amount: -trade.amount,  // Negative for sells
                    price: trade.price,
                    fee: trade.fee,
                    mev_loss: None,
                });
                
                // Close position if fully sold
                if position.amount <= Decimal::ZERO {
                    let closed = ClosedPosition {
                        token: position.token.clone(),
                        entry_timestamp: position.entry_timestamp,
                        exit_timestamp: trade.timestamp,
                        total_amount: position.trades.iter()
                            .filter(|t| t.amount > Decimal::ZERO)
                            .map(|t| t.amount)
                            .sum(),
                        entry_price: position.cost_basis,
                        exit_price: trade.price,
                        realized_pnl: position.realized_pnl,
                        total_fees: position.trades.iter()
                            .map(|t| t.fee)
                            .sum(),
                        holding_period: trade.timestamp.signed_duration_since(position.entry_timestamp)
                            .to_std()
                            .unwrap_or_default(),
                    };
                    
                    self.closed_positions.write().await.push(closed);
                    positions.remove(&trade.base_token);
                }
            }
        }
        
        Ok(())
    }

    pub async fn get_position(&self, token: &str) -> Option<Position> {
        self.positions.read().await.get(token).cloned()
    }

    pub async fn get_all_positions(&self) -> Vec<Position> {
        self.positions.read().await.values().cloned().collect()
    }

    pub async fn calculate_unrealized_pnl(
        &self,
        price_cache: &PriceCache,
    ) -> Result<HashMap<String, PositionPnL>, PortfolioError> {
        let positions = self.positions.read().await;
        let mut pnl_map = HashMap::new();
        
        for (token, position) in positions.iter() {
            if let Ok(Some(price_data)) = price_cache.get_price(token).await {
                let current_value = position.amount * price_data.price;
                let cost_basis_value = position.amount * position.cost_basis;
                let unrealized_pnl = current_value - cost_basis_value;
                let unrealized_pnl_percent = if cost_basis_value > Decimal::ZERO {
                    (unrealized_pnl / cost_basis_value) * Decimal::from(100)
                } else {
                    Decimal::ZERO
                };
                
                pnl_map.insert(token.clone(), PositionPnL {
                    token: token.clone(),
                    amount: position.amount,
                    cost_basis: position.cost_basis,
                    current_price: price_data.price,
                    current_value,
                    unrealized_pnl,
                    unrealized_pnl_percent,
                    realized_pnl: position.realized_pnl,
                });
            }
        }
        
        Ok(pnl_map)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PositionPnL {
    pub token: String,
    pub amount: Decimal,
    pub cost_basis: Decimal,
    pub current_price: Decimal,
    pub current_value: Decimal,
    pub unrealized_pnl: Decimal,
    pub unrealized_pnl_percent: Decimal,
    pub realized_pnl: Decimal,
}
```

### 3. Virtual Portfolio Integration

```rust
pub struct VirtualPortfolio {
    wallet: Arc<VirtualWallet>,
    position_manager: Arc<PositionManager>,
    price_cache: Arc<PriceCache>,
    trade_history: Arc<RwLock<Vec<Trade>>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    slippage_model: Arc<dyn SlippageModel + Send + Sync>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PerformanceMetrics {
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub total_volume: Decimal,
    pub total_fees_paid: Decimal,
    pub total_mev_losses: Decimal,
    pub max_drawdown: Decimal,
    pub sharpe_ratio: Option<f64>,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioSnapshot {
    pub timestamp: DateTime<Utc>,
    pub total_value_usd: Decimal,
    pub balances: HashMap<String, TokenBalance>,
    pub positions: Vec<Position>,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub total_pnl: Decimal,
    pub total_pnl_percent: Decimal,
    pub performance: PerformanceMetrics,
}

impl VirtualPortfolio {
    pub fn new(
        initial_balances: HashMap<String, Decimal>,
        price_cache: Arc<PriceCache>,
        slippage_model: Arc<dyn SlippageModel + Send + Sync>,
    ) -> Self {
        Self {
            wallet: Arc::new(VirtualWallet::new(initial_balances)),
            position_manager: Arc::new(PositionManager::new()),
            price_cache,
            trade_history: Arc::new(RwLock::new(Vec::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics {
                start_time: Utc::now(),
                last_update: Utc::now(),
                ..Default::default()
            })),
            slippage_model,
        }
    }

    pub async fn execute_trade(&self, mut trade: Trade) -> Result<Trade, PortfolioError> {
        // Apply slippage model
        let simulated_price = self.slippage_model.apply_slippage(
            trade.price,
            trade.amount,
            trade.slippage,
            trade.mev_protected,
        ).await?;
        
        trade.price = simulated_price;
        
        // Calculate actual amounts with slippage
        match trade.action {
            TradeAction::Buy => {
                let quote_amount = trade.amount * trade.price + trade.fee;
                
                // Update wallet balances
                self.wallet.update_balance(
                    &trade.quote_token,
                    -quote_amount,
                    TransactionType::Trade,
                ).await?;
                
                self.wallet.update_balance(
                    &trade.base_token,
                    trade.amount,
                    TransactionType::Trade,
                ).await?;
            }
            TradeAction::Sell => {
                let quote_amount = trade.amount * trade.price - trade.fee;
                
                // Update wallet balances
                self.wallet.update_balance(
                    &trade.base_token,
                    -trade.amount,
                    TransactionType::Trade,
                ).await?;
                
                self.wallet.update_balance(
                    &trade.quote_token,
                    quote_amount,
                    TransactionType::Trade,
                ).await?;
            }
            TradeAction::Swap => {
                // For swaps, handle as sell + buy
                // This is simplified - real implementation would use Jupiter quote
                self.wallet.update_balance(
                    &trade.base_token,
                    -trade.amount,
                    TransactionType::Trade,
                ).await?;
                
                let output_amount = trade.amount * trade.price - trade.fee;
                self.wallet.update_balance(
                    &trade.quote_token,
                    output_amount,
                    TransactionType::Trade,
                ).await?;
            }
        }
        
        // Update position
        self.position_manager.update_position(&trade).await?;
        
        // Record trade
        self.trade_history.write().await.push(trade.clone());
        
        // Update performance metrics
        let mut metrics = self.performance_metrics.write().await;
        metrics.total_trades += 1;
        metrics.total_volume += trade.amount * trade.price;
        metrics.total_fees_paid += trade.fee;
        
        if !trade.mev_protected {
            metrics.total_mev_losses += trade.amount * trade.price * Decimal::from_str("0.002").unwrap();
        }
        
        metrics.last_update = Utc::now();
        
        Ok(trade)
    }

    pub async fn get_snapshot(&self) -> Result<PortfolioSnapshot, PortfolioError> {
        // Update USD values
        let total_balance_value = self.wallet.update_usd_values(&self.price_cache).await?;
        
        // Get positions and calculate P&L
        let positions = self.position_manager.get_all_positions().await;
        let position_pnl = self.position_manager.calculate_unrealized_pnl(&self.price_cache).await?;
        
        let unrealized_pnl: Decimal = position_pnl.values()
            .map(|p| p.unrealized_pnl)
            .sum();
        
        let realized_pnl: Decimal = position_pnl.values()
            .map(|p| p.realized_pnl)
            .sum();
        
        let position_value: Decimal = position_pnl.values()
            .map(|p| p.current_value)
            .sum();
        
        let total_value = total_balance_value + position_value;
        let total_pnl = unrealized_pnl + realized_pnl;
        
        // Calculate initial value (assumes 10000 USDC start)
        let initial_value = Decimal::from(10000);
        let total_pnl_percent = if initial_value > Decimal::ZERO {
            (total_pnl / initial_value) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };
        
        Ok(PortfolioSnapshot {
            timestamp: Utc::now(),
            total_value_usd: total_value,
            balances: self.wallet.get_all_balances().await,
            positions,
            unrealized_pnl,
            realized_pnl,
            total_pnl,
            total_pnl_percent,
            performance: self.performance_metrics.read().await.clone(),
        })
    }

    pub async fn get_trade_history(&self) -> Vec<Trade> {
        self.trade_history.read().await.clone()
    }
}

// Slippage model trait
#[async_trait]
pub trait SlippageModel: Send + Sync {
    async fn apply_slippage(
        &self,
        base_price: Decimal,
        amount: Decimal,
        max_slippage_bps: u16,
        mev_protected: bool,
    ) -> Result<Decimal, PortfolioError>;
}

// Fixed slippage model for MVP
pub struct FixedSlippageModel {
    base_slippage_percent: Decimal,
}

#[async_trait]
impl SlippageModel for FixedSlippageModel {
    async fn apply_slippage(
        &self,
        base_price: Decimal,
        _amount: Decimal,
        _max_slippage_bps: u16,
        mev_protected: bool,
    ) -> Result<Decimal, PortfolioError> {
        let slippage = if mev_protected {
            self.base_slippage_percent
        } else {
            self.base_slippage_percent + Decimal::from_str("0.2").unwrap() // +0.2% MEV impact
        };
        
        Ok(base_price * (Decimal::ONE - slippage / Decimal::from(100)))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PortfolioError {
    #[error("Insufficient balance: {token} - available: {available}, required: {required}")]
    InsufficientBalance {
        token: String,
        available: Decimal,
        required: Decimal,
    },
    #[error("Insufficient position: {token} - available: {available}, required: {required}")]
    InsufficientPosition {
        token: String,
        available: Decimal,
        required: Decimal,
    },
    #[error("Token not found: {0}")]
    TokenNotFound(String),
    #[error("No position for token: {0}")]
    NoPosition(String),
    #[error("Price data unavailable for token: {0}")]
    PriceUnavailable(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_virtual_wallet_balance_updates() {
        let mut initial = HashMap::new();
        initial.insert("SOL".to_string(), Decimal::from(10));
        initial.insert("USDC".to_string(), Decimal::from(1000));
        
        let wallet = VirtualWallet::new(initial);
        
        // Test deposit
        let new_balance = wallet.update_balance(
            "SOL",
            Decimal::from(5),
            TransactionType::Deposit,
        ).await.unwrap();
        assert_eq!(new_balance, Decimal::from(15));
        
        // Test withdrawal
        let new_balance = wallet.update_balance(
            "USDC",
            Decimal::from(-100),
            TransactionType::Trade,
        ).await.unwrap();
        assert_eq!(new_balance, Decimal::from(900));
        
        // Test insufficient balance
        let result = wallet.update_balance(
            "SOL",
            Decimal::from(-20),
            TransactionType::Trade,
        ).await;
        assert!(matches!(result, Err(PortfolioError::InsufficientBalance { .. })));
    }

    #[tokio::test]
    async fn test_position_tracking() {
        let position_manager = PositionManager::new();
        
        // Buy trade
        let buy_trade = Trade {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            action: TradeAction::Buy,
            base_token: "SOL".to_string(),
            quote_token: "USDC".to_string(),
            amount: Decimal::from(10),
            price: Decimal::from(150),
            fee: Decimal::from(2),
            slippage: Decimal::from_str("0.1").unwrap(),
            priority_fee: Some(5000),
            tx_signature: None,
            transfer_fee: None,
            extension_data: None,
            mev_protected: true,
            latency_ms: 100,
        };
        
        position_manager.update_position(&buy_trade).await.unwrap();
        
        let position = position_manager.get_position("SOL").await.unwrap();
        assert_eq!(position.amount, Decimal::from(10));
        assert_eq!(position.cost_basis, Decimal::from_str("150.2").unwrap()); // (10 * 150 + 2) / 10
        
        // Partial sell
        let sell_trade = Trade {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            action: TradeAction::Sell,
            base_token: "SOL".to_string(),
            quote_token: "USDC".to_string(),
            amount: Decimal::from(5),
            price: Decimal::from(160),
            fee: Decimal::from(1),
            ..buy_trade.clone()
        };
        
        position_manager.update_position(&sell_trade).await.unwrap();
        
        let position = position_manager.get_position("SOL").await.unwrap();
        assert_eq!(position.amount, Decimal::from(5));
        assert!(position.realized_pnl > Decimal::ZERO); // Should have profit
    }

    #[tokio::test]
    async fn test_pnl_calculation() {
        let price_cache = Arc::new(MockPriceCache::new());
        let position_manager = PositionManager::new();
        
        // Create position
        let trade = Trade {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            action: TradeAction::Buy,
            base_token: "SOL".to_string(),
            quote_token: "USDC".to_string(),
            amount: Decimal::from(10),
            price: Decimal::from(100),
            fee: Decimal::from(2),
            ..Default::default()
        };
        
        position_manager.update_position(&trade).await.unwrap();
        
        // Set current price
        price_cache.set_price("SOL", Decimal::from(120)).await;
        
        // Calculate P&L
        let pnl_map = position_manager.calculate_unrealized_pnl(&price_cache).await.unwrap();
        let sol_pnl = pnl_map.get("SOL").unwrap();
        
        assert_eq!(sol_pnl.current_price, Decimal::from(120));
        assert_eq!(sol_pnl.current_value, Decimal::from(1200)); // 10 * 120
        assert!(sol_pnl.unrealized_pnl > Decimal::from(195)); // ~200 profit minus fees
        assert!(sol_pnl.unrealized_pnl_percent > Decimal::from(19)); // ~20% profit
    }

    #[tokio::test]
    async fn test_token_2022_fees() {
        let mut initial = HashMap::new();
        initial.insert("TRANSFER_FEE_TOKEN".to_string(), Decimal::from(1000));
        
        let wallet = VirtualWallet::new(initial);
        
        // Simulate token with 100 bps (1%) transfer fee
        let mut balances = wallet.balances.write().await;
        balances.get_mut("TRANSFER_FEE_TOKEN").unwrap().has_transfer_fee = true;
        balances.get_mut("TRANSFER_FEE_TOKEN").unwrap().transfer_fee_bps = Some(100);
        drop(balances);
        
        // Deposit with transfer fee
        let new_balance = wallet.update_balance(
            "TRANSFER_FEE_TOKEN",
            Decimal::from(100),
            TransactionType::Deposit,
        ).await.unwrap();
        
        // Should receive 99 after 1% fee
        assert_eq!(new_balance, Decimal::from(1099)); // 1000 + 100 - 1
        
        // Check fee tracking
        let fees = wallet.total_fees_paid.read().await;
        assert_eq!(fees.get("TRANSFER_FEE_TOKEN"), Some(&Decimal::from(1)));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_trading_flow() {
    // Initialize portfolio
    let mut initial_balances = HashMap::new();
    initial_balances.insert("SOL".to_string(), Decimal::from(0));
    initial_balances.insert("USDC".to_string(), Decimal::from(10000));
    
    let price_cache = Arc::new(MockPriceCache::new());
    let slippage_model = Arc::new(FixedSlippageModel {
        base_slippage_percent: Decimal::from_str("0.5").unwrap(),
    });
    
    let portfolio = VirtualPortfolio::new(initial_balances, price_cache.clone(), slippage_model);
    
    // Set initial prices
    price_cache.set_price("SOL", Decimal::from(100)).await;
    price_cache.set_price("USDC", Decimal::from(1)).await;
    
    // Execute buy trade
    let buy_trade = Trade {
        id: uuid::Uuid::new_v4(),
        timestamp: Utc::now(),
        action: TradeAction::Buy,
        base_token: "SOL".to_string(),
        quote_token: "USDC".to_string(),
        amount: Decimal::from(10),
        price: Decimal::from(100),
        fee: Decimal::from(2),
        slippage: Decimal::from_str("0.5").unwrap(),
        mev_protected: true,
        ..Default::default()
    };
    
    let executed_trade = portfolio.execute_trade(buy_trade).await.unwrap();
    
    // Verify slippage applied
    assert!(executed_trade.price < Decimal::from(100));
    
    // Get snapshot
    let snapshot = portfolio.get_snapshot().await.unwrap();
    
    // Verify balances
    assert_eq!(snapshot.balances.get("SOL").unwrap().amount, Decimal::from(10));
    assert!(snapshot.balances.get("USDC").unwrap().amount < Decimal::from(9000)); // Spent ~1000
    
    // Update price and check P&L
    price_cache.set_price("SOL", Decimal::from(120)).await;
    
    let snapshot = portfolio.get_snapshot().await.unwrap();
    assert!(snapshot.unrealized_pnl > Decimal::ZERO);
    assert!(snapshot.total_pnl_percent > Decimal::ZERO);
}
```

## Dependencies

- **Task 1**: Uses Trade and MEV models
- **Task 2**: Stores trades in database
- **Task 5**: Uses Redis price cache for real-time valuations

## Integration Points

- **Paper Trader**: Core component for simulated trading
- **MEV Simulator**: Provides realistic slippage estimates
- **Price Feed**: Updates portfolio valuations
- **TUI**: Displays portfolio snapshot and P&L

## Performance Considerations

- All balance updates are O(1) operations
- Position tracking uses efficient weighted average
- P&L calculations cached where possible
- Concurrent read access via RwLock

## Future Enhancements

- Dynamic slippage based on order book depth
- Multi-asset portfolio optimization
- Risk metrics (Sharpe ratio, max drawdown)
- Tax lot accounting for realized gains
- Performance attribution analysis