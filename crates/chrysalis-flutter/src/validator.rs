//! Flutter SDK validation.

use crate::{FlutterError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use which::which;

/// Flutter SDK validator.
#[derive(Debug)]
pub struct FlutterValidator {
    flutter_path: PathBuf,
}

impl FlutterValidator {
    /// Create a new validator.
    ///
    /// If `flutter_path` is None, searches for Flutter in PATH.
    pub fn new(flutter_path: Option<PathBuf>) -> Result<Self> {
        let flutter_path = if let Some(path) = flutter_path {
            path
        } else {
            which("flutter").map_err(|_| FlutterError::SdkNotFound)?
        };

        Ok(Self { flutter_path })
    }

    /// Get the Flutter executable path.
    pub fn flutter_path(&self) -> &Path {
        &self.flutter_path
    }

    /// Validate Flutter SDK installation.
    pub fn validate(&self) -> Result<()> {
        // Check if Flutter executable exists
        if !self.flutter_path.exists() {
            return Err(FlutterError::SdkNotFound);
        }

        // Run `flutter --version` to verify it works
        let output = Command::new(&self.flutter_path).arg("--version").output()?;

        if !output.status.success() {
            return Err(FlutterError::CommandFailed {
                command: "flutter --version".to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(())
    }

    /// Get Flutter version.
    pub fn version(&self) -> Result<String> {
        let output = Command::new(&self.flutter_path).arg("--version").output()?;

        if !output.status.success() {
            return Err(FlutterError::CommandFailed {
                command: "flutter --version".to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let version_output = String::from_utf8_lossy(&output.stdout);

        // Extract version from first line (e.g., "Flutter 3.16.0")
        let version = version_output
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .ok_or_else(|| FlutterError::InvalidVersion(version_output.to_string()))?;

        Ok(version.to_string())
    }

    /// Validate Flutter project directory.
    pub fn validate_project<P: AsRef<Path>>(&self, project_dir: P) -> Result<()> {
        let project_dir = project_dir.as_ref();

        // Check if directory exists
        if !project_dir.exists() {
            return Err(FlutterError::ProjectNotFound(project_dir.to_path_buf()));
        }

        // Check for pubspec.yaml
        let pubspec = project_dir.join("pubspec.yaml");
        if !pubspec.exists() {
            return Err(FlutterError::MissingPubspec(project_dir.to_path_buf()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        // This test will only pass if Flutter is installed
        if let Ok(validator) = FlutterValidator::new(None) {
            assert!(validator.flutter_path().exists());
        }
    }

    #[test]
    fn test_validation() {
        // This test will only pass if Flutter is installed
        if let Ok(validator) = FlutterValidator::new(None) {
            assert!(validator.validate().is_ok());
        }
    }
}
