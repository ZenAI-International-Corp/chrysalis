//! File system scanner.

use crate::{BuildError, FileInfo, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use walkdir::WalkDir;

/// File scanner.
pub struct Scanner {
    /// Root directory to scan.
    root: PathBuf,
    
    /// Glob patterns to exclude.
    exclude_patterns: Vec<glob::Pattern>,
}

impl Scanner {
    /// Create a new scanner.
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        
        if !root.exists() {
            return Err(BuildError::DirectoryNotFound(root));
        }
        
        Ok(Self {
            root,
            exclude_patterns: Vec::new(),
        })
    }

    /// Add exclude pattern.
    pub fn exclude(mut self, pattern: &str) -> Result<Self> {
        let pattern = glob::Pattern::new(pattern)
            .map_err(|e| BuildError::GlobPattern(e.to_string()))?;
        self.exclude_patterns.push(pattern);
        Ok(self)
    }

    /// Add multiple exclude patterns.
    pub fn exclude_many<I, S>(mut self, patterns: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for pattern in patterns {
            let pattern = glob::Pattern::new(pattern.as_ref())
                .map_err(|e| BuildError::GlobPattern(e.to_string()))?;
            self.exclude_patterns.push(pattern);
        }
        Ok(self)
    }

    /// Scan directory and return all files.
    pub fn scan(&self) -> Result<Vec<FileInfo>> {
        info!("Scanning directory: {}", self.root.display());
        
        let mut files = Vec::new();
        
        for entry in WalkDir::new(&self.root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !self.is_excluded(e.path()))
        {
            let entry = entry.map_err(|e| {
                BuildError::Other(anyhow::anyhow!("Walk directory error: {}", e))
            })?;
            
            if entry.file_type().is_file() {
                let absolute = entry.path().to_path_buf();
                let relative = pathdiff::diff_paths(&absolute, &self.root)
                    .ok_or_else(|| BuildError::InvalidPath(absolute.clone()))?;
                
                let metadata = entry.metadata().map_err(|e| {
                    BuildError::Other(anyhow::anyhow!("Failed to read metadata for {}: {}", absolute.display(), e))
                })?;
                
                let file_info = FileInfo::new(absolute, relative, metadata.len());
                files.push(file_info);
            }
        }
        
        debug!("Found {} files", files.len());
        Ok(files)
    }

    /// Check if path should be excluded.
    fn is_excluded(&self, path: &Path) -> bool {
        if let Some(relative) = pathdiff::diff_paths(path, &self.root) {
            for pattern in &self.exclude_patterns {
                if pattern.matches_path(&relative) {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scanner() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        
        // Create test files
        fs::write(root.join("file1.txt"), "content1").unwrap();
        fs::write(root.join("file2.js"), "content2").unwrap();
        fs::create_dir(root.join("subdir")).unwrap();
        fs::write(root.join("subdir/file3.css"), "content3").unwrap();
        
        let scanner = Scanner::new(root).unwrap();
        let files = scanner.scan().unwrap();
        
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_scanner_with_exclude() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        
        fs::write(root.join("file1.txt"), "content1").unwrap();
        fs::write(root.join("file2.js"), "content2").unwrap();
        fs::write(root.join("file3.map"), "sourcemap").unwrap();
        
        let scanner = Scanner::new(root)
            .unwrap()
            .exclude("*.map")
            .unwrap();
        let files = scanner.scan().unwrap();
        
        assert_eq!(files.len(), 2);
        assert!(!files.iter().any(|f| f.ext == ".map"));
    }
}
