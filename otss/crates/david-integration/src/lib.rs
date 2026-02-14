//! OTSS Integration - Broker adapter implementations
//!
//! This crate provides broker adapter implementations for:
//! - DAS Trader (primary target)
//! - Generic broker API interfaces
//! - Order management through broker APIs
//!
//! All implementations follow the port/adapter pattern with the core
//! crate handling domain logic.

#![allow(dead_code, unused_imports)]

pub mod adapters;
pub mod das;
pub mod generic;
pub mod types;

pub use adapters::{BrokerAdapter, ConnectionConfig, ExecutionResult};
pub use das::{DasTraderAdapter, DasTraderConfig};
pub use generic::GenericBrokerAdapter;
pub use types::{BrokerOrder, BrokerOrderStatus, ExecutionFill};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the integration module
pub fn init() {
    tracing::info!("Initializing OTSS Integration module v{}", VERSION);
}

/// List available broker adapters
pub fn available_adapters() -> Vec<&str> {
    vec![
        "das-trader",
        "generic",
    ]
}

/// Get default connection config
pub fn default_connection_config() -> ConnectionConfig {
    ConnectionConfig {
        host: "localhost".to_string(),
        port: 8080,
        use_tls: true,
        api_key: None,
    }
}
