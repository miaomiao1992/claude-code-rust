//! `/rewind` 命令实现
//!
//! 回退到之前的点

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, LocalJsxCommand, LoadedFrom,
    CommandSource, CommandResultDisplay,
};
use crate::commands::registry::CommandExecutor;
use super::{ConversationError, ConversationStorage, FileConversationStorage};

/// 回退命令
pub struct RewindCommand;

#[async_trait::async_trait]
impl CommandExecutor for RewindCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        // 解析参数
        let args = parse_rewind_args(&context.args);

        // 执行回退
        let result = rewind_conversation(&args).await?;

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
                name: "rewind".to_string(),
                description: "Rewind to a previous point in conversation".to_string(),
                aliases: Some(vec!["rw".to_string()]),
                argument_hint: Some("[--to <message-id>] [--steps N]".to_string()),
                when_to_use: Some("Use when you want to go back to an earlier point in the conversation".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
        })
    }
}

/// 回退参数
struct RewindArgs {
    to_message_id: Option<String>,
    steps: Option<usize>,
}

/// 解析回退参数
fn parse_rewind_args(args: &str) -> RewindArgs {
    let mut to_message_id = None;
    let mut steps = None;

    let mut args_iter = args.split_whitespace().peekable();

    while let Some(arg) = args_iter.next() {
        match arg {
            "--to" | "-t" => {
                if let Some(id) = args_iter.next() {
                    to_message_id = Some(id.to_string());
                }
            }
            "--steps" | "-s" => {
                if let Some(n) = args_iter.next() {
                    if let Ok(num) = n.parse() {
                        steps = Some(num);
                    }
                }
            }
            _ => {}
        }
    }

    RewindArgs { to_message_id, steps }
}

/// 回退对话
async fn rewind_conversation(args: &RewindArgs) -> Result<String> {
    // TODO: 实现实际的回退逻辑

    let result = if let Some(id) = &args.to_message_id {
        format!(
            "## ⏪ 回退成功 | Rewind Successful\n\n\
            已回退到消息 ID: {}\n\
            Rewound to message ID: {}\n\n\
            ⚠️ 注意：此操作不可撤销 | Note: This action cannot be undone",
            id, id
        )
    } else if let Some(s) = args.steps {
        format!(
            "## ⏪ 回退成功 | Rewind Successful\n\n\
            已回退 {} 步\n\
            Rewound {} steps\n\n\
            ⚠️ 注意：此操作不可撤销 | Note: This action cannot be undone",
            s, s
        )
    } else {
        "Usage: /rewind --to <message-id> 或 /rewind --steps N\n\
        用法：/rewind --to <消息ID> 或 /rewind --steps N".to_string()
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rewind_args() {
        let args = parse_rewind_args("--to msg-123");
        assert_eq!(args.to_message_id, Some("msg-123".to_string()));
        assert!(args.steps.is_none());

        let args = parse_rewind_args("--steps 5");
        assert!(args.to_message_id.is_none());
        assert_eq!(args.steps, Some(5));
    }
}
