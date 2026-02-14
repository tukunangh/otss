//! OTSS Integration - Broker adapter trait
//!
//! Defines the interface for broker adapters.

use serde::{Deserialize, Serialize};

trait BrokerAdapter {
    /// Connect to the broker
    fn connect(&mut self) -> anyhow::Result<()>;
    /// Disconnect from the broker
    fn disconnect(&mut self) -> anyhow::Result<()>;
    /// Check if connected
    fn is_connected(&self) -> bool;
}  /// Connection configuration for broker adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Broker host
    pub host: String,
    /// Broker port
    pub port: u16,
    /// Use TLS encryption
    pub use_tls: bool,
    /// API key
    pub api_key: Option<String>,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            use_tls: true,
            api_key: None,
        }
    }
}
/// Result of an order execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether the execution was successful
    pub success: bool,
    /// Execution details
    pub message: String,
}

/// Broker adapter errors
#[derive(Debug, thiserror::Error)]
pub enum BrokerError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Order rejected: {0}")]
    OrderRejected(String),
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Rate limited")]
    RateLimited,
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    #[error("Insufficient funds")]
    InsufficientFunds,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_config_default() {
        let config = ConnectionConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert!(config.use_tls);
    }

    #[test]
    fn test_execution_result() {
        let result = ExecutionResult {
            success: true,
            message: "Order executed".to_string(),
        };
        assert!(result.success);
    }
}
