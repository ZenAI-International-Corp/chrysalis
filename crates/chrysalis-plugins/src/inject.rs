//! Injection plugin for chunk loader.

use crate::{Plugin, PluginError, Result};
use crate::minify::minify_html;
use chrysalis_config::InjectConfig;
use chrysalis_core::BuildContext;
use std::collections::HashMap;
use tracing::{info, warn};

/// Chunk loader template - based on JS version's approach.
const CHUNK_LOADER_TEMPLATE: &str = r#"
(function() {
  'use strict';
  
  // Chunk manifest
  const MANIFEST = {{manifest}};
  const BASE_URL = window.location.origin + window.location.pathname.replace(/\/[^\/]*$/, '/');
  
  // Cache for loaded chunks
  const chunkCache = new Map();
  const loadingPromises = new Map();
  
  /**
   * Load a single chunk using XHR (returns Uint8Array)
   */
  function loadChunk(url) {
    const fullUrl = BASE_URL + url;
    
    // Check cache
    if (chunkCache.has(url)) {
      return Promise.resolve(chunkCache.get(url));
    }
    
    // Check if already loading
    if (loadingPromises.has(url)) {
      return loadingPromises.get(url);
    }
    
    const promise = new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();
      xhr.open('GET', fullUrl, true);
      xhr.responseType = 'arraybuffer';
      
      xhr.onload = function() {
        if (xhr.status === 200) {
          const data = new Uint8Array(xhr.response);
          chunkCache.set(url, data);
          loadingPromises.delete(url);
          resolve(data);
        } else {
          loadingPromises.delete(url);
          reject(new Error(`Failed to load chunk: ${url} (status: ${xhr.status})`));
        }
      };
      
      xhr.onerror = function() {
        loadingPromises.delete(url);
        reject(new Error(`Network error loading chunk: ${url}`));
      };
      
      xhr.send();
    });
    
    loadingPromises.set(url, promise);
    return promise;
  }
  
  // Export public API for stub files to use
  window.ChunkLoader = {
    loadChunk: loadChunk,
    manifest: MANIFEST,
    cache: chunkCache,
  };
  
  // Export for debugging
  window.__CHRYSALIS__ = {
    manifest: MANIFEST,
    chunkCache: chunkCache,
    loadChunk: loadChunk,
  };
})();
"#;

/// Inject plugin adds chunk loader to HTML.
pub struct InjectPlugin {
    config: InjectConfig,
}

impl InjectPlugin {
    /// Create a new inject plugin.
    pub fn new(config: InjectConfig) -> Self {
        Self { config }
    }

    /// Generate chunk manifest.
    /// Maps parent file names (with hash) to their chunk file names (with hash).
    fn generate_manifest(&self, ctx: &BuildContext) -> HashMap<String, Vec<String>> {
        let mut manifest = HashMap::new();

        for (parent_path, chunk_paths) in ctx.chunks().iter() {
            if let Some(parent_file) = ctx.get_file(parent_path) {
                let parent_name = parent_file.name.clone();
                
                // Get chunk names from the chunk paths (already in correct order)
                let chunk_names: Vec<String> = chunk_paths
                    .iter()
                    .filter_map(|chunk_path| {
                        ctx.get_file(chunk_path).map(|f| f.name.clone())
                    })
                    .collect();

                if !chunk_names.is_empty() {
                    manifest.insert(parent_name, chunk_names);
                }
            }
        }

        manifest
    }

    /// Extract base name from a file name (removing hash and extension).
    /// e.g., "main.dart.abc123.js" -> "main.dart"
    ///       "main.dart.js" -> "main.dart"
    fn extract_base_name(name: &str) -> String {
        // Split by dots
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() < 2 {
            return name.to_string();
        }
        
        // Remove extension
        let without_ext = &parts[..parts.len() - 1];
        
        // Check if second-to-last part is a hash (8 hex chars)
        if without_ext.len() >= 2 {
            let potential_hash = without_ext[without_ext.len() - 1];
            if potential_hash.len() == 8 && potential_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                // Remove hash part
                return without_ext[..without_ext.len() - 1].join(".");
            }
        }
        
