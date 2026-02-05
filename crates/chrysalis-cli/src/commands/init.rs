//! Init command - generate default configuration.

use anyhow::{Result, bail};
use chrysalis_config::Config;
use console::style;
use std::path::PathBuf;
use tracing::info;

pub async fn execute(config_path: PathBuf, force: bool) -> Result<()> {
    println!();
    println!(
        "{}",
        style("Initializing Chrysalis configuration...").cyan()
    );
    println!();

    // Check if config already exists
    if config_path.exists() && !force {
        bail!(
            "Configuration file already exists: {}\nUse --force to overwrite",
            config_path.display()
        );
    }

    // Generate default config
    let config = Config::default();

    // Save to file
    config.save(&config_path)?;

    info!("Configuration written to: {}", config_path.display());

    println!("{}", style("✓ Configuration file created!").green());
    println!();
    println!(
        "Edit {} to customize your build:",
        style(config_path.display()).yellow()
    );
    println!();
    println!("  build:              - Build system settings");
    println!("  env:                - Environment variables");
    println!("  platforms:          - Platform configurations");
    println!("    platforms.web:           - Web platform");
    println!("      platforms.web.flutter: - Flutter settings");
    println!("      platforms.web.plugins: - Plugin configurations");
    println!();
    println!("To build your project:");
    println!("  chrysalis build                    # Build web (default)");
    println!("  chrysalis build --platform windows # Build for Windows");
    println!("  chrysalis build --all              # Build all enabled platforms");
    println!();

    Ok(())
}
