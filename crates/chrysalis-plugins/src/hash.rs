//! Hashing plugin for content-based filenames.

use crate::{Plugin, Result};
use chrysalis_config::HashConfig;
use chrysalis_core::{BuildContext, FileNaming};
use glob::Pattern;
use tracing::{info, warn};

/// Hash plugin adds content hashes to filenames.
pub struct HashPlugin {
    config: HashConfig,
    include_patterns: Vec<Pattern>,
    exclude_patterns: Vec<Pattern>,
}

impl HashPlugin {
    /// Create a new hash plugin.
    pub fn new(config: HashConfig) -> Result<Self> {
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

        Ok(Self {
            config,
            include_patterns,
            exclude_patterns,
        })
    }

    /// Check if file should be hashed.
    fn should_hash(&self, relative_path: &std::path::Path) -> bool {
        // Flutter framework files that must keep their original names
        if let Some(file_name) = relative_path.file_name() {
            let name = file_name.to_string_lossy();
            if chrysalis_core::is_flutter_framework_file(&name) {
                return false;
            }
        }
        
        // Check exclude patterns
        for pattern in &self.exclude_patterns {
            if pattern.matches_path(relative_path) {
                return false;
            }
        }

        // Check include patterns
        for pattern in &self.include_patterns {
            if pattern.matches_path(relative_path) {
                return true;
            }
        }

        false
    }

    /// Replace file references in content using the file mapping.
    fn replace_references(&self, content: &str, ctx: &BuildContext) -> String {
        let mut result = content.to_string();
        
        // Get file mapping (old relative path -> new relative path)
        let file_mapping = ctx.file_mapping();
        
        // Sort by length (longest first) to avoid partial replacements
        let mut mappings: Vec<_> = file_mapping.iter().collect();
        mappings.sort_by(|a, b| b.0.as_os_str().len().cmp(&a.0.as_os_str().len()));
        
        for (old_path, new_path) in mappings {
            let old_str = old_path.to_string_lossy();
            let new_str = new_path.to_string_lossy();
            
            // Extract just the filename for both
            let old_filename = old_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let new_filename = new_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            
            if old_filename == new_filename {
                continue;
            }
            
            // Replace patterns:
            // 1. Quoted filename: "main.dart.js" -> "main.dart.abc123.js"
            result = result.replace(&format!("\"{}\"", old_filename), &format!("\"{}\"", new_filename));
            result = result.replace(&format!("'{}'", old_filename), &format!("'{}'", new_filename));
            result = result.replace(&format!("`{}`", old_filename), &format!("`{}`", new_filename));
            
            // 2. Quoted full path
            result = result.replace(&format!("\"{}\"", old_str), &format!("\"{}\"", new_str));
            result = result.replace(&format!("'{}'", old_str), &format!("'{}'", new_str));
            
            // 3. src/href attributes: src=filename or src="filename"
            result = result.replace(&format!("src={}", old_filename), &format!("src={}", new_filename));
            result = result.replace(&format!("href={}", old_filename), &format!("href={}", new_filename));
        }
        
        result
    }
}

#[async_trait::async_trait]
impl Plugin for HashPlugin {
    fn name(&self) -> &str {
        "hash"
    }

    async fn execute(&self, ctx: &mut BuildContext) -> Result<()> {
        if !self.config.enabled {
            info!("Hashing disabled");
            return Ok(());
        }

        info!("Adding content hashes to filenames...");
        let hash_length = ctx.config().hash_length;

        // Phase 1: Rename files with hash suffix
        info!("  Phase 1: Adding hash suffixes...");
        let files_to_hash: Vec<_> = ctx
            .files()
            .filter(|f| self.should_hash(&f.relative))
            .map(|f| f.absolute.clone())
            .collect();

        for file_path in files_to_hash {
            // Load content and calculate hash
            let (new_path, file_name) = {
                let file = ctx.get_file_mut(&file_path).unwrap();

                // Load content for hashing
                if let Err(e) = file.load_content() {
                    warn!("Failed to load {}: {}", file.name, e);
                    continue;
                }

                let content = file.content.as_ref().unwrap();
                let hash = chrysalis_core::calculate_hash(content, hash_length);

                // Generate new filename
                let new_name = FileNaming::add_hash(&file.name, &hash);
                let new_path = file.absolute.parent().unwrap().join(&new_name);
                
                (new_path, file.name.clone())
            };

            // Rename file
            if let Err(e) = ctx.rename_file(&file_path, &new_path) {
                warn!("Failed to rename {}: {}", file_name, e);
                continue;
            }

            ctx.stats_mut().record_hash();
        }

        info!("  ✓ Renamed {} files", ctx.stats().hashed_files);

        // Phase 2: Update references in text files
        info!("  Phase 2: Updating file references...");
        let text_files: Vec<_> = ctx
            .files()
            .filter(|f| f.is_js() || f.is_html() || f.is_css() || f.is_json())
            .map(|f| f.absolute.clone())
            .collect();

        let mut updated_count = 0;
        for file_path in text_files {
            // Load content first
            let content = {
                let file = ctx.get_file_mut(&file_path).unwrap();

                // Load content
                if let Err(e) = file.load_content() {
                    warn!("Failed to load {}: {}", file.name, e);
                    continue;
                }

                match file.content_as_str() {
                    Some(s) => s.to_string(),
                    None => continue,
                }
            };

            // Replace references (now ctx is not borrowed)
            let new_content = self.replace_references(&content, ctx);
            
            if new_content != content {
                let new_bytes = new_content.into_bytes();
                
                // Write back
                chrysalis_core::write_file_content(&file_path, &new_bytes)?;
                
                let file = ctx.get_file_mut(&file_path).unwrap();
                file.set_content(new_bytes);
                updated_count += 1;
            }
        }

        info!("  ✓ Updated {} files with new references", updated_count);
        info!("✓ Hashed {} files", ctx.stats().hashed_files);
        Ok(())
    }
}