        without_ext.join(".")
    }

    /// Generate chunk loader script.
    fn generate_loader(&self, manifest: &HashMap<String, Vec<String>>) -> Result<String> {
        let manifest_json = serde_json::to_string(manifest)
            .map_err(|e| PluginError::InjectionFailed(format!("Failed to serialize manifest: {}", e)))?;

        let loader = CHUNK_LOADER_TEMPLATE.replace("{{manifest}}", &manifest_json);
        Ok(loader)
    }

    /// Update file references in HTML to use hashed versions.
    fn update_file_references(&self, html_content: &str, ctx: &BuildContext) -> String {
        let mut result = html_content.to_string();
        
        // Build a map of original names to hashed names
        let mut name_map = HashMap::new();
        for file in ctx.files() {
            // If the file has been hashed, it will have a hash in its name
            if file.name.contains('.') {
                // Extract the base name without hash
                // e.g., "flutter_bootstrap.e9a99a30.js" -> "flutter_bootstrap.js"
                let parts: Vec<&str> = file.name.split('.').collect();
                if parts.len() >= 3 {
                    // Check if the second-to-last part looks like a hash (8 hex chars)
                    let potential_hash = parts[parts.len() - 2];
                    if potential_hash.len() == 8 && potential_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                        // Reconstruct original name without hash
                        let mut original_parts = parts.clone();
                        original_parts.remove(parts.len() - 2);
                        let original_name = original_parts.join(".");
                        name_map.insert(original_name, file.name.clone());
                    }
                }
            }
        }
        
        // Update references in HTML - handle quoted, unquoted, and compressed formats
        for (original, hashed) in name_map.iter() {
            // Pattern 1: src=filename (no quotes, compressed HTML)
            result = result.replace(&format!("src={}", original), &format!("src={}", hashed));
            // Pattern 2: src="filename"
            result = result.replace(&format!("src=\"{}\"", original), &format!("src=\"{}\"", hashed));
            // Pattern 3: src='filename'
            result = result.replace(&format!("src='{}'", original), &format!("src='{}'", hashed));
            // Pattern 4: href=filename (no quotes, compressed HTML)
            result = result.replace(&format!("href={}", original), &format!("href={}", hashed));
            // Pattern 5: href="filename"
            result = result.replace(&format!("href=\"{}\"", original), &format!("href=\"{}\"", hashed));
            // Pattern 6: href='filename'
            result = result.replace(&format!("href='{}'", original), &format!("href='{}'", hashed));
        }
        
        result
    }

    /// Inject loader into HTML file.
    fn inject_into_html(&self, html_content: &str, loader_script: &str) -> String {
        // Find </head> tag and inject before it
        if let Some(pos) = html_content.find("</head>") {
            let mut result = String::with_capacity(html_content.len() + loader_script.len() + 20);
            result.push_str(&html_content[..pos]);
            result.push_str("<script>");
            result.push_str(loader_script);
            result.push_str("</script>");
            result.push_str(&html_content[pos..]);
            result
        } else {
            // If no </head>, inject at beginning of <body>
            if let Some(pos) = html_content.find("<body") {
                if let Some(end) = html_content[pos..].find('>') {
                    let insert_pos = pos + end + 1;
                    let mut result = String::with_capacity(html_content.len() + loader_script.len() + 20);
                    result.push_str(&html_content[..insert_pos]);
                    result.push_str("<script>");
                    result.push_str(loader_script);
                    result.push_str("</script>");
                    result.push_str(&html_content[insert_pos..]);
                    return result;
                }
            }
            
            // Fallback: just prepend
            format!("<script>{}</script>{}", loader_script, html_content)
        }
    }

    /// Update stub files to use correct hashed file names.
    fn update_stub_references(&self, ctx: &mut BuildContext) -> Result<()> {
        info!("  Updating stub file references...");

        // Find all stub files (files that were chunked)
        let stub_files: Vec<_> = ctx
            .chunks()
            .keys()
            .filter_map(|parent_path| {
                ctx.get_file(parent_path).map(|f| (f.absolute.clone(), f.name.clone()))
            })
            .collect();

        for (stub_path, hashed_name) in stub_files {
            // Extract the original name without hash
            let original_name = Self::extract_base_name(&hashed_name);
            let original_full = format!("{}.js", original_name);

            // Update the stub file content
            let updated_content = {
                let file = ctx.get_file_mut(&stub_path).unwrap();

                // Load content
                if let Err(e) = file.load_content() {
                    warn!("Failed to load stub {}: {}", file.name, e);
                    continue;
                }

                let mut content = match file.content_as_str() {
                    Some(s) => s.to_string(),
                    None => continue,
                };

                // Update the fileName variable to use the hashed name
                // Old: const fileName = 'main.dart.js';
                // New: const fileName = 'main.dart.abc123.js';
                content = content.replace(
                    &format!("const fileName = '{}';", original_full),
                    &format!("const fileName = '{}';", hashed_name),
                );
                content = content.replace(
                    &format!("const fileName = \"{}\";", original_full),
                    &format!("const fileName = \"{}\";", hashed_name),
                );

                content.into_bytes()
            };

            // Write back
            chrysalis_core::write_file_content(&stub_path, &updated_content)?;

            let file = ctx.get_file_mut(&stub_path).unwrap();
            file.set_content(updated_content);
            info!("    Updated stub: {}", file.name);
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Plugin for InjectPlugin {
    fn name(&self) -> &str {
        "inject"
    }

    async fn execute(&self, ctx: &mut BuildContext) -> Result<()> {
        if !self.config.enabled {
            info!("Injection disabled");
            return Ok(());
        }

        // Check if there are any chunks
        if ctx.chunks().is_empty() {
            info!("No chunks to inject loader for");
            return Ok(());
        }

        info!("Injecting chunk loader...");

        // Generate manifest
        let manifest = self.generate_manifest(ctx);
        info!("  Manifest entries: {}", manifest.len());

        // Generate loader script
        let loader_script = self.generate_loader(&manifest)?;
        
        // Minify loader if possible
        let loader_script = if self.config.inline_manifest {
            // Already minified by template
            loader_script
        } else {
            loader_script
        };

        // Find HTML files and inject
        // Only inject into index.html (the main entry point)
        let html_files: Vec<_> = ctx
            .files()
            .filter(|f| f.is_html() && f.name == "index.html")
            .map(|f| f.absolute.clone())
            .collect();

        for html_path in html_files {
            // Load HTML content and get a copy
            let html_content = {
                let file = ctx.get_file_mut(&html_path).unwrap();

                // Load HTML content
                if let Err(e) = file.load_content() {
                    warn!("Failed to load {}: {}", file.name, e);
                    continue;
                }

                match file.content_as_str() {
                    Some(s) => s.to_string(),
                    None => {
                        warn!("HTML file {} is not valid UTF-8", file.name);
                        continue;
                    }
                }
            };

            // Update file references to use hashed versions
            let updated_html = self.update_file_references(&html_content, ctx);
            
            // Inject loader
            let injected_html = self.inject_into_html(&updated_html, &loader_script);

            // Minify HTML (index.html was skipped by minify plugin, so this is the first minification)
            let new_html = match minify_html(injected_html.as_bytes()) {
                Ok(minified) => minified,
                Err(e) => {
                    warn!("Failed to minify HTML after injection: {}", e);
                    injected_html.into_bytes()
                }
            };

            // Write back
            chrysalis_core::write_file_content(&html_path, &new_html)?;
            
            let file = ctx.get_file_mut(&html_path).unwrap();
            file.set_content(new_html);

            info!("  Injected into: {}", file.name);
        }

        // Update chunk references in stub files (after hashing)
        self.update_stub_references(ctx)?;

        info!("✓ Chunk loader injected");
        Ok(())
    }
}
