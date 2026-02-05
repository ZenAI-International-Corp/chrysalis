//! Flutter command executor.

use crate::{FlutterError, FlutterValidator, Result};
use chrysalis_config::{EnvConfig, EnvLoader, FlutterConfig, Platform};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::{debug, info, warn};

/// Flutter command executor.
#[derive(Debug)]
pub struct FlutterExecutor {
    validator: FlutterValidator,
    project_dir: PathBuf,
    platform: Platform,
    config: FlutterConfig,
    env_config: EnvConfig,
    mode: Option<String>,
}

impl FlutterExecutor {
    /// Create a new executor.
    pub fn new<P: AsRef<Path>>(
        project_dir: P,
        platform: Platform,
        config: FlutterConfig,
        env_config: EnvConfig,
        mode: Option<String>,
    ) -> Result<Self> {
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
            platform,
            config,
            env_config,
            mode,
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

    /// Run `flutter build` for the configured platform.
    pub fn build(&self) -> Result<()> {
        info!("Running flutter build {}...", self.platform);

        // Load environment variables
        let mut env_loader = EnvLoader::new(&self.project_dir, self.mode.clone());
        env_loader.load().map_err(|e| FlutterError::CommandFailed {
            command: "load environment variables".to_string(),
            exit_code: -1,
            stderr: format!("Failed to load environment variables: {}", e),
        })?;

        // Get filtered environment variables
        let env_vars =
            env_loader.get_filtered(self.env_config.prefix(), self.env_config.whitelist());

        debug!("Loaded {} environment variables", env_vars.len());
        for (key, value) in &env_vars {
            debug!("  {}={}", key, value);
        }

        // Build Flutter command arguments
        let mut args = vec!["build".to_string(), self.platform.as_str().to_string()];

        // Add platform-specific arguments
        match self.platform {
            Platform::Web => {
                args.extend(self.build_args_web());
            }
            _ => {
                // Future: add platform-specific arguments for other platforms
                if self.config.release {
                    args.push("--release".to_string());
                } else {
                    args.push("--profile".to_string());
                }
            }
        }

        // Add dart-define for each environment variable
        for (key, value) in &env_vars {
            args.push(format!("--dart-define={}={}", key, value));
        }

        // Add dart-define for MODE if mode is specified
        if let Some(ref mode) = self.mode {
            let mode_var = format!("{}_MODE", self.env_config.prefix().trim_end_matches('_'));
            args.push(format!("--dart-define={}={}", mode_var, mode));
            info!("Build mode: {}", mode);
        }

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

    /// Get web-specific build arguments.
    fn build_args_web(&self) -> Vec<String> {
        self.config.build_args()
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

    /// Get the platform.
    pub fn platform(&self) -> Platform {
        self.platform
    }
}
