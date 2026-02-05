//! Build command implementation.

use anyhow::{Context, Result};
use chrysalis_config::{Config, Platform};
use chrysalis_core::BuildContext;
use chrysalis_flutter::FlutterExecutor;
use chrysalis_plugins::{ChunkPlugin, HashPlugin, InjectPlugin, MinifyPlugin, Plugin};
use console::style;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{error, info, warn};

pub async fn execute(
    config_path: PathBuf,
    project_dir: Option<PathBuf>,
    platforms: Vec<Platform>,
    build_all: bool,
    clean: bool,
    mode: Option<String>,
) -> Result<()> {
    let start = Instant::now();

    println!();
    println!(
        "{}",
        style("╔═══════════════════════════════════════════════════╗").cyan()
    );
    println!(
        "{}",
        style("║           CHRYSALIS 🦋                            ║").cyan()
    );
    println!(
        "{}",
        style("║   Modern Build System for Flutter                 ║").cyan()
    );
    println!(
        "{}",
        style("╚═══════════════════════════════════════════════════╝").cyan()
    );
    println!();

    // Determine project directory
    let project_dir = project_dir
        .or_else(|| std::env::current_dir().ok())
        .context("Failed to determine project directory")?;

    info!("Project directory: {}", project_dir.display());

    // Load configuration
    let config = if config_path.exists() {
        info!("Loading config from: {}", config_path.display());
        Config::from_file(&config_path)?
    } else {
        info!("Using default configuration");
        Config::default()
    };

    // Validate configuration
    config.validate()?;

    // Determine which platforms to build
    let platforms_to_build = if build_all {
        // Build all enabled platforms from config
        let enabled = config.platforms.enabled_platforms();
        if enabled.is_empty() {
            return Err(anyhow::anyhow!("No platforms enabled in configuration"));
        }
        info!("Building all enabled platforms: {}", enabled.join(", "));
        // Convert enabled platforms to Platform enum
        enabled
            .iter()
            .filter_map(|p| p.parse::<Platform>().ok())
            .collect::<Vec<_>>()
    } else {
        // Use platforms from CLI argument
        platforms
    };

    if platforms_to_build.is_empty() {
        return Err(anyhow::anyhow!("No platforms specified for build"));
    }

    // Build each platform
    for (idx, platform) in platforms_to_build.iter().enumerate() {
        if idx > 0 {
            println!();
            println!("{}", style("─".repeat(50)).dim());
            println!();
        }

        info!("Building platform: {}", platform);

        // Currently only web platform is fully supported
        match platform {
            Platform::Web => {
                build_web_platform(&config, &project_dir, clean, mode.clone()).await?;
            }
            _ => {
                warn!(
                    "Platform {} is not yet fully supported, skipping post-processing",
                    platform
                );
                // For other platforms, just run flutter build
                build_other_platform(&config, &project_dir, *platform, mode.clone()).await?;
            }
        }
    }

    let elapsed = start.elapsed();
    println!();
    println!(
        "{}",
        style("✓ All builds completed successfully!").green().bold()
    );
    println!("  Total build time: {:.2}s", elapsed.as_secs_f64());
    println!();

    Ok(())
}

