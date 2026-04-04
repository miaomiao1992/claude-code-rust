//! `/export` 命令实现
//!
//! 导出对话

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, LocalJsxCommand, LoadedFrom,
    CommandSource, CommandResultDisplay,
};
use crate::commands::registry::CommandExecutor;
use super::{ConversationError, ConversationStorage, FileConversationStorage};

/// 导出命令
pub struct ExportCommand;

#[async_trait::async_trait]
impl CommandExecutor for ExportCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        // 解析参数
        let args = parse_export_args(&context.args);

        // 获取当前对话消息
        let messages = get_current_messages(&context).await?;

        if messages.is_empty() {
            return Ok(CommandResult {
                content: "No messages to export.\n没有消息可以导出。".to_string(),
                display: CommandResultDisplay::User,
                should_query: false,
                ..Default::default()
            });
        }

        // 执行导出
        let export_result = export_conversation(
            &messages, &args, &context
        ).await?;

        Ok(CommandResult {
            content: export_result,
            display: CommandResultDisplay::User,
            should_query: false,
            ..Default::default()
        })
    }

    fn command(&self) -> Command {
        Command::LocalJsx(LocalJsxCommand {
            base: CommandBase {
                name: "export".to_string(),
                description: "Export conversation to file".to_string(),
                aliases: Some(vec!["exp".to_string()]),
                argument_hint: Some("[--format <format>] [--output <path>]".to_string()),
                when_to_use: Some("Use when you want to save the conversation to a file".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
        })
    }
}

/// 导出参数
struct ExportArgs {
    format: ExportFormat,
    output_path: Option<String>,
    include_metadata: bool,
}

/// 导出格式
#[derive(Debug, Clone, Copy)]
enum ExportFormat {
    Markdown,
    Json,
    Text,
    Html,
}

impl ExportFormat {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => ExportFormat::Markdown,
            "json" => ExportFormat::Json,
            "text" | "txt" => ExportFormat::Text,
            "html" | "htm" => ExportFormat::Html,
            _ => ExportFormat::Markdown,
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Markdown => "md",
            ExportFormat::Json => "json",
            ExportFormat::Text => "txt",
            ExportFormat::Html => "html",
        }
    }

    fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::Markdown => "text/markdown",
            ExportFormat::Json => "application/json",
            ExportFormat::Text => "text/plain",
            ExportFormat::Html => "text/html",
        }
    }
}

/// 解析导出参数
fn parse_export_args(args: &str) -> ExportArgs {
    let mut format = ExportFormat::Markdown;
    let mut output_path = None;
    let mut include_metadata = true;

    let mut args_iter = args.split_whitespace().peekable();

    while let Some(arg) = args_iter.next() {
        match arg {
            "--format" | "-f" => {
                if let Some(f) = args_iter.next() {
                    format = ExportFormat::from_str(f);
                }
            }
            "--output" | "-o" => {
                if let Some(path) = args_iter.next() {
                    output_path = Some(path.to_string());
                }
            }
            "--no-metadata" => {
                include_metadata = false;
            }
            _ => {}
        }
    }

    ExportArgs {
        format,
        output_path,
        include_metadata,
    }
}

/// 当前对话消息
async fn get_current_messages(_context: &CommandContext) -> Result<Vec<super::ConversationMessage>> {
    // TODO: 从实际对话状态获取
    Ok(vec![
        super::ConversationMessage {
            id: "1".to_string(),
            role: "user".to_string(),
            content: "Hello!".to_string(),
            timestamp: chrono::Utc::now(),
        },
        super::ConversationMessage {
            id: "2".to_string(),
            role: "assistant".to_string(),
            content: "Hi there! How can I help?".to_string(),
            timestamp: chrono::Utc::now(),
        },
    ])
}

/// 导出对话
async fn export_conversation(
    messages: &[super::ConversationMessage],
    args: &ExportArgs,
    context: &CommandContext,
) -> Result<String> {
    // 生成导出内容
    let content = match args.format {
        ExportFormat::Markdown => export_as_markdown(messages, args),
        ExportFormat::Json => export_as_json(messages, args),
        ExportFormat::Text => export_as_text(messages, args),
        ExportFormat::Html => export_as_html(messages, args),
    };

    // 确定输出路径
    let output_path = if let Some(path) = &args.output_path {
        std::path::PathBuf::from(path)
    } else {
        generate_default_output_path(&context.cwd, args.format)
    };

    // 写入文件
    std::fs::write(&output_path, content)?;

    // 格式化结果
    let result = format!(
        "## 📤 导出成功 | Export Successful\n\n\
        **格式 | Format**: {:?}\n\
        **文件路径 | Path**: `{}`\n\
        **消息数 | Messages**: {}\n\
        **文件大小 | Size**: {} bytes\n\n\
        文件已保存！File saved successfully!",
        args.format,
        output_path.display(),
        messages.len(),
        std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0)
    );

    Ok(result)
}

/// 生成默认输出路径
fn generate_default_output_path(base_dir: &std::path::Path, format: ExportFormat) -> std::path::PathBuf {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("claude_conversation_{}.{}", timestamp, format.extension());
    base_dir.join(filename)
}

