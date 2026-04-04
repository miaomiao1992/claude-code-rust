//! `/pr-comments` 命令实现
//!
//! 获取Pull Request评论

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, PromptCommand, LoadedFrom, CommandSource,
};
use crate::commands::registry::CommandExecutor;

/// PR评论命令
pub struct PrCommentsCommand;

#[async_trait::async_trait]
impl CommandExecutor for PrCommentsCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        // 解析参数
        let args = parse_pr_comments_args(&context.args);

        // 获取PR评论
        let comments = fetch_pr_comments(&context.cwd, &args).await?;

        // 构建提示
        let prompt = build_comments_prompt(&comments, &args);

        // 返回结果
        Ok(CommandResult {
            content: prompt,
            should_query: true,  // 需要AI分析评论
            ..Default::default()
        })
    }

    fn command(&self) -> Command {
        Command::Prompt(PromptCommand {
            base: CommandBase {
                name: "pr-comments".to_string(),
                description: "Get and analyze Pull Request comments".to_string(),
                aliases: Some(vec!["prc".to_string()]),
                argument_hint: Some("[pr-number] [--remote <remote>]".to_string()),
                when_to_use: Some("Use when you want to analyze comments on a Pull Request".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
            progress_message: "Fetching and analyzing PR comments...".to_string(),
            content_length: 3000,
            arg_names: Some(vec!["pr_number".to_string(), "remote".to_string()]),
            allowed_tools: Some(vec!["git".to_string(), "read_file".to_string(), "web_fetch".to_string()]),
            model: Some("claude-3-5-sonnet-20241022".to_string()),
            source: CommandSource::Builtin,
            plugin_info: None,
            disable_non_interactive: None,
            context: None,
            agent: None,
            effort: Some(crate::commands::types::EffortValue::Medium),
            paths: None,
        })
    }
}

/// PR评论参数
struct PrCommentsArgs {
    pr_number: Option<u32>,
    remote: Option<String>,
    include_resolved: bool,
    limit: usize,
}

/// 解析PR评论命令参数
fn parse_pr_comments_args(args: &str) -> PrCommentsArgs {
    let mut pr_number = None;
    let mut remote = None;
    let mut include_resolved = false;
    let mut limit = 50; // 默认限制

    let mut args_iter = args.split_whitespace().peekable();

    while let Some(arg) = args_iter.next() {
        match arg {
            "--remote" | "-r" => {
                if let Some(r) = args_iter.next() {
                    remote = Some(r.to_string());
                }
            }
            "--include-resolved" | "-i" => {
                include_resolved = true;
            }
            "--limit" | "-l" => {
                if let Some(l) = args_iter.next() {
                    if let Ok(num) = l.parse() {
                        limit = num;
                    }
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

    PrCommentsArgs {
        pr_number,
        remote,
        include_resolved,
        limit,
    }
}

/// PR评论
#[derive(Debug)]
struct PrComment {
    author: String,
    body: String,
    created_at: String,
    resolved: bool,
    file_path: Option<String>,
    line_number: Option<u32>,
    is_review: bool,
}

/// 获取PR评论
async fn fetch_pr_comments(cwd: &std::path::Path, args: &PrCommentsArgs) -> Result<Vec<PrComment>> {
    // 如果有PR编号，尝试从GitHub获取评论
    if let Some(pr_number) = args.pr_number {
        return fetch_comments_from_github(pr_number, args).await;
    }

    // 否则，获取当前PR的评论（通过git config或其他方式）
    // 这里简化为返回空列表
    Ok(Vec::new())
}

/// 从GitHub获取评论
async fn fetch_comments_from_github(pr_number: u32, args: &PrCommentsArgs) -> Result<Vec<PrComment>> {
    // TODO: 实现GitHub API调用
    // 目前返回模拟数据

    let mut comments = Vec::new();

    // 模拟一些评论
    comments.push(PrComment {
        author: "reviewer1".to_string(),
        body: "This looks good overall, but can you add tests for the new feature?".to_string(),
        created_at: "2024-01-15T10:30:00Z".to_string(),
        resolved: false,
        file_path: Some("src/feature.rs".to_string()),
        line_number: Some(42),
        is_review: true,
    });

    comments.push(PrComment {
        author: "author".to_string(),
        body: "Thanks for the review! I've added tests in the latest commit.".to_string(),
        created_at: "2024-01-15T11:15:00Z".to_string(),
        resolved: true,
        file_path: None,
        line_number: None,
        is_review: false,
    });

    comments.push(PrComment {
        author: "reviewer2".to_string(),
        body: "There's a potential race condition in the async code. Consider using a Mutex.".to_string(),
        created_at: "2024-01-15T12:00:00Z".to_string(),
        resolved: false,
        file_path: Some("src/async_module.rs".to_string()),
        line_number: Some(123),
        is_review: true,
    });

    // 根据参数过滤
    let filtered_comments: Vec<PrComment> = comments
        .into_iter()
        .filter(|c| args.include_resolved || !c.resolved)
        .take(args.limit)
        .collect();

    Ok(filtered_comments)
}

/// 构建评论提示
fn build_comments_prompt(comments: &[PrComment], args: &PrCommentsArgs) -> String {
    let mut prompt = String::new();

    prompt.push_str("# Pull Request Comments Analysis\n\n");

    if let Some(pr_number) = args.pr_number {
        prompt.push_str(&format!("**PR #{}**\n\n", pr_number));
    } else {
        prompt.push_str("**Current Pull Request**\n\n");
    }

    if comments.is_empty() {
        prompt.push_str("No comments found or no PR specified.\n");
        prompt.push_str("Please specify a PR number: `/pr-comments #123`\n");
        return prompt;
    }

    prompt.push_str(&format!("Found {} comment(s):\n\n", comments.len()));

    let unresolved_count = comments.iter().filter(|c| !c.resolved).count();
    let resolved_count = comments.len() - unresolved_count;

    prompt.push_str(&format!("- {} unresolved comment(s)\n", unresolved_count));
    prompt.push_str(&format!("- {} resolved comment(s)\n", resolved_count));

    // 按文件分组
    let mut comments_by_file: std::collections::HashMap<Option<String>, Vec<&PrComment>> =
        std::collections::HashMap::new();

    for comment in comments {
        comments_by_file
            .entry(comment.file_path.clone())
            .or_default()
            .push(comment);
    }

    // 显示评论
    for (file_path, file_comments) in comments_by_file {
        if let Some(path) = file_path {
            prompt.push_str(&format!("\n## File: {}\n\n", path));
        } else {
            prompt.push_str("\n## General Comments\n\n");
        }

        for comment in file_comments {
            let status = if comment.resolved {
                "✅ RESOLVED"
            } else {
                "⚠️ UNRESOLVED"
            };

            prompt.push_str(&format!("### {} by {}\n", status, comment.author));

            if let Some(line) = comment.line_number {
                prompt.push_str(&format!("Line: {}\n", line));
            }

            prompt.push_str(&format!("Time: {}\n", comment.created_at));

            if comment.is_review {
                prompt.push_str("Type: Review Comment\n");
            }

            prompt.push_str("\n");
            prompt.push_str(&comment.body);
            prompt.push_str("\n\n---\n\n");
        }
    }

    // 分析指导
    prompt.push_str("\n## Analysis Instructions\n\n");
    prompt.push_str("Please analyze the above comments and provide:\n");
    prompt.push_str("1. **Summary**: Key themes and concerns from the comments\n");
    prompt.push_str("2. **Priority**: Which comments need immediate attention\n");
    prompt.push_str("3. **Action Items**: Specific changes or responses needed\n");
    prompt.push_str("4. **Response Strategy**: How to respond to reviewers\n");
    prompt.push_str("5. **Next Steps**: Recommended actions for the PR author\n");

    if unresolved_count > 0 {
        prompt.push_str(&format!(
            "\n**Note**: There are {} unresolved comments that need attention.\n",
            unresolved_count
        ));
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pr_comments_args() {
        let args = parse_pr_comments_args("#123 --remote origin --limit 20");
        assert_eq!(args.pr_number, Some(123));
        assert_eq!(args.remote, Some("origin".to_string()));
        assert_eq!(args.limit, 20);
        assert!(!args.include_resolved);

        let args = parse_pr_comments_args("456 -i");
        assert_eq!(args.pr_number, Some(456));
        assert!(args.include_resolved);
        assert_eq!(args.limit, 50); // 默认值
    }

    #[test]
    fn test_build_comments_prompt() {
        let comments = vec![
            PrComment {
                author: "test1".to_string(),
                body: "Test comment 1".to_string(),
                created_at: "2024-01-01".to_string(),
                resolved: false,
                file_path: Some("file.rs".to_string()),
                line_number: Some(10),
                is_review: true,
            },
            PrComment {
                author: "test2".to_string(),
                body: "Test comment 2".to_string(),
                created_at: "2024-01-02".to_string(),
                resolved: true,
                file_path: None,
                line_number: None,
                is_review: false,
            },
        ];

        let args = PrCommentsArgs {
            pr_number: Some(123),
            remote: None,
            include_resolved: true,
            limit: 50,
        };

        let prompt = build_comments_prompt(&comments, &args);
        assert!(prompt.contains("PR #123"));
        assert!(prompt.contains("Test comment 1"));
        assert!(prompt.contains("Test comment 2"));
        assert!(prompt.contains("File: file.rs"));
        assert!(prompt.contains("Analysis Instructions"));
    }
}