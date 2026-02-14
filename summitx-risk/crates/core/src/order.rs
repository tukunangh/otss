//! Core domain types for order management
//!
//! These types are used by the OrderExecutionPort trait and throughout
//! the trading system for order lifecycle management.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Unique order identifier (newtype wrapper around String)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(pub String);

impl OrderId {
    /// Create a new OrderId from a string-like value
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the inner String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for OrderId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for OrderId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Order side (Buy or Sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderSide {
    /// Buy order (long position)
    Buy,
    /// Sell order (short position)
    Sell,
}

impl OrderSide {
    /// Returns true if this is a buy side
    pub const fn is_buy(self) -> bool {
        matches!(self, OrderSide::Buy)
    }

    /// Returns true if this is a sell side
    pub const fn is_sell(self) -> bool {
        matches!(self, OrderSide::Sell)
    }

    /// Returns the signed quantity based on side
    /// Buy = positive, Sell = negative
    pub fn sign(self) -> Decimal {
        match self {
            OrderSide::Buy => Decimal::ONE,
            OrderSide::Sell => Decimal::NEGATIVE_ONE,
        }
    }
}

impl Default for OrderSide {
    fn default() -> Self {
        OrderSide::Buy
    }
}

/// Time in Force - order duration policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeInForce {
    /// Day order - expires at end of trading day
    Day,
    /// Good Till Cancelled - remains active until filled or cancelled
    GTC,
    /// Immediate or Cancel - fill immediately, cancel remaining
    IOC,
    /// Fill or Kill - fill completely immediately or cancel entirely
    FOK,
}

impl TimeInForce {
    /// Returns true if the order allows partial fills
    pub const fn allows_partial(self) -> bool {
        matches!(self, TimeInForce::Day | TimeInForce::GTC | TimeInForce::IOC)
    }

    /// Returns true if the order must be filled immediately
    pub const fn requires_immediate(self) -> bool {
        matches!(self, TimeInForce::IOC | TimeInForce::FOK)
    }

    /// Returns true if the order persists across trading sessions
    pub const fn persists(self) -> bool {
        matches!(self, TimeInForce::GTC)
    }
}

impl Default for TimeInForce {
    fn default() -> Self {
        TimeInForce::Day
    }
}

/// Order status - state machine states
///
/// Lifecycle: Pending → Submitted → [PartialFill] → [Filled | Cancelled | Rejected]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Order created but not yet submitted to exchange
    Pending,
    /// Order submitted to exchange, awaiting acknowledgment
    Submitted,
    /// Order partially filled, remaining quantity still open
    PartialFill,
    /// Order completely filled
    Filled,
    /// Order cancelled by user or system
    Cancelled,
    /// Order rejected by exchange or risk system
    Rejected,
}

impl OrderStatus {
    /// Returns true if the order is in a terminal state
    pub const fn is_terminal(self) -> bool {
        matches!(self, OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected)
    }

    /// Returns true if the order is still open (may receive fills)
    pub const fn is_open(self) -> bool {
        matches!(self, OrderStatus::Submitted | OrderStatus::PartialFill)
    }

    /// Returns true if the order can be cancelled
    pub const fn can_cancel(self) -> bool {
        matches!(self, OrderStatus::Pending | OrderStatus::Submitted | OrderStatus::PartialFill)
    }

    /// Transition to a new state if valid
    pub fn transition(self, to: OrderStatus) -> Result<OrderStatus, OrderStatusTransitionError> {
        match (self, to) {
            // From Pending
            (OrderStatus::Pending, OrderStatus::Submitted) => Ok(to),
            (OrderStatus::Pending, OrderStatus::Cancelled) => Ok(to),
            (OrderStatus::Pending, OrderStatus::Rejected) => Ok(to),

            // From Submitted
            (OrderStatus::Submitted, OrderStatus::PartialFill) => Ok(to),
            (OrderStatus::Submitted, OrderStatus::Filled) => Ok(to),
            (OrderStatus::Submitted, OrderStatus::Cancelled) => Ok(to),
            (OrderStatus::Submitted, OrderStatus::Rejected) => Ok(to),

            // From PartialFill
            (OrderStatus::PartialFill, OrderStatus::PartialFill) => Ok(to),
            (OrderStatus::PartialFill, OrderStatus::Filled) => Ok(to),
            (OrderStatus::PartialFill, OrderStatus::Cancelled) => Ok(to),

            // Terminal states - no out transitions
            (state, _) if state.is_terminal() => {
                Err(OrderStatusTransitionError::TerminalState(state))
            }

            // All other transitions are invalid
            (from, to) => Err(OrderStatusTransitionError::InvalidTransition { from, to }),
        }
    }
}

