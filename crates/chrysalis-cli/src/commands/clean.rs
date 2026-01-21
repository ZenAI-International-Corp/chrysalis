//! Clean command - remove build artifacts.

use anyhow::{Context, Result};
use chrysalis_flutter::FlutterExecutor;
use chrysalis_config::FlutterConfig;
use console::style;
use std::path::PathBuf;

pub async fn execute(project_dir: Option<PathBuf>) -> Result<()> {
    println!();
    println!("{}", style("Cleaning build artifacts...").cyan());
    println!();

    let project_dir = project_dir
        .or_else(|| std::env::current_dir().ok())
        .context("Failed to determine project directory")?;

    // Create Flutter executor
    let config = FlutterConfig::default();
    let executor = FlutterExecutor::new(&project_dir, config)?;

    // Run flutter clean
    executor.clean()?;

    println!("{}", style("✓ Clean completed successfully!").green());
    println!();

    Ok(())
}
