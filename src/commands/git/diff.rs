//! `/diff` 命令实现
//!
//! 查看未提交的Git更改

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, LocalJsxCommand, LoadedFrom,
    CommandResultDisplay,
};
use crate::commands::registry::CommandExecutor;
use super::GitError;

/// 差异命令
pub struct DiffCommand;

#[async_trait::async_trait]
impl CommandExecutor for DiffCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        // 解析参数
        let args = parse_diff_args(&context.args);

        // 检查Git仓库
        super::ensure_git_repository(&context.cwd)?;

        // 获取差异
        let diff_output = get_git_diff(&context.cwd, &args.files, args.staged, args.cached).await?;

        if diff_output.is_empty() {
            return Ok(CommandResult {
                content: "No changes to show.".to_string(),
                display: CommandResultDisplay::User,
                should_query: false,
                ..Default::default()
            });
        }

        // 格式化输出
        let formatted_diff = format_diff_output(&diff_output, &args);

        Ok(CommandResult {
            content: formatted_diff,
            display: CommandResultDisplay::User,
            should_query: false,
            ..Default::default()
        })
    }

    fn command(&self) -> Command {
        Command::LocalJsx(LocalJsxCommand {
            base: CommandBase {
                name: "diff".to_string(),
                description: "View uncommitted Git changes".to_string(),
                aliases: Some(vec!["d".to_string()]),
                argument_hint: Some("[files] [--staged] [--cached]".to_string()),
                when_to_use: Some("Use when you want to see what changes have been made but not yet committed".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
        })
    }
}

/// 差异参数
struct DiffArgs {
    files: Vec<String>,
    staged: bool,
    cached: bool,
}

/// 解析差异命令参数
fn parse_diff_args(args: &str) -> DiffArgs {
    let mut files = Vec::new();
    let mut staged = false;
    let mut cached = false;

    for arg in args.split_whitespace() {
        match arg {
            "--staged" | "--cached" => {
                staged = true;
                cached = true;
            }
            "--unstaged" => {
                staged = false;
                cached = false;
            }
            arg if arg.starts_with("--") => {
                // 忽略其他选项
            }
            _ => {
                files.push(arg.to_string());
            }
        }
    }

    DiffArgs { files, staged, cached }
}

/// 获取Git差异
async fn get_git_diff(
    cwd: &std::path::Path,
    files: &[String],
    staged: bool,
    cached: bool,
) -> Result<String> {
    let mut diff_args = vec!["diff"];

    if staged || cached {
        diff_args.push("--staged");
    }

    // 添加颜色输出
    diff_args.push("--color=always");

    // 添加统一差异格式
    diff_args.push("--unified=5");

    if !files.is_empty() {
        diff_args.push("--");
        diff_args.extend(files.iter().map(|s| s.as_str()));
    }

    super::execute_git_command(cwd, &diff_args)
        .or_else(|_| {
            // 如果差异为空，尝试获取状态
            if files.is_empty() {
                super::execute_git_command(cwd, &["status", "--porcelain"])
            } else {
                Err(ClaudeError::Other("git diff: No output".to_string()))
            }
        })
        .map_err(Into::into)
}

/// 格式化差异输出
fn format_diff_output(diff: &str, args: &DiffArgs) -> String {
    let mut output = String::new();

    if args.staged || args.cached {
        output.push_str("## Staged Changes\n\n");
    } else {
        output.push_str("## Unstaged Changes\n\n");
    }

    if diff.lines().count() > 100 {
        // 如果差异太大，只显示摘要
        let file_changes = diff
            .lines()
            .filter(|line| line.starts_with("diff --git"))
            .count();

        let insertions = diff.lines().filter(|line| line.starts_with('+')).count();
        let deletions = diff.lines().filter(|line| line.starts_with('-')).count();

        output.push_str(&format!(
            "Summary: {} files changed, {} insertions(+), {} deletions(-)\n\n",
            file_changes, insertions, deletions
        ));
        output.push_str("(Diff is too large to display here. Use `git diff` in terminal for full view.)\n");
    } else {
        output.push_str("```diff\n");
        output.push_str(diff);
        if !diff.ends_with('\n') {
            output.push('\n');
        }
        output.push_str("```\n");
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diff_args() {
        let args = parse_diff_args("file1.rs file2.rs --staged");
        assert_eq!(args.files, vec!["file1.rs", "file2.rs"]);
        assert!(args.staged);
        assert!(args.cached);

        let args = parse_diff_args("--unstaged");
        assert!(args.files.is_empty());
        assert!(!args.staged);
        assert!(!args.cached);
    }

    #[test]
    fn test_format_diff_output() {
        let diff = "diff --git a/file.txt b/file.txt\nindex abc123..def456 100644\n--- a/file.txt\n+++ b/file.txt\n@@ -1,3 +1,4 @@\n line1\n-line2\n+line2 modified\n line3\n+line4";

        let args = DiffArgs {
            files: vec![],
            staged: false,
            cached: false,
        };

        let formatted = format_diff_output(diff, &args);
        assert!(formatted.contains("Unstaged Changes"));
        assert!(formatted.contains("```diff"));
    }
}