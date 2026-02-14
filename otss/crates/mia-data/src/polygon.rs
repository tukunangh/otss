//! OTSS Data - Polygon.io adapter
//!
//! Provides WebSocket and REST API access to Polygon for:
//! - Real-time trades and quotes
//! - Aggregates/OHLCV
//! - Historical data

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Polygon client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonConfig {
    /// API key
    pub api_key: String,
    /// WebSocket endpoint
    pub ws_endpoint: String,
    /// REST endpoint
    pub rest_endpoint: String,
}

impl Default for PolygonConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            ws_endpoint: "wss://socket.polygon.io".to_string(),
            rest_endpoint: "https://api.polygon.io".to_string(),
        }
    }
}

/// Polygon data client
#[derive(Debug, Clone)]
pub struct PolygonClient {
    config: PolygonConfig,
    connected: bool,
    last_seq_num: u64,
}

impl PolygonClient {
    /// Create a new Polygon client with the given configuration
    pub fn new(config: PolygonConfig) -> Self {
        Self {
            config,
            connected: false,
            last_seq_num: 0,
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &PolygonConfig {
        &self.config
    }

    /// Connect to Polygon WebSocket
    pub async fn connect(&mut self) -> anyhow::Result<()> {
        tracing::info!("Connecting to Polygon.io: {}", self.config.ws_endpoint);
        // TODO: Implement actual Polygon connection
        self.connected = true;
        Ok(())
    }

    /// Disconnect from Polygon
    pub async fn disconnect(&mut self) -> anyhow::Result<()> {
        tracing::info!("Disconnecting from Polygon.io");
        self.connected = false;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Authenticate with API key
    pub async fn authenticate(&mut self) -> anyhow::Result<()> {
        tracing::info!("Authenticating with Polygon.io");
        // Authentication is done during connection
        Ok(())
    }

    /// Subscribe to trades for given symbols
    pub async fn subscribe_trades(&mut self, symbols: &[impl AsRef<str>]) -> anyhow::Result<()> {
        let symbols: Vec<_> = symbols.iter().map(|s| s.as_ref()).collect();
        tracing::info!("Subscribing to trades via Polygon: {:?}", symbols);
        Ok(())
    }

    /// Subscribe to quotes (L1) for given symbols
    pub async fn subscribe_quotes(&mut self, symbols: &[impl AsRef<str>]) -> anyhow::Result<()> {
        let symbols: Vec<_> = symbols.iter().map(|s| s.as_ref()).collect();
        tracing::info!("Subscribing to quotes via Polygon: {:?}", symbols);
        Ok(())
    }

    /// Request aggregates/bars for a symbol
    pub async fn get_aggregates(
        &self,
        symbol: &str,
        multiplier: u32,
        timespan: &str,
        from: &str,
        to: &str,
    ) -> anyhow::Result<Vec<crate::types::OHLCV>> {
        tracing::info!(
            "Requesting {}m {} aggregates for {} from {} to {}",
            multiplier,
            timespan,
            symbol,
            from,
            to
        );
        // TODO: Implement REST API call
        Ok(vec![])
    }

    /// Get last trade for a symbol
    pub async fn get_last_trade(&self, symbol: &str) -> anyhow::Result<crate::types::Trade> {
        tracing::info!("Getting last trade for {}", symbol);
        Ok(crate::types::Trade {
            trade_id: "0".to_string(),
            symbol: symbol.to_string(),
            price: Decimal::new(0, 0),
            size: 0,
            side: crate::types::TradeSide::Unknown,
            exchange: "POLY".to_string(),
            timestamp: 0,
        })
    }
}

/// Polygon adapter
#[derive(Debug, Clone)]
pub struct PolygonAdapter {
    client: PolygonClient,
}

impl PolygonAdapter {
    /// Create a new adapter
    pub fn new(client: PolygonClient) -> Self {
        Self { client }
    }

    /// Get the client reference
    pub fn client(&self) -> &PolygonClient {
        &self.client
    }

    /// Get mutable client reference
    pub fn client_mut(&mut self) -> &mut PolygonClient {
        &mut self.client
    }
}

/// Polygon trade message
#[derive(Debug, Clone, Deserialize)]
pub struct PolygonTradeMsg {
    pub ev: String,
    pub sym: String,
    pub x: u32,
    pub i: String,
    pub z: u8,
    pub p: Decimal,
    pub s: u64,
    pub t: u64,
    pub q: u64,
    pub trf: u32,
}

/// Polygon quote message
#[derive(Debug, Clone, Deserialize)]
pub struct PolygonQuoteMsg {
    pub ev: String,
    pub sym: String,
    pub bx: u32,
    pub bp: Decimal,
    pub bs: u64,
    pub ax: u32,
    pub ap: Decimal,
    pub as_: u64,
    pub t: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polygon_config_default() {
        let config = PolygonConfig::default();
        assert_eq!(config.ws_endpoint, "wss://socket.polygon.io");
    }

    #[test]
    fn test_polygon_client_created() {
        let config = PolygonConfig::default();
        let client = PolygonClient::new(config);
        assert!(!client.is_connected());
    }
}
