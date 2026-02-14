//! Core domain types for order management with simplified state machine
//!
//! Implements the Order State Machine for CORE-003:
//! - New → Pending → Filled/PartiallyFilled → [Cancelled|Rejected]
//!
//! All financial values use `Decimal` for precision.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Unique order identifier (newtype wrapper)
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

impl std::fmt::Display for OrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Symbol/trading pair identifier (e.g., "BTC-USD")
pub type Symbol = String;

/// Quantity type using Decimal for precision
pub type Quantity = Decimal;

/// Price type using Decimal for precision
pub type Price = Decimal;

/// Timestamp type
pub type Timestamp = DateTime<Utc>;

/// Order side (Buy or Sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    /// Returns true if this is a buy side
    pub const fn is_buy(self) -> bool {
        matches!(self, Side::Buy)
    }

    /// Returns true if this is a sell side
    pub const fn is_sell(self) -> bool {
        matches!(self, Side::Sell)
    }

    /// Returns the signed quantity based on side
    /// Buy = positive, Sell = negative
    pub fn sign(self) -> Decimal {
        match self {
            Side::Buy => Decimal::ONE,
            Side::Sell => Decimal::NEGATIVE_ONE,
        }
    }
}

impl Default for Side {
    fn default() -> Self {
        Side::Buy
    }
}

/// Order state machine with simplified states
///
/// Lifecycle: New → Pending → Filled/PartiallyFilled
///              ↓
///            Cancelled/Rejected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderState {
    /// Order created but not yet submitted
    New,
    /// Order submitted to exchange with assigned ID
    Pending { id: OrderId },
    /// Order completely filled
    Filled,
    /// Order partially filled with quantities tracked
    PartiallyFilled { filled: Quantity, remaining: Quantity },
    /// Order cancelled with reason
    Cancelled { reason: String },
    /// Order rejected with reason
    Rejected { reason: String },
}

impl OrderState {
    /// Returns true if the order is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, OrderState::Filled | OrderState::Cancelled { .. } | OrderState::Rejected { .. })
    }

    /// Returns true if order can be submitted (New state)
    pub fn can_submit(&self) -> bool {
        matches!(self, OrderState::New)
    }

    /// Returns true if order can be filled (Pending or PartiallyFilled)
    pub fn can_fill(&self) -> bool {
        matches!(self, OrderState::Pending { .. } | OrderState::PartiallyFilled { .. })
    }

    /// Returns true if order can be cancelled (New or Pending)
    pub fn can_cancel(&self) -> bool {
        matches!(self, OrderState::New | OrderState::Pending { .. })
    }

    /// Returns true if order can be rejected (New or Pending)
    pub fn can_reject(&self) -> bool {
        matches!(self, OrderState::New | OrderState::Pending { .. })
    }

    /// Returns the order ID if in Pending state
    pub fn pending_id(&self) -> Option<&OrderId> {
        match self {
            OrderState::Pending { id } => Some(id),
            _ => None,
        }
    }

    /// Returns the filled quantity if in PartiallyFilled state
    pub fn filled_quantity(&self) -> Option<Quantity> {
        match self {
            OrderState::PartiallyFilled { filled, .. } => Some(*filled),
            OrderState::Filled => Some(Decimal::ZERO), // Total filled, but we don't track qty here
            _ => None,
        }
    }

    /// Returns the remaining quantity if tracked
    pub fn remaining_quantity(&self) -> Option<Quantity> {
        match self {
            OrderState::PartiallyFilled { remaining, .. } => Some(*remaining),
            _ => None,
        }
    }

    /// Returns the cancelled reason if in Cancelled state
    pub fn cancelled_reason(&self) -> Option<&str> {
        match self {
            OrderState::Cancelled { reason } => Some(reason.as_str()),
            _ => None,
        }
    }

    /// Returns the rejected reason if in Rejected state
    pub fn rejected_reason(&self) -> Option<&str> {
        match self {
            OrderState::Rejected { reason } => Some(reason.as_str()),
            _ => None,
        }
    }
}

impl Default for OrderState {
    fn default() -> Self {
        OrderState::New
    }
}

