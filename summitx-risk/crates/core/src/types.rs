//! Domain types for the Risk Engine
//!
//! All financial values use `rust_decimal::Decimal` for precision.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Symbol/trading pair identifier (e.g., "BTC-USD")
pub type Symbol = String;

/// Unique order identifier
pub type OrderId = u64;

/// Order side (Buy/Sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    /// Returns the signed quantity based on side
    /// Buy = positive, Sell = negative
    pub fn sign(self) -> Decimal {
        match self {
            Side::Buy => Decimal::ONE,
            Side::Sell => Decimal::NEGATIVE_ONE,
        }
    }

    /// Returns true if this is a buy
    pub fn is_buy(self) -> bool {
        matches!(self, Side::Buy)
    }

    /// Returns true if this is a sell
    pub fn is_sell(self) -> bool {
        matches!(self, Side::Sell)
    }
}

/// Order state machine states
/// 
/// Lifecycle: Pending → Submitted → [PartialFill] → [Filled | Cancelled | Rejected]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderState {
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

impl OrderState {
    /// Returns true if the order is in a terminal state
    pub fn is_terminal(self) -> bool {
        matches!(self, OrderState::Filled | OrderState::Cancelled | OrderState::Rejected)
    }

    /// Returns true if the order can still receive fills
    pub fn can_fill(self) -> bool {
        matches!(self, OrderState::Submitted | OrderState::PartialFill)
    }

    /// Returns true if the order can be cancelled
    pub fn can_cancel(self) -> bool {
        matches!(self, OrderState::Pending | OrderState::Submitted | OrderState::PartialFill)
    }

    /// Returns true if the order is active (submitted or partially filled)
    pub fn is_active(self) -> bool {
        matches!(self, OrderState::Submitted | OrderState::PartialFill)
    }

    /// Transition to a new state if valid
    /// 
    /// Returns Err if the transition is not allowed by the state machine
    pub fn transition(self, new_state: OrderState) -> Result<OrderState, StateTransitionError> {
        match (self, new_state) {
            // From Pending
            (OrderState::Pending, OrderState::Submitted) => Ok(new_state),
            (OrderState::Pending, OrderState::Cancelled) => Ok(new_state),
            (OrderState::Pending, OrderState::Rejected) => Ok(new_state),
            
            // From Submitted
            (OrderState::Submitted, OrderState::PartialFill) => Ok(new_state),
            (OrderState::Submitted, OrderState::Filled) => Ok(new_state),
            (OrderState::Submitted, OrderState::Cancelled) => Ok(new_state),
            (OrderState::Submitted, OrderState::Rejected) => Ok(new_state),
            
            // From PartialFill
            (OrderState::PartialFill, OrderState::PartialFill) => Ok(new_state),
            (OrderState::PartialFill, OrderState::Filled) => Ok(new_state),
            (OrderState::PartialFill, OrderState::Cancelled) => Ok(new_state),
            
            // Terminal states - no transitions allowed
            (OrderState::Filled, _) |
            (OrderState::Cancelled, _) |
            (OrderState::Rejected, _) => Err(StateTransitionError::TerminalState(self)),
            
            // All other transitions are invalid
            (from, to) => Err(StateTransitionError::InvalidTransition { from, to }),
        }
    }
}

/// Error when an invalid state transition is attempted
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum StateTransitionError {
    #[error("Cannot transition from terminal state: {0:?}")]
    TerminalState(OrderState),
    #[error("Invalid transition from {from:?} to {to:?}")]
    InvalidTransition { from: OrderState, to: OrderState },
}

/// Position tracking per symbol
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// Symbol for this position
    pub symbol: Symbol,
    /// Net quantity (positive = long, negative = short)
    pub quantity: Decimal,
    /// Average entry price
    pub avg_price: Decimal,
    /// Total realized PnL for this symbol
    pub realized_pnl: Decimal,
    /// Total unrealized PnL (requires mark price)
    pub unrealized_pnl: Decimal,
    /// Last update timestamp
    pub last_update_ns: u64,
}

