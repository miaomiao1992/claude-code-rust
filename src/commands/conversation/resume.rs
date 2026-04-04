//! `/resume` 命令实现
//!
//! 恢复之前的对话

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, LocalJsxCommand, LoadedFrom,
    CommandSource, CommandResultDisplay,
};
use crate::commands::registry::CommandExecutor;
use super::{ConversationError, ConversationStorage, FileConversationStorage};

/// 恢复命令
pub struct ResumeCommand;

#[async_trait::async_trait]
impl CommandExecutor for ResumeCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        // 解析参数
        let args = parse_resume_args(&context.args);

        // 获取存储
        let storage = get_conversation_storage();

        // 列出可用对话
        let conversations = storage.list()?;

        if conversations.is_empty() {
            return Ok(CommandResult {
                content: "No saved conversations found.\n没有找到保存的对话。\n\nStart a new conversation by typing your message.\n输入消息开始新对话。".to_string(),
                display: CommandResultDisplay::User,
                should_query: false,
                ..Default::default()
            });
        }

        // 如果有指定对话ID或名称，尝试加载
        let selected_conversation = if let Some(query) = args.query {
            find_conversation(&conversations, &query)
        } else {
            // 默认选择最近的对话
            conversations.first().cloned()
        };

        match selected_conversation {
            Some(info) => {
                // 加载对话
                let (_, messages) = storage.load(&info.id)?;

                let result = format_resume_result(&info, &messages, &args
                );

                Ok(CommandResult {
                    content: result,
                    display: CommandResultDisplay::User,
                    should_query: false,
                    ..Default::default()
                })
            }
            None => {
                // 显示可用对话列表
                let list = format_conversation_list(&conversations
                );

                Ok(CommandResult {
                    content: list,
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
                name: "resume".to_string(),
                description: "Resume a previous conversation".to_string(),
                aliases: Some(vec!["r".to_string()]),
                argument_hint: Some("[conversation-id-or-name]".to_string()),
                when_to_use: Some("Use when you want to continue a previous conversation".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
        })
    }
}

/// 恢复参数
struct ResumeArgs {
    query: Option<String>,
    list_only: bool,
}

/// 解析恢复参数
fn parse_resume_args(args: &str) -> ResumeArgs {
    let trimmed = args.trim();

    if trimmed.is_empty() {
        ResumeArgs {
            query: None,
            list_only: false,
        }
    } else if trimmed == "--list" || trimmed == "-l" {
        ResumeArgs {
            query: None,
            list_only: true,
        }
    } else {
        ResumeArgs {
            query: Some(trimmed.to_string()),
            list_only: false,
        }
    }
}

/// 获取对话存储
fn get_conversation_storage() -> impl ConversationStorage {
    let storage_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("claude-code")
        .join("conversations");

    FileConversationStorage::new(storage_dir)
}

/// 查找对话
fn find_conversation(
    conversations: &[super::ConversationInfo],
    query: &str,
) -> Option<super::ConversationInfo> {
    // 先尝试匹配ID
    for conv in conversations {
        if conv.id == query {
            return Some(conv.clone());
        }
    }

    // 再尝试匹配名称（不区分大小写）
    let query_lower = query.to_lowercase();
    for conv in conversations {
        if conv.name.to_lowercase().contains(&query_lower) {
            return Some(conv.clone());
        }
    }

    None
}

/// 格式化恢复结果
fn format_resume_result(
    info: &super::ConversationInfo,
    messages: &[super::ConversationMessage],
    _args: &ResumeArgs,
) -> String {
    let mut result = String::new();

    result.push_str("## 🔄 已恢复对话 | Conversation Resumed\n\n");
    result.push_str(&format!("**名称 | Name**: {}\n", info.name));
    result.push_str(&format!("**ID**: {}\n", info.id));
    result.push_str(&format!(
        "**更新时间 | Updated**: {}\n",
        info.updated_at.format("%Y-%m-%d %H:%M:%S")
    ));
    result.push_str(&format!(
        "**消息数 | Messages**: {}\n",
        messages.len()
    ));
    result.push_str(&format!("**Token数 | Tokens**: {}\n", info.token_count));

    result.push_str("\n### 最后几条消息 | Recent Messages\n\n");

    // 显示最后几条消息
    let recent_messages = messages.iter().rev().take(5).collect::<Vec<_>();
    for msg in recent_messages.iter().rev() {
        let role_emoji = match msg.role.as_str() {
            "user" => "👤",
            "assistant" => "🤖",
            "system" => "⚙️",
            _ => "💬",
        };

        let content_preview = if msg.content.len() > 100 {
            format!("{}...", &msg.content[..100])
        } else {
            msg.content.clone()
        };

        result.push_str(&format!(
            "{} **{}**: {}\n\n",
            role_emoji,
            msg.role,
            content_preview
        ));
    }

    result.push_str("\n💡 您可以继续对话了 | You can continue the conversation now.\n");

    result
}

/// 格式化对话列表
fn format_conversation_list(conversations: &[super::ConversationInfo]) -> String {
    let mut result = String::new();

    result.push_str("## 📋 可用对话 | Available Conversations\n\n");
    result.push_str("Use `/resume <ID>` or `/resume <Name>` to resume a conversation.\n\n");

    for (i, conv) in conversations.iter().enumerate().take(10) {
        result.push_str(&format!(
            "{}. **{}**\n",
            i + 1,
            conv.name
        ));
        result.push_str(&format!("   ID: `{}`\n", conv.id));
        result.push_str(&format!(
            "   Updated: {} | Messages: {}\n\n",
            conv.updated_at.format("%Y-%m-%d %H:%M"),
            conv.message_count
        ));
    }

    if conversations.len() > 10 {
        result.push_str(&format!(
            "... and {} more conversations\n",
            conversations.len() - 10
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resume_args() {
        let args = parse_resume_args("");
        assert!(args.query.is_none());
        assert!(!args.list_only);

        let args = parse_resume_args("my-conversation");
        assert_eq!(args.query, Some("my-conversation".to_string()));

        let args = parse_resume_args("--list");
        assert!(args.list_only);
    }

    #[test]
    fn test_find_conversation() {
        let conversations = vec![
            super::super::ConversationInfo {
                id: "conv-1".to_string(),
                name: "Test Conversation".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                message_count: 10,
                token_count: 500,
            },
        ];

        let found = find_conversation(&conversations, "conv-1");
        assert!(found.is_some());

        let found = find_conversation(&conversations, "Test");
        assert!(found.is_some());

        let found = find_conversation(&conversations, "nonexistent");
        assert!(found.is_none());
    }
}