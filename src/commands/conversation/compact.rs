//! `/compact` 命令实现
//!
//! 压缩对话 (保留摘要)

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, PromptCommand, LoadedFrom,
    CommandSource, CommandResultDisplay,
};
use crate::commands::registry::CommandExecutor;
use super::{ConversationError, ConversationStorage, FileConversationStorage, ConversationMessage};

/// 压缩命令
pub struct CompactCommand;

#[async_trait::async_trait]
impl CommandExecutor for CompactCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        // 解析参数
        let args = parse_compact_args(&context.args);

        // 获取当前对话的消息（从上下文或存储）
        let messages = get_current_messages(&context).await?;

        if messages.is_empty() {
            return Ok(CommandResult {
                content: "No messages to compact.\n没有消息可以压缩。".to_string(),
                display: CommandResultDisplay::User,
                should_query: false,
                ..Default::default()
            });
        }

        // 构建压缩提示
        let compact_prompt = build_compact_prompt(&messages, &args);

        Ok(CommandResult {
            content: compact_prompt,
            display: CommandResultDisplay::User,
            should_query: true,  // 需要AI生成摘要
            ..Default::default()
        })
    }

    fn command(&self) -> Command {
        Command::Prompt(PromptCommand {
            base: CommandBase {
                name: "compact".to_string(),
                description: "Compact conversation while preserving summary".to_string(),
                aliases: Some(vec!["cmp".to_string()]),
                argument_hint: Some("[--keep-last N] [--strategy <strategy>]".to_string()),
                when_to_use: Some("Use when the conversation is getting too long and you want to preserve the key points".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
            progress_message: "Analyzing conversation and generating compact summary...".to_string(),
            content_length: 3000,
            arg_names: Some(vec!["keep_last".to_string(), "strategy".to_string()]),
            allowed_tools: Some(vec!["summarize".to_string()]),
            model: Some("claude-3-5-sonnet-20241022".to_string()),
            source: CommandSource::Builtin,
            plugin_info: None,
            disable_non_interactive: None,
            context: None,
            agent: None,
            effort: Some(crate::commands::types::EffortValue::High),
            paths: None,
        })
    }
}

/// 压缩参数
struct CompactArgs {
    keep_last: usize,
    strategy: CompactStrategy,
}

/// 压缩策略
#[derive(Debug, Clone, Copy)]
enum CompactStrategy {
    Summary,
    KeyPoints,
    FullContext,
}

impl CompactStrategy {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "summary" => CompactStrategy::Summary,
            "keypoints" | "key-points" => CompactStrategy::KeyPoints,
            "full" | "fullcontext" | "full-context" => CompactStrategy::FullContext,
            _ => CompactStrategy::Summary,
        }
    }

    fn description(&self) -> &'static str {
        match self {
            CompactStrategy::Summary => "Generate a concise summary of the conversation",
            CompactStrategy::KeyPoints => "Extract key decisions and action items only",
            CompactStrategy::FullContext => "Preserve full context with detailed summary",
        }
    }
}

/// 解析压缩参数
fn parse_compact_args(args: &str) -> CompactArgs {
    let mut keep_last = 3;  // 默认保留最后3条消息
    let mut strategy = CompactStrategy::Summary;

    let mut args_iter = args.split_whitespace().peekable();

    while let Some(arg) = args_iter.next() {
        match arg {
            "--keep-last" | "-k" => {
                if let Some(n) = args_iter.next() {
                    if let Ok(num) = n.parse() {
                        keep_last = num;
                    }
                }
            }
            "--strategy" | "-s" => {
                if let Some(s) = args_iter.next() {
                    strategy = CompactStrategy::from_str(s);
                }
            }
            _ => {}
        }
    }

    CompactArgs { keep_last, strategy }
}

/// 获取当前对话消息
async fn get_current_messages(_context: &CommandContext) -> Result<Vec<ConversationMessage>> {
    // TODO: 从实际对话状态获取消息
    // 这里返回模拟数据作为示例

    Ok(vec![
        ConversationMessage {
            id: "1".to_string(),
            role: "user".to_string(),
            content: "Hello, I need help with my Rust project.".to_string(),
            timestamp: chrono::Utc::now(),
        },
        ConversationMessage {
            id: "2".to_string(),
            role: "assistant".to_string(),
            content: "I'd be happy to help! What specific issue are you facing?".to_string(),
            timestamp: chrono::Utc::now(),
        },
        ConversationMessage {
            id: "3".to_string(),
            role: "user".to_string(),
            content: "I'm trying to implement async/await but getting lifetime errors.".to_string(),
            timestamp: chrono::Utc::now(),
        },
    ])
}

