//! Command handlers.

mod build;
mod init;
mod clean;

use crate::cli::{Args, Command};
use anyhow::Result;

/// Execute command based on CLI arguments.
pub async fn execute(args: Args) -> Result<()> {
    match args.command.unwrap_or_default() {
        Command::Build {
            skip_pub_get,
            skip_minify,
            skip_hash,
            skip_chunk,
            clean,
        } => {
            build::execute(
                args.config,
                args.project_dir,
                skip_pub_get,
                skip_minify,
                skip_hash,
                skip_chunk,
                clean,
            )
            .await
        }
        Command::Init { force } => init::execute(args.config, force).await,
        Command::Clean => clean::execute(args.project_dir).await,
        Command::Version => {
            println!("chrysalis {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
