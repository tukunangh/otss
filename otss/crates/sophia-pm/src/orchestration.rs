//! OTSS PM - Orchestration
//!
//! Provides system-wide orchestration for OTSS services.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// System configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// System name
    pub name: String,
    /// Environment
    pub environment: String,
    /// Service configurations
    pub services: HashMap<String, ServiceConfig>,
}

impl Default for SystemConfig {
    fn default() -> Self {
        let mut services = HashMap::new();
        services.insert(
            "trading-engine".to_string(),
            ServiceConfig {
                enabled: true,
                priority: 1,
            },
        );
        services.insert(
            "market-data".to_string(),
            ServiceConfig {
                enabled: true,
                priority: 2,
            },
        );
        services.insert(
            "broker-integration".to_string(),
            ServiceConfig {
                enabled: true,
                priority: 2,
            },
        );

        Self {
            name: "OTSS".to_string(),
            environment: "development".to_string(),
            services,
        }
    }
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Whether service is enabled
    pub enabled: bool,
    /// Startup priority (lower = start first)
    pub priority: u32,
}

/// System state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemState {
    /// Initializing
    Initializing,
    /// Running
    Running,
    /// Degraded
    Degraded,
    /// Stopping
    Stopping,
    /// Stopped
    Stopped,
}

/// System orchestrator
#[derive(Debug, Clone)]
pub struct Orchestrator {
    config: SystemConfig,
    state: SystemState,
}

impl Orchestrator {
    /// Create a new orchestrator
    pub fn new(config: SystemConfig) -> Self {
        Self {
            config,
            state: SystemState::Initializing,
        }
    }

    /// Get current state
    pub fn state(&self) -> SystemState {
        self.state
    }

    /// Start the system
    pub async fn start(&mut self) -> anyhow::Result<()> {
        tracing::info!("Starting OTSS system...");
        self.state = SystemState::Running;
        tracing::info!("OTSS system started");
        Ok(())
    }

    /// Stop the system
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        tracing::info!("Stopping OTSS system...");
        self.state = SystemState::Stopping;
        // TODO: Stop services in reverse priority order
        self.state = SystemState::Stopped;
        tracing::info!("OTSS system stopped");
        Ok(())
    }

    /// Get config reference
    pub fn config(&self) -> &SystemConfig {
        &self.config
    }

    /// List enabled services
    pub fn enabled_services(&self) -> Vec<(&String, &ServiceConfig)> {
        self.config
            .services
            .iter()
            .filter(|(_, config)| config.enabled)
            .collect()
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new(SystemConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_config_default() {
        let config = SystemConfig::default();
        assert_eq!(config.name, "OTSS");
        assert!(!config.services.is_empty());
    }

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = Orchestrator::default();
        assert_eq!(orchestrator.state(), SystemState::Initializing);
    }

    #[test]
    fn test_active_services() {
        let orchestrator = Orchestrator::default();
        let services = orchestrator.enabled_services();
        assert!(!services.is_empty());
    }
}
