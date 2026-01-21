//! Utility functions for build system.

use crate::{BuildError, Result};
use std::path::Path;

/// Calculate MD5 hash of content.
pub fn calculate_hash(content: &[u8], length: usize) -> String {
    let digest = md5::compute(content);
    let hash = format!("{:x}", digest);
    hash[..length.min(hash.len())].to_string()
}

/// Read file content from disk.
pub fn read_file_content<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let path = path.as_ref();
    std::fs::read(path).map_err(|source| BuildError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Write file content to disk.
pub fn write_file_content<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<()> {
    let path = path.as_ref();
    std::fs::write(path, content).map_err(|source| BuildError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Format bytes to human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let bytes_f = bytes as f64;
    let exp = (bytes_f.ln() / 1024_f64.ln()).floor() as usize;
    let exp = exp.min(UNITS.len() - 1);
    let value = bytes_f / 1024_f64.powi(exp as i32);
    
    format!("{:.2} {}", value, UNITS[exp])
}

/// Check if a file is a Flutter framework file that must not be modified.
/// 
/// These files are expected by Flutter with specific names and should not be:
/// - Renamed (hashed)
/// - Chunked
/// - Modified in structure
/// 
/// Examples: `flutter_service_worker.js`, `manifest.json`, `version.json`
pub fn is_flutter_framework_file(name: &str) -> bool {
    matches!(
        name,
        "flutter_service_worker.js" | "manifest.json" | "version.json"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let content = b"hello world";
        let hash = calculate_hash(content, 8);
        assert_eq!(hash.len(), 8);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_is_flutter_framework_file() {
        assert!(is_flutter_framework_file("flutter_service_worker.js"));
        assert!(is_flutter_framework_file("manifest.json"));
        assert!(is_flutter_framework_file("version.json"));
        
        assert!(!is_flutter_framework_file("main.dart.js"));
        assert!(!is_flutter_framework_file("index.html"));
        assert!(!is_flutter_framework_file("flutter_bootstrap.js"));
    }
}
