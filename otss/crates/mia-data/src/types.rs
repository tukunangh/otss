//! OTSS Data - Market data types
//!
//! Common types for market data representation.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::SystemTime;

/// Trade event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trade {
    /// Trade ID
    pub trade_id: String,
    /// Symbol
    pub symbol: String,
    /// Trade price
    pub price: Decimal,
    /// Trade size
    pub size: u64,
    /// Trade side from aggressor perspective
    pub side: TradeSide,
    /// Exchange where trade executed
    pub exchange: String,
    /// Timestamp
    pub timestamp: u64,
}

/// Trade side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeSide {
    /// Buyer-initiated
    Buy,
    /// Seller-initiated
    Sell,
    /// Unknown/unspecified
    Unknown,
}

/// Market book level (L2)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BookLevel {
    /// Price level
    pub price: Decimal,
    /// Size at this level
    pub size: u64,
    /// Number of orders
    pub count: u32,
}

/// Market book (order book)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketBook {
    /// Symbol
    pub symbol: String,
    /// Bid levels (sorted high to low)
    pub bids: Vec<BookLevel>,
    /// Ask levels (sorted low to high)
    pub asks: Vec<BookLevel>,
    /// Timestamp
    pub timestamp: u64,
}

/// OHLCV bar
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OHLCV {
    /// Open price
    pub open: Decimal,
    /// High price
    pub high: Decimal,
    /// Low price
    pub low: Decimal,
    /// Close price
    pub close: Decimal,
    /// Volume
    pub volume: u64,
    /// Timestamp
    pub timestamp: u64,
}

/// Market data event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarketData {
    /// Trade event
    Trade(Trade),
    /// Book update
    Book(MarketBook),
    /// OHLCV bar
    OHLCV(OHLCV),
    /// Heartbeat/ping
    Heartbeat,
}

/// Data subscription
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataSubscription {
    /// Symbols to subscribe to
    pub symbols: Vec<String>,
    /// Data types to receive
    pub data_types: Vec<DataType>,
}

/// Types of market data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// Trades only
    Trades,
    /// Quotes/L2
    Quotes,
    /// OHLCV bars
    Bars,
    /// All data
    All,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_creation() {
        let trade = Trade {
            trade_id: "12345".to_string(),
            symbol: "AAPL".to_string(),
            price: Decimal::from_str("150.25").unwrap(),
            size: 100,
            side: TradeSide::Buy,
            exchange: "NASDAQ".to_string(),
            timestamp: 1700000000000,
        };
        assert_eq!(trade.symbol, "AAPL");
    }

    #[test]
    fn test_market_book_creation() {
        let book = MarketBook {
            symbol: "TSLA".to_string(),
            bids: vec![BookLevel {
                price: Decimal::from_str("200.00").unwrap(),
                size: 500,
                count: 10,
            }],
            asks: vec![BookLevel {
                price: Decimal::from_str("200.05").unwrap(),
                size: 300,
                count: 5,
            }],
            timestamp: 1700000000000,
        };
        assert_eq!(book.symbol, "TSLA");
    }
}
