//! Chunking plugin for large file splitting.

use crate::{Plugin, PluginError, Result};
use chrysalis_config::ChunkConfig;
use chrysalis_core::{BuildContext, FileInfo, FileNaming};
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

/// Chunk metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    /// Parent file path.
    pub parent: String,
    /// Total number of chunks.
    pub total_chunks: usize,
    /// Chunk paths (relative).
    pub chunks: Vec<String>,
}

/// Chunk plugin splits large files into smaller chunks.
pub struct ChunkPlugin {
    config: ChunkConfig,
    chunk_size: usize,
    min_size: usize,
    include_patterns: Vec<Pattern>,
    exclude_patterns: Vec<Pattern>,
}

impl ChunkPlugin {
    /// Create a new chunk plugin.
    pub fn new(config: ChunkConfig) -> Result<Self> {
        let include_patterns = config
            .include
            .iter()
            .map(|p| Pattern::new(p))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Invalid include pattern: {}", e))?;

        let exclude_patterns = config
            .exclude
            .iter()
            .map(|p| Pattern::new(p))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Invalid exclude pattern: {}", e))?;

        let chunk_size = config.chunk_size_bytes();
        let min_size = config.min_chunk_size_bytes();

        Ok(Self {
            config,
            chunk_size,
            min_size,
            include_patterns,
            exclude_patterns,
        })
    }

    /// Check if file should be chunked.
    fn should_chunk(&self, file: &FileInfo) -> bool {
        // Flutter framework files that must not be modified
        // Note: index.html is handled separately by inject plugin
        if chrysalis_core::is_flutter_framework_file(&file.name) || file.name == "index.html" {
            return false;
        }

        // File must be large enough
        if file.size < self.min_size as u64 {
            return false;
        }

        // Check exclude patterns first
        for pattern in &self.exclude_patterns {
            if pattern.matches_path(&file.relative) {
                return false;
            }
        }

        // Check include patterns
        for pattern in &self.include_patterns {
            if pattern.matches_path(&file.relative) {
                return true;
            }
        }

        false
    }

    /// Split file into chunks.
    fn split_into_chunks(&self, file: &FileInfo) -> Result<Vec<Vec<u8>>> {
        let content = file
            .content
            .as_ref()
            .ok_or_else(|| PluginError::ChunkingFailed {
                file: file.absolute.clone(),
                reason: "Content not loaded".to_string(),
            })?;

        let mut chunks = Vec::new();
        let mut offset = 0;

        while offset < content.len() {
            let end = (offset + self.chunk_size).min(content.len());
            chunks.push(content[offset..end].to_vec());
            offset = end;
        }

        Ok(chunks)
    }

    /// Generate a stub loader for chunked JS files.
    /// The stub will lookup chunks from the global ChunkLoader manifest at runtime.
    fn generate_stub(
        &self,
        file_name: &str,
        _chunk_paths: &[PathBuf],
        _build_dir: &std::path::Path,
    ) -> Result<String> {
        // Generate stub that looks up chunks from manifest at runtime
        // This way, the chunk file names can be hashed after this stub is created
        let stub = format!(
            r#"// Chrysalis chunked file stub
(async function() {{
  const fileName = '{file_name}';
  const maxRetries = 3;
  let retryCount = 0;

  async function loadWithRetry() {{
    try {{
      // Wait for ChunkLoader to be available
      if (!window.ChunkLoader || !window.ChunkLoader.manifest) {{
        if (retryCount < maxRetries) {{
          retryCount++;
          await new Promise(resolve => setTimeout(resolve, 100));
          return loadWithRetry();
        }}
        throw new Error('ChunkLoader not available after ' + maxRetries + ' retries');
      }}

      // Get chunks from manifest (injected at build time with actual hashed names)
      const chunks = window.ChunkLoader.manifest[fileName];
      if (!chunks || chunks.length === 0) {{
        throw new Error('No chunks found in manifest for: ' + fileName);
      }}

      // Load all chunks in parallel using XHR
      const chunkData = await Promise.all(chunks.map(chunk => window.ChunkLoader.loadChunk(chunk)));

      // Merge chunks
      const totalLength = chunkData.reduce((sum, data) => sum + data.length, 0);
      const merged = new Uint8Array(totalLength);
      let offset = 0;
      for (const data of chunkData) {{
        merged.set(data, offset);
        offset += data.length;
      }}

      // Execute the code
      const text = new TextDecoder().decode(merged);
      const script = document.createElement('script');
      script.textContent = text;
      document.head.appendChild(script);
    }} catch (e) {{
      console.error('[Chrysalis] Failed to load chunked file:', e);
      throw e;
    }}
  }}

  await loadWithRetry();
}})();
"#,
            file_name = file_name
        );

        Ok(stub)
    }
}