impl Position {
    /// Create a new empty position
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            quantity: Decimal::ZERO,
            avg_price: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
            unrealized_pnl: Decimal::ZERO,
            last_update_ns: 0,
        }
    }

    /// Returns true if position is flat (zero quantity)
    pub fn is_flat(&self) -> bool {
        self.quantity == Decimal::ZERO
    }
    
    /// Returns true if position is long (positive quantity)
    pub fn is_long(&self) -> bool {
        self.quantity > Decimal::ZERO
    }
    
    /// Returns true if position is short (negative quantity)
    pub fn is_short(&self) -> bool {
        self.quantity < Decimal::ZERO
    }

    /// Returns the absolute position size
    pub fn abs_quantity(&self) -> Decimal {
        self.quantity.abs()
    }

    /// Calculate notional value at a given price
    pub fn notional_value(&self, price: &Decimal) -> Decimal {
        self.quantity.abs() * price
    }

    /// Apply a fill to this position
    /// 
    /// Updates quantity, average price, and realized PnL
    pub fn apply_fill(
        &mut self,
        fill_qty: &Decimal,
        fill_price: &Decimal,
        side: Side,
        timestamp_ns: u64,
    ) {
        let signed_fill = *fill_qty * side.sign();
        let new_quantity = self.quantity + signed_fill;
        
        // Calculate realized PnL if reducing position
        let current_qty_abs = self.quantity.abs();
        let fill_qty_abs = fill_qty.abs();
        
        if (self.quantity > Decimal::ZERO && side == Side::Sell) ||
           (self.quantity < Decimal::ZERO && side == Side::Buy) {
            // Reducing position - realize PnL
            let close_qty = fill_qty_abs.min(current_qty_abs);
            let entry_value = close_qty * self.avg_price;
            let exit_value = close_qty * fill_price;
            
            let trade_pnl = if self.quantity > Decimal::ZERO {
                exit_value - entry_value  // Long: sell higher = profit
            } else {
                entry_value - exit_value  // Short: buy lower = profit
            };
            
            self.realized_pnl += trade_pnl;
        }
        
        // Update average price for new position
        if new_quantity == Decimal::ZERO {
            // Position closed
            self.avg_price = Decimal::ZERO;
        } else if new_quantity.abs() > self.quantity.abs() {
            // Increasing position size - update average price
            let current_notional = self.quantity.abs() * self.avg_price;
            let fill_notional = fill_qty_abs * fill_price;
            let total_qty = self.quantity.abs() + fill_qty_abs;
            
            if total_qty > Decimal::ZERO {
                self.avg_price = (current_notional + fill_notional) / total_qty;
            }
        } else if new_quantity.abs() < self.quantity.abs() {
            // Reducing position size - keep existing average price (realized PnL already captured)
            // Average price of remaining shares doesn't change
        } else {
            // Flipped side or same size - new position at fill price
            self.avg_price = *fill_price;
        }
        
        self.quantity = new_quantity;
        self.last_update_ns = timestamp_ns;
    }

    /// Update unrealized PnL given current mark price
    pub fn update_unrealized_pnl(&mut self, mark_price: &Decimal) {
        if self.is_flat() {
            self.unrealized_pnl = Decimal::ZERO;
        } else if self.is_long() {
            self.unrealized_pnl = self.quantity * (mark_price - self.avg_price);
        } else {
            self.unrealized_pnl = self.quantity.abs() * (self.avg_price - mark_price);
        }
    }
}

/// Risk limit configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RiskLimit {
    /// Maximum absolute position size per symbol
    pub max_position: Decimal,
    /// Daily loss limit (negative number, e.g., -5000)
    pub daily_loss_limit: Decimal,
    /// Maximum number of concurrent positions
    pub max_concurrent_positions: Option<usize>,
    /// Maximum order size per transaction
    pub max_order_size: Option<Decimal>,
    /// Maximum notional value per order
    pub max_notional_per_order: Option<Decimal>,
}

impl RiskLimit {
    /// Create default risk limits
    pub fn new(max_position: Decimal, daily_loss_limit: Decimal) -> Self {
        Self {
            max_position,
            daily_loss_limit,
            max_concurrent_positions: None,
            max_order_size: None,
            max_notional_per_order: None,
        }
    }

    /// Set maximum concurrent positions
    pub fn with_max_concurrent_positions(mut self, max: usize) -> Self {
        self.max_concurrent_positions = Some(max);
        self
    }

    /// Set maximum order size
    pub fn with_max_order_size(mut self, max: Decimal) -> Self {
        self.max_order_size = Some(max);
        self
    }

    /// Set maximum notional per order
    pub fn with_max_notional(mut self, max: Decimal) -> Self {
        self.max_notional_per_order = Some(max);
        self
    }
}

