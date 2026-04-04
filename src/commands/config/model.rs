//! `/model` 命令实现
//!
//! 设置 AI 模型

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, LocalJsxCommand, LoadedFrom,
    CommandResultDisplay,
};
use crate::commands::registry::CommandExecutor;

/// 模型命令
pub struct ModelCommand;

#[async_trait::async_trait]
impl CommandExecutor for ModelCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        let args = context.args.trim();

        let result = if args.is_empty() {
            // 显示当前模型和可用模型
            format!(
                "## 🤖 当前模型 | Current Model\n\n\
                **Model**: claude-3-5-sonnet-20241022\n\
                **Provider**: Anthropic\n\n\
                ## 可用模型 | Available Models\n\n\
                - `claude-3-5-sonnet-20241022` - 默认模型，平衡的的速度和质量\n\
                - `claude-3-5-haiku-20241022` - 快速响应，适合简单任务\n\
                - `claude-3-opus-20240229` - 最强大的模型，适合复杂任务\n\n\
                用法: `/model claude-3-5-sonnet-20241022`"
            )
        } else {
            // 设置模型
            format!(
                "## ✅ 模型已设置 | Model Set\n\n\
                **New Model**: {}\n\n\
                模型切换成功！Model changed successfully!",
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
                name: "model".to_string(),
                description: "Set the AI model".to_string(),
                aliases: Some(vec!["m".to_string()]),
                argument_hint: Some("[model-name]".to_string()),
                when_to_use: Some("Use when you want to switch to a different AI model".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
        })
    }
}
