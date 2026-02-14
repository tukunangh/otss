//! OTSS PM - Project Management and orchestration
//!
//! This crate provides tools for:
//! - System orchestration
//! - Service lifecycle management
//! - Configuration management
//! - Health monitoring and alerting

#![allow(dead_code, unused_imports)]

pub mod orchestration;
pub mod config;
pub mod health;
pub mod lifecycle;

pub use orchestration::{Orchestrator, SystemConfig, SystemState};
pub use config::{ConfigLoader, ConfigManager};
pub use health::SystemMonitor;
pub use lifecycle::{Service, ServiceManager};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the PM module
pub fn init() {
    tracing::info!("Initializing OTSS PM module v{}", VERSION);
}

/// Get PM module status
pub fn status() -> String {
    format!("OTSS PM v{}", VERSION)
}
