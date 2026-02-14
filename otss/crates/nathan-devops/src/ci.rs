//! OTSS DevOps - CI/CD configuration
//!
//! Defines CI/CD pipeline structures and configuration generation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CI/CD pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CICDConfig {
    /// Pipeline name
    pub name: String,
    /// Repository URL
    pub repository: String,
    /// Default branch
    pub default_branch: String,
    /// Pipeline stages
    pub stages: Vec<PipelineStage>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
}

impl Default for CICDConfig {
    fn default() -> Self {
        Self {
            name: "OTSS Pipeline".to_string(),
            repository: "https://github.com/otss/otss".to_string(),
            default_branch: "main".to_string(),
            stages: vec![
                PipelineStage {
                    name: "build".to_string(),
                    script: vec!["cargo build --workspace".to_string()],
                },
                PipelineStage {
                    name: "test".to_string(),
                    script: vec!["cargo test --workspace".to_string()],
                },
            ],
            env_vars: HashMap::new(),
        }
    }
}

/// A single pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    /// Stage name
    pub name: String,
    /// Scripts to execute
    pub script: Vec<String>,
}

/// Pipeline configuration manager
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    config: CICDConfig,
}

impl PipelineConfig {
    /// Create a new pipeline config
    pub fn new(config: CICDConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn config(&self) -> &CICDConfig {
        &self.config
    }

    /// Generate GitHub Actions workflow YAML
    pub fn generate_github_actions(&self) -> String {
        let mut yaml = String::from("name: OTSS CI\n\n");
        yaml.push_str("on:\n  push:\n    branches: [ main ]\n");
        yaml.push_str("  pull_request:\n    branches: [ main ]\n\n");
        yaml.push_str("jobs:\n  ci:\n    runs-on: ubuntu-latest\n");
        yaml.push_str("    steps:\n      - uses: actions/checkout@v3\n");
        
        for stage in &self.config.stages {
            yaml.push_str(&format!("      # {} stage\n", stage.name));
            for script in &stage.script {
                yaml.push_str(&format!("      - run: {}\n", script));
            }
        }
        
        yaml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cicd_config_default() {
        let config = CICDConfig::default();
        assert_eq!(config.name, "OTSS Pipeline");
        assert!(!config.stages.is_empty());
    }
}
