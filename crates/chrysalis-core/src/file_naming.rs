//! Unified file naming conventions.

use std::path::Path;

/// File naming utilities.
pub struct FileNaming;

impl FileNaming {
    /// Add hash to filename: `filename.ext` -> `filename.{hash}.ext`
    ///
    /// # Examples
    ///
    /// ```
    /// use chrysalis_core::FileNaming;
    ///
    /// let name = FileNaming::add_hash("main.dart.js", "abc12345");
    /// assert_eq!(name, "main.dart.abc12345.js");
    /// ```
    pub fn add_hash(filename: &str, hash: &str) -> String {
        let path = Path::new(filename);
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        
        format!("{}.{}{}", stem, hash, ext)
    }

    /// Add chunk suffix to hashed filename: `filename.{hash}.ext` -> `filename.{hash}.chunk{N}.ext`
    ///
    /// # Examples
    ///
    /// ```
    /// use chrysalis_core::FileNaming;
    ///
    /// let name = FileNaming::add_chunk_suffix("main.dart.abc12345.js", 0);
    /// assert_eq!(name, "main.dart.abc12345.chunk0.js");
    /// ```
    pub fn add_chunk_suffix(hashed_filename: &str, chunk_index: usize) -> String {
        let path = Path::new(hashed_filename);
        let ext = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        
        format!("{}.chunk{}{}", stem, chunk_index, ext)
    }

    /// Extract hash from hashed filename.
    ///
    /// Returns None if filename doesn't contain a hash.
    pub fn extract_hash(filename: &str) -> Option<String> {
        let path = Path::new(filename);
        let stem = path.file_stem()?.to_string_lossy();
        
        // Hash is the last component before extension (8 chars)
        let parts: Vec<&str> = stem.split('.').collect();
        if parts.len() < 2 {
            return None;
        }
        
        let last = parts.last()?;
        if last.len() == 8 && last.chars().all(|c| c.is_ascii_hexdigit()) {
            Some(last.to_string())
        } else {
            None
        }
    }

    /// Check if filename has hash.
    pub fn has_hash(filename: &str) -> bool {
        Self::extract_hash(filename).is_some()
    }

    /// Remove hash from filename: `filename.{hash}.ext` -> `filename.ext`
    pub fn remove_hash(filename: &str) -> String {
        if let Some(hash) = Self::extract_hash(filename) {
            filename.replace(&format!(".{}", hash), "")
        } else {
            filename.to_string()
        }
    }

    /// Get original filename from any processed filename.
    ///
    /// Removes both hash and chunk suffixes.
    pub fn get_original_name(filename: &str) -> String {
        let path = Path::new(filename);
        let ext = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        
        // Remove .chunkN suffix if present
        let stem = if let Some(idx) = stem.rfind(".chunk") {
            &stem[..idx]
        } else {
            &stem
        };
        
        // Remove hash (8 hex chars) if present
        let parts: Vec<&str> = stem.split('.').collect();
        let stem = if parts.len() >= 2 {
            let last = parts.last().unwrap();
            if last.len() == 8 && last.chars().all(|c| c.is_ascii_hexdigit()) {
                parts[..parts.len() - 1].join(".")
            } else {
                stem.to_string()
            }
        } else {
            stem.to_string()
        };
        
        format!("{}{}", stem, ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_hash() {
        assert_eq!(FileNaming::add_hash("main.dart.js", "abc12345"), "main.dart.abc12345.js");
        assert_eq!(FileNaming::add_hash("style.css", "def67890"), "style.def67890.css");
    }

    #[test]
    fn test_add_chunk_suffix() {
        assert_eq!(
            FileNaming::add_chunk_suffix("main.dart.abc12345.js", 0),
            "main.dart.abc12345.chunk0.js"
        );
        assert_eq!(
            FileNaming::add_chunk_suffix("main.dart.abc12345.js", 5),
            "main.dart.abc12345.chunk5.js"
        );
    }

    #[test]
    fn test_extract_hash() {
        assert_eq!(
            FileNaming::extract_hash("main.dart.abc12345.js"),
            Some("abc12345".to_string())
        );
        assert_eq!(FileNaming::extract_hash("main.dart.js"), None);
    }

    #[test]
    fn test_has_hash() {
        assert!(FileNaming::has_hash("main.dart.abc12345.js"));
        assert!(!FileNaming::has_hash("main.dart.js"));
    }

    #[test]
    fn test_remove_hash() {
        assert_eq!(FileNaming::remove_hash("main.dart.abc12345.js"), "main.dart.js");
        assert_eq!(FileNaming::remove_hash("main.dart.js"), "main.dart.js");
    }

    #[test]
    fn test_get_original_name() {
        assert_eq!(
            FileNaming::get_original_name("main.dart.abc12345.chunk0.js"),
            "main.dart.js"
        );
        assert_eq!(
            FileNaming::get_original_name("main.dart.abc12345.js"),
            "main.dart.js"
        );
        assert_eq!(
            FileNaming::get_original_name("main.dart.js"),
            "main.dart.js"
        );
    }
}
