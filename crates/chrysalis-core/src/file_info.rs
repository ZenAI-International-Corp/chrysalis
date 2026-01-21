//! File information structure.

use std::path::{Path, PathBuf};

/// Information about a file in the build.
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Absolute path to the file.
    pub absolute: PathBuf,

    /// Relative path from build directory.
    pub relative: PathBuf,

    /// File name (with extension).
    pub name: String,

    /// File size in bytes.
    pub size: u64,

    /// Parent directory (relative to build directory).
    pub dir: PathBuf,

    /// File extension (including the dot, e.g., ".js").
    pub ext: String,

    /// File content (lazy loaded).
    pub content: Option<Vec<u8>>,

    /// Whether the file has been modified.
    pub modified: bool,
}

impl FileInfo {
    /// Create a new FileInfo.
    pub fn new<P: AsRef<Path>>(
        absolute: P,
        relative: P,
        size: u64,
    ) -> Self {
        let absolute = absolute.as_ref().to_path_buf();
        let relative = relative.as_ref().to_path_buf();
        let name = relative
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let dir = relative.parent().unwrap_or(Path::new("")).to_path_buf();
        let ext = relative
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();

        Self {
            absolute,
            relative,
            name,
            size,
            dir,
            ext,
            content: None,
            modified: false,
        }
    }

    /// Check if this is a JavaScript file.
    pub fn is_js(&self) -> bool {
        self.ext == ".js"
    }

    /// Check if this is a CSS file.
    pub fn is_css(&self) -> bool {
        self.ext == ".css"
    }

    /// Check if this is an HTML file.
    pub fn is_html(&self) -> bool {
        self.ext == ".html"
    }

    /// Check if this is a JSON file.
    pub fn is_json(&self) -> bool {
        self.ext == ".json"
    }

    /// Check if this file matches a glob pattern.
    pub fn matches_pattern(&self, pattern: &glob::Pattern) -> bool {
        pattern.matches_path(&self.relative)
    }

    /// Load file content into memory.
    pub fn load_content(&mut self) -> std::io::Result<&[u8]> {
        if self.content.is_none() {
            self.content = Some(std::fs::read(&self.absolute)?);
        }
        Ok(self.content.as_ref().unwrap())
    }

    /// Get file content as string (if UTF-8).
    pub fn content_as_str(&self) -> Option<&str> {
        self.content
            .as_ref()
            .and_then(|c| std::str::from_utf8(c).ok())
    }

    /// Update file content.
    pub fn set_content(&mut self, content: Vec<u8>) {
        self.size = content.len() as u64;
        self.content = Some(content);
        self.modified = true;
    }

    /// Clear content from memory.
    pub fn clear_content(&mut self) {
        self.content = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_info_creation() {
        let file = FileInfo::new("build/web/main.dart.js", "main.dart.js", 1024);
        assert_eq!(file.name, "main.dart.js");
        assert_eq!(file.ext, ".js");
        assert_eq!(file.size, 1024);
        assert!(file.is_js());
    }

    #[test]
    fn test_file_type_checks() {
        let js_file = FileInfo::new("build/web/app.js", "app.js", 100);
        assert!(js_file.is_js());
        assert!(!js_file.is_css());

        let css_file = FileInfo::new("build/web/style.css", "style.css", 200);
        assert!(css_file.is_css());
        assert!(!css_file.is_js());
    }
}