/// Build web platform with full post-processing pipeline.
async fn build_web_platform(
    config: &Config,
    project_dir: &PathBuf,
    clean: bool,
    mode: Option<String>,
) -> Result<()> {
    let web_config = &config.platforms.web;

    if !web_config.enabled {
        warn!("Web platform is disabled in configuration, skipping");
        return Ok(());
    }

    // Clean output directory if requested
    if clean || config.build.clean_before_build {
        if let Some(output_dir) = &web_config.output_dir {
            let output_path = project_dir.join(output_dir);
            if output_path.exists() {
                info!("Cleaning output directory...");
                std::fs::remove_dir_all(&output_path)
                    .context("Failed to clean output directory")?;
            }
        }
    }

    // Phase 1: Flutter build
    println!("{}", style("Phase 1: Flutter Build").yellow().bold());
    println!("{}", style("─".repeat(50)).dim());

    let flutter_executor = FlutterExecutor::new(
        project_dir,
        Platform::Web,
        web_config.flutter.clone(),
        config.env.clone(),
        mode,
    )?;

    // Run pub get
    if web_config.flutter.run_pub_get {
        flutter_executor.pub_get()?;
    }

    // Run flutter build
    flutter_executor.build()?;

    println!();

    // Phase 2: Copy build artifacts to output directory
    let flutter_build_dir = project_dir.join(web_config.flutter_build_dir());
    let processing_dir = if let Some(output_dir) = &web_config.output_dir {
        let output_path = project_dir.join(output_dir);

        println!("{}", style("Phase 2: Copy Build Artifacts").yellow().bold());
        println!("{}", style("─".repeat(50)).dim());

        info!(
            "Copying {} -> {}",
            flutter_build_dir.display(),
            output_path.display()
        );

        // Copy Flutter build output to output directory
        chrysalis_core::copy_dir_all(&flutter_build_dir, &output_path)
            .context("Failed to copy build artifacts")?;

        info!("✓ Build artifacts copied to {}", output_path.display());
        println!();

        output_path
    } else {
        // Process in-place
        info!(
            "Processing files in-place at {}",
            flutter_build_dir.display()
        );
        flutter_build_dir
    };

    // Phase 3: Post-processing
    println!("{}", style("Phase 3: Post-Processing").yellow().bold());
    println!("{}", style("─".repeat(50)).dim());

    let mut ctx = BuildContext::new(&processing_dir, web_config.exclude_patterns.clone())?;
    ctx.scan()?;

    // Build plugin pipeline
    let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();

    // Determine if inject plugin will run (only if both chunk and inject are enabled)
    let will_inject = web_config.plugins.chunk.enabled && web_config.plugins.inject.enabled;

    // Phase 1: Minify
    if web_config.plugins.minify.enabled {
        // Skip index.html during minification if inject plugin will handle it
        plugins.push(Box::new(MinifyPlugin::new(
            web_config.plugins.minify.clone(),
            will_inject,
        )));
    }

    // Phase 2: Chunk (BEFORE hashing, so Flutter can reference main.dart.js)
    if web_config.plugins.chunk.enabled {
        plugins.push(Box::new(ChunkPlugin::new(
            web_config.plugins.chunk.clone(),
        )?));
    }

    // Phase 3: Hash (AFTER chunking, so stub and chunks get hashed together)
    if web_config.plugins.hash.enabled {
        plugins.push(Box::new(HashPlugin::new(web_config.plugins.hash.clone())?));
    }

    // Phase 4: Inject (updates references to hashed files)
    if web_config.plugins.chunk.enabled && web_config.plugins.inject.enabled {
        plugins.push(Box::new(InjectPlugin::new(
            web_config.plugins.inject.clone(),
        )));
    }

    // Execute plugins
    for plugin in plugins {
        info!("Running plugin: {}", plugin.name());
        if let Err(e) = plugin.execute(&mut ctx).await {
            error!("Plugin '{}' failed: {}", plugin.name(), e);
            return Err(e.into());
        }
    }

    println!();

    // Print summary
    println!("{}", style("Build Summary").green().bold());
    println!("{}", style("═".repeat(50)).dim());

    let stats = ctx.stats();
    println!("  Platform:         web");
    println!("  Total files:      {}", stats.total_files);
    println!("  Minified files:   {}", stats.minified_files);
    println!("  Hashed files:     {}", stats.hashed_files);
    println!("  Chunked files:    {}", stats.chunked_files);
    println!("  Total chunks:     {}", stats.total_chunks);
    println!(
        "  Bytes saved:      {}",
        chrysalis_core::format_bytes(stats.bytes_saved)
    );

    if stats.original_size > 0 {
        println!("  Compression:      {:.1}%", stats.compression_ratio());
    }

    println!("  Output:           {}", processing_dir.display());
    println!();

    Ok(())
}

/// Build other platforms (no post-processing yet).
async fn build_other_platform(
    config: &Config,
    project_dir: &PathBuf,
    platform: Platform,
    mode: Option<String>,
) -> Result<()> {
    info!("Building platform: {}", platform);

    // For now, use default Flutter config for non-web platforms
    let flutter_config = chrysalis_config::FlutterConfig::default();

    let flutter_executor = FlutterExecutor::new(
        project_dir,
        platform,
        flutter_config,
        config.env.clone(),
        mode,
    )?;

    // Run pub get
    if flutter_executor.config().run_pub_get {
        flutter_executor.pub_get()?;
    }

    // Run flutter build
    flutter_executor.build()?;

    println!();
    println!("{}", style("Build Summary").green().bold());
    println!("{}", style("═".repeat(50)).dim());
    println!("  Platform:         {}", platform);
    println!(
        "  Output:           {}",
        flutter_executor.flutter_build_dir().display()
    );
    println!();

    Ok(())
}
