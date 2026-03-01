//! Risk Manager - CORE-001 Foundation
//!
//! Central coordinator for pre-trade risk validation.
//! Pure functions with zero I/O.
//!
//! # Core Responsibilities
//!
//! - Position tracking per symbol
//! - Daily loss limit checking
//! - Per-symbol position limits
//! - Order validation before submission
//!
//! # Example
//!
//! ```
//! use rust_decimal_macros::dec;
//! use summitx_risk_core::risk::RiskManager;
//! use summitx_risk_core::RiskLimit;
//! use summitx_risk_core::types::Side;
//!
//! let limits = RiskLimit::new(dec!(1000), dec!(-5000));
//! let manager = RiskManager::new(limits);
//!
//! // Validate an order
//! let result = manager.validate_order("BTC", &dec!(100), &dec!(10), Side::Buy);
//! ```

use crate::types::{
    Fill, Position, RiskLimit, RiskViolation, Side, Symbol,
    ValidationContext,
};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Result of a risk check
#[derive(Debug, Clone, PartialEq)]
pub enum RiskCheckResult {
    /// Risk check passed
    Pass,
    /// Risk check failed with violation(s)
    Fail(RiskViolation),
}

impl RiskCheckResult {
    /// Returns true if the check passed
    pub fn is_pass(&self) -> bool {
        matches!(self, RiskCheckResult::Pass)
    }

    /// Returns true if the check failed
    pub fn is_fail(&self) -> bool {
        matches!(self, RiskCheckResult::Fail(_))
    }

    /// Get the violation if failed
    pub fn violation(&self) -> Option<&RiskViolation> {
        match self {
            RiskCheckResult::Fail(v) => Some(v),
            _ => None,
        }
    }
}

/// Central risk manager for pre-trade validation
///
/// Manages position tracking and enforces risk limits.
/// All operations are pure functions - no I/O, no side effects.
#[derive(Debug, Clone)]
pub struct RiskManager {
    /// Risk limits configuration
    limits: RiskLimit,
    /// Current positions per symbol
    positions: HashMap<Symbol, Position>,
    /// Daily realized PnL
    daily_realized_pnl: Decimal,
    /// Daily unrealized PnL (requires mark prices)
    daily_unrealized_pnl: Decimal,
}

impl RiskManager {
    /// Create a new RiskManager with the given limits
    pub fn new(limits: RiskLimit) -> Self {
        Self {
            limits,
            positions: HashMap::new(),
            daily_realized_pnl: Decimal::ZERO,
            daily_unrealized_pnl: Decimal::ZERO,
        }
    }

    /// Get current positions snapshot
    pub fn positions(&self) -> &HashMap<Symbol, Position> {
        &self.positions
    }