/// Error type for invalid order state transitions
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum OrderError {
    /// Cannot submit order from current state
    #[error("Cannot submit order from state: {0:?}")]
    InvalidSubmit(OrderState),

    /// Cannot fill order from current state
    #[error("Cannot fill order from state: {0:?}")]
    InvalidFill(OrderState),

    /// Cannot cancel order from current state
    #[error("Cannot cancel order from state: {0:?}")]
    InvalidCancel(OrderState),

    /// Cannot reject order from current state
    #[error("Cannot reject order from state: {0:?}")]
    InvalidReject(OrderState),

    /// Order is already in terminal state
    #[error("Order is in terminal state: {0:?}")]
    TerminalState(OrderState),

    /// Transition not allowed from current state
    #[error("Invalid transition from {from:?} to {to:?}")]
    InvalidTransition { from: OrderState, to: String },

    /// Fill quantity exceeds remaining quantity
    #[error("Fill quantity {fill_qty} exceeds remaining {remaining}")]
    OverFill { fill_qty: Quantity, remaining: Quantity },

    /// Fill quantity must be positive
    #[error("Fill quantity must be positive, got: {0}")]
    InvalidFillQuantity(Quantity),

    /// Cancel reason cannot be empty
    #[error("Cancel reason cannot be empty")]
    EmptyCancelReason,

    /// Reject reason cannot be empty
    #[error("Reject reason cannot be empty")]
    EmptyRejectReason,
}

/// Order struct with state machine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    /// Unique order identifier (generated at creation)
    pub id: OrderId,
    /// Trading symbol (e.g., "BTC-USD")
    pub symbol: Symbol,
    /// Order side (Buy or Sell)
    pub side: Side,
    /// Order quantity (always positive)
    pub quantity: Quantity,
    /// Limit price (None for market orders)
    pub price: Option<Price>,
    /// Current order state
    pub state: OrderState,
    /// Creation timestamp
    pub created_at: Timestamp,
}

impl Order {
    /// Create a new order in New state
    pub fn new(
        id: impl Into<OrderId>,
        symbol: impl Into<Symbol>,
        side: Side,
        quantity: Quantity,
    ) -> Self {
        Self {
            id: id.into(),
            symbol: symbol.into(),
            side,
            quantity,
            price: None,
            state: OrderState::New,
            created_at: Utc::now(),
        }
    }

    /// Set a limit price
    pub fn with_price(mut self, price: Price) -> Self {
        self.price = Some(price);
        self
    }

