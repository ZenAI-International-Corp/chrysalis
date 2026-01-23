//! Flutter command executor.

use crate::{FlutterError, FlutterValidator, Result};
use chrysalis_config::FlutterConfig;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::{debug, info, warn};

/// Flutter command executor.
#[derive(Debug)]
pub struct FlutterExecutor {
    validator: FlutterValidator,
    project_dir: PathBuf,
    config: FlutterConfig,
}

impl FlutterExecutor {
    /// Create a new executor.
    pub fn new<P: AsRef<Path>>(project_dir: P, config: FlutterConfig) -> Result<Self> {
        let project_dir = project_dir.as_ref().to_path_buf();
        let validator = FlutterValidator::new(config.flutter_path.clone())?;

        // Validate Flutter SDK
        validator.validate()?;

        // Validate project
        validator.validate_project(&project_dir)?;

        info!(
            "Flutter SDK found at: {}",
            validator.flutter_path().display()
        );

        if let Ok(version) = validator.version() {
            info!("Flutter version: {}", version);
        }

        Ok(Self {
            validator,
            project_dir,
            config,
        })
    }

    /// Run `flutter pub get`.
    pub fn pub_get(&self) -> Result<()> {
        if !self.config.run_pub_get {
            debug!("Skipping 'flutter pub get' (disabled in config)");
            return Ok(());
        }

        info!("Running flutter pub get...");

        let output = Command::new(self.validator.flutter_path())
            .current_dir(&self.project_dir)
            .args(["pub", "get"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            return Err(FlutterError::CommandFailed {
                command: "flutter pub get".to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        info!("✓ Dependencies installed");
        Ok(())
    }

    /// Run `flutter build web`.
    pub fn build_web(&self) -> Result<()> {
        info!("Running flutter build web...");

        let args = self.config.build_args();
        debug!("Flutter build args: {:?}", args);

        let mut cmd = Command::new(self.validator.flutter_path());
        cmd.current_dir(&self.project_dir)
            .args(&args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd.status()?;

        if !status.success() {
            return Err(FlutterError::CommandFailed {
                command: format!("flutter {}", args.join(" ")),
                exit_code: status.code().unwrap_or(-1),
                stderr: "See output above".to_string(),
            });
        }

        // Verify build output exists
        let build_output = self.project_dir.join(&self.config.target_dir);
        if !build_output.exists() {
            warn!(
                "Build output directory not found: {}",
                build_output.display()
            );
            return Err(FlutterError::BuildOutputNotFound(build_output));
        }

        info!("✓ Flutter build completed: {}", build_output.display());
        Ok(())
    }

    /// Run `flutter clean`.
    pub fn clean(&self) -> Result<()> {
        info!("Running flutter clean...");

        let output = Command::new(self.validator.flutter_path())
            .current_dir(&self.project_dir)
            .arg("clean")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            return Err(FlutterError::CommandFailed {
                command: "flutter clean".to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        info!("✓ Project cleaned");
        Ok(())
    }

    /// Get the build output directory.
    pub fn build_output_dir(&self) -> PathBuf {
        self.project_dir.join(&self.config.target_dir)
    }

    /// Get the project directory.
    pub fn project_dir(&self) -> &Path {
        &self.project_dir
    }

    /// Get the Flutter configuration.
    pub fn config(&self) -> &FlutterConfig {
        &self.config
    }
}
