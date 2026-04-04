//! 系统诊断命令模块

use crate::error::{ClaudeError, Result};
use crate::commands::registry::{CommandLoader, CommandRegistry};

pub mod doctor;

pub use doctor::DoctorCommand;

/// 系统命令加载器
pub struct SystemCommandLoader;

#[async_trait::async_trait]
impl CommandLoader for SystemCommandLoader {
    async fn load(&self, registry: &CommandRegistry) -> Result<()> {
        registry.register(DoctorCommand).await;

        tracing::debug!("Loaded system diagnostic commands");

        Ok(())
    }

    fn name(&self) -> &str {
        "system"
    }
}
