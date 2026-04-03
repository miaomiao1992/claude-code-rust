//! 工具系统模块
//! 
//! 这个模块实现了完整的工具系统架构，包括：
//! - 工具类型系统
//! - 工具权限系统
//! - 工具注册系统
//! - 核心工具实现

pub mod types;
pub mod permissions;
pub mod base;
pub mod registry;
pub mod file_tools;
pub mod search_tools;
pub mod command_tools;
pub mod web_tools;
pub mod skill_tools;
pub mod message_tools;
pub mod task_tools;
pub mod plan_tools;
pub mod git_tools;
pub mod user_tools;
pub mod lsp_tools;
pub mod time_tools;
pub mod cron_tools;
pub mod team_tools;
pub mod tool_search;

// 重新导出主要类型
pub use types::{
    ToolMetadata, ToolResult, ToolUseContext, ToolInputSchema,
    ToolCategory, ToolPermissionLevel, ValidationResult, PermissionResult,
    PermissionMode, PermissionBehavior, ToolPermissionContext,
};
pub use base::{Tool, ToolBuilder};
pub use registry::{ToolRegistry, ToolManager, ToolLoader};
pub use permissions::{PermissionChecker, ModeChecker};
pub use file_tools::{FileReadTool, FileEditTool, FileWriteTool};
pub use search_tools::{GlobTool, GrepTool};
pub use command_tools::{BashTool, PowerShellTool};
pub use web_tools::{WebFetchTool, WebSearchTool};
pub use skill_tools::SkillTool;
pub use message_tools::SendMessageTool;
pub use task_tools::TaskCreateTool;
pub use plan_tools::{EnterPlanModeTool, ExitPlanModeTool};
pub use git_tools::EnterWorktreeTool;
pub use user_tools::AskUserQuestionTool;
pub use lsp_tools::LSPTool;
pub use time_tools::SleepTool;
pub use cron_tools::CronCreateTool;
pub use team_tools::TeamCreateTool;
pub use tool_search::ToolSearchTool;

use crate::error::Result;

