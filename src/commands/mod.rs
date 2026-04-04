//! 命令系统模块
//! 
//! 这个模块实现了完整的命令系统架构，包括：
//! - 命令类型系统
//! - 命令注册系统
//! - 命令执行流程
//! - 核心命令实现

pub mod types;
pub mod registry;
pub mod executor;
pub mod builtin;
pub mod cli;

// CLI 命令模块
pub mod interactive;
pub mod query;
pub mod config;
pub mod auth;
pub mod ultraplan;
pub mod voice;
pub mod mcp;
pub mod plugins;
pub mod analytics;
pub mod buddy;
pub mod git;
pub mod upgrade;
pub mod conversation;
pub mod system;

// 重新导出主要类型
pub use types::{Command, CommandContext, CommandResult, CommandBase};
pub use registry::{CommandRegistry, CommandManager, CommandLoader, CommandExecutor};
pub use executor::{CmdExecutor, CommandRouter, UserInput, ExecuteResult};
pub use builtin::{
    VersionCommand, HelpCommand, ClearCommand, ExitCommand,
    ConfigCommand, McpCommand, StatusCommand,
};
pub use upgrade::{UpgradeCommand, run as upgrade};
pub use ultraplan::{
    UltraplanService, UltraplanConfig, UltraplanSession, UltraplanPhase,
    UltraplanResult, ExecutionTarget, PlanEvaluation, build_ultraplan_prompt,
};
pub use cli::{
    CommandHistory, CommandCompleter, ColorOutput, CliInterface, BatchProcessor,
};

use crate::error::Result;
use crate::state::AppState;
use self::git::GitCommandLoader;
use self::conversation::ConversationCommandLoader;
use self::config::ConfigCommandLoader;
use self::system::SystemCommandLoader;

/// 初始化命令系统
pub async fn init() -> Result<CommandManager> {
    let mut manager = CommandManager::new();

    // 注册核心命令加载器
    manager.add_loader(BuiltinCommandLoader);
    manager.add_loader(GitCommandLoader);
    manager.add_loader(ConversationCommandLoader);
    manager.add_loader(ConfigCommandLoader);
    manager.add_loader(SystemCommandLoader);

    // 加载所有命令
    manager.load_all().await?;

    tracing::info!("Command system initialized with {} commands",
        manager.registry().len().await);

    Ok(manager)
}

/// 初始化命令系统（带应用状态）
pub async fn init_with_state(app_state: AppState) -> Result<CommandManager> {
    let mut manager = CommandManager::new();

    // 注册核心命令加载器
    manager.add_loader(BuiltinCommandLoader);
    manager.add_loader(GitCommandLoader);
    manager.add_loader(ConversationCommandLoader);
    manager.add_loader(ConfigCommandLoader);
    manager.add_loader(SystemCommandLoader);

    // 注册 Buddy 命令加载器
    manager.add_loader(BuddyCommandLoader { app_state });

    // 加载所有命令
    manager.load_all().await?;

    tracing::info!("Command system initialized with {} commands",
        manager.registry().len().await);

    Ok(manager)
}

/// 内置命令加载器
struct BuiltinCommandLoader;

#[async_trait::async_trait]
impl CommandLoader for BuiltinCommandLoader {
    async fn load(&self, registry: &CommandRegistry) -> Result<()> {
        // 注册核心命令
        registry.register(VersionCommand).await;
        registry.register(HelpCommand).await;
        registry.register(ClearCommand).await;
        registry.register(ExitCommand).await;
        registry.register(ConfigCommand).await;
        registry.register(McpCommand).await;
        registry.register(StatusCommand).await;
        registry.register(UpgradeCommand).await;

        tracing::debug!("Loaded {} builtin commands", 8);
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "builtin"
    }
}

/// Buddy 命令加载器
struct BuddyCommandLoader {
    app_state: AppState,
}

#[async_trait::async_trait]
impl CommandLoader for BuddyCommandLoader {
    async fn load(&self, registry: &CommandRegistry) -> Result<()> {
        // 注册 Buddy 命令
        let buddy_executor = buddy::BuddyCommandExecutor::new(self.app_state.clone());
        registry.register(buddy_executor).await;
        
        tracing::debug!("Loaded Buddy command");
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "buddy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_init_command_system() {
        let manager = init().await.unwrap();
        assert!(manager.registry().len().await >= 7);
    }
    
    #[tokio::test]
    async fn test_builtin_commands_loaded() {
        let manager = init().await.unwrap();
        
        assert!(manager.registry().has("version").await);
        assert!(manager.registry().has("help").await);
        assert!(manager.registry().has("clear").await);
        assert!(manager.registry().has("exit").await);
        assert!(manager.registry().has("config").await);
        assert!(manager.registry().has("mcp").await);
        assert!(manager.registry().has("status").await);
    }
}