impl Default for OrderStatus {
    fn default() -> Self {
        OrderStatus::Pending
    }
}

/// Error when an invalid order status transition is attempted
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum OrderStatusTransitionError {
    #[error("Cannot transition from terminal state: {0:?}")]
    TerminalState(OrderStatus),
    #[error("Invalid transition from {from:?} to {to:?}")]
    InvalidTransition { from: OrderStatus, to: OrderStatus },
}

/// Order represents a trading order
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    /// Unique order identifier
    pub id: OrderId,
    /// Trading symbol (e.g., "BTC-USD")
    pub symbol: String,
    /// Order side (Buy or Sell)
    pub side: OrderSide,
    /// Order quantity (always positive)
    pub quantity: Decimal,
    /// Limit price (None for market orders)
    pub price: Option<Decimal>,
    /// Time in force policy
    pub tif: TimeInForce,
    /// Current order status
    pub status: OrderStatus,
}

impl Order {
    /// Create a new order
    pub fn new(
        id: impl Into<OrderId>,
        symbol: impl Into<String>,
        side: OrderSide,
        quantity: Decimal,
    ) -> Self {
        Self {
            id: id.into(),
            symbol: symbol.into(),
            side,
            quantity,
            price: None,
            tif: TimeInForce::Day,
            status: OrderStatus::Pending,
        }
    }

    /// Set a limit price
    pub fn with_price(mut self, price: Decimal) -> Self {
        self.price = Some(price);
        self
    }

    /// Set time in force
    pub fn with_tif(mut self, tif: TimeInForce) -> Self {
        self.tif = tif;
        self
    }

    /// Set initial status
    pub fn with_status(mut self, status: OrderStatus) -> Self {
        self.status = status;
        self
    }

    /// Returns true if this is a limit order
    pub fn is_limit(&self) -> bool {
        self.price.is_some()
    }

    /// Returns true if this is a market order
    pub fn is_market(&self) -> bool {
        self.price.is_none()
    }

    /// Attempt to transition to a new status
    pub fn transition_status(&mut self, new_status: OrderStatus) -> Result<(), OrderStatusTransitionError> {
        self.status = self.status.transition(new_status)?;
        Ok(())
    }
}

/// Fill represents an executed trade (partial or complete)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fill {
    /// Order ID that this fill belongs to
    pub order_id: OrderId,
    /// Unique fill identifier (exchange-assigned)
    pub fill_id: String,
    /// Symbol that was traded
    pub symbol: String,
    /// Quantity filled
    pub quantity: Decimal,
    /// Price at which the fill occurred
    pub price: Decimal,
    /// Timestamp of the fill (UTC)
    pub timestamp: DateTime<Utc>,
}

impl Fill {
    /// Create a new fill
    pub fn new(
        order_id: impl Into<OrderId>,
        fill_id: impl Into<String>,
        symbol: impl Into<String>,
        quantity: Decimal,
        price: Decimal,
    ) -> Self {
        Self {
            order_id: order_id.into(),
            fill_id: fill_id.into(),
            symbol: symbol.into(),
            quantity,
            price,
            timestamp: Utc::now(),
        }
    }

