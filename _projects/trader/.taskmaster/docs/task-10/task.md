# Task 10: Develop Terminal-Based User Interface (TUI)

## Overview
This task implements a sophisticated terminal-based user interface (TUI) for the Solana trading platform using Rust's ratatui library. The interface provides real-time visual feedback, keyboard navigation, and performance monitoring with a 10Hz refresh rate via Redis Streams.

## Architecture Context
According to the architecture.md, the TUI serves as the primary interface for traders to monitor positions, execute trades, and track system health. It integrates with:
- Redis Streams for real-time event updates (10Hz refresh rate)
- Virtual Portfolio for position tracking
- System Health monitoring with circuit breaker status
- QuestDB for historical data queries

## Implementation Requirements

### 1. Core TUI Framework
Build the foundational terminal interface with:
- **Layout Management**: Split terminal into logical panels (header, main content, footer)
- **Tab Navigation**: Portfolio, Trades, Order Entry, System Health views
- **Event Loop**: Dual handling of keyboard input and data updates
- **10Hz Refresh Rate**: Render updates every 100ms without flicker

### 2. Visual Components

#### P&L Sparkline Charts
```rust
pub fn render_pnl_chart(&self, f: &mut Frame, area: Rect) {
    let portfolio = self.portfolio.blocking_read();
    let pnl_data: Vec<u64> = portfolio.pnl_history
        .iter()
        .map(|&pnl| ((pnl + 100.0) * 100.0) as u64)
        .collect();

    let sparkline = Sparkline::default()
        .block(Block::default()
            .title("P&L %")
            .borders(Borders::ALL))
        .data(&pnl_data)
        .style(Style::default().fg(
            if pnl_data.last().unwrap_or(&0) > &10000 { 
                Color::Green 
            } else { 
                Color::Red 
            }
        ));

    f.render_widget(sparkline, area);
}
```

#### Position List with Real-time Updates
```rust
pub fn render_positions(&self, f: &mut Frame, area: Rect) {
    let portfolio = self.portfolio.read().await;
    
    let positions: Vec<ListItem> = portfolio.positions.iter()
        .map(|(token, position)| {
            let pnl_percent = position.unrealized_pnl_percent;
            let style = if pnl_percent >= 0.0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            
            ListItem::new(format!(
                "{:<8} │ Amount: {:<12.4} │ Entry: ${:<8.4} │ Current: ${:<8.4} │ P&L: {:>+7.2}%",
                token,
                position.amount,
                position.cost_basis,
                position.current_price,
                pnl_percent * 100.0
            )).style(style)
        })
        .collect();
    
    let positions_widget = List::new(positions)
        .block(Block::default()
            .title("Active Positions")
            .borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray));
    
    f.render_widget(positions_widget, area);
}
```

#### Trade History with MEV Status
```rust
pub fn render_trades(&self, f: &mut Frame, area: Rect) {
    let trades = self.trade_history.lock().await;
    
    let rows: Vec<Row> = trades.iter()
        .rev()
        .take(20)
        .map(|trade| {
            let mev_indicator = if trade.mev_protected {
                "🛡️" 
            } else if trade.mev_impacted {
                "⚠️"
            } else {
                "  "
            };
            
            Row::new(vec![
                Cell::from(trade.timestamp.format("%H:%M:%S").to_string()),
                Cell::from(format!("{:?}", trade.action)),
                Cell::from(format!("{}/{}", trade.base_token, trade.quote_token)),
                Cell::from(format!("{:.4}", trade.amount)),
                Cell::from(format!("${:.4}", trade.price)),
                Cell::from(format!("{:.2}%", trade.slippage * 100.0)),
                Cell::from(mev_indicator),
            ])
        })
        .collect();
    
    let table = Table::new(rows)
        .header(Row::new(vec!["Time", "Action", "Pair", "Amount", "Price", "Slippage", "MEV"]))
        .block(Block::default()
            .title("Recent Trades")
            .borders(Borders::ALL))
        .widths(&[
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(4),
        ]);
    
    f.render_widget(table, area);
}
```

### 3. Real-time Data Integration

