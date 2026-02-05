//! Environment variable loader with Vite-like priority system.

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Environment variable loader that follows Vite's priority system.
///
/// # Priority (highest to lowest):
/// 1. Existing system environment variables
/// 2. `.env.[mode].local`
/// 3. `.env.[mode]`
/// 4. `.env.local` (not loaded when mode is "test")
/// 5. `.env`
#[derive(Debug)]
pub struct EnvLoader {
    /// Project directory
    project_dir: PathBuf,
    /// Build mode (e.g., "development", "production", "staging")
    mode: Option<String>,
    /// Loaded environment variables
    env_vars: HashMap<String, String>,
}

impl EnvLoader {
    /// Create a new environment loader.
    pub fn new<P: AsRef<Path>>(project_dir: P, mode: Option<String>) -> Self {
        Self {
            project_dir: project_dir.as_ref().to_path_buf(),
            mode,
            env_vars: HashMap::new(),
        }
    }

    /// Load environment variables following Vite's priority system.
    ///
    /// Variables are loaded in order from lowest to highest priority.
    /// Higher priority values will overwrite lower priority ones.
    pub fn load(&mut self) -> Result<(), std::io::Error> {
        // Load in priority order (lowest to highest)
        // Lower priority files are loaded first so higher priority can override

        // 1. .env (lowest priority)
        self.load_env_file(".env")?;

        // 2. .env.local (skip if mode is "test")
        if self.mode.as_deref() != Some("test") {
            self.load_env_file(".env.local")?;
        }

        // 3. .env.[mode]
        if let Some(ref mode) = self.mode {
            self.load_env_file(&format!(".env.{}", mode))?;
        }

        // 4. .env.[mode].local (highest priority from files)
        if let Some(ref mode) = self.mode {
            self.load_env_file(&format!(".env.{}.local", mode))?;
        }

        // 5. System environment variables have highest priority
        // Merge with loaded env vars, preferring system env vars
        for (key, value) in env::vars() {
            self.env_vars.insert(key, value);
        }

        Ok(())
    }

    /// Load a single .env file if it exists.
    fn load_env_file(&mut self, filename: &str) -> Result<(), std::io::Error> {
        let path = self.project_dir.join(filename);

        if !path.exists() {
            debug!("Env file not found: {}", path.display());
            return Ok(());
        }

        info!("Loading env file: {}", path.display());

        match dotenvy::from_path_iter(&path) {
            Ok(iter) => {
                for item in iter {
                    match item {
                        Ok((key, value)) => {
                            // Only set if not already set (preserving priority)
                            self.env_vars.entry(key.clone()).or_insert_with(|| {
                                debug!("Loaded env var from {}: {}={}", filename, key, value);
                                value
                            });
                        }
                        Err(e) => {
                            debug!("Failed to parse line in {}: {}", filename, e);
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Failed to load env file {}: {}", filename, e);
            }
        }

        Ok(())
    }

    /// Get all loaded environment variables.
    pub fn env_vars(&self) -> &HashMap<String, String> {
        &self.env_vars
    }

    /// Get environment variables that match the given prefix.
    pub fn get_prefixed(&self, prefix: &str) -> HashMap<String, String> {
        self.env_vars
            .iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }

    /// Get environment variables that match the prefix or are in the whitelist.
    pub fn get_filtered(&self, prefix: &str, whitelist: &[String]) -> HashMap<String, String> {
        self.env_vars
            .iter()
            .filter(|(key, _)| key.starts_with(prefix) || whitelist.contains(key))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }

    /// Get the mode.
    pub fn mode(&self) -> Option<&str> {
        self.mode.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_loader_creation() {
        let loader = EnvLoader::new("/tmp", Some("production".to_string()));
        assert_eq!(loader.mode(), Some("production"));
    }

    #[test]
    fn test_get_prefixed() {
        let mut loader = EnvLoader::new("/tmp", None);
        loader
            .env_vars
            .insert("PUBLIC_API_KEY".to_string(), "123".to_string());
        loader.env_vars.insert(
            "PUBLIC_BASE_URL".to_string(),
            "http://example.com".to_string(),
        );
        loader
            .env_vars
            .insert("OTHER_VAR".to_string(), "value".to_string());

        let prefixed = loader.get_prefixed("PUBLIC_");
        assert_eq!(prefixed.len(), 2);
        assert_eq!(prefixed.get("PUBLIC_API_KEY"), Some(&"123".to_string()));
        assert_eq!(
            prefixed.get("PUBLIC_BASE_URL"),
            Some(&"http://example.com".to_string())
        );
        assert_eq!(prefixed.get("OTHER_VAR"), None);
    }

    #[test]
    fn test_get_filtered() {
        let mut loader = EnvLoader::new("/tmp", None);
        loader
            .env_vars
            .insert("PUBLIC_API_KEY".to_string(), "123".to_string());
        loader
            .env_vars
            .insert("OTHER_VAR".to_string(), "value".to_string());
        loader
            .env_vars
            .insert("WHITELISTED".to_string(), "allowed".to_string());

        let whitelist = vec!["WHITELISTED".to_string()];
        let filtered = loader.get_filtered("PUBLIC_", &whitelist);

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered.get("PUBLIC_API_KEY"), Some(&"123".to_string()));
        assert_eq!(filtered.get("WHITELISTED"), Some(&"allowed".to_string()));
        assert_eq!(filtered.get("OTHER_VAR"), None);
    }
}