impl Default for RiskLimit {
    fn default() -> Self {
        Self {
            max_position: Decimal::from(1000),
            daily_loss_limit: Decimal::from(-5000),
            max_concurrent_positions: None,
            max_order_size: None,
            max_notional_per_order: None,
        }
    }
}

/// Risk violation reason
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RiskViolation {
    #[error("Position limit exceeded: symbol={symbol}, current={current}, proposed={proposed}, max={max}")]
    PositionLimitExceeded {
        symbol: String,
        current: Decimal,
        proposed: Decimal,
        max: Decimal,
    },

    #[error("Daily loss limit exceeded: realized={realized}, limit={limit}, excess={excess}")]
    DailyLossExceeded {
        realized: Decimal,
        limit: Decimal,
        excess: Decimal,
    },

    #[error("Order size exceeded: size={size}, max={max}")]
    OrderSizeExceeded {
        size: Decimal,
        max: Decimal,
    },

    #[error("Notional value exceeded: notional={notional}, max={max}")]
    NotionalExceeded {
        notional: Decimal,
        max: Decimal,
    },

    #[error("Concurrent position limit exceeded: current={current}, max={max}")]
    ConcurrentPositionLimitExceeded {
        current: usize,
        max: usize,
    },

    #[error("Multiple violations: {0:?}")]
    Multiple(Vec<Box<RiskViolation>>),
}

/// A trade fill
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fill {
    pub order_id: OrderId,
    pub symbol: Symbol,
    pub side: Side,
    pub quantity: Decimal,
    pub price: Decimal,
    pub timestamp_ns: u64,
}

impl Fill {
    /// Create a new fill
    pub fn new(
        order_id: OrderId,
        symbol: impl Into<String>,
        side: Side,
        quantity: Decimal,
        price: Decimal,
        timestamp_ns: u64,
    ) -> Self {
        Self {
            order_id,
            symbol: symbol.into(),
            side,
            quantity,
            price,
            timestamp_ns,
        }
    }

    /// Calculate fill notional value
    pub fn notional(&self) -> Decimal {
        self.quantity * self.price
    }
}

/// Order representation for risk validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub symbol: Symbol,
    pub side: Side,
    pub quantity: Decimal,
    pub price: Option<Decimal>, // None for market orders
    pub state: OrderState,
    pub filled_quantity: Decimal,
    pub created_ns: u64,
}

impl Order {
    /// Create a new order
    pub fn new(
        id: OrderId,
        symbol: impl Into<String>,
        side: Side,
        quantity: Decimal,
    ) -> Self {
        Self {
            id,
            symbol: symbol.into(),
            side,
            quantity,
            price: None,
            state: OrderState::Pending,
            filled_quantity: Decimal::ZERO,
            created_ns: 0,
        }
    }

    /// Set limit price
    pub fn with_limit_price(mut self, price: Decimal) -> Self {
        self.price = Some(price);
        self
    }

    /// Set creation timestamp
    pub fn with_timestamp(mut self, ns: u64) -> Self {
        self.created_ns = ns;
        self
    }

    /// Update order state
    pub fn transition_state(&mut self,
        new_state: OrderState,
    ) -> Result<(), StateTransitionError> {
        let next = self.state.transition(new_state)?;
        self.state = next;
        Ok(())
    }

    /// Apply a fill to the order
    pub fn apply_fill(&mut self,
        fill_qty: &Decimal,
        fill_price: &Decimal,
        timestamp_ns: u64,
    ) -> Result<(), StateTransitionError> {
        self.filled_quantity += fill_qty;
        
        let new_state = if self.filled_quantity >= self.quantity {
            OrderState::Filled
        } else {
            OrderState::PartialFill
        };
        
        self.transition_state(new_state)?;
        
        // Update average fill price tracking if needed
        if self.price.is_none() {
            // For market orders, could track VWAP here
            let _ = (fill_qty, fill_price, timestamp_ns); // Mark as used
        }
        
        Ok(())
    }

    /// Get remaining quantity to fill
    pub fn remaining_quantity(&self) -> Decimal {
        self.quantity - self.filled_quantity
    }

    /// Returns true if order is completely filled
    pub fn is_filled(&self) -> bool {
        self.remaining_quantity() == Decimal::ZERO
    }
}

/// Context passed to risk validation
#[derive(Debug, Clone)]
pub struct ValidationContext<'a> {
    /// Current positions snapshot
    pub positions: &'a HashMap<Symbol, Position>,
    /// Current daily realized PnL
    pub daily_realized_pnl: Decimal,
    /// Current timestamp
    pub timestamp_ns: u64,
}

