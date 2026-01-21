//! Plugin trait and context.

use crate::Result;
use chrysalis_core::BuildContext;

/// Plugin trait for build transformations.
#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin name.
    fn name(&self) -> &str;

    /// Execute the plugin.
    async fn execute(&self, ctx: &mut BuildContext) -> Result<()>;
}

/// Plugin execution context with progress tracking.
pub struct PluginContext {
    /// Plugin name.
    pub name: String,
}

impl PluginContext {
    /// Create a new plugin context.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
}

// Re-export async_trait for plugins
pub use async_trait;
