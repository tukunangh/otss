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
//! use summitx_risk_core::{RiskManager, RiskLimit, Side};
//!
//! // Create risk configuration
//! let limits = RiskLimit::new(dec!(1000), dec!(-5000));
//!
//! // Create risk manager
//! let manager = RiskManager::new(limits);
//!
//! // Check order against risk limits
//! let result = manager.validate_order("BTC", &dec!(100), &dec!(10), Side::Buy);
//! assert!(result.is_pass());
//! ```

pub mod order;
pub mod risk;
pub mod types;

pub use order::{
    OrderId, OrderSide, OrderStatus, TimeInForce, Order, Fill,
    OrderStatusTransitionError,
};
pub use risk::{RiskManager, RiskCheckResult};
pub use types::{
    Position, RiskLimit, RiskViolation, OrderState, Side, Symbol,
    ValidationContext,
};

/// Core version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Target validation latency: < 100 microseconds
pub const TARGET_LATENCY_US: u64 = 100;