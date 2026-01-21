//! Chrysalis CLI - Modern build system for Flutter Web.

mod cli;
mod commands;
mod logger;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let args = cli::Args::parse();

    // Initialize logger
    logger::init(args.verbose, args.debug)?;

    // Execute command
    commands::execute(args).await
}
