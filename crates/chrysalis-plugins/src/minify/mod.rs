//! Minification plugin.

mod css;
mod html;
mod js;
mod json;

use crate::{Plugin, Result};
use chrysalis_config::MinifyConfig;
use chrysalis_core::BuildContext;
use tracing::{info, warn};

pub use css::minify_css;
pub use html::minify_html;
pub use js::minify_js;
pub use json::minify_json;

/// Minification plugin.
pub struct MinifyPlugin {
    config: MinifyConfig,
    skip_index_html: bool,
}

impl MinifyPlugin {
    /// Create a new minify plugin.
    ///
    /// # Arguments
    /// * `config` - Minification configuration
    /// * `skip_index_html` - Whether to skip index.html (true if inject plugin will handle it)
    pub fn new(config: MinifyConfig, skip_index_html: bool) -> Self {
        Self {
            config,
            skip_index_html,
        }
    }
}

#[async_trait::async_trait]
impl Plugin for MinifyPlugin {
    fn name(&self) -> &str {
        "minify"
    }

    async fn execute(&self, ctx: &mut BuildContext) -> Result<()> {
        if !self.config.enabled {
            info!("Minification disabled");
            return Ok(());
        }

        info!("Minifying files...");
        let mut minified_count = 0;

        // Collect files to process
        let files: Vec<_> = ctx.files().map(|f| f.absolute.clone()).collect();

        for file_path in files {
            let file = ctx.get_file_mut(&file_path).unwrap();

            // Skip index.html if inject plugin will handle it
            if self.skip_index_html && file.is_html() && file.name == "index.html" {
                info!("  Skipping index.html (will be minified after injection)");
                continue;
            }

            // Load content
            if let Err(e) = file.load_content() {
                warn!("Failed to load {}: {}", file.name, e);
                continue;
            }

            let original_size = file.size;
            let content = file.content.as_ref().unwrap();

            let minified = if file.is_js() && self.config.minify_js {
                match minify_js(content) {
                    Ok(m) => Some(m),
                    Err(e) => {
                        warn!("Failed to minify JS {}: {}", file.name, e);
                        None
                    }
                }
            } else if file.is_css() && self.config.minify_css {
                match minify_css(content) {
                    Ok(m) => Some(m),
                    Err(e) => {
                        warn!("Failed to minify CSS {}: {}", file.name, e);
                        None
                    }
                }
            } else if file.is_html() && self.config.minify_html {
                match minify_html(content) {
                    Ok(m) => Some(m),
                    Err(e) => {
                        warn!("Failed to minify HTML {}: {}", file.name, e);
                        None
                    }
                }
            } else if file.is_json() && self.config.minify_json {
                match minify_json(content) {
                    Ok(m) => Some(m),
                    Err(e) => {
                        warn!("Failed to minify JSON {}: {}", file.name, e);
                        None
                    }
                }
            } else {
                None
            };

            if let Some(minified_content) = minified {
                let new_size = minified_content.len() as u64;

                // Write to disk
                chrysalis_core::write_file_content(&file_path, &minified_content)?;

                // Update file
                let file = ctx.get_file_mut(&file_path).unwrap();
                file.set_content(minified_content);

                // Record stats
                ctx.stats_mut().record_minification(original_size, new_size);
                minified_count += 1;
            }
        }

        info!("✓ Minified {} files", minified_count);
        Ok(())
    }
}