#### Redis Streams Event Subscriber
```rust
pub struct EventSubscriber {
    redis_conn: Arc<Mutex<redis::aio::Connection>>,
    stream_key: String,
    consumer_group: String,
}

impl EventSubscriber {
    pub async fn subscribe(&self) -> Result<impl Stream<Item = Event>> {
        let mut conn = self.redis_conn.lock().await;
        
        // Create consumer group if it doesn't exist
        let _: Result<(), _> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&self.stream_key)
            .arg(&self.consumer_group)
            .arg("$")
            .arg("MKSTREAM")
            .query_async(&mut *conn).await;
        
        // Return stream of events
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        let redis_conn = self.redis_conn.clone();
        let stream_key = self.stream_key.clone();
        let consumer_group = self.consumer_group.clone();
        
        tokio::spawn(async move {
            loop {
                let mut conn = redis_conn.lock().await;
                
                match redis::cmd("XREADGROUP")
                    .arg("GROUP")
                    .arg(&consumer_group)
                    .arg("consumer-1")
                    .arg("BLOCK")
                    .arg(100) // 100ms timeout for 10Hz
                    .arg("STREAMS")
                    .arg(&stream_key)
                    .arg(">")
                    .query_async::<_, Vec<StreamReadReply>>(&mut *conn).await
                {
                    Ok(replies) => {
                        for reply in replies {
                            for message in reply.messages {
                                if let Ok(event) = Event::from_redis_message(message) {
                                    let _ = tx.send(event).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Redis stream error: {}", e);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });
        
        Ok(ReceiverStream::new(rx))
    }
}
```

#### System Health Dashboard
```rust
pub fn render_system_health(&self, f: &mut Frame, area: Rect) {
    let health = self.system_health.lock().await;
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Circuit breaker status
            Constraint::Length(5),  // Latency metrics
            Constraint::Min(5),     // Node health
        ])
        .split(area);
    
    // Circuit breaker status
    let cb_status = match health.circuit_breaker_state {
        CircuitState::Closed => ("Closed", Color::Green, "Trading Active"),
        CircuitState::Open => ("Open", Color::Red, "Trading PAUSED - High Latency"),
        CircuitState::HalfOpen => ("Half-Open", Color::Yellow, "Testing Recovery"),
    };
    
    let cb_widget = Paragraph::new(format!("Circuit Breaker: {} - {}", cb_status.0, cb_status.2))
        .style(Style::default().fg(cb_status.1))
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(cb_widget, chunks[0]);
    
    // Latency metrics
    let latency_text = vec![
        Line::from(format!("Node Latency P99: {}ms", health.node_latency_p99.as_millis())),
        Line::from(format!("Trade Execution P99: {}ms", health.trade_latency_p99.as_millis())),
        Line::from(format!("Jupiter Response: {}ms", health.jupiter_latency.as_millis())),
    ];
    
    let latency_widget = Paragraph::new(latency_text)
        .block(Block::default()
            .title("Latency Metrics")
            .borders(Borders::ALL));
    
    f.render_widget(latency_widget, chunks[1]);
}
```

### 4. Keyboard Navigation

```rust
impl TradingTui {
    async fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Tab => {
                self.active_tab = match self.active_tab {
                    Tab::Portfolio => Tab::Trades,
                    Tab::Trades => Tab::OrderEntry,
                    Tab::OrderEntry => Tab::SystemHealth,
                    Tab::SystemHealth => Tab::Portfolio,
                };
            }
            KeyCode::Char('q') => {
                return Err(Error::UserExit);
            }
            KeyCode::Char('r') => {
                // Force refresh
                self.force_refresh().await?;
            }
            KeyCode::Enter => {
                if self.active_tab == Tab::OrderEntry {
                    self.submit_order().await?;
                }
            }
            KeyCode::Up | KeyCode::Down => {
                if self.active_tab == Tab::Portfolio {
                    self.navigate_positions(key.code).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Performance Considerations

1. **Non-blocking Rendering**: Use `tokio::select!` to handle both UI updates and data events
2. **Buffered Updates**: Batch multiple data updates within the 100ms window
3. **Lazy Loading**: Only fetch visible data (e.g., last 20 trades)
4. **Memory Management**: Limit history buffers to prevent unbounded growth

## Integration Points

- **Virtual Portfolio**: Read-only access via Arc<RwLock<VirtualPortfolio>>
- **Redis Streams**: Subscribe to "ui-events" stream with 10Hz polling
- **System Health**: Monitor circuit breaker and latency metrics
- **Trade Executor**: Submit orders through shared Arc<PaperTradeExecutor>

## Error Handling

```rust
pub enum TuiError {
    Terminal(std::io::Error),
    Redis(redis::RedisError),
    Render(String),
    DataAccess(String),
}

impl TradingTui {
    async fn safe_render(&mut self) -> Result<(), TuiError> {
        match self.render().await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Log error but don't crash
                eprintln!("Render error: {}", e);
                // Attempt recovery
                self.reset_terminal()?;
                Ok(())
            }
        }
    }
}
```

## Testing Approach

1. **Mock Data Generators**: Create realistic position and trade data
2. **Event Stream Simulation**: Test with high-frequency Redis events
3. **Performance Benchmarks**: Verify 10Hz refresh without lag
4. **Keyboard Navigation Tests**: Validate all key combinations
5. **Error Recovery Tests**: Simulate terminal resize, data errors

## Dependencies
- Task 5: Redis Cache Implementation (for real-time data)
- Task 7: Virtual Portfolio Manager (for position data)