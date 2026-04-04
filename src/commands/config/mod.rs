//! 配置与模型命令模块

use crate::error::{ClaudeError, Result};
use crate::commands::registry::{CommandLoader, CommandRegistry};
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, LocalJsxCommand, LoadedFrom,
    CommandResultDisplay,
};
use crate::commands::registry::CommandExecutor;

pub mod model;
pub mod effort;
pub mod fast;

pub use model::ModelCommand;
pub use effort::EffortCommand;
pub use fast::FastCommand;

/// 配置命令加载器
pub struct ConfigCommandLoader;

#[async_trait::async_trait]
impl CommandLoader for ConfigCommandLoader {
    async fn load(&self, registry: &CommandRegistry) -> Result<()> {
        registry.register(ModelCommand).await?;
        registry.register(EffortCommand).await?;
        registry.register(FastCommand).await?;

        tracing::debug!("Loaded config commands");

        Ok(())
    }

    fn name(&self) -> &str {
        "config"
    }
}
