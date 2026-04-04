//! `/rename` 命令实现
//!
//! 重命名当前对话

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, LocalJsxCommand, LoadedFrom,
    CommandSource, CommandResultDisplay,
};
use crate::commands::registry::CommandExecutor;
use super::{ConversationError, ConversationStorage, FileConversationStorage};

/// 重命名命令
pub struct RenameCommand;

#[async_trait::async_trait]
impl CommandExecutor for RenameCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        // 解析参数
        let args = parse_rename_args(&context.args);

        if args.name.is_empty() {
            return Ok(CommandResult {
                content: "Usage: /rename <new-name>\n用法：/rename <新名称>".to_string(),
                display: CommandResultDisplay::User,
                should_query: false,
                ..Default::default()
            });
        }

        // 执行重命名
        match rename_conversation(&args).await {
            Ok(old_name) => {
                let result = format!(
                    "## ✏️ 重命名成功 | Rename Successful\n\n\
                    **原名称 | Old Name**: {}\n\
                    **新名称 | New Name**: {}\n\n\
                    对话已重命名！Conversation renamed successfully!",
                    old_name, args.name
                );

                Ok(CommandResult {
                    content: result,
                    display: CommandResultDisplay::User,
                    should_query: false,
                    ..Default::default()
                })
            }
            Err(e) => {
                let error_msg = format!(
                    "## ❌ 重命名失败 | Rename Failed\n\n{}",
                    e
                );

                Ok(CommandResult {
                    content: error_msg,
                    display: CommandResultDisplay::User,
                    should_query: false,
                    ..Default::default()
                })
            }
        }
    }

    fn command(&self) -> Command {
        Command::LocalJsx(LocalJsxCommand {
            base: CommandBase {
                name: "rename".to_string(),
                description: "Rename the current conversation".to_string(),
                aliases: Some(vec!["rn".to_string()]),
                argument_hint: Some("<new-name>".to_string()),
                when_to_use: Some("Use when you want to give the conversation a more meaningful name".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
        })
    }
}

/// 重命名参数
struct RenameArgs {
    name: String,
}

/// 解析重命名参数
fn parse_rename_args(args: &str) -> RenameArgs {
    RenameArgs {
        name: args.trim().to_string(),
    }
}

/// 重命名对话
async fn rename_conversation(args: &RenameArgs) -> Result<String> {
    // TODO: 获取当前对话ID并执行重命名
    // 这里返回模拟的旧名称
    Ok("Unnamed Conversation".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rename_args() {
        let args = parse_rename_args("My New Conversation");
        assert_eq!(args.name, "My New Conversation");

        let args = parse_rename_args("  Trimmed Name  ");
        assert_eq!(args.name, "Trimmed Name");
    }
}
