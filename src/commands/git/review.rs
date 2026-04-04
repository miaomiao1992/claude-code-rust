//! `/review` 命令实现
//!
//! 审查Pull Request

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, PromptCommand, LoadedFrom, CommandSource,
};
use crate::commands::registry::CommandExecutor;
// use super::GitError;  // 未使用

/// 审查命令
pub struct ReviewCommand;

#[async_trait::async_trait]
impl CommandExecutor for ReviewCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        // 解析参数
        let args = parse_review_args(&context.args);

        // 检查Git仓库
        super::ensure_git_repository(&context.cwd)?;

        // 获取PR信息
        let pr_info = get_pr_info(&context.cwd, &args).await?;

        // 获取差异
        let diff = get_pr_diff(&context.cwd, &args).await?;

        // 构建审查提示
        let review_prompt = build_review_prompt(&pr_info, &diff, &args);

        // 返回结果（实际审查由AI模型完成）
        Ok(CommandResult {
            content: review_prompt,
            should_query: true,  // 需要AI模型进行审查
            ..Default::default()
        })
    }

    fn command(&self) -> Command {
        Command::Prompt(PromptCommand {
            base: CommandBase {
                name: "review".to_string(),
                description: "Review a Pull Request".to_string(),
                aliases: Some(vec!["rv".to_string()]),
                argument_hint: Some("[pr-number] [--remote <remote>]".to_string()),
                when_to_use: Some("Use when you want to review a Pull Request with AI assistance".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
            progress_message: "Analyzing PR changes and preparing review...".to_string(),
            content_length: 4000,
            arg_names: Some(vec!["pr_number".to_string(), "remote".to_string()]),
            allowed_tools: Some(vec!["git".to_string(), "read_file".to_string(), "web_fetch".to_string()]),
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

/// 审查参数
struct ReviewArgs {
    pr_number: Option<u32>,
    remote: Option<String>,
    branch: Option<String>,
}

/// 解析审查命令参数
fn parse_review_args(args: &str) -> ReviewArgs {
    let mut pr_number = None;
    let mut remote = None;
    let mut branch = None;

    let mut args_iter = args.split_whitespace().peekable();

    while let Some(arg) = args_iter.next() {
        match arg {
            "--remote" | "-r" => {
                if let Some(r) = args_iter.next() {
                    remote = Some(r.to_string());
                }
            }
            "--branch" | "-b" => {
                if let Some(b) = args_iter.next() {
                    branch = Some(b.to_string());
                }
            }
            arg if arg.starts_with('#') => {
                if let Ok(num) = arg[1..].parse() {
                    pr_number = Some(num);
                }
            }
            arg if arg.parse::<u32>().is_ok() => {
                pr_number = Some(arg.parse().unwrap());
            }
            _ => {
                // 忽略未知参数
            }
        }
    }

    ReviewArgs {
        pr_number,
        remote,
        branch,
    }
}

/// 获取PR信息
async fn get_pr_info(cwd: &std::path::Path, args: &ReviewArgs) -> Result<PrInfo> {
    // 如果有PR编号，尝试从GitHub获取信息
    if let Some(pr_number) = args.pr_number {
        return fetch_pr_info_from_github(pr_number, args).await;
    }

    // 否则，获取当前分支的差异
    let current_branch = super::execute_git_command(cwd, &["branch", "--show-current"])?
        .trim()
        .to_string();

    let base_branch = args.branch.clone()
        .unwrap_or_else(|| "main".to_string());

    Ok(PrInfo {
        title: format!("Review changes from {} to {}", current_branch, base_branch),
        description: String::new(),
        author: String::new(),
        url: None,
        base_branch,
        head_branch: current_branch,
    })
}

/// PR信息
struct PrInfo {
    title: String,
    description: String,
    author: String,
    url: Option<String>,
    base_branch: String,
    head_branch: String,
}

/// 从GitHub获取PR信息
async fn fetch_pr_info_from_github(pr_number: u32, args: &ReviewArgs) -> Result<PrInfo> {
    // TODO: 实现GitHub API调用
    // 目前返回模拟数据

    Ok(PrInfo {
        title: format!("PR #{}: Sample Pull Request", pr_number),
        description: "This is a sample pull request description.".to_string(),
        author: "author@example.com".to_string(),
        url: Some(format!("https://github.com/owner/repo/pull/{}", pr_number)),
        base_branch: "main".to_string(),
        head_branch: format!("feature-{}", pr_number),
    })
}

/// 获取PR差异
async fn get_pr_diff(cwd: &std::path::Path, args: &ReviewArgs) -> Result<String> {
    let base_branch = args.branch.clone()
        .unwrap_or_else(|| "main".to_string());

    let head_branch = if let Some(pr_number) = args.pr_number {
        format!("pull/{}/head", pr_number)
    } else {
        super::execute_git_command(cwd, &["branch", "--show-current"])?
            .trim()
            .to_string()
    };

    // 获取差异
    super::execute_git_command(cwd, &["diff", &format!("{}..{}", base_branch, head_branch)])
        .or_else(|_| super::execute_git_command(cwd, &["diff", &base_branch]))
        .map_err(Into::into)
}

/// 构建审查提示
fn build_review_prompt(pr_info: &PrInfo, diff: &str, args: &ReviewArgs) -> String {
    let mut prompt = String::new();

    prompt.push_str("# Pull Request Review\n\n");

    if let Some(url) = &pr_info.url {
        prompt.push_str(&format!("**PR URL**: {}\n", url));
    }

    prompt.push_str(&format!("**Title**: {}\n", pr_info.title));
    prompt.push_str(&format!("**Author**: {}\n", pr_info.author));
    prompt.push_str(&format!("**Base Branch**: {}\n", pr_info.base_branch));
    prompt.push_str(&format!("**Head Branch**: {}\n", pr_info.head_branch));

    if !pr_info.description.is_empty() {
        prompt.push_str(&format!("\n**Description**:\n{}\n", pr_info.description));
    }

    prompt.push_str("\n## Changes to Review\n\n");

    if diff.is_empty() {
        prompt.push_str("No changes found to review.\n");
    } else if diff.lines().count() > 500 {
        prompt.push_str(&format!(
            "Large diff ({} lines). Key changes:\n",
            diff.lines().count()
        ));

        // 提取关键文件
        let files: Vec<&str> = diff
            .lines()
            .filter(|line| line.starts_with("diff --git"))
            .map(|line| {
                if let Some(start) = line.find(" b/") {
                    &line[start + 3..]
                } else {
                    line
                }
            })
            .collect();

        for file in files.iter().take(10) {
            prompt.push_str(&format!("- {}\n", file));
        }

        if files.len() > 10 {
            prompt.push_str(&format!("- ... and {} more files\n", files.len() - 10));
        }

        prompt.push_str("\n(Full diff is available for analysis)\n");
    } else {
        prompt.push_str("```diff\n");
        prompt.push_str(diff);
        if !diff.ends_with('\n') {
            prompt.push('\n');
        }
        prompt.push_str("```\n");
    }

    prompt.push_str("\n## Review Instructions\n\n");
    prompt.push_str("Please review the above changes and provide:\n");
    prompt.push_str("1. **Code Quality**: Any code style issues, potential bugs, or improvements\n");
    prompt.push_str("2. **Security**: Any security vulnerabilities or concerns\n");
    prompt.push_str("3. **Performance**: Any performance implications\n");
    prompt.push_str("4. **Testing**: Suggestions for test coverage\n");
    prompt.push_str("5. **Documentation**: Any missing or unclear documentation\n");
    prompt.push_str("6. **Overall Assessment**: Should this PR be approved, revised, or rejected?\n");

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_review_args() {
        let args = parse_review_args("#123 --remote origin");
        assert_eq!(args.pr_number, Some(123));
        assert_eq!(args.remote, Some("origin".to_string()));

        let args = parse_review_args("456 -r upstream");
        assert_eq!(args.pr_number, Some(456));
        assert_eq!(args.remote, Some("upstream".to_string()));
    }

    #[test]
    fn test_build_review_prompt() {
        let pr_info = PrInfo {
            title: "Test PR".to_string(),
            description: "Test description".to_string(),
            author: "test@example.com".to_string(),
            url: Some("https://github.com/test/test/pull/1".to_string()),
            base_branch: "main".to_string(),
            head_branch: "feature".to_string(),
        };

        let diff = "diff --git a/file.txt b/file.txt\n+++ added line";
        let args = ReviewArgs {
            pr_number: Some(1),
            remote: None,
            branch: None,
        };

        let prompt = build_review_prompt(&pr_info, diff, &args);
        assert!(prompt.contains("Test PR"));
        assert!(prompt.contains("test@example.com"));
        assert!(prompt.contains("Code Quality"));
    }
}