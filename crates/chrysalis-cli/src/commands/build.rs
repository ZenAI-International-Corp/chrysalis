//! Build command implementation.

use anyhow::{Context, Result};
use chrysalis_config::Config;
use chrysalis_core::BuildContext;
use chrysalis_flutter::FlutterExecutor;
use chrysalis_plugins::{ChunkPlugin, HashPlugin, InjectPlugin, MinifyPlugin, Plugin};
use console::style;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{error, info};

pub async fn execute(
    config_path: PathBuf,
    project_dir: Option<PathBuf>,
    skip_pub_get: bool,
    skip_minify: bool,
    skip_hash: bool,
    skip_chunk: bool,
    clean: bool,
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
        style("║   Modern Build System for Flutter Web            ║").cyan()
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
    let mut config = if config_path.exists() {
        info!("Loading config from: {}", config_path.display());
        Config::from_file(&config_path)?
    } else {
        info!("Using default configuration");
        Config::default()
    };

    // Apply CLI overrides
    if skip_pub_get {
        config.flutter.run_pub_get = false;
    }
    if skip_minify {
        config.plugins.minify.enabled = false;
    }
    if skip_hash {
        config.plugins.hash.enabled = false;
    }
    if skip_chunk {
        config.plugins.chunk.enabled = false;
    }
    if clean {
        config.build.clean_before_build = true;
    }

    // Validate configuration
    config.validate()?;

    // Clean if requested
    if config.build.clean_before_build {
        info!("Cleaning build directory...");
        let build_dir = project_dir.join(&config.build.build_dir);
        if build_dir.exists() {
            std::fs::remove_dir_all(&build_dir).context("Failed to clean build directory")?;
        }
    }

    // Phase 1: Flutter build
    println!("{}", style("Phase 1: Flutter Build").yellow().bold());
    println!("{}", style("─".repeat(50)).dim());

    let flutter_executor = FlutterExecutor::new(&project_dir, config.flutter.clone())?;

    // Run pub get
    if config.flutter.run_pub_get {
        flutter_executor.pub_get()?;
    }

    // Run flutter build web
    flutter_executor.build_web()?;

    println!();

    // Phase 2: Post-processing
    println!("{}", style("Phase 2: Post-Processing").yellow().bold());
    println!("{}", style("─".repeat(50)).dim());

    let build_dir = flutter_executor.build_output_dir();
    let mut ctx = BuildContext::new(&build_dir, config.build.clone())?;
    ctx.scan()?;

    // Build plugin pipeline
    let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();

    // Determine if inject plugin will run (only if both chunk and inject are enabled)
    let will_inject = config.plugins.chunk.enabled && config.plugins.inject.enabled;

    // Phase 1: Minify
    if config.plugins.minify.enabled {
        // Skip index.html during minification if inject plugin will handle it
        plugins.push(Box::new(MinifyPlugin::new(
            config.plugins.minify.clone(),
            will_inject,
        )));
    }

    // Phase 2: Chunk (BEFORE hashing, so Flutter can reference main.dart.js)
    if config.plugins.chunk.enabled {
        let chunk_plugin = ChunkPlugin::new(
            config.plugins.chunk.clone(),
            config.build.chunk_size_bytes(),
            config.build.min_chunk_size_bytes(),
        )?;
        plugins.push(Box::new(chunk_plugin));
    }

    // Phase 3: Hash (AFTER chunking, so stub and chunks get hashed together)
    if config.plugins.hash.enabled {
        plugins.push(Box::new(HashPlugin::new(config.plugins.hash.clone())?));
    }

    // Phase 4: Inject (updates references to hashed files)
    if config.plugins.chunk.enabled && config.plugins.inject.enabled {
        plugins.push(Box::new(InjectPlugin::new(config.plugins.inject.clone())));
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

    let elapsed = start.elapsed();
    println!("  Build time:       {:.2}s", elapsed.as_secs_f64());

    println!();
    println!(
        "{}",
        style("✓ Build completed successfully!").green().bold()
    );
    println!("  Output: {}", build_dir.display());
    println!();

    Ok(())
}