/// 初始化工具系统
pub async fn init() -> Result<ToolManager> {
    let mut manager = ToolManager::new();
    
    // 注册核心工具加载器
    manager.add_loader(BuiltinToolLoader);
    
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
        
        // 注册网络工具
        registry.register(WebFetchTool::default()).await;
        registry.register(WebSearchTool::default()).await;
        
        // 注册系统工具
        registry.register(SkillTool).await;
        registry.register(SendMessageTool).await;
        registry.register(TaskCreateTool).await;
        registry.register(EnterPlanModeTool).await;
        registry.register(ExitPlanModeTool).await;
        registry.register(SleepTool).await;
        registry.register(CronCreateTool).await;
        registry.register(ToolSearchTool).await;
        
        // 注册Git工具
        registry.register(EnterWorktreeTool).await;
        
        // 注册用户交互工具
        registry.register(AskUserQuestionTool).await;
        
        // 注册开发工具
        registry.register(LSPTool).await;
        
        // 注册团队工具
        registry.register(TeamCreateTool).await;
        
        tracing::debug!("Loaded {} builtin tools", 21);
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "builtin"
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
        "WebFetch".to_string(),
        "WebSearch".to_string(),
        "Skill".to_string(),
        "SendMessage".to_string(),
        "TaskCreate".to_string(),
        "EnterPlanMode".to_string(),
        "ExitPlanMode".to_string(),
        "EnterWorktree".to_string(),
        "AskUserQuestion".to_string(),
        "LSP".to_string(),
        "Sleep".to_string(),
        "CronCreate".to_string(),
        "TeamCreate".to_string(),
        "ToolSearch".to_string(),
    ]
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
                "WebFetch".to_string(),
                "WebSearch".to_string(),
                "Skill".to_string(),
                "SendMessage".to_string(),
                "TaskCreate".to_string(),
                "EnterPlanMode".to_string(),
                "ExitPlanMode".to_string(),
                "EnterWorktree".to_string(),
                "AskUserQuestion".to_string(),
                "LSP".to_string(),
                "Sleep".to_string(),
                "CronCreate".to_string(),
                "TeamCreate".to_string(),
                "ToolSearch".to_string(),
            ],
            ToolPreset::Simple => vec![
                "Read".to_string(),
                "Glob".to_string(),
                "Grep".to_string(),
                "WebFetch".to_string(),
                "WebSearch".to_string(),
                "Skill".to_string(),
                "SendMessage".to_string(),
                "TaskCreate".to_string(),
                "EnterPlanMode".to_string(),
                "ExitPlanMode".to_string(),
                "EnterWorktree".to_string(),
                "AskUserQuestion".to_string(),
                "LSP".to_string(),
                "Sleep".to_string(),
                "CronCreate".to_string(),
                "TeamCreate".to_string(),
                "ToolSearch".to_string(),
            ],
            ToolPreset::Full => get_tool_names(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_init_tool_system() {
        let manager = init().await.unwrap();
        assert!(manager.registry().len().await >= 7);
    }
    
    #[tokio::test]
    async fn test_builtin_tools_loaded() {
        let manager = init().await.unwrap();
        
        assert!(manager.registry().has("Read").await);
        assert!(manager.registry().has("Edit").await);
        assert!(manager.registry().has("Write").await);
        assert!(manager.registry().has("Glob").await);
        assert!(manager.registry().has("Grep").await);
        assert!(manager.registry().has("Bash").await);
        assert!(manager.registry().has("PowerShell").await);
        assert!(manager.registry().has("WebFetch").await);
        assert!(manager.registry().has("WebSearch").await);
        assert!(manager.registry().has("Skill").await);
        assert!(manager.registry().has("SendMessage").await);
        assert!(manager.registry().has("TaskCreate").await);
        assert!(manager.registry().has("EnterPlanMode").await);
        assert!(manager.registry().has("ExitPlanMode").await);
        assert!(manager.registry().has("EnterWorktree").await);
        assert!(manager.registry().has("AskUserQuestion").await);
        assert!(manager.registry().has("LSP").await);
        assert!(manager.registry().has("Sleep").await);
        assert!(manager.registry().has("CronCreate").await);
        assert!(manager.registry().has("TeamCreate").await);
        assert!(manager.registry().has("ToolSearch").await);
    }
    
    #[tokio::test]
    async fn test_tool_aliases() {
        let manager = init().await.unwrap();
        
        assert!(manager.registry().has("read").await);
        assert!(manager.registry().has("cat").await);
        assert!(manager.registry().has("edit").await);
        assert!(manager.registry().has("bash").await);
        assert!(manager.registry().has("webfetch").await);
        assert!(manager.registry().has("fetch").await);
        assert!(manager.registry().has("websearch").await);
        assert!(manager.registry().has("search").await);
        assert!(manager.registry().has("skill").await);
        assert!(manager.registry().has("sendmessage").await);
        assert!(manager.registry().has("send").await);
        assert!(manager.registry().has("taskcreate").await);
        assert!(manager.registry().has("task").await);
        assert!(manager.registry().has("enterplanmode").await);
        assert!(manager.registry().has("plan").await);
        assert!(manager.registry().has("exitplanmode").await);
        assert!(manager.registry().has("exitplan").await);
        assert!(manager.registry().has("enterworktree").await);
        assert!(manager.registry().has("worktree").await);
        assert!(manager.registry().has("askuserquestion").await);
        assert!(manager.registry().has("ask").await);
        assert!(manager.registry().has("lsp").await);
        assert!(manager.registry().has("language").await);
        assert!(manager.registry().has("sleep").await);
        assert!(manager.registry().has("wait").await);
        assert!(manager.registry().has("croncreate").await);
        assert!(manager.registry().has("cron").await);
        assert!(manager.registry().has("teamcreate").await);
        assert!(manager.registry().has("team").await);
        assert!(manager.registry().has("toolsearch").await);
        assert!(manager.registry().has("tools").await);
        assert!(manager.registry().has("searchtools").await);
    }
    
    #[test]
    fn test_tool_preset() {
        let default = ToolPreset::Default;
        assert_eq!(default.tool_names().len(), 20);
        
        let simple = ToolPreset::Simple;
        assert_eq!(simple.tool_names().len(), 17);
        
        let full = ToolPreset::Full;
        assert!(full.tool_names().len() >= 21);
    }
}
