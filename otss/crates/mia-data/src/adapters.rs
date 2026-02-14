//! OTSS Data - Data adapter trait and configuration
//!
//! Defines the interface for market data adapters.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Data source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSource {
    /// Databento feed
    Databento,
    /// Polygon.io feed
    Polygon,
    /// CSV file
    File,
}

impl fmt::Display for DataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataSource::Databento => write!(f, "databento"),
            DataSource::Polygon => write!(f, "polygon"),
            DataSource::File => write!(f, "file"),
        }
    }
}

/// Feed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedConfig {
    /// Data source
    pub source: DataSource,
    /// API key
    pub api_key: Option<String>,
    /// Base URL for API
    pub url: Option<String>,
    /// Buffer size for incoming data
    pub buffer_size: usize,
    /// Reconnect on disconnect
    pub auto_reconnect: bool,
}

impl Default for FeedConfig {
    fn default() -> Self {
        Self {
            source: DataSource::Polygon,
            api_key: None,
            url: None,
            buffer_size: 10000,
            auto_reconnect: true,
        }
    }
}

/// Data adapter trait
pub trait DataAdapter {
    /// Connect to the data source
    fn connect(&mut self) -> anyhow::Result<()>;
    /// Disconnect from the data source
    fn disconnect(&mut self) -> anyhow::Result<()>;
    /// Check if connected
    fn is_connected(&self) -> bool;
    /// Subscribe to symbols
    fn subscribe(&mut self, symbols: &[impl AsRef<str>]) -> anyhow::Result<()>;
    /// Unsubscribe from symbols
    fn unsubscribe(&mut self, symbols: &[impl AsRef<str>]) -> anyhow::Result<()>;
}

/// Feed status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedStatus {
    /// Not connected
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected and receiving data
    Connected,
    /// Error state
    Error,
}

impl fmt::Display for FeedStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeedStatus::Disconnected => write!(f, "disconnected"),
            FeedStatus::Connecting => write!(f, "connecting"),
            FeedStatus::Connected => write!(f, "connected"),
            FeedStatus::Error => write!(f, "error"),
        }
    }
}

/// Adapter errors
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Rate limited")]
    RateLimited,
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_source_display() {
        assert_eq!(DataSource::Databento.to_string(), "databento");
        assert_eq!(DataSource::Polygon.to_string(), "polygon");
    }

    #[test]
    fn test_feed_config_default() {
        let config = FeedConfig::default();
        assert!(config.auto_reconnect);
        assert_eq!(config.buffer_size, 10000);
    }
}
