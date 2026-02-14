//! OTSS Data - Databento adapter
//!
//! Provides MBP-10 (Market-By-Price L2) data from Databento.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Databento client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabentoConfig {
    /// API key
    pub api_key: String,
    /// Dataset (e.g., "XNAS.ITCH")
    pub dataset: String,
    /// Schema (e.g., "mbp-10")
    pub schema: String,
}

impl Default for DatabentoConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            dataset: "XNAS.ITCH".to_string(),
            schema: "mbp-10".to_string(),
        }
    }
}

/// Databento data client
#[derive(Debug, Clone)]
pub struct DatabentoClient {
    config: DatabentoConfig,
    connected: bool,
}

impl DatabentoClient {
    /// Create a new Databento client with the given configuration
    pub fn new(config: DatabentoConfig) -> Self {
        Self {
            config,
            connected: false,
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &DatabentoConfig {
        &self.config
    }

    /// Connect to Databento
    pub async fn connect(&mut self) -> anyhow::Result<()> {
        tracing::info!("Connecting to Databento dataset: {}", self.config.dataset);
        // TODO: Implement actual Databento connection
        self.connected = true;
        Ok(())
    }

    /// Disconnect from Databento
    pub async fn disconnect(&mut self) -> anyhow::Result<()> {
        tracing::info!("Disconnecting from Databento");
        self.connected = false;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Subscribe to live data for given symbols
    pub async fn subscribe(&mut self, symbols: &[impl AsRef<str>]) -> anyhow::Result<()> {
        let symbols: Vec<_> = symbols.iter().map(|s| s.as_ref()).collect();
        tracing::info!("Subscribing to symbols via Databento: {:?}", symbols);
        Ok(())
    }

    /// Request historical data
    pub async fn request_historical(
        &self,
        symbols: &[impl AsRef<str>],
        start_ts: u64,
        end_ts: u64,
    ) -> anyhow::Result<Vec<crate::types::MarketData>> {
        tracing::info!(
            "Requesting historical data from Databento for {:?} [{} - {}]",
            symbols.iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
            start_ts,
            end_ts
        );
        // TODO: Implement historical data request
        Ok(vec![])
    }
}

/// Databento adapter
#[derive(Debug, Clone)]
pub struct DatabentoAdapter {
    client: DatabentoClient,
}

impl DatabentoAdapter {
    /// Create a new adapter
    pub fn new(client: DatabentoClient) -> Self {
        Self { client }
    }

    /// Get the client reference
    pub fn client(&self) -> &DatabentoClient {
        &self.client
    }

    /// Get mutable client reference
    pub fn client_mut(&mut self) -> &mut DatabentoClient {
        &mut self.client
    }
}

/// Databento MBP record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MBPRecord {
    /// Symbol
    pub symbol: String,
    /// Timestamp
    pub ts_event: u64,
    /// Trade price (if there's a trade at this event)
    pub trade_price: Option<Decimal>,
    /// Trade size (if there's a trade at this event)
    pub trade_size: u64,
    /// Bid levels
    pub bids: Vec<crate::types::BookLevel>,
    /// Ask levels
    pub asks: Vec<crate::types::BookLevel>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_databento_config_default() {
        let config = DatabentoConfig::default();
        assert_eq!(config.dataset, "XNAS.ITCH");
        assert_eq!(config.schema, "mbp-10");
    }

    #[test]
    fn test_databento_client_created() {
        let config = DatabentoConfig::default();
        let client = DatabentoClient::new(config);
        assert!(!client.is_connected());
    }
}
