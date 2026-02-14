//! OTSS DevOps - Health checks and monitoring
//!
//! Provides health check utilities for the CI/CD infrastructure.

use serde::{Deserialize, Serialize};
use std::fmt;

/// System health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Degraded performance
    Degraded,
    /// Service unavailable
    Unhealthy,
    /// Unknown status
    Unknown,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

impl HealthStatus {
    /// Check if status is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Check if status needs attention
    pub fn needs_attention(&self) -> bool {
        matches!(self, HealthStatus::Degraded | HealthStatus::Unhealthy)
    }
}

/// Individual health check result
#[derive(Debug, Clone)]
pub struct HealthCheck {
    /// Name of the component
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Additional details
    pub details: String,
    /// Response time in milliseconds
    pub response_time_ms: u64,
}

/// System-wide health information
#[derive(Debug, Clone)]
pub struct SystemHealth {
    /// Overall status
    pub overall: HealthStatus,
    /// Individual component health
    pub components: Vec<HealthCheck>,
}

impl SystemHealth {
    /// Create new system health
    pub fn new() -> Self {
        Self {
            overall: HealthStatus::Unknown,
            components: Vec::new(),
        }
    }

    /// Add a component health check
    pub fn add_check(&mut self, check: HealthCheck) {
        self.components.push(check);
        self.update_overall();
    }

    /// Update overall status based on components
    fn update_overall(&mut self) {
        if self.components.iter().any(|c| c.status == HealthStatus::Unhealthy) {
            self.overall = HealthStatus::Unhealthy;
        } else if self.components.iter().any(|c| c.status == HealthStatus::Degraded) {
            self.overall = HealthStatus::Degraded;
        } else if self.components.iter().any(|c| c.status == HealthStatus::Unknown) {
            self.overall = HealthStatus::Unknown;
        } else {
            self.overall = HealthStatus::Healthy;
        }
    }
}

impl Default for SystemHealth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
    }

    #[test]
    fn test_health_status_checks() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Degraded.is_healthy());
        assert!(HealthStatus::Degraded.needs_attention());
    }
}
