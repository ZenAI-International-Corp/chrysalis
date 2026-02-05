//! CLI argument parsing.

use chrysalis_config::Platform;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Chrysalis - Modern build system for Flutter
#[derive(Parser, Debug)]
#[command(name = "chrysalis")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "chrysalis.yaml")]
    pub config: PathBuf,

    /// Project directory (defaults to current directory)
    #[arg(short, long)]
    pub project_dir: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Enable debug output
    #[arg(short, long)]
    pub debug: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Build the Flutter project
    Build {
        /// Target platform(s): web, windows, macos, linux, android, ios
        #[arg(short, long, value_delimiter = ',', default_value = "web")]
        platform: Vec<Platform>,

        /// Build all enabled platforms
        #[arg(long)]
        all: bool,

        /// Clean before build
        #[arg(long)]
        clean: bool,

        /// Build mode (e.g., development, production, staging)
        #[arg(short, long)]
        mode: Option<String>,
    },

    /// Generate default configuration file
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },

    /// Clean build artifacts
    Clean,

    /// Show version information
    Version,
}

impl Default for Command {
    fn default() -> Self {
        Self::Build {
            platform: vec![Platform::Web],
            all: false,
            clean: false,
            mode: None,
        }
    }
}
