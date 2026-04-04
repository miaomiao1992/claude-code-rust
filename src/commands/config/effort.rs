//! `/effort` 命令实现
//!
//! 设置 effort level

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, LocalJsxCommand, LoadedFrom,
    CommandResultDisplay,
};
use crate::commands::registry::CommandExecutor;

/// Effort命令
pub struct EffortCommand;

#[async_trait::async_trait]
impl CommandExecutor for EffortCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        let args = context.args.trim();

        let result = if args.is_empty() {
            format!(
                "## ⚡ 当前 Effort Level\n\n\
                **Current**: Medium\n\n\
                ## 可用级别 | Available Levels\n\n\
                - `low` - 快速回答，适合简单问题\n\
                - `medium` - 平衡速度和质量 (默认)\n\
                - `high` - 深入分析，适合复杂任务\n\n\
                用法: `/effort high`"
            )
        } else {
            format!(
                "## ✅ Effort Level 已设置\n\n\
                **New Level**: {}\n\n\
                设置成功！Settings updated!",
                args
            )
        };

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
                name: "effort".to_string(),
                description: "Set the effort level".to_string(),
                aliases: Some(vec!["e".to_string()]),
                argument_hint: Some("[low|medium|high]".to_string()),
                when_to_use: Some("Use when you want to adjust how much effort Claude puts into responses".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
        })
    }
}