/// 导出为Markdown
fn export_as_markdown(
    messages: &[super::ConversationMessage],
    args: &ExportArgs,
) -> String {
    let mut output = String::new();

    // 元数据
    if args.include_metadata {
        output.push_str("---\n");
        output.push_str(&format!("title: Claude Code Conversation\n"));
        output.push_str(&format!("date: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        output.push_str(&format!("messages: {}\n", messages.len()));
        output.push_str("---\n\n");
    }

    output.push_str("# Claude Code Conversation\n\n");

    for msg in messages {
        let role_title = match msg.role.as_str() {
            "user" => "👤 User",
            "assistant" => "🤖 Claude",
            "system" => "⚙️ System",
            _ => &msg.role,
        };

        output.push_str(&format!("## {} ({})\n\n", role_title, msg.timestamp.format("%H:%M:%S")));
        output.push_str(&format!("{}\n\n", msg.content));
    }

    output
}

/// 导出为JSON
fn export_as_json(
    messages: &[super::ConversationMessage],
    args: &ExportArgs,
) -> String {
    let data = if args.include_metadata {
        serde_json::json!({
            "metadata": {
                "exported_at": chrono::Utc::now().to_rfc3339(),
                "message_count": messages.len(),
                "version": env!("CARGO_PKG_VERSION"),
            },
            "messages": messages.iter().map(|m| serde_json::json!({
                "id": m.id,
                "role": m.role,
                "content": m.content,
                "timestamp": m.timestamp.to_rfc3339(),
            })).collect::<Vec<_>(),
        })
    } else {
        serde_json::json!(messages.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": m.content,
        })).collect::<Vec<_>())
    };

    serde_json::to_string_pretty(&data).unwrap_or_default()
}

/// 导出为纯文本
fn export_as_text(
    messages: &[super::ConversationMessage],
    args: &ExportArgs,
) -> String {
    let mut output = String::new();

    if args.include_metadata {
        output.push_str("Claude Code Conversation\n");
        output.push_str("========================\n\n");
        output.push_str(&format!("Date: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        output.push_str(&format!("Messages: {}\n\n", messages.len()));
        output.push_str("========================\n\n");
    }

    for msg in messages {
        let role_label = match msg.role.as_str() {
            "user" => "USER",
            "assistant" => "CLAUDE",
            "system" => "SYSTEM",
            _ => &msg.role.to_uppercase(),
        };

        output.push_str(&format!("[{} - {}]\n", role_label, msg.timestamp.format("%H:%M:%S")));
        output.push_str(&format!("{}\n", msg.content));
        output.push_str("\n");
    }

    output
}

/// 导出为HTML
fn export_as_html(
    messages: &[super::ConversationMessage],
    args: &ExportArgs,
) -> String {
    let mut output = String::new();

    output.push_str(&format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Claude Code Conversation</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; background: #f5f5f5; }}
        .message {{ margin: 20px 0; padding: 15px; border-radius: 10px; }}
        .user {{ background: #007bff; color: white; margin-left: 50px; }}
        .assistant {{ background: white; border: 1px solid #ddd; margin-right: 50px; }}
        .system {{ background: #e9ecef; text-align: center; font-style: italic; }}
        .role {{ font-weight: bold; margin-bottom: 5px; font-size: 14px; }}
        .timestamp {{ font-size: 12px; opacity: 0.7; margin-top: 10px; }}
        .content {{ line-height: 1.6; }}
        pre {{ background: #f4f4f4; padding: 10px; border-radius: 5px; overflow-x: auto; }}
        code {{ background: #f4f4f4; padding: 2px 5px; border-radius: 3px; }}
    </style>
</head>
<body>
    <h1>Claude Code Conversation</h1>
"#));

    if args.include_metadata {
        output.push_str(&format!(
            "    <p><strong>Exported:</strong> {} | <strong>Messages:</strong> {}</p>\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            messages.len()
        ));
    }

    for msg in messages {
        let css_class = match msg.role.as_str() {
            "user" => "user",
            "assistant" => "assistant",
            _ => "system",
        };

        let role_display = match msg.role.as_str() {
            "user" => "You",
            "assistant" => "Claude",
            _ => "System",
        };

        output.push_str(&format!(
            r#"    <div class="message {}">
        <div class="role">{}</div>
        <div class="content">{}    </div>
        <div class="timestamp">{}</div>
    </div>
"#,
            css_class,
            role_display,
            html_escape(&msg.content),
            msg.timestamp.format("%Y-%m-%d %H:%M:%S")
        ));
    }

    output.push_str(&format!(r#"</body>
</html>"#));

    output
}

/// HTML转义
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_export_args() {
        let args = parse_export_args("--format json --output test.json");
        assert!(matches!(args.format, ExportFormat::Json));
        assert_eq!(args.output_path, Some("test.json".to_string()));
        assert!(args.include_metadata);

        let args = parse_export_args("--no-metadata");
        assert!(!args.include_metadata);
    }

    #[test]
    fn test_export_format_from_str() {
        assert!(matches!(ExportFormat::from_str("md"), ExportFormat::Markdown));
        assert!(matches!(ExportFormat::from_str("json"), ExportFormat::Json));
        assert!(matches!(ExportFormat::from_str("txt"), ExportFormat::Text));
        assert!(matches!(ExportFormat::from_str("html"), ExportFormat::Html));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<test>"), "&lt;test&gt;");
        assert_eq!(html_escape("\"hello\""), "&quot;hello&quot;");
    }
}