impl<'a> ValidationContext<'a> {
    /// Create new validation context
    pub fn new(
        positions: &'a HashMap<Symbol, Position>,
        daily_realized_pnl: Decimal,
        timestamp_ns: u64,
    ) -> Self {
        Self {
            positions,
            daily_realized_pnl,
            timestamp_ns,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_side_sign() {
        assert_eq!(Side::Buy.sign(), Decimal::ONE);
        assert_eq!(Side::Sell.sign(), Decimal::NEGATIVE_ONE);
    }

    #[test]
    fn test_order_state_transitions() {
        // Valid transitions
        assert_eq!(
            OrderState::Pending.transition(OrderState::Submitted).unwrap(),
            OrderState::Submitted
        );
        assert_eq!(
            OrderState::Submitted.transition(OrderState::PartialFill).unwrap(),
            OrderState::PartialFill
        );
        assert_eq!(
            OrderState::PartialFill.transition(OrderState::Filled).unwrap(),
            OrderState::Filled
        );
        assert_eq!(
            OrderState::Pending.transition(OrderState::Cancelled).unwrap(),
            OrderState::Cancelled
        );

        // Invalid transitions
        assert!(OrderState::Filled.transition(OrderState::Pending).is_err());
        assert!(OrderState::Cancelled.transition(OrderState::Filled).is_err());
        assert!(OrderState::Pending.transition(OrderState::Filled).is_err());
    }

    #[test]
    fn test_position_apply_fill() {
        let mut pos = Position::new("BTC");
        
        // Buy 10 at 100
        pos.apply_fill(&dec!(10), &dec!(100), Side::Buy, 1000);
        assert_eq!(pos.quantity, dec!(10));
        assert_eq!(pos.avg_price, dec!(100));
        
        // Buy 5 at 110
        pos.apply_fill(&dec!(5), &dec!(110), Side::Buy, 2000);
        assert_eq!(pos.quantity, dec!(15));
        assert_eq!(pos.avg_price, dec!(103.33333333333333333333333333));
        
        // Sell 10 at 120 (realize profit)
        pos.apply_fill(&dec!(10), &dec!(120), Side::Sell, 3000);
        assert_eq!(pos.quantity, dec!(5));
        // Realized PnL = 10 * (120 - 103.33...) ≈ 166.67
        assert!(pos.realized_pnl > Decimal::ZERO);
        
        // Sell remaining 5 at 110
        pos.apply_fill(&dec!(5), &dec!(110), Side::Sell, 4000);
        assert!(pos.is_flat());
    }

    #[test]
    fn test_position_short() {
        let mut pos = Position::new("BTC");
        
        // Short 10 at 100
        pos.apply_fill(&dec!(10), &dec!(100), Side::Sell, 1000);
        assert_eq!(pos.quantity, dec!(-10));
        assert_eq!(pos.avg_price, dec!(100));
        assert!(pos.is_short());
        
        // Close short by buying at 90 (profit)
        pos.apply_fill(&dec!(10), &dec!(90), Side::Buy, 2000);
        assert!(pos.is_flat());
        assert_eq!(pos.realized_pnl, dec!(100)); // 10 * (100 - 90)
    }

    #[test]
    fn test_order_apply_fill() {
        let mut order = Order::new(1, "BTC", Side::Buy, dec!(100))
            .with_timestamp(1000);
        
        order.transition_state(OrderState::Submitted).unwrap();
        
        // Partial fill
        order.apply_fill(&dec!(30), &dec!(50000), 2000).unwrap();
        assert_eq!(order.state, OrderState::PartialFill);
        assert_eq!(order.filled_quantity, dec!(30));
        assert_eq!(order.remaining_quantity(), dec!(70));
        
        // Complete fill
        order.apply_fill(&dec!(70), &dec!(50100), 3000).unwrap();
        assert_eq!(order.state, OrderState::Filled);
        assert!(order.is_filled());
    }

    #[test]
    fn test_risk_violation_display() {
        let violation = RiskViolation::PositionLimitExceeded {
            symbol: "BTC".to_string(),
            current: dec!(900),
            proposed: dec!(1100),
            max: dec!(1000),
        };
        let msg = format!("{}", violation);
        assert!(msg.contains("Position limit exceeded"));
        assert!(msg.contains("BTC"));
    }
}