    /// Set creation timestamp
    pub fn with_timestamp(mut self, ts: Timestamp) -> Self {
        self.created_at = ts;
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

    /// Returns the remaining quantity based on state
    pub fn remaining_quantity(&self) -> Quantity {
        match &self.state {
            OrderState::New => self.quantity,
            OrderState::Pending { .. } => self.quantity,
            OrderState::Filled => Decimal::ZERO,
            OrderState::PartiallyFilled { remaining, .. } => *remaining,
            OrderState::Cancelled { .. } => Decimal::ZERO,
            OrderState::Rejected { .. } => self.quantity,
        }
    }

    /// Submit the order: New → Pending
    ///
    /// # Arguments
    /// * `exchange_id` - The ID assigned by the exchange
    ///
    /// # Errors
    /// Returns OrderError if order is not in New state
    pub fn submit(&mut self, exchange_id: OrderId) -> Result<(), OrderError> {
        if !self.state.can_submit() {
            return Err(OrderError::InvalidSubmit(self.state.clone()));
        }

        self.state = OrderState::Pending { id: exchange_id };
        Ok(())
    }

    /// Fill the order by specified quantity
    ///
    /// Transitions:
    /// - Pending with full quantity → Filled
    /// - Pending with partial → PartiallyFilled
    /// - PartiallyFilled → Filled (if remaining becomes 0) or updated PartiallyFilled
    ///
    /// # Arguments
    /// * `fill_qty` - The quantity filled in this transaction
    ///
    /// # Errors
    /// Returns OrderError if:
    /// - Order is not in fillable state
    /// - Fill quantity is not positive
    /// - Fill quantity exceeds remaining quantity
    pub fn fill(&mut self, fill_qty: Quantity) -> Result<(), OrderError> {
        if !self.state.can_fill() {
            return Err(OrderError::InvalidFill(self.state.clone()));
        }

        if fill_qty <= Decimal::ZERO {
            return Err(OrderError::InvalidFillQuantity(fill_qty));
        }

        let remaining = match &self.state {
            OrderState::Pending { .. } => self.quantity,
            OrderState::PartiallyFilled { remaining, .. } => *remaining,
            _ => return Err(OrderError::InvalidFill(self.state.clone())),
        };

        if fill_qty > remaining {
            return Err(OrderError::OverFill { fill_qty, remaining });
        }

        let new_filled = self.quantity - remaining + fill_qty;
        let new_remaining = remaining - fill_qty;

        self.state = if new_remaining == Decimal::ZERO {
            OrderState::Filled
        } else {
            OrderState::PartiallyFilled {
                filled: new_filled,
                remaining: new_remaining,
            }
        };

        Ok(())
    }

    /// Cancel the order: New/Pending → Cancelled
    ///
    /// # Arguments
    /// * `reason` - The cancellation reason
    ///
    /// # Errors
    /// Returns OrderError if:
    /// - Order is not in cancellable state
    /// - Reason is empty
    pub fn cancel(&mut self, reason: String) -> Result<(), OrderError> {
        if !self.state.can_cancel() {
            return Err(OrderError::InvalidCancel(self.state.clone()));
        }

        if reason.trim().is_empty() {
            return Err(OrderError::EmptyCancelReason);
        }

        self.state = OrderState::Cancelled { reason };
        Ok(())
    }

    /// Reject the order: New/Pending → Rejected
    ///
    /// # Arguments
    /// * `reason` - The rejection reason
    ///
    /// # Errors
    /// Returns OrderError if:
    /// - Order is not in rejectable state
    /// - Reason is empty
    pub fn reject(&mut self, reason: String) -> Result<(), OrderError> {
        if !self.state.can_reject() {
            return Err(OrderError::InvalidReject(self.state.clone()));
        }

        if reason.trim().is_empty() {
            return Err(OrderError::EmptyRejectReason);
        }

        self.state = OrderState::Rejected { reason };
        Ok(())
    }

    /// Attempt to transition to a new state (for internal use)
    ///
    /// This is primarily exposed for advanced use cases.
    /// For standard transitions, use submit(), fill(), cancel(), or reject().
    pub fn transition_to(&mut self, new_state: OrderState) -> Result<(), OrderError> {
        // Validate the transition based on current state
        match (&self.state, &new_state) {
            // From New
            (OrderState::New, OrderState::Pending { .. }) => {
                self.state = new_state;
                Ok(())
            }
            (OrderState::New, OrderState::Cancelled { .. }) => {
                self.state = new_state;
                Ok(())
            }
            (OrderState::New, OrderState::Rejected { .. }) => {
                self.state = new_state;
                Ok(())
            }

            // From Pending
            (OrderState::Pending { .. }, OrderState::Pending { .. }) => {
                // Update the exchange ID
                self.state = new_state;
                Ok(())
            }
            (OrderState::Pending { .. }, OrderState::Filled) => {
                self.state = new_state;
                Ok(())
            }
            (OrderState::Pending { .. }, OrderState::PartiallyFilled { .. }) => {
                self.state = new_state;
                Ok(())
            }
            (OrderState::Pending { .. }, OrderState::Cancelled { .. }) => {
                self.state = new_state;
                Ok(())
            }
            (OrderState::Pending { .. }, OrderState::Rejected { .. }) => {
                self.state = new_state;
                Ok(())
            }

            // From PartiallyFilled
            (OrderState::PartiallyFilled { .. }, OrderState::Filled) => {
                self.state = new_state;
                Ok(())
            }
            (OrderState::PartiallyFilled { .. }, OrderState::PartiallyFilled { .. }) => {
                // Update fill tracking
                self.state = new_state;
                Ok(())
            }
            (OrderState::PartiallyFilled { .. }, OrderState::Cancelled { .. }) => {
                self.state = new_state;
                Ok(())
            }

            // No transitions from terminal states
            (s, _) if s.is_terminal() => Err(OrderError::TerminalState(s.clone())),

            // All others are invalid
            (from, _) => Err(OrderError::InvalidTransition {
                from: from.clone(),
                to: format!("{:?}", new_state),
            }),
        }
    }
}

impl Default for Order {
    fn default() -> Self {
        Self::new("default", "DEFAULT", Side::Buy, Decimal::ZERO)
    }
}

/// State transition recording for audit trails
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: OrderState,
    pub to: OrderState,
    pub timestamp: Timestamp,
}

