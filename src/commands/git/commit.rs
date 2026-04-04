//! `/commit` 命令实现
//!
//! 创建git提交，自动生成提交消息

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, PromptCommand, LoadedFrom, CommandSource,
};
use crate::commands::registry::CommandExecutor;
use super::GitError;

/// 提交命令
pub struct CommitCommand;

#[async_trait::async_trait]
impl CommandExecutor for CommitCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        // 解析参数
        let args = parse_commit_args(&context.args);

        // 检查Git仓库
        super::ensure_git_repository(&context.cwd)?;

        // 获取变更
        let changes = get_git_changes(&context.cwd, &args.files).await?;

        if changes.is_empty() && args.files.is_empty() {
            return Err(GitError::NoChangesToCommit.into());
        }

        // 生成或获取提交消息
        let commit_message = if let Some(msg) = args.message {
            msg
        } else {
            generate_commit_message(&changes, &context).await?
        };

        // 执行提交
        let result = execute_git_commit(&context.cwd, &commit_message, &args.files).await?;

        // 返回结果
        Ok(CommandResult {
            content: format!("✅ Commit created: {}\n\n{}", result.commit_hash, commit_message),
            should_query: false,
            ..Default::default()
        })
    }

    fn command(&self) -> Command {
        Command::Prompt(PromptCommand {
            base: CommandBase {
                name: "commit".to_string(),
                description: "Create a git commit with AI-generated message".to_string(),
                aliases: Some(vec!["c".to_string()]),
                argument_hint: Some("[files] [--message <msg>]".to_string()),
                when_to_use: Some("Use when you want to commit changes with a well-written commit message".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
            progress_message: "Analyzing changes and generating commit message...".to_string(),
            content_length: 2000,
            arg_names: Some(vec!["files".to_string(), "message".to_string()]),
            allowed_tools: Some(vec!["git".to_string(), "read_file".to_string()]),
            model: Some("claude-3-5-sonnet-20241022".to_string()),
            source: CommandSource::Builtin,
            plugin_info: None,
            disable_non_interactive: None,
            context: None,
            agent: None,
            effort: None,
            paths: None,
        })
    }
}

/// 提交参数
struct CommitArgs {
    files: Vec<String>,
    message: Option<String>,
}

/// 解析提交命令参数
fn parse_commit_args(args: &str) -> CommitArgs {
    let mut files = Vec::new();
    let mut message = None;
    let mut args_iter = args.split_whitespace().peekable();

    while let Some(arg) = args_iter.next() {
        if arg == "--message" || arg == "-m" {
            if let Some(msg) = args_iter.next() {
                message = Some(msg.to_string());
            }
        } else {
            files.push(arg.to_string());
        }
    }

    CommitArgs { files, message }
}

/// 获取Git变更
async fn get_git_changes(cwd: &std::path::Path, files: &[String]) -> Result<String> {
    if files.is_empty() {
        // 获取所有暂存区变更
        super::execute_git_command(cwd, &["diff", "--staged", "--stat"])
            .or_else(|_| super::execute_git_command(cwd, &["status", "--porcelain"]))
    } else {
        // 获取指定文件的变更
        let mut args = vec!["diff", "--"];
        args.extend(files.iter().map(|s| s.as_str()));
        super::execute_git_command(cwd, &args)
    }
}

/// 生成提交消息
async fn generate_commit_message(changes: &str, context: &CommandContext) -> Result<String> {
    // TODO: 集成AI模型生成提交消息
    // 目前使用简单启发式方法
    let message = if changes.contains("feat:") || changes.contains("feature") {
        "feat: add new feature".to_string()
    } else if changes.contains("fix:") || changes.contains("bug") {
        "fix: resolve issue".to_string()
    } else if changes.contains("docs:") {
        "docs: update documentation".to_string()
    } else if changes.contains("refactor:") {
        "refactor: improve code structure".to_string()
    } else {
        "chore: update code".to_string()
    };

    Ok(message)
}

/// 执行Git提交
async fn execute_git_commit(
    cwd: &std::path::Path,
    message: &str,
    files: &[String],
) -> Result<CommitResult> {
    // 如果有指定文件，先添加到暂存区
    if !files.is_empty() {
        let mut add_args = vec!["add"];
        add_args.extend(files.iter().map(|s| s.as_str()));
        super::execute_git_command(cwd, &add_args)?;
    }

    // 执行提交
    let commit_output = super::execute_git_command(cwd, &["commit", "-m", message])?;

    // 提取提交哈希
    let commit_hash = extract_commit_hash(&commit_output);

    Ok(CommitResult {
        commit_hash,
        message: message.to_string(),
    })
}

/// 提交结果
struct CommitResult {
    commit_hash: String,
    message: String,
}

/// 从Git输出中提取提交哈希
fn extract_commit_hash(output: &str) -> String {
    for line in output.lines() {
        if line.starts_with("[") && line.contains("]") {
            if let Some(start) = line.find(' ') {
                let hash_part = &line[start + 1..];
                if let Some(end) = hash_part.find(' ') {
                    return hash_part[..end].to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commit_args() {
        let args = parse_commit_args("file1.rs file2.rs --message \"feat: add feature\"");
        assert_eq!(args.files, vec!["file1.rs", "file2.rs"]);
        assert_eq!(args.message, Some("feat: add feature".to_string()));

        let args = parse_commit_args("-m \"fix: bug\"");
        assert!(args.files.is_empty());
        assert_eq!(args.message, Some("fix: bug".to_string()));
    }

    #[test]
    fn test_extract_commit_hash() {
        let output = "[main abc1234] feat: add feature\n 2 files changed, 30 insertions(+)";
        assert_eq!(extract_commit_hash(output), "abc1234");

        let output = "no hash here";
        assert_eq!(extract_commit_hash(output), "unknown");
    }
}