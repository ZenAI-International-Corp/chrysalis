//! Core build system for Chrysalis.
//!
//! This crate provides:
//! - Build context and state management
//! - File scanning and filtering
//! - File naming conventions
//! - Hash calculation utilities

mod context;
mod error;
mod file_info;
mod file_naming;
mod scanner;
mod stats;
mod utils;

pub use context::BuildContext;
pub use error::{BuildError, Result};
pub use file_info::FileInfo;
pub use file_naming::FileNaming;
pub use scanner::Scanner;
pub use stats::BuildStats;
pub use utils::{
    calculate_hash, format_bytes, is_flutter_framework_file, read_file_content, write_file_content,
};
