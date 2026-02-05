//! Clean command - remove build artifacts.

use anyhow::{Context, Result};
use chrysalis_config::{Config, EnvConfig, FlutterConfig, Platform};
use chrysalis_flutter::FlutterExecutor;
use console::style;
use std::path::PathBuf;

pub async fn execute(project_dir: Option<PathBuf>) -> Result<()> {
    println!();
    println!("{}", style("Cleaning build artifacts...").cyan());
    println!();

    let project_dir = project_dir
        .or_else(|| std::env::current_dir().ok())
        .context("Failed to determine project directory")?;

    // Load config to get build directories
    let config_path = project_dir.join("chrysalis.toml");
    let config = if config_path.exists() {
        Config::from_file(&config_path)?
    } else {
        Config::default()
    };

    // Clean Flutter artifacts using Flutter executor
    let flutter_config = FlutterConfig::default();
    let env_config = EnvConfig::default();
    let executor = FlutterExecutor::new(
        &project_dir,
        Platform::Web, // Platform doesn't matter for clean
        flutter_config,
        env_config,
        None,
    )?;

    // Run flutter clean
    executor.clean()?;

    // Also remove build directory
    let build_dir = project_dir.join(&config.platforms.web.build_dir);
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir).context("Failed to remove build directory")?;
        println!("  Removed: {}", build_dir.display());
    }

    println!("{}", style("✓ Clean completed successfully!").green());
    println!();

    Ok(())
}