impl StateTransition {
    /// Create a new state transition record
    pub fn new(from: OrderState, to: OrderState) -> Self {
        Self {
            from,
            to,
            timestamp: Utc::now(),
        }
    }
}

/// Container for order state machine types (re-exported in lib.rs)
pub mod prelude {
    pub use super::{
        Order, OrderError, OrderId, OrderState, Quantity, Price, Side, Symbol, Timestamp,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    // Test 1: OrderId creation and conversion
    #[test]
    fn test_order_id_creation() {
        let id1 = OrderId::new("order-123");
        let id2 = OrderId::from("order-123");
        let id3 = OrderId::from("order-456".to_string());

        assert_eq!(id1.as_str(), "order-123");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_eq!(id1.clone().into_string(), "order-123");
    }

    // Test 2: OrderId Display trait
    #[test]
    fn test_order_id_display() {
        let id = OrderId::new("test-order");
        assert_eq!(format!("{}", id), "test-order");
    }

    // Test 3: Side enum functionality
    #[test]
    fn test_side_properties() {
        assert!(Side::Buy.is_buy());
        assert!(!Side::Buy.is_sell());
        assert_eq!(Side::Buy.sign(), Decimal::ONE);

        assert!(Side::Sell.is_sell());
        assert!(!Side::Sell.is_buy());
        assert_eq!(Side::Sell.sign(), Decimal::NEGATIVE_ONE);
    }

    // Test 4: Order creation with builder pattern
    #[test]
    fn test_order_creation() {
        let order = Order::new("order-1", "BTC-USD", Side::Buy, dec!(100))
            .with_price(dec!(50000));

        assert_eq!(order.id.as_str(), "order-1");
        assert_eq!(order.symbol, "BTC-USD");
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.quantity, dec!(100));
        assert_eq!(order.price, Some(dec!(50000)));
        assert_eq!(order.state, OrderState::New);
        assert!(order.is_limit());
        assert!(!order.is_market());
    }

    // Test 5: Market order creation
    #[test]
    fn test_market_order() {
        let order = Order::new("order-2", "ETH-USD", Side::Sell, dec!(50));

        assert!(order.is_market());
        assert!(!order.is_limit());
        assert_eq!(order.price, None);
    }

    // Test 6: Submit transition New -> Pending
    #[test]
    fn test_submit_transition() {
        let mut order = Order::new("order-3", "BTC-USD", Side::Buy, dec!(100));
        let exchange_id = OrderId::new("ex-123");

        assert!(order.state.can_submit());
        assert_eq!(order.state, OrderState::New);

        order.submit(exchange_id.clone()).unwrap();

        assert_eq!(order.state.pending_id(), Some(&exchange_id));
        assert!(!order.state.can_submit());
    }

