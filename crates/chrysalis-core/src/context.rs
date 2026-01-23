//! Build context - shared state across all plugins.

use crate::{BuildError, BuildStats, FileInfo, Result, Scanner};
use chrysalis_config::BuildConfig;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Build context maintains state across all build plugins.
#[derive(Debug)]
pub struct BuildContext {
    /// Build directory.
    build_dir: PathBuf,

    /// Build configuration.
    config: BuildConfig,

    /// All files indexed by absolute path.
    files: HashMap<PathBuf, FileInfo>,

    /// File mapping: old relative path -> new relative path.
    file_mapping: HashMap<PathBuf, PathBuf>,

    /// Chunk information: parent file -> chunk files.
    chunks: HashMap<PathBuf, Vec<PathBuf>>,

    /// File dependencies: file -> set of dependencies.
    dependencies: HashMap<PathBuf, HashSet<PathBuf>>,

    /// Build statistics.
    stats: BuildStats,
}

impl BuildContext {
    /// Create a new build context.
    pub fn new<P: AsRef<Path>>(build_dir: P, config: BuildConfig) -> Result<Self> {
        let build_dir = build_dir.as_ref().to_path_buf();

        if !build_dir.exists() {
            return Err(BuildError::DirectoryNotFound(build_dir));
        }

        Ok(Self {
            build_dir,
            config,
            files: HashMap::new(),
            file_mapping: HashMap::new(),
            chunks: HashMap::new(),
            dependencies: HashMap::new(),
            stats: BuildStats::new(),
        })
    }

    /// Scan the build directory and index all files.
    pub fn scan(&mut self) -> Result<()> {
        info!("Scanning build directory: {}", self.build_dir.display());

        let scanner = Scanner::new(&self.build_dir)?.exclude_many(&self.config.exclude_patterns)?;

        let files = scanner.scan()?;
        self.stats.total_files = files.len();

        for file in files {
            self.files.insert(file.absolute.clone(), file);
        }

        info!("Found {} files", self.stats.total_files);
        Ok(())
    }

    /// Get a file by absolute path.
    pub fn get_file<P: AsRef<Path>>(&self, path: P) -> Option<&FileInfo> {
        self.files.get(path.as_ref())
    }

    /// Get a mutable reference to a file.
    pub fn get_file_mut<P: AsRef<Path>>(&mut self, path: P) -> Option<&mut FileInfo> {
        self.files.get_mut(path.as_ref())
    }

    /// Get all files.
    pub fn files(&self) -> impl Iterator<Item = &FileInfo> {
        self.files.values()
    }

    /// Get all files matching a predicate.
    pub fn files_matching<F>(&self, predicate: F) -> impl Iterator<Item = &FileInfo>
    where
        F: Fn(&FileInfo) -> bool,
    {
        self.files.values().filter(move |f| predicate(f))
    }

    /// Get mutable iterator over all files.
    pub fn files_mut(&mut self) -> impl Iterator<Item = &mut FileInfo> {
        self.files.values_mut()
    }

    /// Add a new file to the context.
    pub fn add_file(&mut self, file: FileInfo) -> Result<()> {
        if self.files.contains_key(&file.absolute) {
            return Err(BuildError::FileAlreadyExists(file.absolute.clone()));
        }

        self.files.insert(file.absolute.clone(), file);
        Ok(())
    }

    /// Remove a file from the context.
    pub fn remove_file<P: AsRef<Path>>(&mut self, path: P) -> Option<FileInfo> {
        self.files.remove(path.as_ref())
    }

    /// Rename a file and update all mappings.
    pub fn rename_file<P: AsRef<Path>>(&mut self, old_path: P, new_path: P) -> Result<()> {
        let old_path = old_path.as_ref();
        let new_path = new_path.as_ref();

        let mut file = self
            .files
            .remove(old_path)
            .ok_or_else(|| BuildError::FileNotFound(old_path.to_path_buf()))?;

        // Rename physical file
        std::fs::rename(&file.absolute, new_path).map_err(|source| BuildError::Io {
            path: old_path.to_path_buf(),
            source,
        })?;

        let old_relative = file.relative.clone();
        let new_relative = pathdiff::diff_paths(new_path, &self.build_dir)
            .ok_or_else(|| BuildError::InvalidPath(new_path.to_path_buf()))?;

        // Update file info
        file.absolute = new_path.to_path_buf();
        file.relative = new_relative.clone();
        file.name = new_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        file.dir = new_relative.parent().unwrap_or(Path::new("")).to_path_buf();

        // Update mappings
        self.files.insert(new_path.to_path_buf(), file);
        self.file_mapping.insert(old_relative, new_relative);

        // Update chunks mapping if this file is a parent of chunks
        if let Some(chunk_paths) = self.chunks.remove(old_path) {
            self.chunks.insert(new_path.to_path_buf(), chunk_paths);
        }

        // Update chunks mapping if this file is a chunk (appears in any chunk list)
        for (_parent, chunk_paths) in self.chunks.iter_mut() {
            for chunk_path in chunk_paths.iter_mut() {
                if chunk_path == old_path {
                    *chunk_path = new_path.to_path_buf();
                }
            }
        }

        debug!("Renamed: {} -> {}", old_path.display(), new_path.display());
        Ok(())
    }

    /// Add chunk information.
    pub fn add_chunk_info<P: AsRef<Path>>(&mut self, parent: P, chunks: Vec<PathBuf>) {
        let num_chunks = chunks.len();
        self.chunks.insert(parent.as_ref().to_path_buf(), chunks);
        self.stats.record_chunk(num_chunks);
    }

    /// Get chunk information.
    pub fn get_chunk_info<P: AsRef<Path>>(&self, parent: P) -> Option<&Vec<PathBuf>> {
        self.chunks.get(parent.as_ref())
    }

    /// Add file dependency.
    pub fn add_dependency<P: AsRef<Path>, Q: AsRef<Path>>(&mut self, file: P, dependency: Q) {
        self.dependencies
            .entry(file.as_ref().to_path_buf())
            .or_insert_with(HashSet::new)
            .insert(dependency.as_ref().to_path_buf());
    }

    /// Get file dependencies.
    pub fn get_dependencies<P: AsRef<Path>>(&self, file: P) -> Option<&HashSet<PathBuf>> {
        self.dependencies.get(file.as_ref())
    }

    /// Get build statistics.
    pub fn stats(&self) -> &BuildStats {
        &self.stats
    }

    /// Get mutable build statistics.
    pub fn stats_mut(&mut self) -> &mut BuildStats {
        &mut self.stats
    }

    /// Get build directory.
    pub fn build_dir(&self) -> &Path {
        &self.build_dir
    }

    /// Get build configuration.
    pub fn config(&self) -> &BuildConfig {
        &self.config
    }

    /// Get file mapping.
    pub fn file_mapping(&self) -> &HashMap<PathBuf, PathBuf> {
        &self.file_mapping
    }

    /// Get all chunks.
    pub fn chunks(&self) -> &HashMap<PathBuf, Vec<PathBuf>> {
        &self.chunks
    }
}
