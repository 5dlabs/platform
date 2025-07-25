# Task 11: Implement Stop-Loss and Take-Profit Monitoring

## Overview
This task implements a high-frequency monitoring system that continuously checks positions against stop-loss and take-profit conditions, executing orders automatically when triggered. The system operates at 100ms intervals as specified in the PRD, providing rapid response to market movements.

## Architecture Context
According to the architecture.md, the monitoring system integrates with:
- Virtual Portfolio for position tracking
- Price Cache (Redis) for sub-millisecond price access
- Paper Trade Executor for automatic order execution
- QuestDB for recording triggered orders with metadata

## Implementation Requirements

### 1. Position Monitoring Service

The core monitoring service runs at precisely 100ms intervals:

```rust
pub struct OrderMonitor {
    portfolio: Arc<RwLock<VirtualPortfolio>>,
    price_cache: Arc<PriceCache>,
    trade_executor: Arc<PaperTradeExecutor>,
    orders: RwLock<HashMap<String, Vec<ConditionalOrder>>>,
    monitoring_handle: Option<JoinHandle<()>>,
}

impl OrderMonitor {
    pub async fn start_monitoring(&self) -> JoinHandle<()> {
        let portfolio = self.portfolio.clone();
        let price_cache = self.price_cache.clone();
        let trade_executor = self.trade_executor.clone();
        let orders = self.orders.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            
            loop {
                interval.tick().await;
                let start = Instant::now();
                
                // Get current positions
                let positions = portfolio.read().await.get_positions();
                
                // Check each position in parallel
                let mut tasks = Vec::new();
                for (token, position) in positions {
                    let price_cache = price_cache.clone();
                    let orders = orders.clone();
                    let trade_executor = trade_executor.clone();
                    
                    tasks.push(tokio::spawn(async move {
                        monitor_position(token, position, price_cache, orders, trade_executor).await
                    }));
                }
                
                // Wait for all checks to complete
                futures::future::join_all(tasks).await;
                
                // Log monitoring cycle time
                let elapsed = start.elapsed();
                if elapsed > Duration::from_millis(50) {
                    warn!("Monitoring cycle took {}ms", elapsed.as_millis());
                }
            }
        })
    }
}
```

### 2. Order Condition System

Support multiple order types with flexible conditions:

```rust
pub enum OrderCondition {
    StopLoss {
        trigger_price: Decimal,
        is_trailing: bool,
        trail_distance: Option<Decimal>, // Can be absolute or percentage
        high_water_mark: Option<Decimal>, // For trailing stops
    },
    TakeProfit {
        trigger_price: Decimal,
        partial_percentage: Option<Decimal>, // Allow partial profit taking
    },
    TimeBasedExit {
        trigger_after: Duration,
        created_at: DateTime<Utc>,
    },
    CompositeCondition {
        conditions: Vec<OrderCondition>,
        logic: ConditionLogic, // AND/OR
    },
}

pub struct ConditionalOrder {
    id: Uuid,
    token: String,
    condition: OrderCondition,
    action: OrderAction,
    amount: OrderAmount,
    created_at: DateTime<Utc>,
    metadata: HashMap<String, String>,
}

pub enum OrderAction {
    Sell { quote_token: String },
    Buy { quote_token: String },
    Swap { target_token: String },
}

pub enum OrderAmount {
    FullPosition,
    FixedAmount(Decimal),
    Percentage(Decimal),
}

impl OrderCondition {
    pub fn is_triggered(&self, current_price: Decimal, position: &Position) -> bool {
        match self {
            OrderCondition::StopLoss { trigger_price, is_trailing, trail_distance, high_water_mark } => {
                if *is_trailing {
                    // Update high water mark if price increased
                    let new_high = current_price.max(high_water_mark.unwrap_or(current_price));
                    let trail_amount = trail_distance.unwrap_or(Decimal::from(5)); // Default 5%
                    let stop_price = new_high * (Decimal::ONE - trail_amount / Decimal::from(100));
                    current_price <= stop_price
                } else {
                    current_price <= *trigger_price
                }
            }
            OrderCondition::TakeProfit { trigger_price, .. } => {
                current_price >= *trigger_price
            }
            OrderCondition::TimeBasedExit { trigger_after, created_at } => {
                Utc::now() - *created_at >= *trigger_after
            }
            OrderCondition::CompositeCondition { conditions, logic } => {
                match logic {
                    ConditionLogic::And => conditions.iter().all(|c| c.is_triggered(current_price, position)),
                    ConditionLogic::Or => conditions.iter().any(|c| c.is_triggered(current_price, position)),
                }
            }
        }
    }
}
```

