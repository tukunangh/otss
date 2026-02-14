//! OTSS Integration - Broker adapter types
//!
//! Common types for broker integration.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Broker order types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokerOrderType {
    /// Market order
    Market,
    /// Limit order
    Limit { price: Decimal },
    /// Stop order
    Stop { stop_price: Decimal },
    /// Stop limit order
    StopLimit { stop_price: Decimal, limit_price: Decimal },
}

/// Broker order side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokerOrderSide {
    /// Buy
    Buy,
    /// Sell
    Sell,
    /// Sell short
    SellShort,
}

/// Time in force
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeInForce {
    /// Good till cancelled
    Day,
    /// Good till cancelled
    GTC,
    /// Immediate or cancel
    IOC,
    /// Fill or kill
    FOK,
}

/// Order to send to a broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerOrder {
    /// Symbol
    pub symbol: String,
    /// Order type
    pub order_type: BrokerOrderType,
    /// Side
    pub side: BrokerOrderSide,
    /// Quantity
    pub quantity: u64,
    /// Time in force
    pub tif: TimeInForce,
}

/// Order status from broker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokerOrderStatus {
    /// Pending submission
    Pending,
    /// Submitted to broker
    Submitted,
    /// Partially filled
    PartiallyFilled,
    /// Completely filled
    Filled,
    /// Cancelled
    Cancelled,
    /// Rejected
    Rejected { reason: String },
}

impl fmt::Display for BrokerOrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrokerOrderStatus::Pending => write!(f, "PENDING"),
            BrokerOrderStatus::Submitted => write!(f, "SUBMITTED"),
            BrokerOrderStatus::PartiallyFilled => write!(f, "PARTIALLY_FILLED"),
            BrokerOrderStatus::Filled => write!(f, "FILLED"),
            BrokerOrderStatus::Cancelled => write!(f, "CANCELLED"),
            BrokerOrderStatus::Rejected { reason } => write!(f, "REJECTED: {}", reason),
        }
    }
}

/// Execution fill from broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFill {
    /// Symbol
    pub symbol: String,
    /// Fill ID from broker
    pub fill_id: String,
    /// Quantity filled
    pub quantity: u64,
    /// Fill price
    pub price: Decimal,
    /// Side
    pub side: BrokerOrderSide,
    /// Timestamp
    pub timestamp: u64,
}

/// Account info from broker
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccountInfo {
    /// Account ID
    pub account_id: String,
    /// Account type
    pub account_type: AccountType,
    /// Buying power
    pub buying_power: Decimal,
    /// Cash balance
    pub cash_balance: Decimal,
}

/// Account type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    /// Cash account
    Cash,
    /// Margin account
    Margin,
}

/// Position from broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerPosition {
    /// Symbol
    pub symbol: String,
    /// Quantity (positive for long, negative for short)
    pub quantity: i64,
    /// Average cost
    pub avg_cost: Decimal,
    /// Market price
    pub market_price: Decimal,
    /// Unrealized P&L
    pub unrealized_pnl: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_broker_order_creation() {
        let order = BrokerOrder {
            symbol: "AAPL".to_string(),
            order_type: BrokerOrderType::Market,
            side: BrokerOrderSide::Buy,
            quantity: 100,
            tif: TimeInForce::Day,
        };
        assert_eq!(order.quantity, 100);
    }

    #[test]
    fn test_order_status_display() {
        assert_eq!(BrokerOrderStatus::Filled.to_string(), "FILLED");
        let rejected = BrokerOrderStatus::Rejected {
            reason: "test".to_string(),
        };
        assert_eq!(rejected.to_string(), "REJECTED: test");
    }

    #[test]
    fn test_limit_order_price() {
        let price = Decimal::from_str("150.00").unwrap();
        let order_type = BrokerOrderType::Limit { price };
        
        match order_type {
            BrokerOrderType::Limit { price: p } => assert_eq!(p, price),
            _ => panic!("Expected Limit order type"),
        }
    }
}
