//! SummitX Risk Engine - CORE-001 Foundation
//!
//! Pure domain logic for pre-trade risk validation.
//! Zero I/O - all dependencies injected through function parameters.
//!
//! # Core Components
//!
//! - [`RiskManager`]: Central risk validation coordinator
//! - [`Position`]: Position tracking per symbol
//! - [`RiskLimit`]: Configurable risk boundaries
//! - [`RiskViolation`]: Risk check failure reasons
//! - [`Order`]: Order with simplified state machine (CORE-003)
//! - [`OrderState`]: Explicit order lifecycle state machine
//!
//! # Financial Precision
//!
//! All prices and quantities use [`Decimal`](rust_decimal::Decimal).
//! Never use `f64` for financial calculations.
//!
//! # Example
//!
//! ```
//! use rust_decimal_macros::dec;
//! use summitx_risk_core::{Order, Side, OrderId};
//!
//! // Create an order
//! let mut order = Order::new("order-1", "BTC-USD", Side::Buy, dec!(100))
//!     .with_price(dec!(50000));
//!
//! // Submit to exchange
//! order.submit(OrderId::new("ex-123")).unwrap();
//!
//! // Fill partially
//! order.fill(dec!(50)).unwrap();
//! ```

pub mod order;
pub mod risk;
pub mod types;

// CORE-003: Export simplified order state machine types from order module
pub use order::{
    Order, OrderError, OrderId, OrderState, Quantity, Price, Side, Symbol, Timestamp,
    StateTransition,
};

// Export types from risk module
pub use risk::{RiskManager, RiskCheckResult};

// Export types from types module (legacy, for backwards compatibility)
pub use types::{
    // New types module uses u64 for OrderId, export separately if needed
    Position, RiskLimit, RiskViolation, ValidationContext,
    // Old OrderState is different from new OrderState - only export if needed
    OrderState as TypesOrderState, 
    StateTransitionError,
    Fill, Side as TypesSide,
};

/// Core version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Target validation latency: < 100 microseconds
pub const TARGET_LATENCY_US: u64 = 100;