### 3. Integration with Paper Trade Executor

Execute orders automatically when conditions are met:

```rust
async fn monitor_position(
    token: String,
    position: Position,
    price_cache: Arc<PriceCache>,
    orders: Arc<RwLock<HashMap<String, Vec<ConditionalOrder>>>>,
    trade_executor: Arc<PaperTradeExecutor>,
) -> Result<()> {
    // Get current price from cache (sub-millisecond access)
    let current_price = match price_cache.get_price(&token).await? {
        Some(price) => Decimal::from_f64(price).ok_or(Error::InvalidPrice)?,
        None => return Ok(()), // Skip if price not available
    };
    
    // Get orders for this token
    let mut orders_write = orders.write().await;
    let token_orders = match orders_write.get_mut(&token) {
        Some(orders) => orders,
        None => return Ok(()),
    };
    
    // Check each order condition
    let mut triggered_indices = Vec::new();
    for (i, order) in token_orders.iter_mut().enumerate() {
        // Update trailing stop high water mark if needed
        if let OrderCondition::StopLoss { high_water_mark, is_trailing: true, .. } = &mut order.condition {
            *high_water_mark = Some(current_price.max(high_water_mark.unwrap_or(current_price)));
        }
        
        if order.condition.is_triggered(current_price, &position) {
            triggered_indices.push(i);
        }
    }
    
    // Execute triggered orders (in reverse to preserve indices)
    for i in triggered_indices.into_iter().rev() {
        let order = token_orders.remove(i);
        
        // Calculate order amount
        let trade_amount = match order.amount {
            OrderAmount::FullPosition => position.amount,
            OrderAmount::FixedAmount(amount) => amount.to_f64().unwrap_or(0.0),
            OrderAmount::Percentage(pct) => position.amount * (pct / Decimal::from(100)).to_f64().unwrap_or(0.0),
        };
        
        // Create trade parameters
        let params = TradeParams {
            action: match order.action {
                OrderAction::Sell { .. } => TradeAction::Sell,
                OrderAction::Buy { .. } => TradeAction::Buy,
                OrderAction::Swap { .. } => TradeAction::Swap,
            },
            base_token: token.clone(),
            quote_token: extract_quote_token(&order.action),
            amount: trade_amount,
            is_base_input: true,
            simulate_mev: true,
            priority_fee: 5000, // Default priority fee
            metadata: Some(json!({
                "order_type": format!("{:?}", order.condition),
                "order_id": order.id.to_string(),
                "trigger_price": current_price.to_string(),
            })),
        };
        
        // Execute trade
        match trade_executor.execute_trade(params).await {
            Ok(result) => {
                info!(
                    "Executed conditional order {} for {}: {:?}",
                    order.id, token, result
                );
                
                // Record to event stream
                publish_order_executed_event(&order, &result).await?;
            }
            Err(e) => {
                error!(
                    "Failed to execute conditional order {} for {}: {}",
                    order.id, token, e
                );
            }
        }
    }
    
    Ok(())
}
```

### 4. Order Management API

Provide methods to add, update, and remove conditional orders:

