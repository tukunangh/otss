//! OTSS Core - Portfolio and position management module
//!
//! Handles portfolio tracking, position calculations, and P&L.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{Money, Price, Quantity, Symbol};

/// A position in a particular instrument
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    symbol: Symbol,
    quantity: Quantity,
    average_cost: Price,
    market_price: Option<Price>,
}

impl Position {
    /// Create a new position
    pub fn new(symbol: Symbol, quantity: Quantity, average_cost: Price) -> Self {
        Self {
            symbol,
            quantity,
            average_cost,
            market_price: None,
        }
    }

    /// Get the symbol
    pub fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    /// Get the quantity (positive for long, negative for short)
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Check if this is a long position
    pub fn is_long(&self) -> bool {
        self.quantity.is_positive()
    }

    /// Check if this is a short position
    pub fn is_short(&self) -> bool {
        !self.quantity.is_zero() && !self.quantity.is_positive()
    }

    /// Get the average cost basis
    pub fn average_cost(&self) -> Price {
        self.average_cost
    }

    /// Update the market price
    pub fn update_market_price(&mut self, price: Price) {
        self.market_price = Some(price);
    }

    /// Calculate unrealized P&L
    pub fn unrealized_pnl(&self) -> Option<Money> {
        self.market_price.map(|market| {
            let qty = self.quantity.to_decimal();
            let cost = self.average_cost.to_decimal();
            let price_diff = market.to_decimal() - cost;
            Money::from_decimal(price_diff * qty)
        })
    }

    /// Calculate market value
    pub fn market_value(&self) -> Option<Money> {
        self.market_price.map(|price| {
            Money::from_trade(price, self.quantity.abs())
        })
    }

    /// Calculate the total cost basis
    pub fn cost_basis(&self) -> Money {
        Money::from_trade(self.average_cost, self.quantity.abs())
    }
}

/// Represents a trading portfolio
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Portfolio {
    /// Positions keyed by symbol
    positions: HashMap<String, Position>,
    /// Available cash
    cash: Money,
    /// Total equity
    total_equity: Money,
}

impl Portfolio {
    /// Create a new portfolio with initial cash
    pub fn with_cash(initial_cash: Money) -> Self {
        Self {
            positions: HashMap::new(),
            cash: initial_cash,
            total_equity: initial_cash,
        }
    }

    /// Get a position by symbol
    pub fn position(&self, symbol: &Symbol) -> Option<&Position> {
        self.positions.get(symbol.as_str())
    }

    /// Get mutable reference to a position
    pub fn position_mut(&mut self, symbol: &Symbol) -> Option<&mut Position> {
        self.positions.get_mut(symbol.as_str())
    }

    /// Add or update a position
    pub fn update_position(&mut self, position: Position) {
        let symbol = position.symbol().as_str().to_string();
        self.positions.insert(symbol, position);
    }

    /// Get all positions
    pub fn positions(&self) -> impl Iterator<Item = &Position> {
        self.positions.values()
    }

    /// Get available cash
    pub fn cash(&self) -> Money {
        self.cash
    }

    /// Add cash (e.g., deposit, dividend)
    pub fn add_cash(&mut self, amount: Money) {
        self.cash = self.cash + amount;
    }

    /// Remove cash (e.g., withdrawal)
    pub fn remove_cash(&mut self, amount: Money) {
        self.cash = self.cash - amount;
    }

    /// Calculate total market value of all positions
    pub fn positions_value(&self) -> Money {
        self.positions
            .values()
            .filter_map(|p| p.market_value())
            .fold(Money::from_decimal(Decimal::ZERO), |acc, v| acc + v)
    }

    /// Calculate total equity (cash + positions value)
    pub fn total_equity(&self) -> Money {
        self.cash + self.positions_value()
    }

    /// Calculate total unrealized P&L
    pub fn total_unrealized_pnl(&self) -> Money {
        self.positions
            .values()
            .filter_map(|p| p.unrealized_pnl())
            .fold(Money::from_decimal(Decimal::ZERO), |acc, pnl| acc + pnl)
    }

    /// Get the number of positions
    pub fn position_count(&self) -> usize {
        self.positions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_position_creation() {
        let sym = Symbol::new("AAPL");
        let qty = Quantity::from(100);
        let price = Price::from_str("150.00").unwrap();
        let pos = Position::new(sym, qty, price);

        assert_eq!(pos.quantity().to_string(), "100");
        assert!(pos.is_long());
    }

    #[test]
    fn test_portfolio_creation() {
        let cash = Money::from_str("100000.00").unwrap();
        let portfolio = Portfolio::with_cash(cash);

        assert_eq!(portfolio.cash().to_string(), "$100000.00");
        assert_eq!(portfolio.position_count(), 0);
    }
}