    // Test 7: Cannot submit from non-New state
    #[test]
    fn test_cannot_submit_from_pending() {
        let mut order = Order::new("order-4", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();

        let result = order.submit(OrderId::new("ex-2"));
        assert!(matches!(result, Err(OrderError::InvalidSubmit(_))));
    }

    // Test 8: Cannot submit from terminal state
    #[test]
    fn test_cannot_submit_from_terminal() {
        let mut order = Order::new("order-5", "BTC-USD", Side::Buy, dec!(100));
        order.cancel("User cancelled".to_string()).unwrap();

        let result = order.submit(OrderId::new("ex-1"));
        assert!(matches!(result, Err(OrderError::InvalidSubmit(_))));
    }

    // Test 9: Fill transition Pending -> Filled
    #[test]
    fn test_fill_from_pending_full() {
        let mut order = Order::new("order-6", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();

        order.fill(dec!(100)).unwrap();

        assert_eq!(order.state, OrderState::Filled);
        assert!(order.state.is_terminal());
        assert!(!order.state.can_fill());
    }

    // Test 10: Fill transition Pending -> PartiallyFilled
    #[test]
    fn test_fill_from_pending_partial() {
        let mut order = Order::new("order-7", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();

        order.fill(dec!(30)).unwrap();

        assert_eq!(
            order.state,
            OrderState::PartiallyFilled {
                filled: dec!(30),
                remaining: dec!(70)
            }
        );
        assert!(!order.state.is_terminal());
    }

    // Test 11: Fill transition PartiallyFilled -> Filled
    #[test]
    fn test_fill_partially_filled_complete() {
        let mut order = Order::new("order-8", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();
        order.fill(dec!(40)).unwrap();

        order.fill(dec!(60)).unwrap();

        assert_eq!(order.state, OrderState::Filled);
    }

    // Test 12: Fill transition PartiallyFilled -> PartiallyFilled
    #[test]
    fn test_fill_partially_filled_incremental() {
        let mut order = Order::new("order-9", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();
        order.fill(dec!(30)).unwrap();

        order.fill(dec!(50)).unwrap();

        assert_eq!(
            order.state,
            OrderState::PartiallyFilled {
                filled: dec!(80),
                remaining: dec!(20)
            }
        );
    }

    // Test 13: Cannot overfill
    #[test]
    fn test_cannot_overfill() {
        let mut order = Order::new("order-10", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();

        let result = order.fill(dec!(150));
        assert!(matches!(result, Err(OrderError::OverFill { .. })));
    }

    // Test 14: Cannot fill with zero or negative
    #[test]
    fn test_cannot_fill_non_positive() {
        let mut order = Order::new("order-11", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();

        let result = order.fill(Decimal::ZERO);
        assert!(matches!(result, Err(OrderError::InvalidFillQuantity(_))));

        let result = order.fill(dec!(-10));
        assert!(matches!(result, Err(OrderError::InvalidFillQuantity(_))));
    }

    // Test 15: Cancel transition New -> Cancelled
    #[test]
    fn test_cancel_from_new() {
        let mut order = Order::new("order-12", "BTC-USD", Side::Buy, dec!(100));

        order.cancel("User requested".to_string()).unwrap();

        assert_eq!(order.state.cancelled_reason(), Some("User requested"));
        assert!(matches!(order.state, OrderState::Cancelled { .. }));
    }

    // Test 16: Cancel transition Pending -> Cancelled
    #[test]
    fn test_cancel_from_pending() {
        let mut order = Order::new("order-13", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();

        order.cancel("Exchange requested".to_string()).unwrap();

        assert!(matches!(order.state, OrderState::Cancelled { .. }));
    }

    // Test 17: Cannot cancel from filled
    #[test]
    fn test_cannot_cancel_from_filled() {
        let mut order = Order::new("order-14", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();
        order.fill(dec!(100)).unwrap();

        let result = order.cancel("Attempt".to_string());
        assert!(matches!(result, Err(OrderError::InvalidCancel(_))));
    }

    // Test 18: Cannot cancel with empty reason
    #[test]
    fn test_cannot_cancel_empty_reason() {
        let mut order = Order::new("order-15", "BTC-USD", Side::Buy, dec!(100));

        let result = order.cancel("".to_string());
        assert!(matches!(result, Err(OrderError::EmptyCancelReason)));

        let result = order.cancel("   ".to_string());
        assert!(matches!(result, Err(OrderError::EmptyCancelReason)));
    }

    // Test 19: Reject transition New -> Rejected
    #[test]
    fn test_reject_from_new() {
        let mut order = Order::new("order-16", "BTC-USD", Side::Buy, dec!(100));

        order.reject("Risk limit exceeded".to_string()).unwrap();

        assert_eq!(order.state.rejected_reason(), Some("Risk limit exceeded"));
        assert!(matches!(order.state, OrderState::Rejected { .. }));
    }

    // Test 20: Reject transition Pending -> Rejected
    #[test]
    fn test_reject_from_pending() {
        let mut order = Order::new("order-17", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();

        order.reject("Exchange rejected".to_string()).unwrap();

        assert!(matches!(order.state, OrderState::Rejected { .. }));
    }

    // Test 21: Cannot reject from filled
    #[test]
    fn test_cannot_reject_from_filled() {
        let mut order = Order::new("order-18", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();
        order.fill(dec!(100)).unwrap();

        let result = order.reject("Attempt".to_string());
        assert!(matches!(result, Err(OrderError::InvalidReject(_))));
    }

    // Test 22: Cannot reject with empty reason
    #[test]
    fn test_cannot_reject_empty_reason() {
        let mut order = Order::new("order-19", "BTC-USD", Side::Buy, dec!(100));

        let result = order.reject("".to_string());
        assert!(matches!(result, Err(OrderError::EmptyRejectReason)));
    }

    // Test 23: Cannot fill from New state
    #[test]
    fn test_cannot_fill_from_new() {
        let mut order = Order::new("order-20", "BTC-USD", Side::Buy, dec!(100));

        let result = order.fill(dec!(50));
        assert!(matches!(result, Err(OrderError::InvalidFill(_))));
    }

    // Test 24: Cannot fill from cancelled
    #[test]
    fn test_cannot_fill_from_cancelled() {
        let mut order = Order::new("order-21", "BTC-USD", Side::Buy, dec!(100));
        order.cancel("User".to_string()).unwrap();

        let result = order.fill(dec!(50));
        assert!(matches!(result, Err(OrderError::InvalidFill(_))));
    }

    // Test 25: remaining_quantity helper
    #[test]
    fn test_remaining_quantity() {
        let order = Order::new("order-22", "BTC-USD", Side::Buy, dec!(100));
        assert_eq!(order.remaining_quantity(), dec!(100));

        // Test after being partially filled
        let mut order = Order::new("order-23", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();
        order.fill(dec!(30)).unwrap();
        assert_eq!(order.remaining_quantity(), dec!(70));

        // Test after being filled
        let mut order = Order::new("order-24", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();
        order.fill(dec!(100)).unwrap();
        assert_eq!(order.remaining_quantity(), Decimal::ZERO);
    }

    // Test 26: Complete order lifecycle
    #[test]
    fn test_complete_lifecycle_submit_fill() {
        let mut order = Order::new("order-25", "BTC-USD", Side::Buy, dec!(100))
            .with_price(dec!(50000));

        // Submit
        order.submit(OrderId::new("ex-123")).unwrap();
        assert!(matches!(order.state, OrderState::Pending { .. }));

        // Fill completely
        order.fill(dec!(100)).unwrap();
        assert_eq!(order.state, OrderState::Filled);
    }

    // Test 27: Complete order lifecycle submit partial fill then complete fill
    #[test]
    fn test_complete_lifecycle_partial_fill_complete() {
        let mut order = Order::new("order-26", "BTC-USD", Side::Buy, dec!(100));

        // Submit
        order.submit(OrderId::new("ex-123")).unwrap();

        // Partial fill
        order.fill(dec!(30)).unwrap();
        assert!(matches!(order.state, OrderState::PartiallyFilled { filled, remaining } if filled == dec!(30) && remaining == dec!(70)));

        // Fill remaining (cannot cancel from PartiallyFilled per CORE-003 spec)
        order.fill(dec!(70)).unwrap();
        assert!(matches!(order.state, OrderState::Filled));
    }

    // Test 28: OrderState helper methods
    #[test]
    fn test_order_state_helpers() {
        let filled = OrderState::PartiallyFilled { filled: dec!(30), remaining: dec!(70) };
        assert_eq!(filled.filled_quantity(), Some(dec!(30)));
        assert_eq!(filled.remaining_quantity(), Some(dec!(70)));
        assert!(!filled.is_terminal());
        assert!(filled.can_fill());
        // Per CORE-003: cancel is only allowed from New/Pending state, not PartiallyFilled
        assert!(!filled.can_cancel());
    }

    // Test 29: StateTransition recording
    #[test]
    fn test_state_transition() {
        let from = OrderState::New;
        let to = OrderState::Pending { id: OrderId::new("ex-1") };
        let transition = StateTransition::new(from, to);

        assert_eq!(transition.from, OrderState::New);
        assert!(matches!(transition.to, OrderState::Pending { .. }));
    }

    // Test 30: Multiple partial fills
    #[test]
    fn test_multiple_partial_fills() {
        let mut order = Order::new("order-27", "BTC-USD", Side::Buy, dec!(100));
        order.submit(OrderId::new("ex-1")).unwrap();

        // Multiple fills
        order.fill(dec!(10)).unwrap();
        order.fill(dec!(20)).unwrap();
        order.fill(dec!(30)).unwrap();
        order.fill(dec!(40)).unwrap();

        assert_eq!(order.state, OrderState::Filled);
    }
}