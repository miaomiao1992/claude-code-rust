//! Claude Code Tools Framework
//!
//! 完整的工具定义与执行框架，包括：
//! - 工具类型系统
//! - 工具权限系统
//! - 工具注册和执行
//! - 与API工具调用集成

pub mod types;
pub mod permissions;
pub mod base;
pub mod registry;
pub mod error;
pub mod file_tools;
pub mod search_tools;
pub mod command_tools;
pub mod web_tools;
pub mod skill_tools;
pub mod message_tools;
pub mod task_tools;
pub mod plan_tools;
#[cfg(feature = "git-worktree")]
pub mod git_tools;
pub mod user_tools;
pub mod lsp_tools;
pub mod time_tools;
pub mod cron_tools;
pub mod team_tools;
pub mod tool_search;
pub mod api_integration;

// 重新导出主要类型
use crate::types::ToolExecutionOptions;
pub use types::{
    ToolMetadata, ToolResult, ToolUseContext, ToolInputSchema,
    ToolCategory, ToolPermissionLevel, ValidationResult, PermissionResult,
    PermissionMode, PermissionBehavior, ToolPermissionContext,
};
pub use base::{Tool, ToolBuilder};
pub use registry::{ToolRegistry, ToolManager, ToolLoader};
pub use permissions::{PermissionChecker, ModeChecker};

// 工具实现
pub use file_tools::{FileReadTool, FileEditTool, FileWriteTool};
pub use search_tools::{GlobTool, GrepTool};
pub use command_tools::{BashTool, PowerShellTool};
pub use time_tools::SleepTool;
pub use tool_search::ToolSearchTool;

use anyhow::Result;

/// 工具系统配置
#[derive(Debug, Clone)]
pub struct ToolSystemConfig {
    /// 是否启用API工具调用集成
    pub enable_api_tool_use: bool,
    /// 默认权限模式
    pub default_permission_mode: PermissionMode,
    /// 工具预设
    pub tool_preset: ToolPreset,
}

impl Default for ToolSystemConfig {
    fn default() -> Self {
        Self {
            enable_api_tool_use: false,
            default_permission_mode: PermissionMode::Default,
            tool_preset: ToolPreset::Default,
        }
    }
}

/// 工具预设
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolPreset {
    /// 默认预设
    Default,
    /// 简单预设（只读工具）
    Simple,
    /// 完整预设（所有工具）
    Full,
}

impl ToolPreset {
    /// 获取预设的工具名称
    pub fn tool_names(&self) -> Vec<String> {
        match self {
            ToolPreset::Default => vec![
                "Read".to_string(),
                "Edit".to_string(),
                "Write".to_string(),
                "Glob".to_string(),
                "Grep".to_string(),
                "Bash".to_string(),
                "PowerShell".to_string(),
                "Sleep".to_string(),
                "ToolSearch".to_string(),
            ],
            ToolPreset::Simple => vec![
                "Read".to_string(),
                "Glob".to_string(),
                "Grep".to_string(),
                "Sleep".to_string(),
                "ToolSearch".to_string(),
            ],
            ToolPreset::Full => self::get_tool_names(),
        }
    }
}

/// 获取所有工具名称
pub fn get_tool_names() -> Vec<String> {
    vec![
        "Read".to_string(),
        "Edit".to_string(),
        "Write".to_string(),
        "Glob".to_string(),
        "Grep".to_string(),
        "Bash".to_string(),
        "PowerShell".to_string(),
        "Sleep".to_string(),
        "ToolSearch".to_string(),
    ]
}

/// 初始化工具系统
pub async fn init(config: ToolSystemConfig) -> Result<ToolManager> {
    let execution_options = ToolExecutionOptions {
        enable_api_tool_use: config.enable_api_tool_use,
        ..Default::default()
    };
    let manager = ToolManager::new(execution_options);

    // 注册内置工具加载器
    manager.add_loader(Box::new(BuiltinToolLoader));

    // 加载所有工具
    manager.load_all().await?;

    tracing::info!("Tool system initialized with {} tools",
        manager.registry().len().await);

    Ok(manager)
}

/// 内置工具加载器
struct BuiltinToolLoader;

#[async_trait::async_trait]
impl ToolLoader for BuiltinToolLoader {
    async fn load(&self, registry: &ToolRegistry) -> Result<()> {
        // 注册文件操作工具
        registry.register(FileReadTool).await;
        registry.register(FileEditTool).await;
        registry.register(FileWriteTool).await;

        // 注册代码搜索工具
        registry.register(GlobTool).await;
        registry.register(GrepTool).await;

        // 注册命令执行工具
        registry.register(BashTool).await;
        registry.register(PowerShellTool).await;

        // 注册系统工具
        registry.register(SleepTool).await;
        registry.register(ToolSearchTool).await;

        tracing::debug!("Loaded {} builtin tools", 9);

        Ok(())
    }

    fn name(&self) -> &str {
        "builtin"
    }
}