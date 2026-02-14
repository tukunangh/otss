//! OTSS Data - Market data adapters for Databento and Polygon
//!
//! This crate provides real-time and historical market data interfaces
//! for the OTSS trading system, supporting:
//! - Databento MBP-10 (Market-By-Price L2) feeds
//! - Polygon.io WebSocket and REST APIs
//! - Symbol mapping and normalization
//! - Trade/book data processing

#![allow(dead_code, unused_imports)]

pub mod adapters;
pub mod databento;
pub mod polygon;
pub mod types;

pub use adapters::{DataAdapter, DataSource, FeedConfig};
pub use databento::{DatabentoAdapter, DatabentoClient};
pub use polygon::{PolygonAdapter, PolygonClient};
pub use types::{BookLevel, MarketBook, MarketData, Trade, OHLCV};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the data module
pub fn init() {
    tracing::info!("Initializing OTSS Data module v{}", VERSION);
}

/// Get available data sources
pub fn available_sources() -> Vec<DataSource> {
    vec![
        DataSource::Databento,
        DataSource::Polygon,
    ]
}
