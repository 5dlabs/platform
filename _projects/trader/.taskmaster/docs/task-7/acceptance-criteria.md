# Task 7: Acceptance Criteria

## Functional Requirements

### 1. Virtual Wallet
- [ ] **Token Support**:
  - All MVP tokens configured (SOL, USDC, BONK, JitoSOL, RAY)
  - Correct decimal precision for each token
  - Mint addresses match mainnet values
  - Extensible for new tokens
- [ ] **Balance Management**:
  - Track balances with Decimal precision
  - Validate sufficient balance before deductions
  - Return InsufficientBalance error when needed
  - Atomic balance updates
- [ ] **Token-2022 Support**:
  - Transfer fee calculation
  - Fee deduction from received amount
  - Fee tracking per token
  - Configurable fee rates
- [ ] **Transaction Logging**:
  - Record all balance changes
  - Include timestamp and type
  - Track balance after transaction
  - Calculate total fees paid

### 2. Position Tracking
- [ ] **Position Management**:
  - Create new positions on first buy
  - Update existing positions
  - Remove positions when fully sold
  - Track position metadata
- [ ] **Cost Basis**:
  - Weighted average calculation
  - Include fees in cost basis
  - Update on each buy
  - Maintain accuracy with Decimal
- [ ] **Trade Recording**:
  - Store all trades per position
  - Include MEV loss estimates
  - Track fees per trade
  - Maintain chronological order
- [ ] **P&L Tracking**:
  - Calculate realized P&L on sells
  - Track cumulative realized P&L
  - Separate from unrealized P&L
  - Include all fees in calculations

### 3. P&L Calculations
- [ ] **Unrealized P&L**:
  - Use current market prices
  - Calculate per position
  - Sum for portfolio total
  - Express as amount and percentage
- [ ] **Realized P&L**:
  - Calculate on position reduction
  - Based on cost basis
  - Include fees in calculation
  - Track cumulatively
- [ ] **Portfolio Valuation**:
  - Sum all token balances in USD
  - Add position values
  - Calculate total portfolio value
  - Track value over time
- [ ] **Performance Metrics**:
  - Win/loss ratio
  - Total volume traded
  - Total fees paid
  - MEV losses incurred

### 4. Trade Execution
- [ ] **Order Types**:
  - Buy orders update balances correctly
  - Sell orders check position size
  - Swaps handle as atomic operation
  - All include appropriate fees
- [ ] **Slippage Application**:
  - Fixed 0.5% for MVP
  - Additional 0.2% without MEV protection
  - Applied to execution price
  - Logged in trade record
- [ ] **Balance Updates**:
  - Deduct quote currency on buy
  - Add base currency on buy
  - Opposite for sells
  - Handle fees correctly
- [ ] **Error Handling**:
  - Insufficient balance errors
  - Insufficient position errors
  - Token not found errors
  - Price unavailable errors

## Non-Functional Requirements

### Performance
- [ ] Balance updates complete in <1ms
- [ ] Position calculations <5ms
- [ ] Portfolio snapshot <10ms
- [ ] Support 100 trades/second

### Reliability
- [ ] Thread-safe with RwLock
- [ ] No race conditions
- [ ] Consistent state after errors
- [ ] Recoverable from failures

### Accuracy
- [ ] Decimal precision maintained
- [ ] No floating point errors
- [ ] Rounding rules consistent
- [ ] Fees calculated exactly

## Test Cases

### Wallet Tests
```rust
// Test 1: Balance updates
Input: Initial 1000 USDC, deduct 100
Expected: Balance = 900 USDC

// Test 2: Insufficient balance
Input: 100 SOL balance, try deduct 150
Expected: InsufficientBalance error

// Test 3: Token-2022 fee
Input: Receive 100 tokens with 1% fee
Expected: Balance increases by 99
```

### Position Tests
```rust
// Test 1: New position
Input: Buy 10 SOL at $100
Expected: Position with cost basis $100.20 (with fees)

// Test 2: Position update
Input: Buy 5 more SOL at $120
Expected: 15 SOL with weighted average cost basis

// Test 3: Partial sell
Input: Sell 5 SOL from 15 SOL position
Expected: 10 SOL remaining, realized P&L calculated
```

### P&L Tests
```rust
// Test 1: Unrealized gain
Input: Buy at $100, current price $120
Expected: 20% unrealized gain

// Test 2: Realized loss
Input: Buy at $100, sell at $90
Expected: -10% realized loss (plus fees)

// Test 3: Portfolio P&L
Input: Multiple positions with gains/losses
Expected: Accurate total P&L calculation
```

### Trade Execution Tests
```rust
// Test 1: Buy execution
Input: Buy 10 SOL with 1000 USDC
Expected: SOL +10, USDC -1002 (with fees)

// Test 2: Slippage application
Input: Trade without MEV protection
Expected: 0.7% total slippage applied

// Test 3: Swap execution
Input: Swap 10 SOL for USDC
Expected: SOL -10, USDC increased by rate
```

## Integration Tests

### Full Trading Scenario
```rust
1. Start with 10,000 USDC
2. Buy 50 SOL at $100 (cost: $5,000)
3. Buy 1M BONK at $0.00001 (cost: $10)
4. SOL price rises to $120
5. Sell 25 SOL at $120 (receive: $3,000)
6. Calculate portfolio snapshot

Expected:
- 25 SOL remaining
- 1M BONK remaining
- ~$8,000 USDC
- Unrealized P&L on remaining SOL
- Realized P&L from SOL sale
- Accurate total portfolio value
```

### Concurrent Operations
```rust
// 10 threads simultaneously:
- 5 updating different token balances
- 3 calculating P&L
- 2 executing trades

Expected:
- All operations complete
- Final state consistent
- No deadlocks
- Correct final balances
```

## Definition of Done

- [ ] All unit tests pass with >90% coverage
- [ ] Integration tests demonstrate scenarios
- [ ] No race conditions in concurrent tests
- [ ] Performance benchmarks meet targets
- [ ] Documentation includes:
  - API usage examples
  - Token configuration
  - P&L calculation methods
  - Error handling guide
- [ ] Code reviewed for:
  - Decimal precision handling
  - Thread safety
  - Error propagation
  - Clean architecture
- [ ] Metrics validated against manual calculations
- [ ] Slippage model produces realistic results
- [ ] Token-2022 fees handled correctly