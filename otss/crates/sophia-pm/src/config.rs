//! OTSS PM - Configuration management
//!
//! Provides configuration loading and management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration manager
#[derive(Debug, Clone)]
pub struct ConfigManager {
    configs: HashMap<String, ConfigValue>,
    config_path: Option<PathBuf>,
}

/// Configuration value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Number value
    Number(f64),
    /// Boolean value
    Bool(bool),
    /// Nested config
    Nested(HashMap<String, ConfigValue>),
    /// Array of values
    Array(Vec<ConfigValue>),
}

/// Config loader
#[derive(Debug, Clone, Default)]
pub struct ConfigLoader;

impl ConfigLoader {
    /// Create a new config loader
    pub fn new() -> Self {
        Self
    }

    /// Load configuration from a file
    pub fn from_file(&self, _path: &PathBuf) -> anyhow::Result<ConfigManager> {
        tracing::info!("Loading configuration...");
        // TODO: Implement file loading
        Ok(ConfigManager::default())
    }

    /// Load from JSON string
    pub fn from_json(&self, json: &str) -> anyhow::Result<ConfigManager> {
        let configs: HashMap<String, ConfigValue> = serde_json::from_str(json)?;
        Ok(ConfigManager {
            configs,
            config_path: None,
        })
    }
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a configuration value
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.configs.get(key)
    }

    /// Set a configuration value
    pub fn set(&mut self, key: String, value: ConfigValue) {
        self.configs.insert(key, value);
    }

    /// Get string value
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.configs.get(key) {
            Some(ConfigValue::String(s)) => Some(s),
            _ => None,
        }
    }

    /// Get boolean value
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.configs.get(key) {
            Some(ConfigValue::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    /// Get number value
    pub fn get_number(&self, key: &str) -> Option<f64> {
        match self.configs.get(key) {
            Some(ConfigValue::Number(n)) => Some(*n),
            _ => None,
        }
    }

    /// Check if a key exists
    pub fn has(&self, key: &str) -> bool {
        self.configs.contains_key(key)
    }

    /// Load from file
    pub fn load_file(&mut self, path: PathBuf) -> anyhow::Result<()> {
        tracing::info!("Loading config from: {:?}", path);
        self.config_path = Some(path);
        Ok(())
    }

    /// Save to file
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = &self.config_path {
            tracing::info!("Saving config to: {:?}", path);
        }
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self {
            configs: HashMap::new(),
            config_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_default() {
        let manager = ConfigManager::default();
        assert!(!manager.has("test"));
    }

    #[test]
    fn test_set_get() {
        let mut manager = ConfigManager::new();
        manager.set("key".to_string(), ConfigValue::String("value".to_string()));
        assert_eq!(manager.get_string("key"), Some("value"));
    }

    #[test]
    fn test_get_string() {
        let mut manager = ConfigManager::new();
        manager.set("name".to_string(), ConfigValue::String("test".to_string()));
        assert_eq!(manager.get_string("name"), Some("test"));
    }

    #[test]
    fn test_get_bool() {
        let mut manager = ConfigManager::new();
        manager.set("enabled".to_string(), ConfigValue::Bool(true));
        assert_eq!(manager.get_bool("enabled"), Some(true));
    }
}