/// 构建压缩提示
fn build_compact_prompt(messages: &[ConversationMessage], args: &CompactArgs) -> String {
    let mut prompt = String::new();

    prompt.push_str("# 🗜️ 对话压缩 | Conversation Compact\n\n");
    prompt.push_str(&format!("**策略 | Strategy**: {:?}\n", args.strategy));
    prompt.push_str(&format!("**保留消息数 | Keep Last**: {}\n", args.keep_last));
    prompt.push_str(&format!("**总消息数 | Total Messages**: {}\n\n", messages.len()));

    // 显示要压缩的消息（除了最后N条）
    let messages_to_compact = if messages.len() > args.keep_last {
        &messages[..messages.len() - args.keep_last]
    } else {
        &[]
    };

    if messages_to_compact.is_empty() {
        prompt.push_str("⚠️ 消息数量较少，无需压缩。\n");
        prompt.push_str("   Too few messages to compact.\n");
        return prompt;
    }

    prompt.push_str("## 待压缩内容 | Content to Compact\n\n");

    for msg in messages_to_compact {
        let role_emoji = match msg.role.as_str() {
            "user" => "👤",
            "assistant" => "🤖",
            "system" => "⚙️",
            _ => "💬",
        };

        prompt.push_str(&format!("{} **{}** ({}):\n",
            role_emoji,
            msg.role,
            msg.timestamp.format("%H:%M")
        ));
        prompt.push_str(&format!("{}\n\n", msg.content));
    }

    // 显示将保留的消息
    if args.keep_last > 0 && messages.len() > args.keep_last {
        prompt.push_str("## 将保留的消息 | Messages to Keep\n\n");

        let kept_messages = &messages[messages.len() - args.keep_last..];
        for msg in kept_messages {
            let role_emoji = match msg.role.as_str() {
                "user" => "👤",
                "assistant" => "🤖",
                "system" => "⚙️",
                _ => "💬",
            };

            prompt.push_str(&format!("{} **{}**: {}\n",
                role_emoji,
                msg.role,
                if msg.content.len() > 50 {
                    format!("{}...", &msg.content[..50])
                } else {
                    msg.content.clone()
                }
            ));
        }
        prompt.push('\n');
    }

    // 压缩指令
    prompt.push_str("## 压缩指令 | Compact Instructions\n\n");
    prompt.push_str(&format!("Strategy: {}\n\n", args.strategy.description()));

    prompt.push_str("Please generate a compact summary that:\n");
    prompt.push_str("1. **Preserves key context** - Keep important facts and decisions\n");
    prompt.push_str("2. **Maintains continuity** - Ensure the summary flows naturally\n");
    prompt.push_str("3. **Highlights action items** - Note any pending tasks or decisions\n");
    prompt.push_str("4. **Captures intent** - Preserve the user's goals and requirements\n\n");

    prompt.push_str("Output format:\n");
    prompt.push_str("```\n");
    prompt.push_str("## 对话摘要 | Conversation Summary\n\n");
    prompt.push_str("[Concise summary of the discussion]\n\n");
    prompt.push_str("### 关键点 | Key Points\n");
    prompt.push_str("- Point 1\n");
    prompt.push_str("- Point 2\n\n");
    prompt.push_str("### 待办事项 | Action Items\n");
    prompt.push_str("- [ ] Task 1\n");
    prompt.push_str("- [ ] Task 2\n");
    prompt.push_str("```\n");

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compact_args() {
        let args = parse_compact_args("--keep-last 5 --strategy keypoints");
        assert_eq!(args.keep_last, 5);
        match args.strategy {
            CompactStrategy::KeyPoints => (),
            _ => panic!("Expected KeyPoints strategy"),
        }

        let args = parse_compact_args("");
        assert_eq!(args.keep_last, 3);
        match args.strategy {
            CompactStrategy::Summary => (),
            _ => panic!("Expected Summary strategy by default"),
        }
    }

    #[test]
    fn test_compact_strategy_from_str() {
        assert!(matches!(
            CompactStrategy::from_str("summary"),
            CompactStrategy::Summary
        ));
        assert!(matches!(
            CompactStrategy::from_str("key-points"),
            CompactStrategy::KeyPoints
        ));
        assert!(matches!(
            CompactStrategy::from_str("full"),
            CompactStrategy::FullContext
        ));
    }
}