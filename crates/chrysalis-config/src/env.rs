//! Environment variable configuration.

use crate::{ConfigError, Result};
use serde::{Deserialize, Serialize};

/// Environment variable configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EnvConfig {
    /// Prefix for environment variables to be included (e.g., "PUBLIC_").
    /// Variables starting with this prefix will be passed to --dart-define.
    pub prefix: String,

    /// Whitelist of environment variable names that should be included
    /// even if they don't match the prefix.
    pub whitelist: Vec<String>,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            // Default prefix based on project name
            prefix: "PUBLIC_".to_string(),
            whitelist: Vec::new(),
        }
    }
}

impl EnvConfig {
    /// Validate environment configuration.
    pub fn validate(&self) -> Result<()> {
        // Ensure prefix is not empty
        if self.prefix.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "env.prefix".to_string(),
                reason: "prefix cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// Get the prefix.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Get the whitelist.
    pub fn whitelist(&self) -> &[String] {
        &self.whitelist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_env_config() {
        let config = EnvConfig::default();
        assert_eq!(config.prefix, "PUBLIC_");
        assert!(config.whitelist.is_empty());
    }

    #[test]
    fn test_validate_empty_prefix() {
        let config = EnvConfig {
            prefix: String::new(),
            whitelist: Vec::new(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_valid_config() {
        let config = EnvConfig {
            prefix: "PUBLIC_".to_string(),
            whitelist: vec!["API_KEY".to_string()],
        };
        assert!(config.validate().is_ok());
    }
}
