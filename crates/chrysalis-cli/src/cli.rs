//! CLI argument parsing.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Chrysalis - Modern build system for Flutter Web
#[derive(Parser, Debug)]
#[command(name = "chrysalis")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "chrysalis.toml")]
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
    /// Build the Flutter web project
    Build {
        /// Skip Flutter pub get
        #[arg(long)]
        skip_pub_get: bool,

        /// Skip minification
        #[arg(long)]
        skip_minify: bool,

        /// Skip hashing
        #[arg(long)]
        skip_hash: bool,

        /// Skip chunking
        #[arg(long)]
        skip_chunk: bool,

        /// Clean before build
        #[arg(long)]
        clean: bool,
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
            skip_pub_get: false,
            skip_minify: false,
            skip_hash: false,
            skip_chunk: false,
            clean: false,
        }
    }
}
