# Task 7: Develop Virtual Portfolio for Paper Trading - Autonomous Prompt

You are implementing a comprehensive virtual portfolio system for paper trading that accurately simulates real trading conditions including fees, slippage, and Token-2022 extensions. The system must track balances, positions, and P&L while providing realistic trade execution simulation.

## Context

The paper trading system requires high-fidelity simulation to achieve 85-90% accuracy compared to live trading. Key requirements:
- Track balances for all MVP tokens (SOL, USDC, BONK, JitoSOL, RAY)
- Support Token-2022 transfer fees
- Calculate real-time P&L using Redis-cached prices
- Simulate MEV impact on unprotected trades
- Maintain position tracking with cost basis

## Your Objectives

1. **Implement Virtual Wallet**
   - Track token balances with proper decimals
   - Handle Token-2022 transfer fees
   - Validate sufficient balance before trades
   - Log all transactions
   - Calculate USD values

2. **Build Position Tracking**
   - Weighted average cost basis
   - Track entry/exit timestamps
   - Calculate realized P&L on sells
   - Support partial positions
   - Record all position trades

3. **Create P&L Calculations**
   - Real-time unrealized P&L
   - Track realized P&L
   - Percentage calculations
   - Portfolio-wide metrics
   - MEV loss tracking

4. **Trade Execution Simulation**
   - Apply slippage models
   - Deduct appropriate fees
   - Update balances atomically
   - Record in trade history
   - Track performance metrics

## Implementation Requirements

### Token Information
```rust
SOL: {
    mint: "So11111111111111111111111111111111111111112",
    decimals: 9,
    transfer_fee: None
}
USDC: {
    mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    decimals: 6,
    transfer_fee: None
}
BONK: {
    mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
    decimals: 5,
    transfer_fee: None
}
JitoSOL: {
    mint: "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
    decimals: 9,
    transfer_fee: None
}
RAY: {
    mint: "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
    decimals: 6,
    transfer_fee: None
}
```

### Balance Management
- Check sufficient balance before deductions
- Apply Token-2022 fees if applicable
- Log all balance changes
- Support concurrent access
- Track total fees paid

### Position Updates
- Buy: Add to position, update cost basis
- Sell: Reduce position, calculate realized P&L
- Swap: Handle as sell + buy
- Close position when amount = 0
- Track trade history per position

### Slippage Simulation
- Fixed model for MVP: 0.5% base
- Additional 0.2% for non-MEV protected
- Apply to execution price
- Respect max slippage parameter

## Testing Strategy

Create comprehensive tests for:

1. **Wallet Operations**:
   - Balance updates
   - Insufficient balance errors
   - Token-2022 fee application
   - Concurrent access safety

2. **Position Tracking**:
   - Cost basis calculation
   - Partial sells
   - Position closure
   - Trade history accuracy

3. **P&L Calculations**:
   - Unrealized P&L accuracy
   - Realized P&L on sells
   - Portfolio totals
   - Percentage calculations

4. **Trade Execution**:
   - Buy/sell/swap flows
   - Slippage application
   - Fee deduction
   - MEV impact simulation

## Deliverables

1. **Core Components**:
   - `virtual_wallet.rs` with balance tracking
   - `position_manager.rs` with cost basis
   - `portfolio.rs` integrating all components
   - Error types and models

2. **Features**:
   - Transaction logging
   - Performance metrics
   - Portfolio snapshots
   - Trade history

3. **Slippage Models**:
   - Fixed slippage for MVP
   - MEV impact simulation
   - Extensible interface

4. **Tests**:
   - Unit tests for components
   - Integration test scenarios
   - Performance benchmarks
   - Edge case coverage

## Success Criteria

- All MVP tokens supported with correct decimals
- Token-2022 fees calculated correctly
- P&L calculations match manual verification
- Slippage applied consistently
- No balance inconsistencies
- Thread-safe concurrent operations
- Performance metrics tracked accurately
- 85-90% correlation with live trading patterns