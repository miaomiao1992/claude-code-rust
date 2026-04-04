//! `/fast` 命令实现
//!
//! 切换 fast mode

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, LocalJsxCommand, LoadedFrom,
    CommandResultDisplay,
};
use crate::commands::registry::CommandExecutor;

/// Fast命令
pub struct FastCommand;

#[async_trait::async_trait]
impl CommandExecutor for FastCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        let args = context.args.trim().to_lowercase();

        let (enabled, message) = if args.is_empty() || args == "toggle" {
            // 切换状态
            (true, "Fast mode 已切换 | Fast mode toggled".to_string())
        } else if args == "on" || args == "true" || args == "enable" {
            (true, "Fast mode 已启用 | Fast mode enabled".to_string())
        } else if args == "off" || args == "false" || args == "disable" {
            (false, "Fast mode 已禁用 | Fast mode disabled".to_string())
        } else {
            (true, format!("Fast mode: {}", args))
        };

        let result = format!(
            "## ⚡ {}\n\n\
            **Status**: {}\n\n\
            {}\n\n\
            在 fast mode 下，响应速度更快但可能牺牲一些质量。\n\
            In fast mode, responses are quicker but may sacrifice some quality.",
            if enabled { "Fast Mode Enabled" } else { "Fast Mode Disabled" },
            if enabled { "ON ✅" } else { "OFF ❌" },
            message
        );

        Ok(CommandResult {
            content: result,
            display: CommandResultDisplay::User,
            should_query: false,
            ..Default::default()
        })
    }

    fn command(&self) -> Command {
        Command::LocalJsx(LocalJsxCommand {
            base: CommandBase {
                name: "fast".to_string(),
                description: "Toggle fast mode".to_string(),
                aliases: Some(vec!["f".to_string()]),
                argument_hint: Some("[on|off|toggle]".to_string()),
                when_to_use: Some("Use when you want faster responses at the cost of some quality".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
        })
    }
}