#[async_trait::async_trait]
impl Plugin for ChunkPlugin {
    fn name(&self) -> &str {
        "chunk"
    }

    async fn execute(&self, ctx: &mut BuildContext) -> Result<()> {
        if !self.config.enabled {
            info!("Chunking disabled");
            return Ok(());
        }

        info!("Chunking large files...");
        info!("  Chunk size: {} KB", self.chunk_size / 1024);
        info!("  Min file size: {} KB", self.min_size / 1024);

        // Collect files to chunk
        let files_to_chunk: Vec<_> = ctx
            .files()
            .filter(|f| self.should_chunk(f))
            .map(|f| f.absolute.clone())
            .collect();

        for file_path in files_to_chunk {
            // Load content and split into chunks
            let (chunks, file_name, parent_dir, build_dir) = {
                let file = ctx.get_file_mut(&file_path).unwrap();

                // Load content
                if let Err(e) = file.load_content() {
                    warn!("Failed to load {}: {}", file.name, e);
                    continue;
                }

                info!("  Chunking: {} ({} KB)", file.name, file.size / 1024);

                // Split into chunks
                let chunks = match self.split_into_chunks(file) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Failed to chunk {}: {}", file.name, e);
                        continue;
                    }
                };

                if chunks.len() <= 1 {
                    continue;
                }

                (
                    chunks,
                    file.name.clone(),
                    file.absolute.parent().unwrap().to_path_buf(),
                    ctx.build_dir().to_path_buf(),
                )
            };

            info!("    Created {} chunks", chunks.len());

            // Write chunk files
            let mut chunk_paths = Vec::new();

            for (i, chunk_content) in chunks.iter().enumerate() {
                let chunk_name = FileNaming::add_chunk_suffix(&file_name, i);
                let chunk_path = parent_dir.join(&chunk_name);

                // Write chunk file
                chrysalis_core::write_file_content(&chunk_path, chunk_content)?;

                // Add to context
                let relative = pathdiff::diff_paths(&chunk_path, &build_dir).ok_or_else(|| {
                    PluginError::ChunkingFailed {
                        file: file_path.clone(),
                        reason: "Failed to compute relative path".to_string(),
                    }
                })?;

                let chunk_file = FileInfo::new(&chunk_path, &relative, chunk_content.len() as u64);
                ctx.add_file(chunk_file)?;
                chunk_paths.push(chunk_path);
            }

            // Record chunk info
            ctx.add_chunk_info(&file_path, chunk_paths.clone());

            // Replace original file with a stub loader (for JS files)
            if file_name.ends_with(".js") {
                let stub_content = self.generate_stub(&file_name, &chunk_paths, &build_dir)?;
                chrysalis_core::write_file_content(&file_path, stub_content.as_bytes())?;

                // Update file info in context
                let file = ctx.get_file_mut(&file_path).unwrap();
                file.size = stub_content.len() as u64;
                file.set_content(stub_content.into_bytes());
            } else {
                // For non-JS files, delete the original
                std::fs::remove_file(&file_path).map_err(|e| PluginError::ChunkingFailed {
                    file: file_path.clone(),
                    reason: format!("Failed to delete original file: {}", e),
                })?;
                ctx.remove_file(&file_path);
            }
        }

        info!(
            "✓ Chunked {} files into {} chunks",
            ctx.stats().chunked_files,
            ctx.stats().total_chunks
        );
        Ok(())
    }
}
