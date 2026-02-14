//! OTSS Core - Financial types module
//!
//! Provides precise decimal types for financial calculations.

use rust_decimal::Decimal;
use rust_decimal::prelude::Zero;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};
use std::str::FromStr;

/// A price value represented as a precise decimal
///
/// Using Decimal instead of f64 to avoid floating point rounding errors
/// in financial calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Price(Decimal);

impl Price {
    /// Create a new price from a string representation
    pub fn from_str(amount: &str) -> Result<Self, rust_decimal::Error> {
        Decimal::from_str(amount).map(Self)
    }

    /// Convert to a Decimal
    pub fn to_decimal(&self) -> Decimal {
        self.0
    }

    /// Get zero price
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl From<Decimal> for Price {
    fn from(d: Decimal) -> Self {
        Self(d)
    }
}

impl FromStr for Price {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Decimal::from_str(s).map(Self)
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for Price {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Price {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for Price {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Mul<Decimal> for Price {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self::Output {
        Self(self.0 * rhs)
    }
}

/// A quantity value represented as a precise decimal
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Quantity(Decimal);

impl Quantity {
    /// Create a new quantity from a string representation
    pub fn from_str(amount: &str) -> Result<Self, rust_decimal::Error> {
        Decimal::from_str(amount).map(Self)
    }

    /// Convert to a Decimal
    pub fn to_decimal(&self) -> Decimal {
        self.0
    }

    /// Get zero quantity
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Check if positive
    pub fn is_positive(&self) -> bool {
        self.0.is_sign_positive() && !self.0.is_zero()
    }

    /// Get absolute value
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }
}

impl From<Decimal> for Quantity {
    fn from(d: Decimal) -> Self {
        Self(d)
    }
}

impl From<i64> for Quantity {
    fn from(i: i64) -> Self {
        Self(Decimal::from(i))
    }
}

impl FromStr for Quantity {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Decimal::from_str(s).map(Self)
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for Quantity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Quantity {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul<Decimal> for Quantity {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl AddAssign for Quantity {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Quantity {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

/// A stock/option symbol
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(String);

impl Symbol {
    /// Create a new symbol from a string
    pub fn new(s: &str) -> Self {
        Self(s.to_uppercase())
    }

    /// Get the symbol as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Symbol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("Symbol cannot be empty".to_string())
        } else {
            Ok(Self::new(s))
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Represents a monetary amount
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Money(Decimal);

impl Money {
    /// Create money from decimal
    pub fn from_decimal(d: Decimal) -> Self {
        Self(d)
    }

    /// Create money from string
    pub fn from_str(s: &str) -> Result<Self, rust_decimal::Error> {
        Decimal::from_str(s).map(Self)
    }

    /// Get the decimal value
    pub fn to_decimal(&self) -> Decimal {
        self.0
    }

    /// Calculate money from price * quantity
    pub fn from_trade(price: Price, quantity: Quantity) -> Self {
        Self(price.to_decimal() * quantity.to_decimal())
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:.2}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_arithmetic() {
        let p1 = Price::from_str("100.50").unwrap();
        let p2 = Price::from_str("50.25").unwrap();

        assert_eq!((p1 + p2).to_string(), "150.75");
        assert_eq!((p1 - p2).to_string(), "50.25");
    }

    #[test]
    fn test_symbol_creation() {
        let sym = Symbol::new("aapl");
        assert_eq!(sym.as_str(), "AAPL");
    }

    #[test]
    fn test_money_calculation() {
        let price = Price::from_str("150.00").unwrap();
        let qty = Quantity::from(100);
        let money = Money::from_trade(price, qty);

        assert_eq!(money.to_string(), "$15000.00");
    }
}
