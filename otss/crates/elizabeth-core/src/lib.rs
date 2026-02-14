//! OTSS Core - Core trading engine, order management, and risk management
//!
//! This crate contains the pure domain logic for the OTSS trading system.
//! It is designed to be I/O-free and focuses on:
//! - Order state machines and lifecycle management
//! - Financial types with precision handling
//! - Portfolio risk calculations
//! - Position tracking
//!
//! # Architecture
//!
//! The core is intentionally free of I/O dependencies. All external interactions
//! are handled through ports and adapters in other crates.

pub mod order;
pub mod portfolio;
pub mod risk;
pub mod types;

pub use order::{Order, OrderId, OrderSide, OrderState, OrderType};
pub use portfolio::{Portfolio, Position};
pub use risk::{RiskCheck, RiskConfig, RiskResult};
pub use types::{Price, Quantity, Symbol};

/// The core trading engine that manages order flow and risk
#[derive(Debug, Clone)]
pub struct TradingEngine {
    config: EngineConfig,
}

/// Configuration for the trading engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum orders per second
    pub max_order_rate: u32,
    /// Risk configuration
    pub risk_config: RiskConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_order_rate: 100,
            risk_config: RiskConfig::default(),
        }
    }
}

impl TradingEngine {
    /// Create a new trading engine with the given configuration
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }

    /// Submit an order for processing
    ///
    /// This performs risk checks and state transitions synchronously.
    /// Returns the order ID if accepted.
    pub fn submit_order(&self, order: Order) -> Result<OrderId, EngineError> {
        // TODO: Implement order submission logic
        tracing::info!("Submitting order: {:?}", order);
        Ok(order.id().clone())
    }

    /// Get the current engine configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }
}

/// Errors that can occur in the trading engine
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Order rejected: {0}")]
    OrderRejected(String),
    #[error("Risk check failed: {0}")]
    RiskCheckFailed(String),
    #[error("Invalid state transition")]
    InvalidStateTransition,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = TradingEngine::new(EngineConfig::default());
        assert_eq!(engine.config().max_order_rate, 100);
    }
}
