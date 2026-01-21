//! Flutter integration for Chrysalis.
//!
//! This crate provides functionality to:
//! - Execute Flutter commands
//! - Run `flutter pub get`
//! - Run `flutter build web`
//! - Validate Flutter SDK installation

mod error;
mod executor;
mod validator;

pub use error::{FlutterError, Result};
pub use executor::FlutterExecutor;
pub use validator::FlutterValidator;
