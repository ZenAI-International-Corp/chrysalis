//! Project-level configuration.

use serde::{Deserialize, Serialize};

/// Project-level configuration shared across all platforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    /// Project name
    pub name: Option<String>,

    /// Project version
    pub version: Option<String>,

    /// Project description
    pub description: Option<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: None,
            version: None,
            description: None,
        }
    }
}

impl ProjectConfig {
    /// Validate project configuration.
    pub fn validate(&self) -> crate::Result<()> {
        // No strict validation needed for optional fields
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_project_config() {
        let config = ProjectConfig::default();
        assert!(config.name.is_none());
        assert!(config.version.is_none());
    }

    #[test]
    fn test_validation() {
        let config = ProjectConfig::default();
        assert!(config.validate().is_ok());
    }
}
