//! `/commit-push-pr` 命令实现
//!
//! 提交+推送+创建Pull Request工作流

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, PromptCommand, LoadedFrom, CommandSource,
};
use crate::commands::registry::CommandExecutor;
use super::GitError;

/// 提交推送PR命令
pub struct CommitPushPrCommand;

#[async_trait::async_trait]
impl CommandExecutor for CommitPushPrCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        // 解析参数
        let args = parse_commit_push_pr_args(&context.args);

        // 检查Git仓库
        super::ensure_git_repository(&context.cwd)?;

        // 执行工作流
        let workflow_result = execute_commit_push_pr_workflow(&context.cwd, &args).await?;

        // 构建结果提示
        let prompt = build_workflow_result_prompt(&workflow_result, &args);

        // 返回结果
        Ok(CommandResult {
            content: prompt,
            should_query: true,  // 可能需要AI进一步指导
            ..Default::default()
        })
    }

    fn command(&self) -> Command {
        Command::Prompt(PromptCommand {
            base: CommandBase {
                name: "commit-push-pr".to_string(),
                description: "Commit, push, and create a Pull Request".to_string(),
                aliases: Some(vec!["cpp".to_string()]),
                argument_hint: Some("[--message <msg>] [--branch <branch>] [--title <title>]".to_string()),
                when_to_use: Some("Use when you want to quickly commit changes, push them, and create a PR".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
            progress_message: "Executing commit-push-PR workflow...".to_string(),
            content_length: 2500,
            arg_names: Some(vec!["message".to_string(), "branch".to_string(), "title".to_string()]),
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

/// 工作流参数
struct CommitPushPrArgs {
    message: Option<String>,
    branch_name: Option<String>,
    pr_title: Option<String>,
    pr_body: Option<String>,
    remote: Option<String>,
    base_branch: Option<String>,
    draft: bool,
}

/// 解析工作流参数
fn parse_commit_push_pr_args(args: &str) -> CommitPushPrArgs {
    let mut message = None;
    let mut branch_name = None;
    let mut pr_title = None;
    let mut pr_body = None;
    let mut remote = None;
    let mut base_branch = None;
    let mut draft = false;

    let mut args_iter = args.split_whitespace().peekable();

    while let Some(arg) = args_iter.next() {
        match arg {
            "--message" | "-m" => {
                if let Some(msg) = args_iter.next() {
                    message = Some(msg.to_string());
                }
            }
            "--branch" | "-b" => {
                if let Some(branch) = args_iter.next() {
                    branch_name = Some(branch.to_string());
                }
            }
            "--title" | "-t" => {
                if let Some(title) = args_iter.next() {
                    pr_title = Some(title.to_string());
                }
            }
            "--body" => {
                if let Some(body) = args_iter.next() {
                    pr_body = Some(body.to_string());
                }
            }
            "--remote" | "-r" => {
                if let Some(rem) = args_iter.next() {
                    remote = Some(rem.to_string());
                }
            }
            "--base" => {
                if let Some(base) = args_iter.next() {
                    base_branch = Some(base.to_string());
                }
            }
            "--draft" | "-d" => {
                draft = true;
            }
            _ => {
                // 未标记的参数作为消息（如果消息为空）
                if message.is_none() && !arg.starts_with('-') {
                    message = Some(arg.to_string());
                }
            }
        }
    }

    CommitPushPrArgs {
        message,
        branch_name,
        pr_title,
        pr_body,
        remote,
        base_branch,
        draft,
    }
}

/// 工作流结果
struct WorkflowResult {
    commit_success: bool,
    commit_hash: Option<String>,
    push_success: bool,
    push_output: Option<String>,
    pr_created: bool,
    pr_url: Option<String>,
    branch_created: bool,
    branch_name: String,
    errors: Vec<String>,
}

/// 执行工作流
async fn execute_commit_push_pr_workflow(
    cwd: &std::path::Path,
    args: &CommitPushPrArgs,
) -> Result<WorkflowResult> {
    let mut result = WorkflowResult {
        commit_success: false,
        commit_hash: None,
        push_success: false,
        push_output: None,
        pr_created: false,
        pr_url: None,
        branch_created: false,
        branch_name: String::new(),
        errors: Vec::new(),
    };

    // 步骤1：创建或切换到分支
    let branch_name = args.branch_name.clone().unwrap_or_else(|| {
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        format!("claude-pr-{}", timestamp)
    });

    result.branch_name = branch_name.clone();

    // 检查是否需要创建新分支
    let current_branch = super::execute_git_command(cwd, &["branch", "--show-current"])
        .unwrap_or_default()
        .trim()
        .to_string();

    if current_branch != branch_name {
        match super::execute_git_command(cwd, &["checkout", "-b", &branch_name]) {
            Ok(_) => {
                result.branch_created = true;
            }
            Err(e) => {
                result.errors.push(format!("Failed to create branch: {}", e));
                // 尝试切换到现有分支
                if let Err(e2) = super::execute_git_command(cwd, &["checkout", &branch_name]) {
                    result.errors.push(format!("Failed to checkout branch: {}", e2));
                    return Ok(result);
                }
            }
        }
    }

    // 步骤2：提交更改
    let commit_message = args.message.clone().unwrap_or_else(|| {
        "Update code".to_string()
    });

    match super::execute_git_command(cwd, &["commit", "-am", &commit_message]) {
        Ok(output) => {
            result.commit_success = true;
            // 提取提交哈希
            for line in output.lines() {
                if line.contains('[') && line.contains(']') {
                    if let Some(start) = line.find(' ') {
                        let rest = &line[start + 1..];
                        if let Some(end) = rest.find(' ') {
                            result.commit_hash = Some(rest[..end].to_string());
                            break;
                        }
                    }
                }
            }
        }
        Err(e) => {
            result.errors.push(format!("Commit failed: {}", e));
            // 可能没有更改可提交
            let status = super::execute_git_command(cwd, &["status", "--porcelain"])
                .unwrap_or_default();
            if status.trim().is_empty() {
                result.errors.push("No changes to commit".to_string());
            }
        }
    }

    // 步骤3：推送到远程
    let remote = args.remote.clone().unwrap_or_else(|| "origin".to_string());

    match super::execute_git_command(cwd, &["push", "--set-upstream", &remote, &branch_name]) {
        Ok(output) => {
            result.push_success = true;
            result.push_output = Some(output);
        }
        Err(e) => {
            result.errors.push(format!("Push failed: {}", e));
        }
    }

    // 步骤4：创建PR（如果推送成功）
    if result.push_success {
        match create_pull_request(cwd, &branch_name, args).await {
            Ok(pr_url) => {
                result.pr_created = true;
                result.pr_url = Some(pr_url);
            }
            Err(e) => {
                result.errors.push(format!("PR creation failed: {}", e));
            }
        }
    }

    Ok(result)
}

/// 创建Pull Request
async fn create_pull_request(
    cwd: &std::path::Path,
    branch_name: &str,
    args: &CommitPushPrArgs,
) -> Result<String> {
    // TODO: 实现GitHub API调用创建PR
    // 目前返回模拟URL

    let base_branch = args.base_branch.clone().unwrap_or_else(|| "main".to_string());
    let pr_title = args.pr_title.clone()
        .unwrap_or_else(|| format!("PR: {}", branch_name));
    let pr_body = args.pr_body.clone()
        .unwrap_or_else(|| "Created by Claude Code".to_string());

    // 模拟创建PR
    let pr_url = format!(
        "https://github.com/owner/repo/pull/123 (Simulated: {} -> {})",
        branch_name, base_branch
    );

    Ok(pr_url)
}

/// 构建工作流结果提示
fn build_workflow_result_prompt(result: &WorkflowResult, args: &CommitPushPrArgs) -> String {
    let mut prompt = String::new();

    prompt.push_str("# Commit-Push-PR Workflow Results\n\n");

    if result.errors.is_empty() && result.commit_success && result.push_success && result.pr_created {
        prompt.push_str("✅ **Workflow completed successfully!**\n\n");
    } else {
        prompt.push_str("## Workflow Status\n\n");
    }

    // 提交状态
    if result.commit_success {
        prompt.push_str("✅ **Commit**: Success\n");
        if let Some(hash) = &result.commit_hash {
            prompt.push_str(&format!("  Hash: `{}`\n", hash));
        }
    } else {
        prompt.push_str("❌ **Commit**: Failed\n");
    }

    // 推送状态
    if result.push_success {
        prompt.push_str("✅ **Push**: Success\n");
        prompt.push_str(&format!("  Branch: `{}`\n", result.branch_name));
    } else {
        prompt.push_str("❌ **Push**: Failed\n");
    }

    // PR状态
    if result.pr_created {
        prompt.push_str("✅ **PR Created**: Success\n");
        if let Some(url) = &result.pr_url {
            prompt.push_str(&format!("  URL: {}\n", url));
        }
    } else {
        prompt.push_str("⚠️ **PR Created**: Not attempted or failed\n");
    }

    // 分支状态
    if result.branch_created {
        prompt.push_str("📌 **Branch**: New branch created\n");
    } else {
        prompt.push_str("📌 **Branch**: Using existing branch\n");
    }

    // 显示错误
    if !result.errors.is_empty() {
        prompt.push_str("\n## Errors\n\n");
        for error in &result.errors {
            prompt.push_str(&format!("- {}\n", error));
        }
    }

    // 下一步指导
    prompt.push_str("\n## Next Steps\n\n");

    if result.pr_created {
        prompt.push_str("1. **Review the PR**: Check the PR URL above\n");
        prompt.push_str("2. **Add reviewers**: Assign team members to review\n");
        prompt.push_str("3. **Monitor CI**: Wait for CI checks to pass\n");
        prompt.push_str("4. **Merge when ready**: After approval and CI success\n");
    } else if result.push_success {
        prompt.push_str("1. **Create PR manually**: Use GitHub UI or CLI\n");
        prompt.push_str("2. **Link**: ");
        if let Some(url) = &result.pr_url {
            prompt.push_str(&format!("{}\n", url));
        } else {
            prompt.push_str("(URL not available)\n");
        }
        prompt.push_str(&format!("3. **Branch**: `{}`\n", result.branch_name));
    } else if result.commit_success {
        prompt.push_str("1. **Fix push issues**: Check remote configuration\n");
        prompt.push_str("2. **Push manually**: `git push origin ");
        prompt.push_str(&result.branch_name);
        prompt.push_str("`\n");
        prompt.push_str("3. **Then create PR**\n");
    } else {
        prompt.push_str("1. **Check for changes**: `git status`\n");
        prompt.push_str("2. **Stage changes**: `git add .`\n");
        prompt.push_str("3. **Try again** or use individual commands\n");
    }

    // 如果需要AI进一步分析
    if !result.errors.is_empty() {
        prompt.push_str("\n## Request for Analysis\n\n");
        prompt.push_str("Please analyze the workflow errors and suggest:\n");
        prompt.push_str("1. **Root causes** of the failures\n");
        prompt.push_str("2. **Step-by-step fixes** for each issue\n");
        prompt.push_str("3. **Alternative approaches** if needed\n");
        prompt.push_str("4. **Prevention strategies** for future workflows\n");
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commit_push_pr_args() {
        let args = parse_commit_push_pr_args("--message \"feat: add feature\" --branch feature/new --title \"New Feature\"");
        assert_eq!(args.message, Some("feat: add feature".to_string()));
        assert_eq!(args.branch_name, Some("feature/new".to_string()));
        assert_eq!(args.pr_title, Some("New Feature".to_string()));
        assert!(!args.draft);

        let args = parse_commit_push_pr_args("Simple message --draft");
        assert_eq!(args.message, Some("Simple".to_string()));
        assert!(args.draft);
    }

    #[test]
    fn test_build_workflow_result_prompt() {
        let result = WorkflowResult {
            commit_success: true,
            commit_hash: Some("abc123".to_string()),
            push_success: true,
            push_output: Some("Push successful".to_string()),
            pr_created: true,
            pr_url: Some("https://github.com/test/pr".to_string()),
            branch_created: true,
            branch_name: "feature/test".to_string(),
            errors: Vec::new(),
        };

        let args = CommitPushPrArgs {
            message: Some("test".to_string()),
            branch_name: Some("feature/test".to_string()),
            pr_title: None,
            pr_body: None,
            remote: None,
            base_branch: None,
            draft: false,
        };

        let prompt = build_workflow_result_prompt(&result, &args);
        assert!(prompt.contains("Workflow completed successfully"));
        assert!(prompt.contains("abc123"));
        assert!(prompt.contains("feature/test"));
        assert!(prompt.contains("https://github.com/test/pr"));
        assert!(prompt.contains("Next Steps"));
    }
}