//! OTSS Core - Order management module
//!
//! Handles order lifecycle, state machine, and validation.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::types::{Price, Quantity, Symbol};

/// Unique identifier for an order
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(pub String);

impl OrderId {
    /// Generate a new unique order ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for OrderId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for OrderId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

/// Order side (buy or sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

/// Order type (market, limit, stop, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit(Price),
    Stop(Price),
    StopLimit { stop: Price, limit: Price },
}

/// Order state in the lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderState {
    /// Order created but not yet submitted
    Created,
    /// Order submitted to broker
    Pending,
    /// Order partially filled
    PartiallyFilled,
    /// Order fully filled
    Filled,
    /// Order cancelled
    Cancelled,
    /// Order rejected
    Rejected { reason: String },
}

impl fmt::Display for OrderState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderState::Created => write!(f, "CREATED"),
            OrderState::Pending => write!(f, "PENDING"),
            OrderState::PartiallyFilled => write!(f, "PARTIALLY_FILLED"),
            OrderState::Filled => write!(f, "FILLED"),
            OrderState::Cancelled => write!(f, "CANCELLED"),
            OrderState::Rejected { reason } => write!(f, "REJECTED: {}", reason),
        }
    }
}

/// Represents a trading order
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    id: OrderId,
    symbol: Symbol,
    side: OrderSide,
    quantity: Quantity,
    order_type: OrderType,
    state: OrderState,
    /// Time in force
    tif: TimeInForce,
    /// Current filled quantity
    filled_qty: Quantity,
    /// Average fill price
    avg_fill_price: Option<Price>,
}

/// Time in force options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeInForce {
    /// Good till cancelled
    GTC,
    /// Immediate or cancel
    IOC,
    /// Fill or kill
    FOK,
    /// Day order
    DAY,
}

impl Order {
    /// Create a new market order
    pub fn market(symbol: Symbol, side: OrderSide, quantity: Quantity) -> Self {
        Self::new(symbol, side, quantity, OrderType::Market)
    }

    /// Create a new limit order
    pub fn limit(
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        limit_price: Price,
    ) -> Self {
        Self::new(symbol, side, quantity, OrderType::Limit(limit_price))
    }

    /// Create a new order with the given parameters
    pub fn new(symbol: Symbol, side: OrderSide, quantity: Quantity, order_type: OrderType) -> Self {
        Self {
            id: OrderId::new(),
            symbol,
            side,
            quantity,
            order_type,
            state: OrderState::Created,
            tif: TimeInForce::GTC,
            filled_qty: Quantity::from(0),
            avg_fill_price: None,
        }
    }

    /// Get the order ID
    pub fn id(&self) -> &OrderId {
        &self.id
    }

    /// Get the symbol
    pub fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    /// Get the side
    pub fn side(&self) -> OrderSide {
        self.side
    }

    /// Get the order quantity
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Get the order type
    pub fn order_type(&self) -> &OrderType {
        &self.order_type
    }

    /// Get the current state
    pub fn state(&self) -> OrderState {
        self.state
    }

    /// Get remaining quantity to be filled
    pub fn remaining_qty(&self) -> Quantity {
        self.quantity - self.filled_qty
    }

    /// Check if the order is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, OrderState::Pending | OrderState::PartiallyFilled)
    }

    /// Check if the order is complete (filled, cancelled, or rejected)
    pub fn is_complete(&self) -> bool {
        matches!(
            self.state,
            OrderState::Filled | OrderState::Cancelled | OrderState::Rejected { .. }
        )
    }
}

// Need uuid for generating order IDs
use uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let symbol = Symbol::from_str("AAPL").unwrap();
        let qty = Quantity::from(100);
        let order = Order::market(symbol, OrderSide::Buy, qty);

        assert_eq!(order.symbol().as_str(), "AAPL");
        assert_eq!(order.side(), OrderSide::Buy);
        assert_eq!(order.quantity(), qty);
        assert_eq!(order.state(), OrderState::Created);
    }

    #[test]
    fn test_limit_order() {
        let symbol = Symbol::from_str("TSLA").unwrap();
        let qty = Quantity::from(50);
        let price = Price::from_str("150.00").unwrap();
        let order = Order::limit(symbol, OrderSide::Sell, qty, price);

        assert!(matches!(order.order_type(), OrderType::Limit(_)));
        assert_eq!(order.side(), OrderSide::Sell);
    }
}