    /// Create a fill with a specific timestamp
    pub fn with_timestamp(
        order_id: impl Into<OrderId>,
        fill_id: impl Into<String>,
        symbol: impl Into<String>,
        quantity: Decimal,
        price: Decimal,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            order_id: order_id.into(),
            fill_id: fill_id.into(),
            symbol: symbol.into(),
            quantity,
            price,
            timestamp,
        }
    }

    /// Calculate the notional value of this fill
    pub fn notional(&self) -> Decimal {
        self.quantity * self.price
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_order_id() {
        let id1 = OrderId::new("order-123");
        let id2 = OrderId::from("order-123");
        let id3 = OrderId::from("order-456".to_string());

        assert_eq!(id1.as_str(), "order-123");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_order_side() {
        assert!(OrderSide::Buy.is_buy());
        assert!(!OrderSide::Buy.is_sell());
        assert_eq!(OrderSide::Buy.sign(), Decimal::ONE);

        assert!(OrderSide::Sell.is_sell());
        assert!(!OrderSide::Sell.is_buy());
        assert_eq!(OrderSide::Sell.sign(), Decimal::NEGATIVE_ONE);
    }

    #[test]
    fn test_time_in_force() {
        assert!(TimeInForce::IOC.requires_immediate());
        assert!(TimeInForce::FOK.requires_immediate());
        assert!(!TimeInForce::Day.requires_immediate());

        assert!(TimeInForce::IOC.allows_partial());
        assert!(!TimeInForce::FOK.allows_partial());
        assert!(TimeInForce::GTC.allows_partial());

        assert!(TimeInForce::GTC.persists());
        assert!(!TimeInForce::Day.persists());
        assert!(!TimeInForce::IOC.persists());
    }

    #[test]
    fn test_order_status_transitions() {
        // Valid transitions
        assert!(OrderStatus::Pending.transition(OrderStatus::Submitted).is_ok());
        assert!(OrderStatus::Pending.transition(OrderStatus::Cancelled).is_ok());
        assert!(OrderStatus::Submitted.transition(OrderStatus::PartialFill).is_ok());
        assert!(OrderStatus::PartialFill.transition(OrderStatus::Filled).is_ok());

        // Invalid transitions
        assert!(OrderStatus::Filled.transition(OrderStatus::Pending).is_err());
        assert!(OrderStatus::Filled.transition(OrderStatus::Submitted).is_err());
        assert!(OrderStatus::Pending.transition(OrderStatus::Filled).is_err());
    }

    #[test]
    fn test_order_lifecycle() {
        let mut order = Order::new("abc-123", "BTC-USD", OrderSide::Buy, dec!(100.5))
            .with_price(dec!(50000));

        assert_eq!(order.status, OrderStatus::Pending);
        assert!(order.is_limit());
        assert!(!order.is_market());

        order.transition_status(OrderStatus::Submitted).unwrap();
        assert_eq!(order.status, OrderStatus::Submitted);
    }

    #[test]
    fn test_fill_creation() {
        let fill = Fill::new(
            "order-1",
            "fill-123",
            "BTC-USD",
            dec!(10),
            dec!(50000),
        );

        assert_eq!(fill.order_id.as_str(), "order-1");
        assert_eq!(fill.fill_id, "fill-123");
        assert_eq!(fill.notional(), dec!(500000));
    }

    #[test]
    fn test_fill_with_timestamp() {
        let ts = Utc::now();
        let fill = Fill::with_timestamp(
            "order-1",
            "fill-123",
            "BTC-USD",
            dec!(10),
            dec!(50000),
            ts,
        );

        assert_eq!(fill.timestamp, ts);
    }

    #[test]
    fn test_order_builder_pattern() {
        let order = Order::new("xyz-789", "ETH-USD", OrderSide::Sell, dec!(50))
            .with_price(dec!(3000))
            .with_tif(TimeInForce::GTC)
            .with_status(OrderStatus::Submitted);

        assert_eq!(order.id.as_str(), "xyz-789");
        assert_eq!(order.symbol, "ETH-USD");
        assert_eq!(order.side, OrderSide::Sell);
        assert_eq!(order.tif, TimeInForce::GTC);
        assert_eq!(order.status, OrderStatus::Submitted);
    }
}