    /// Get position for a specific symbol
    pub fn position(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    /// Get mutable position for a symbol (creates if not exists)
    fn position_mut(&mut self, symbol: &str) -> &mut Position {
        let key = symbol.to_string();
        self.positions
            .entry(key.clone())
            .or_insert_with(|| Position::new(key))
    }

    /// Get daily realized PnL
    pub fn daily_realized_pnl(&self) -> Decimal {
        self.daily_realized_pnl
    }

    /// Get current risk limits
    pub fn limits(&self) -> &RiskLimit {
        &self.limits
    }

    /// Update risk limits
    pub fn set_limits(&mut self, limits: RiskLimit) {
        self.limits = limits;
    }

    /// Calculate the proposed position after executing an order
    fn proposed_position(
        &self,
        symbol: &str,
        order_qty: &Decimal,
        side: Side,
    ) -> Decimal {
        let current_qty = self
            .position(symbol)
            .map(|p| p.quantity)
            .unwrap_or(Decimal::ZERO);
        current_qty + (*order_qty * side.sign())
    }

    /// Validate an order against all risk limits
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading symbol
    /// * `quantity` - Order quantity
    /// * `price` - Expected execution price (for notional calculation)
    /// * `side` - Buy or Sell
    ///
    /// # Returns
    ///
    /// `RiskCheckResult::Pass` if all checks pass, otherwise `RiskCheckResult::Fail`
    pub fn validate_order(
        &self,
        symbol: &str,
        quantity: &Decimal,
        price: &Decimal,
        side: Side,
    ) -> RiskCheckResult {
        // Collect all violations
        let mut violations: Vec<Box<RiskViolation>> = Vec::new();

        // Check order size
        if let Some(max_order) = self.limits.max_order_size {
            if *quantity > max_order {
                violations.push(Box::new(RiskViolation::OrderSizeExceeded {
                    size: *quantity,
                    max: max_order,
                }));
            }
        }

        // Check notional value
        if let Some(max_notional) = self.limits.max_notional_per_order {
            let notional = *quantity * price;
            if notional > max_notional {
                violations.push(Box::new(RiskViolation::NotionalExceeded {
                    notional,
                    max: max_notional,
                }));
            }
        }

        // Check position limit
        let proposed = self.proposed_position(symbol, quantity, side);
        let current = self
            .position(symbol)
            .map(|p| p.quantity)
            .unwrap_or(Decimal::ZERO);

        if proposed.abs() > self.limits.max_position {
            violations.push(Box::new(RiskViolation::PositionLimitExceeded {
                symbol: symbol.to_string(),
                current,
                proposed,
                max: self.limits.max_position,
            }));
        }

        // Check concurrent positions limit
        if let Some(max_concurrent) = self.limits.max_concurrent_positions {
            let current_non_flat = self
                .positions
                .values()
                .filter(|p| !p.is_flat())
                .count();

            // Only count if this would open a new position
            let current_has_position = self
                .position(symbol)
                .map(|p| !p.is_flat())
                .unwrap_or(false);

            if !current_has_position && proposed != Decimal::ZERO && current_non_flat >= max_concurrent
            {
                violations.push(Box::new(RiskViolation::ConcurrentPositionLimitExceeded {
                    current: current_non_flat,
                    max: max_concurrent,
                }));
            }
        }

        // Check daily loss limit
        if self.daily_realized_pnl < self.limits.daily_loss_limit {
            violations.push(Box::new(RiskViolation::DailyLossExceeded {
                realized: self.daily_realized_pnl,
                limit: self.limits.daily_loss_limit,
                excess: self.limits.daily_loss_limit - self.daily_realized_pnl,
            }));
        }

        // Return result
        if violations.is_empty() {
            RiskCheckResult::Pass
        } else if violations.len() == 1 {
            RiskCheckResult::Fail(*violations.into_iter().next().unwrap())
        } else {
            RiskCheckResult::Fail(RiskViolation::Multiple(violations))
        }
    }

    /// Validate using explicit context (for testing/composition)
    ///
    /// This allows validation against a specific snapshot of positions
    /// without modifying the manager's state.
    pub fn validate_with_context(
        &self,
        symbol: &str,
        quantity: &Decimal,
        price: &Decimal,
        side: Side,
        context: &ValidationContext<'_>,
    ) -> RiskCheckResult {
        let mut violations: Vec<Box<RiskViolation>> = Vec::new();

        // Check order size
        if let Some(max_order) = self.limits.max_order_size {
            if *quantity > max_order {
                violations.push(Box::new(RiskViolation::OrderSizeExceeded {
                    size: *quantity,
                    max: max_order,
                }));
            }
        }

        // Check notional value
        if let Some(max_notional) = self.limits.max_notional_per_order {
            let notional = *quantity * price;
            if notional > max_notional {
                violations.push(Box::new(RiskViolation::NotionalExceeded {
                    notional,
                    max: max_notional,
                }));
            }
        }

        // Check position limit
        let current = context
            .positions
            .get(symbol)
            .map(|p| p.quantity)
            .unwrap_or(Decimal::ZERO);
        let proposed = current + (*quantity * side.sign());

        if proposed.abs() > self.limits.max_position {
            violations.push(Box::new(RiskViolation::PositionLimitExceeded {
                symbol: symbol.to_string(),
                current,
                proposed,
                max: self.limits.max_position,
            }));
        }

        // Check concurrent positions limit
        if let Some(max_concurrent) = self.limits.max_concurrent_positions {
            let current_non_flat = context
                .positions
                .values()
                .filter(|p| !p.is_flat())
                .count();

            let current_has_position = context
                .positions
                .get(symbol)
                .map(|p| !p.is_flat())
                .unwrap_or(false);

            if !current_has_position && proposed != Decimal::ZERO && current_non_flat >= max_concurrent
            {
                violations.push(Box::new(RiskViolation::ConcurrentPositionLimitExceeded {
                    current: current_non_flat,
                    max: max_concurrent,
                }));
            }
        }

        // Check daily loss limit
        if context.daily_realized_pnl < self.limits.daily_loss_limit {
            violations.push(Box::new(RiskViolation::DailyLossExceeded {
                realized: context.daily_realized_pnl,
                limit: self.limits.daily_loss_limit,
                excess: self.limits.daily_loss_limit - context.daily_realized_pnl,
            }));
        }

        if violations.is_empty() {
            RiskCheckResult::Pass
        } else if violations.len() == 1 {
            RiskCheckResult::Fail(*violations.into_iter().next().unwrap())
        } else {
            RiskCheckResult::Fail(RiskViolation::Multiple(violations))
        }
    }

    /// Apply a fill and update positions
    ///
    /// Updates the position for the symbol and realized PnL.
    /// Returns the updated position.
    pub fn apply_fill(&mut self, fill: &Fill) -> &Position {
        let position = self.position_mut(&fill.symbol);
        let old_realized = position.realized_pnl;

        position.apply_fill(
            &fill.quantity,
            &fill.price,
            fill.side,
            fill.timestamp_ns,
        );

        // Update daily PnL (change in realized PnL)
        let pnl_change = position.realized_pnl - old_realized;
        if pnl_change != Decimal::ZERO {
            self.daily_realized_pnl += pnl_change;
        }

        // Get the key to avoid borrow issues
        let key = fill.symbol.clone();
        self.positions.get(&key).unwrap()
    }

    /// Update unrealized PnL for all positions given current mark prices
    ///
    /// # Arguments
    ///
    /// * `mark_prices` - Map of symbol to current mark price
    pub fn update_unrealized_pnl(&mut self, mark_prices: &HashMap<Symbol, Decimal>) {
        self.daily_unrealized_pnl = Decimal::ZERO;

        for (symbol, position) in self.positions.iter_mut() {
            if let Some(mark_price) = mark_prices.get(symbol) {
                position.update_unrealized_pnl(mark_price);
                self.daily_unrealized_pnl += position.unrealized_pnl;
            }
        }
    }

    /// Get total exposure (sum of absolute positions)
    pub fn total_exposure(&self) -> Decimal {
        self.positions
            .values()
            .map(|p| p.abs_quantity())
            .fold(Decimal::ZERO, |acc, qty| acc + qty)
    }

    /// Get gross notional exposure at given prices
    pub fn gross_notional(&self, prices: &HashMap<Symbol, Decimal>) -> Decimal {
        self.positions
            .values()
            .filter_map(|p| {
                prices.get(&p.symbol).map(|price| p.notional_value(price))
            })
            .fold(Decimal::ZERO, |acc, notional| acc + notional)
    }

    /// Get net notional exposure at given prices
    pub fn net_notional(&self, prices: &HashMap<Symbol, Decimal>) -> Decimal {
        self.positions
            .values()
            .filter_map(|p| {
                prices.get(&p.symbol).map(|price| p.quantity * price)
            })
            .fold(Decimal::ZERO, |acc, notional| acc + notional)
    }

    /// Reset daily PnL (call at market open)
    pub fn reset_daily_pnl(&mut self) {
        self.daily_realized_pnl = Decimal::ZERO;
        self.daily_unrealized_pnl = Decimal::ZERO;
    }

    /// Reset all state (positions and PnL)
    pub fn reset_all(&mut self) {
        self.positions.clear();
        self.daily_realized_pnl = Decimal::ZERO;
        self.daily_unrealized_pnl = Decimal::ZERO;
    }

    /// Get number of non-flat positions
    pub fn position_count(&self) -> usize {
        self.positions.values().filter(|p| !p.is_flat()).count()
    }

    /// Check if we would breach risk limits with this order
    ///
    /// Similar to validate_order but returns bool for quick checks
    pub fn would_breach_limits(
        &self,
        symbol: &str,
        quantity: &Decimal,
        side: Side,
    ) -> bool {
        let proposed = self.proposed_position(symbol, quantity, side);
        proposed.abs() > self.limits.max_position
    }

    /// Get remaining capacity for a symbol
    pub fn remaining_capacity(&self, symbol: &str) -> Decimal {
        let current = self
            .position(symbol)
            .map(|p| p.quantity.abs())
            .unwrap_or(Decimal::ZERO);
        (self.limits.max_position - current).max(Decimal::ZERO)
    }

    /// Get daily loss utilization (0.0 to 1.0 where higher = closer to limit)
    ///
    /// Returns None if daily_loss_limit is zero
    pub fn daily_loss_utilization(&self) -> Option<Decimal> {
        if self.limits.daily_loss_limit == Decimal::ZERO {
            return None;
        }

        // For negative limits: realized_pnl / limit (both negative, ratio is positive)
        // At limit: realized_pnl = -5000, limit = -5000, ratio = 1.0
        // Halfway: realized_pnl = -2500, limit = -5000, ratio = 0.5
        let ratio = self.daily_realized_pnl.abs() / self.limits.daily_loss_limit.abs();
        Some(ratio.min(Decimal::ONE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Fill, Position};
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    fn create_test_manager() -> RiskManager {
        let limits = RiskLimit::new(dec!(1000), dec!(-5000))
            .with_max_order_size(dec!(500))
            .with_max_notional(dec!(50000))
            .with_max_concurrent_positions(5);
        RiskManager::new(limits)
    }

    #[test]
    fn test_risk_manager_new() {
        let manager = create_test_manager();
        assert!(manager.positions().is_empty());
        assert_eq!(manager.daily_realized_pnl(), Decimal::ZERO);
    }

    #[test]
    fn test_validate_order_pass() {
        let manager = create_test_manager();
        // 100 qty * 10 price = 1,000 notional (under 50k limit)
        let result = manager.validate_order("BTC", &dec!(100), &dec!(10), Side::Buy);
        assert!(result.is_pass());
    }

    #[test]
    fn test_validate_order_size_exceeded() {
        // Create manager without notional limit
        let limits = RiskLimit::new(dec!(1000), dec!(-5000))
            .with_max_order_size(dec!(500));
        let manager = RiskManager::new(limits);
        
        let result = manager.validate_order("BTC", &dec!(600), &dec!(100), Side::Buy);
        assert!(result.is_fail());
        
        let violation = result.violation().unwrap();
        assert!(matches!(
            violation,
            RiskViolation::OrderSizeExceeded { size, .. } if *size == dec!(600)
        ));
    }

    #[test]
    fn test_validate_notional_exceeded() {
        // Create manager without position limit, only notional
        let limits = RiskLimit::new(dec!(10000), dec!(-5000))
            .with_max_notional(dec!(50000));
        let manager = RiskManager::new(limits);
        
        // 1000 * 100 = 100,000 notional > 50,000 limit
        let result = manager.validate_order("BTC", &dec!(1000), &dec!(100), Side::Buy);
        assert!(result.is_fail());
        
        let violation = result.violation().unwrap();
        assert!(matches!(
            violation,
            RiskViolation::NotionalExceeded { notional, .. } if *notional == dec!(100000)
        ));
    }

    #[test]
    fn test_validate_position_limit() {
        let mut manager = create_test_manager();
        
        // Add existing position of 800
        let fill = Fill::new(1, "BTC", Side::Buy, dec!(800), dec!(100), 1000);
        manager.apply_fill(&fill);
        assert_eq!(manager.position("BTC").unwrap().quantity, dec!(800));
        
        // Try to buy 500 more = would be 1300 > 1000 limit
        let result = manager.validate_order("BTC", &dec!(500), &dec!(100), Side::Buy);
        assert!(result.is_fail());
        
        if let RiskViolation::PositionLimitExceeded {
            current, proposed, max, ..
        } = result.violation().unwrap()
        {
            assert_eq!(*current, dec!(800));
            assert_eq!(*proposed, dec!(1300));
            assert_eq!(*max, dec!(1000));
        } else {
            panic!("Expected PositionLimitExceeded");
        }
        
        // But buying 100 more would be OK (900 total)
        let result = manager.validate_order("BTC", &dec!(100), &dec!(100), Side::Buy);
        assert!(result.is_pass());
    }

    #[test]
    fn test_validate_daily_loss_exceeded() {
        let mut manager = create_test_manager();
        manager.daily_realized_pnl = dec!(-6000); // Lost 6000, limit is -5000
        
        let result = manager.validate_order("BTC", &dec!(100), &dec!(100), Side::Buy);
        assert!(result.is_fail());
        assert!(matches!(
            result.violation().unwrap(),
            RiskViolation::DailyLossExceeded { realized, limit, .. }
            if *realized == dec!(-6000) && *limit == dec!(-5000)
        ));
    }

    #[test]
    fn test_validate_concurrent_positions() {
        let mut manager = create_test_manager();
        
        // Fill 5 different symbols to reach limit
        for i in 0..5 {
            let symbol = format!("SYM{}", i);
            let fill = Fill::new(i as u64, &symbol, Side::Buy, dec!(100), dec!(100), 1000);
            manager.apply_fill(&fill);
        }
        
        assert_eq!(manager.position_count(), 5);
        
        // Try to open a 6th position
        let result = manager.validate_order("NEW", &dec!(100), &dec!(100), Side::Buy);
        assert!(result.is_fail());
        assert!(matches!(
            result.violation().unwrap(),
            RiskViolation::ConcurrentPositionLimitExceeded { current, max }
            if *current == 5 && *max == 5
        ));
    }

    #[test]
    fn test_apply_fill_updates_position() {
        let mut manager = create_test_manager();
        
        let fill = Fill::new(1, "BTC", Side::Buy, dec!(100), dec!(50000), 1000);
        manager.apply_fill(&fill);
        
        let pos = manager.position("BTC").unwrap();
        assert_eq!(pos.quantity, dec!(100));
        assert_eq!(pos.abs_quantity(), dec!(100));
    }

    #[test]
    fn test_apply_fill_realizes_pnl() {
        let mut manager = create_test_manager();
        
        // Buy 100 at 100
        let fill1 = Fill::new(1, "BTC", Side::Buy, dec!(100), dec!(100), 1000);
        manager.apply_fill(&fill1);
        assert_eq!(manager.daily_realized_pnl(), Decimal::ZERO);
        
        // Sell 100 at 110 (profit of 10 per unit = 1000 total)
        let fill2 = Fill::new(2, "BTC", Side::Sell, dec!(100), dec!(110), 2000);
        manager.apply_fill(&fill2);
        assert_eq!(manager.daily_realized_pnl(), dec!(1000));
        
        // Position is now flat
        assert!(manager.position("BTC").unwrap().is_flat());
    }

    #[test]
    fn test_apply_fill_realizes_loss() {
        let mut manager = create_test_manager();
        
        // Buy 100 at 100
        let fill1 = Fill::new(1, "BTC", Side::Buy, dec!(100), dec!(100), 1000);
        manager.apply_fill(&fill1);
        
        // Sell 50 at 90 (loss of 10 per unit = 500 total)
        let fill2 = Fill::new(2, "BTC", Side::Sell, dec!(50), dec!(90), 2000);
        manager.apply_fill(&fill2);
        assert_eq!(manager.daily_realized_pnl(), dec!(-500));
        
        // Sell remaining 50 at 95 (loss of 5 per unit = 250 total)
        let fill3 = Fill::new(3, "BTC", Side::Sell, dec!(50), dec!(95), 3000);
        manager.apply_fill(&fill3);
        assert_eq!(manager.daily_realized_pnl(), dec!(-750));
    }

    #[test]
    fn test_update_unrealized_pnl() {
        let mut manager = create_test_manager();
        
        let fill = Fill::new(1, "BTC", Side::Buy, dec!(100), dec!(100), 1000);
        manager.apply_fill(&fill);
        
        // Price drops to 90
        let mut prices: HashMap<Symbol, Decimal> = HashMap::new();
        prices.insert("BTC".to_string(), dec!(90));
        manager.update_unrealized_pnl(&prices);
        
        let pos = manager.position("BTC").unwrap();
        assert_eq!(pos.unrealized_pnl, dec!(-1000)); // 100 * (90 - 100)
    }

    #[test]
    fn test_total_exposure() {
        let mut manager = create_test_manager();
        
        // Long 100 BTC, Short 50 ETH
        manager.apply_fill(&Fill::new(1, "BTC", Side::Buy, dec!(100), dec!(100), 1000));
        manager.apply_fill(&Fill::new(2, "ETH", Side::Sell, dec!(50), dec!(10), 2000));
        
        // Total exposure = 100 + 50 = 150
        assert_eq!(manager.total_exposure(), dec!(150));
    }

    #[test]
    fn test_gross_notional() {
        let mut manager = create_test_manager();
        
        manager.apply_fill(&Fill::new(1, "BTC", Side::Buy, dec!(10), dec!(50000), 1000));
        manager.apply_fill(&Fill::new(2, "ETH", Side::Buy, dec!(100), dec!(3000), 2000));
        
        let mut prices = HashMap::new();
        prices.insert("BTC".to_string(), dec!(51000));
        prices.insert("ETH".to_string(), dec!(3100));
        
        // Gross = 10*51000 + 100*3100 = 510,000 + 310,000 = 820,000
        assert_eq!(manager.gross_notional(&prices), dec!(820000));
    }

    #[test]
    fn test_net_notional() {
        let mut manager = create_test_manager();
        
        // Long 10 BTC, Short 5 ETH
        manager.apply_fill(&Fill::new(1, "BTC", Side::Buy, dec!(10), dec!(50000), 1000));
        manager.apply_fill(&Fill::new(2, "ETH", Side::Sell, dec!(5), dec!(3000), 2000));
        
        let mut prices = HashMap::new();
        prices.insert("BTC".to_string(), dec!(51000));
        prices.insert("ETH".to_string(), dec!(3100));
        
        // Net = 10*51000 + (-5)*3100 = 510,000 - 15,500 = 494,500
        assert_eq!(manager.net_notional(&prices), dec!(494500));
    }

    #[test]
    fn test_reset_daily_pnl() {
        let mut manager = create_test_manager();
        manager.daily_realized_pnl = dec!(-1000);
        
        manager.reset_daily_pnl();
        
        assert_eq!(manager.daily_realized_pnl(), Decimal::ZERO);
    }

    #[test]
    fn test_reset_all() {
        let mut manager = create_test_manager();
        manager.apply_fill(&Fill::new(1, "BTC", Side::Buy, dec!(100), dec!(100), 1000));
        manager.daily_realized_pnl = dec!(1000);
        
        manager.reset_all();
        
        assert!(manager.positions().is_empty());
        assert_eq!(manager.daily_realized_pnl(), Decimal::ZERO);
    }

    #[test]
    fn test_remaining_capacity() {
        let mut manager = create_test_manager();
        
        // No position yet - full capacity
        assert_eq!(manager.remaining_capacity("BTC"), dec!(1000));
        
        // 300 position - 700 remaining
        manager.apply_fill(&Fill::new(1, "BTC", Side::Buy, dec!(300), dec!(100), 1000));
        assert_eq!(manager.remaining_capacity("BTC"), dec!(700));
        
        // 900 position - 100 remaining
        manager.apply_fill(&Fill::new(2, "BTC", Side::Buy, dec!(600), dec!(100), 2000));
        assert_eq!(manager.remaining_capacity("BTC"), dec!(100));
        
        // 1100 position - 0 remaining (already exceeded)
        manager.apply_fill(&Fill::new(3, "BTC", Side::Buy, dec!(200), dec!(100), 3000));
        assert_eq!(manager.remaining_capacity("BTC"), Decimal::ZERO);
    }

    #[test]
    fn test_daily_loss_utilization() {
        let mut manager = create_test_manager();
        
        manager.daily_realized_pnl = dec!(-2500); // Half of -5000 limit
        assert_eq!(manager.daily_loss_utilization().unwrap(), dec!(0.5));
        
        manager.daily_realized_pnl = dec!(-5000); // At limit
        assert_eq!(manager.daily_loss_utilization().unwrap(), dec!(1.0));
        
        // Cap at 1.0 if exceeded
        manager.daily_realized_pnl = dec!(-6000);
        assert_eq!(manager.daily_loss_utilization().unwrap(), dec!(1.0));
    }

    #[test]
    fn test_validate_with_context() {
        let manager = create_test_manager();
        
        let mut positions: HashMap<Symbol, Position> = HashMap::new();
        let mut pos = Position::new("BTC");
        pos.quantity = dec!(800);
        positions.insert("BTC".to_string(), pos);
        
        let context = ValidationContext::new(&positions, Decimal::ZERO, 1000);
        
        // Would exceed position limit
        let result = manager.validate_with_context(
            "BTC",
            &dec!(300),
            &dec!(100),
            Side::Buy,
            &context,
        );
        assert!(result.is_fail());
        
        // Would be acceptable
        let result = manager.validate_with_context(
            "BTC",
            &dec!(100),
            &dec!(100),
            Side::Buy,
            &context,
        );
        assert!(result.is_pass());
    }

    #[test]
    fn test_multiple_violations() {
        let mut manager = create_test_manager();
        
        // Create a scenario with multiple violations
        manager.daily_realized_pnl = dec!(-6000); // Exceeds daily loss
        
        // Add existing position
        let fill = Fill::new(1, "BTC", Side::Buy, dec!(900), dec!(100), 1000);
        manager.apply_fill(&fill);
        
        // Order that exceeds both daily loss AND position limit
        let result = manager.validate_order("BTC", &dec!(200), &dec!(100), Side::Buy);
        
        assert!(result.is_fail());
        if let RiskViolation::Multiple(violations) = result.violation().unwrap() {
            assert_eq!(violations.len(), 2);
        } else {
            panic!("Expected Multiple violations");
        }
    }
}