```rust
impl OrderMonitor {
    pub async fn add_stop_loss(
        &self,
        token: &str,
        trigger_price: Decimal,
        is_trailing: bool,
        trail_percentage: Option<Decimal>,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let order = ConditionalOrder {
            id,
            token: token.to_string(),
            condition: OrderCondition::StopLoss {
                trigger_price,
                is_trailing,
                trail_distance: trail_percentage,
                high_water_mark: None,
            },
            action: OrderAction::Sell {
                quote_token: "USDC".to_string(),
            },
            amount: OrderAmount::FullPosition,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        self.orders.write().await
            .entry(token.to_string())
            .or_insert_with(Vec::new)
            .push(order);
        
        info!("Added stop-loss order {} for {}", id, token);
        Ok(id)
    }
    
    pub async fn add_take_profit(
        &self,
        token: &str,
        trigger_price: Decimal,
        partial_percentage: Option<Decimal>,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let order = ConditionalOrder {
            id,
            token: token.to_string(),
            condition: OrderCondition::TakeProfit {
                trigger_price,
                partial_percentage,
            },
            action: OrderAction::Sell {
                quote_token: "USDC".to_string(),
            },
            amount: match partial_percentage {
                Some(pct) => OrderAmount::Percentage(pct),
                None => OrderAmount::FullPosition,
            },
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        self.orders.write().await
            .entry(token.to_string())
            .or_insert_with(Vec::new)
            .push(order);
        
        info!("Added take-profit order {} for {}", id, token);
        Ok(id)
    }
    
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<bool> {
        let mut orders = self.orders.write().await;
        
        for token_orders in orders.values_mut() {
            if let Some(pos) = token_orders.iter().position(|o| o.id == order_id) {
                token_orders.remove(pos);
                info!("Cancelled order {}", order_id);
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    pub async fn get_active_orders(&self) -> Vec<ConditionalOrder> {
        let orders = self.orders.read().await;
        orders.values()
            .flat_map(|v| v.iter().cloned())
            .collect()
    }
}
```

### 5. Performance Optimization

To maintain 100ms monitoring intervals:

1. **Parallel Position Checks**: Process multiple positions concurrently
2. **Redis Cache**: Use sub-millisecond price lookups
3. **Skip Missing Prices**: Don't block on unavailable price data
4. **Efficient Data Structures**: Use HashMap for O(1) order lookups
5. **Async Execution**: Non-blocking trade execution

## Error Handling

```rust
pub enum MonitoringError {
    PriceUnavailable(String),
    TradeExecutionFailed(String),
    InvalidOrderCondition(String),
    MonitoringCycleSlow(Duration),
}

impl OrderMonitor {
    async fn handle_monitoring_error(&self, error: MonitoringError) {
        match error {
            MonitoringError::PriceUnavailable(token) => {
                // Log but continue monitoring other positions
                debug!("Price unavailable for {}, skipping", token);
            }
            MonitoringError::TradeExecutionFailed(msg) => {
                // Alert user but don't stop monitoring
                error!("Trade execution failed: {}", msg);
                self.send_alert("Trade Execution Failed", &msg).await;
            }
            MonitoringError::MonitoringCycleSlow(duration) => {
                // Performance warning
                warn!("Monitoring cycle slow: {:?}", duration);
                if duration > Duration::from_millis(200) {
                    self.send_alert("Performance Warning", "Monitoring lag detected").await;
                }
            }
            _ => {}
        }
    }
}
```

## Testing Strategy

1. **Unit Tests**: Test each order condition type with various scenarios
2. **Integration Tests**: Verify end-to-end order execution flow
3. **Performance Tests**: Ensure 100ms monitoring intervals under load
4. **Accuracy Tests**: Verify precise trigger price matching
5. **Concurrency Tests**: Multiple orders on same position

## Dependencies
- Task 7: Virtual Portfolio Manager (for position data)
- Task 9: Paper Trade Executor (for order execution)
- Task 5: Redis Cache Implementation (for price data)