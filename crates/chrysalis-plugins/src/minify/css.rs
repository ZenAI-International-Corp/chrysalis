//! CSS minification using lightningcss.

use crate::{PluginError, Result};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use std::path::PathBuf;

/// Minify CSS content.
pub fn minify_css(content: &[u8]) -> Result<Vec<u8>> {
    let content_str =
        std::str::from_utf8(content).map_err(|e| PluginError::MinificationFailed {
            file: PathBuf::from("unknown.css"),
            reason: format!("UTF-8 error: {}", e),
        })?;

    let stylesheet = StyleSheet::parse(content_str, ParserOptions::default()).map_err(|e| {
        PluginError::MinificationFailed {
            file: PathBuf::from("unknown.css"),
            reason: format!("Parse error: {:?}", e),
        }
    })?;

    let result = stylesheet
        .to_css(PrinterOptions {
            minify: true,
            ..Default::default()
        })
        .map_err(|e| PluginError::MinificationFailed {
            file: PathBuf::from("unknown.css"),
            reason: format!("Minify error: {:?}", e),
        })?;

    Ok(result.code.into_bytes())
}
