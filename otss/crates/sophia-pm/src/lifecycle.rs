//! OTSS PM - Service lifecycle
//!
//! Manages service lifecycle states.

/// Service state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    /// Not started
    NotStarted,
    /// Starting
    Starting,
    /// Running
    Running,
    /// Stopping
    Stopping,
    /// Stopped
    Stopped,
    /// Error
    Error,
}

/// Service trait
pub trait Service: Send + Sync {
    /// Service name
    fn name(&self) -> &str;
    /// Start the service
    fn start(&mut self) -> anyhow::Result<()>;
    /// Stop the service
    fn stop(&mut self) -> anyhow::Result<()>;
    /// Get current state
    fn state(&self) -> ServiceState;
}

/// Service manager
#[derive(Debug, Clone, Default)]
pub struct ServiceManager {
    /// Services
    services: Vec<Box<dyn Service>>,
}

impl ServiceManager {
    /// Create a new service manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a service
    pub fn add_service(&mut self, service: Box<dyn Service>) {
        self.services.push(service);
    }

    /// Get service count
    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    /// Start all services
    pub fn start_all(&mut self) {
        tracing::info!("Starting {} services", self.services.len());
        for service in &mut self.services {
            if let Err(e) = service.start() {
                tracing::error!("Failed to start {}: {}", service.name(), e);
            }
        }
    }

    /// Stop all services in reverse order
    pub fn stop_all(&mut self) {
        tracing::info!("Stopping {} services", self.services.len());
        for service in self.services.iter_mut().rev() {
            if let Err(e) = service.stop() {
                tracing::error!("Failed to stop {}: {}", service.name(), e);
            }
        }
    }

    /// Get service by name
    pub fn get_service(&self, name: &str) -> Option<&Box<dyn Service>> {
        self.services.iter().find(|s| s.name() == name)
    }
}

/// Service handle
#[derive(Debug, Clone)]
pub struct ServiceHandle {
    /// Service name
    pub name: String,
    /// Service state
    pub state: ServiceState,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestService;

    impl Service for TestService {
        fn name(&self) -> &str {
            "test-service"
        }

        fn start(&mut self) -> anyhow::Result<()> {
            Ok(())
        }

        fn stop(&mut self) -> anyhow::Result<()> {
            Ok(())
        }

        fn state(&self) -> ServiceState {
            ServiceState::Running
        }
    }

    #[test]
    fn test_service_manager() {
        let mut manager = ServiceManager::new();
        assert_eq!(manager.service_count(), 0);
        manager.add_service(Box::new(TestService));
        assert_eq!(manager.service_count(), 1);
    }

    #[test]
    fn test_service_state() {
        assert_ne!(ServiceState::Starting, ServiceState::Running);
    }

    #[test]
    fn test_service_handle() {
        let handle = ServiceHandle {
            name: "test".to_string(),
            state: ServiceState::Running,
        };
        assert_eq!(handle.state, ServiceState::Running);
    }
}
