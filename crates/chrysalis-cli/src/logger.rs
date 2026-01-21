//! Logging configuration.

use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize logger.
pub fn init(verbose: bool, debug: bool) -> Result<()> {
    let filter = if debug {
        EnvFilter::new("debug")
    } else if verbose {
        EnvFilter::new("info")
    } else {
        EnvFilter::new("warn")
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .compact(),
        )
        .init();

    Ok(())
}
