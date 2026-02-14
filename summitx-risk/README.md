# SummitX Risk Engine - CORE-001

Pure domain logic foundation for pre-trade risk validation in trading systems.

## Overview

This crate implements a clean, zero-I/O risk engine following the **CORE-001** specification:

- **Pure functions** - No network calls, no file I/O, no databases
- **Financial precision** - Uses `rust_decimal::Decimal` for all prices/quantities
- **Explicit state machine** - Order lifecycle with clear transitions
- **Dependency injection** - All state through ports/parameters

## Project Structure

```
/projects/summitx-risk/
├── Cargo.toml              # Workspace configuration (2024 edition)
├── crates/
│   └── core/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs      # Public API exports
│           ├── types.rs    # Domain types (Position, RiskLimit, OrderState)
│           └── risk/       # Risk validation logic
│               └── mod.rs  # RiskManager implementation
```

## Core Components

### RiskManager

Central coordinator for pre-trade risk validation:

```rust
use summitx_risk_core::{RiskManager, RiskLimit, Side};
use rust_decimal_macros::dec;

let limits = RiskLimit::new(dec!(1000), dec!(-5000))
    .with_max_order_size(dec!(500))
    .with_max_notional(dec!(50000))
    .with_max_concurrent_positions(5);

let manager = RiskManager::new(limits);

// Validate an order
let result = manager.validate_order("BTC", &dec!(100), &dec!(50000), Side::Buy);
```

### Risk Checks

The `RiskManager.validate_order()` performs these validations:

1. **Order Size Limit** - Maximum quantity per order
2. **Notional Value Limit** - Maximum order value (price × quantity)
3. **Position Limit** - Maximum absolute position per symbol
4. **Concurrent Positions** - Maximum number of open positions
5. **Daily Loss Limit** - Maximum allowable daily loss

### OrderState Machine

Explicit state machine with enforced transitions:

```rust
use summitx_risk_core::OrderState;

// Valid transitions
let state = OrderState::Pending;
let new_state = state.transition(OrderState::Submitted)?; // Ok

// Invalid transitions fail
let bad = state.transition(OrderState::Filled); // Err - InvalidTransition
```

States: `Pending → Submitted → [PartialFill] → [Filled | Cancelled | Rejected]`

### Position Tracking

Accurate PnL calculation with proper cost basis:

```rust
use summitx_risk_core::{Position, Side};
use rust_decimal_macros::dec;

let mut pos = Position::new("BTC");

// Buy 100 at $100
pos.apply_fill(&dec!(100), &dec!(100), Side::Buy, 1000);
// Position: quantity=100, avg_price=100

// Sell 50 at $110 (profit)
pos.apply_fill(&dec!(50), &dec!(110), Side::Sell, 2000);
// Position: quantity=50, realized_pnl=500, avg_price still 100
```

## API Examples

### Basic Usage

```rust
use summitx_risk_core::{
    RiskManager, RiskLimit, Side, Fill, Position
};
use rust_decimal_macros::dec;

// Configure limits
let limits = RiskLimit::new(
    dec!(1000),      // max_position
    dec!(-5000),     // daily_loss_limit
)
.with_max_order_size(dec!(100))
.with_max_notional(dec!(500000))
.with_max_concurrent_positions(10);

// Create manager
let mut manager = RiskManager::new(limits);

// Fill an order
let fill = Fill::new(1, "BTC", Side::Buy, dec!(10), dec!(50000), 1000);
manager.apply_fill(&fill);

// Check position
let pos = manager.position("BTC").unwrap();
assert_eq!(pos.quantity, dec!(10));
```

### Validation with Context

```rust
use summitx_risk_core::{RiskManager, RiskLimit, Side, ValidationContext};
use rust_decimal_macros::dec;
use std::collections::HashMap;

let manager = RiskManager::new(RiskLimit::new(dec!(100), dec!(-1000)));
let positions: HashMap<String, Position> = HashMap::new();

let ctx = ValidationContext::new(
    &positions,
    dec!(0),     // daily_pnl
    1000,        // timestamp
);

let result = manager.validate_with_context(
    "BTC", &dec!(50), &dec!(100), Side::Buy, &ctx
);
```

## Testing

TDD pattern with comprehensive unit tests:

```bash
cd /home/node/.openclaw/projects/summitx-risk
cargo test
```

### Test Coverage

- **Position tracking** - Long/short positions, PnL calculation, flip detection
- **State machine** - Valid/invalid transitions, terminal states
- **Risk validation** - All limit types, multi-violation detection
- **Edge cases** - Position closing, concurrent limits, zero quantities

## Financial Precision

All monetary values use `Decimal` (never `f64`):

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Good
let price = dec!(100.50);
let quantity = dec!(10.5);

// Bad - floating point for money
let bad_price: f64 = 100.50;  // Don't do this!
```

## Zero I/O Principle

This crate is pure domain logic with no external dependencies:

✅ Time is passed as `timestamp_ns: u64`
✅ Prices passed by reference (`&Decimal`)
✅ State returned, not persisted
❌ No network calls
❌ No file I/O
❌ No database queries
❌ No system time

## Risk Violations

```rust
use summitx_risk_core::RiskViolation;

match violation {
    RiskViolation::PositionLimitExceeded { symbol, current, proposed, max } => {
        // Handle position limit breach
    }
    RiskViolation::DailyLossExceeded { realized, limit, excess } => {
        // Handle daily loss breach
    }
    RiskViolation::Multiple(violations) => {
        // Multiple violations detected
    }
    _ => {}
}
```

## Version

- **Rust Edition**: 2024
- **Package Version**: 0.1.0
- **Target Latency**: < 100μs

## License

Internal use only - SummitX Trading Systems