# Task 10: Develop Terminal-Based User Interface (TUI) - Acceptance Criteria

## Functional Requirements

### 1. Core Framework
- [ ] TUI launches successfully and displays initial layout
- [ ] Tab navigation switches between all four views (Portfolio, Trades, Order Entry, System Health)
- [ ] Application maintains 10Hz refresh rate (verified by frame counter)
- [ ] No screen flicker or tearing during updates
- [ ] Terminal resize is handled gracefully without crashes

### 2. Portfolio View
- [ ] Displays all active positions with correct data:
  - [ ] Token symbol
  - [ ] Position amount
  - [ ] Entry price (cost basis)
  - [ ] Current price
  - [ ] Unrealized P&L in both value and percentage
- [ ] P&L values are color-coded (green for profit, red for loss)
- [ ] P&L sparkline chart updates with historical data
- [ ] Total portfolio value is calculated and displayed correctly
- [ ] Positions can be selected with arrow keys

### 3. Trade History View
- [ ] Shows last 20 trades in reverse chronological order
- [ ] Each trade displays:
  - [ ] Timestamp (HH:MM:SS format)
  - [ ] Action (Buy/Sell/Swap)
  - [ ] Token pair
  - [ ] Amount
  - [ ] Execution price
  - [ ] Slippage percentage
  - [ ] MEV status indicator
- [ ] MEV indicators show correctly:
  - [ ] 🛡️ for protected trades
  - [ ] ⚠️ for impacted trades
  - [ ] Empty for unaffected trades
- [ ] Trade list scrolls smoothly with arrow keys

### 4. System Health Dashboard
- [ ] Circuit breaker status displays with correct color:
  - [ ] Green for "Closed" (trading active)
  - [ ] Red for "Open" (trading paused)
  - [ ] Yellow for "Half-Open" (testing recovery)
- [ ] Latency metrics update in real-time:
  - [ ] Node latency P99
  - [ ] Trade execution P99
  - [ ] Jupiter response time
- [ ] Visual alert appears when any latency exceeds 200ms
- [ ] Uptime percentage is calculated correctly
- [ ] Error rate is displayed and updates

### 5. Order Entry Form
- [ ] Token pair can be selected from available options
- [ ] Amount input accepts numeric values only
- [ ] Slippage tolerance validates between 0.5% and 2%
- [ ] MEV protection can be toggled on/off
- [ ] Submit button executes trade when pressed
- [ ] Cancel button clears form and returns to previous view
- [ ] Form shows validation errors for invalid inputs

### 6. Real-time Data Integration
- [ ] Successfully connects to Redis Streams
- [ ] Receives and processes events from "ui-events" stream
- [ ] Updates are reflected in UI within 100ms of event
- [ ] Handles all event types:
  - [ ] Price updates
  - [ ] Trade executions
  - [ ] Position changes
  - [ ] System alerts
  - [ ] Circuit breaker state changes
- [ ] Consumer group prevents duplicate event processing

### 7. Keyboard Navigation
- [ ] Tab key cycles through all views
- [ ] Arrow keys navigate within lists and forms
- [ ] Enter key submits forms and selects items
- [ ] 'q' key exits application with confirmation
- [ ] 'r' key forces data refresh
- [ ] '/' key activates search (where applicable)
- [ ] Esc key cancels current operation
- [ ] All shortcuts are documented in footer

## Performance Requirements

### 1. Rendering Performance
- [ ] Frame render time is consistently <10ms
- [ ] No frames are dropped during normal operation
- [ ] UI remains responsive during high event load (>100 events/second)

### 2. Resource Usage
- [ ] CPU usage stays below 5% during normal operation
- [ ] Memory usage remains stable over 1-hour session
- [ ] No memory leaks detected after 1000 trades
- [ ] File handles are properly managed

### 3. Latency
- [ ] Data updates appear within 100ms of Redis event
- [ ] Keyboard input response time <50ms
- [ ] No perceptible lag during view switching

## Error Handling

### 1. Connection Errors
- [ ] Redis connection loss displays error message
- [ ] UI continues to function with cached data
- [ ] Reconnection attempts are made automatically
- [ ] Stale data indicator appears after 5 seconds

### 2. Data Errors
- [ ] Invalid event data doesn't crash the application
- [ ] Error messages are logged to file
- [ ] UI shows user-friendly error notifications
- [ ] Partial data updates don't corrupt display

### 3. Input Validation
- [ ] Invalid order amounts show clear error message
- [ ] Out-of-range slippage values are rejected
- [ ] Form submission with errors is prevented
- [ ] Error messages disappear when corrected

## Integration Requirements

### 1. Portfolio Integration
- [ ] Successfully reads from VirtualPortfolio via Arc<RwLock>
- [ ] Portfolio updates don't block UI rendering
- [ ] Concurrent access is handled safely

### 2. Trade Execution
- [ ] Orders submitted through PaperTradeExecutor
- [ ] Trade confirmation is displayed
- [ ] Failed trades show error reason

### 3. System Health Monitoring
- [ ] Circuit breaker state is read correctly
- [ ] Latency metrics are current (not stale)
- [ ] Alerts trigger at correct thresholds

## Test Scenarios

### 1. Basic Operation Test
```bash
# Launch TUI
cargo run --bin paper-trader-tui

# Verify:
1. Initial layout displays correctly
2. All views are accessible via Tab
3. Footer shows keyboard shortcuts
4. No errors in terminal output
```

### 2. High-Frequency Update Test
```bash
# Generate 1000 price updates in 10 seconds
./scripts/generate-price-events.sh

# Verify:
1. UI maintains 10Hz refresh rate
2. All updates are displayed
3. No lag or stuttering
4. CPU usage remains below 5%
```

### 3. Error Recovery Test
```bash
# Start TUI, then stop Redis
redis-cli shutdown

# Verify:
1. Error message appears
2. UI remains functional
3. Stale data indicator shows

# Restart Redis
redis-server

# Verify:
1. Connection automatically restored
2. Updates resume
3. Stale indicator disappears
```

### 4. Memory Leak Test
```bash
# Run TUI for 1 hour with continuous updates
./scripts/long-running-test.sh

# Monitor:
1. Memory usage via top/htop
2. Check for increasing trend
3. Verify stable after 1 hour
```

### 5. Terminal Compatibility Test
Test on multiple terminal emulators:
- [ ] Linux: gnome-terminal, konsole, xterm
- [ ] macOS: Terminal.app, iTerm2
- [ ] Windows: Windows Terminal, ConEmu
- [ ] Minimum size: 80x24
- [ ] Maximum size: 200x60

## Acceptance Sign-off

The TUI implementation is considered complete when:
1. All functional requirements are met
2. Performance benchmarks pass
3. Error handling works as specified
4. Integration with other components is verified
5. Test scenarios pass without issues
6. Code review approves implementation quality

### Performance Benchmarks
- Render time P99: <10ms
- Event processing latency P99: <100ms
- CPU usage average: <5%
- Memory growth over 1 hour: <10MB