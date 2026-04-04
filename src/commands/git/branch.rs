//! `/branch` 命令实现
//!
//! 创建对话分支

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, LocalJsxCommand, LoadedFrom,
    CommandResultDisplay,
};
use crate::commands::registry::CommandExecutor;
use super::GitError;

/// 分支命令
pub struct BranchCommand;

#[async_trait::async_trait]
impl CommandExecutor for BranchCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        // 解析参数
        let args = parse_branch_args(&context.args);

        // 检查Git仓库
        super::ensure_git_repository(&context.cwd)?;

        // 创建分支
        let branch_info = create_branch(&context.cwd, &args).await?;

        // 返回结果
        Ok(CommandResult {
            content: format_branch_result(&branch_info, &args),
            display: CommandResultDisplay::User,
            should_query: false,
            ..Default::default()
        })
    }

    fn command(&self) -> Command {
        Command::LocalJsx(LocalJsxCommand {
            base: CommandBase {
                name: "branch".to_string(),
                description: "Create a conversation branch".to_string(),
                aliases: Some(vec!["br".to_string()]),
                argument_hint: Some("[branch-name] [--from <start-point>]".to_string()),
                when_to_use: Some("Use when you want to create a new branch for experimenting with changes".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
        })
    }
}

/// 分支参数
struct BranchArgs {
    name: Option<String>,
    from: Option<String>,
    checkout: bool,
    force: bool,
}

/// 解析分支命令参数
fn parse_branch_args(args: &str) -> BranchArgs {
    let mut name = None;
    let mut from = None;
    let mut checkout = false;
    let mut force = false;

    let mut args_iter = args.split_whitespace().peekable();

    while let Some(arg) = args_iter.next() {
        match arg {
            "--from" | "-f" => {
                if let Some(start_point) = args_iter.next() {
                    from = Some(start_point.to_string());
                }
            }
            "--checkout" | "-c" => {
                checkout = true;
            }
            "--force" | "-F" => {
                force = true;
            }
            "--no-checkout" => {
                checkout = false;
            }
            _ => {
                // 第一个非选项参数作为分支名
                if name.is_none() && !arg.starts_with('-') {
                    name = Some(arg.to_string());
                }
            }
        }
    }

    // 如果没有提供分支名，生成一个默认名
    let name = name.unwrap_or_else(|| generate_branch_name());

    BranchArgs {
        name: Some(name),
        from,
        checkout,
        force,
    }
}

/// 生成分支名
fn generate_branch_name() -> String {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    format!("claude-branch-{}", timestamp)
}

/// 分支信息
struct BranchInfo {
    name: String,
    from: String,
    created: bool,
    checked_out: bool,
    hash: Option<String>,
}

/// 创建分支
async fn create_branch(cwd: &std::path::Path, args: &BranchArgs) -> Result<BranchInfo> {
    let branch_name = args.name.as_ref().unwrap();
    let from_point = args.from.as_deref().unwrap_or("HEAD");

    // 检查起点是否存在
    let from_hash = super::execute_git_command(cwd, &["rev-parse", "--verify", from_point])
        .map_err(|_| GitError::InvalidArguments {
            message: format!("Start point '{}' does not exist", from_point),
        })?;

    // 构建创建命令
    let mut create_args = vec!["branch"];

    if args.force {
        create_args.push("--force");
    }

    create_args.push(branch_name);
    create_args.push(from_point);

    // 创建分支
    let create_output = super::execute_git_command(cwd, &create_args)?;

    let created = !create_output.contains("already exists") || args.force;

    // 如果需要，切换到新分支
    let checked_out = if args.checkout {
        let checkout_output = super::execute_git_command(cwd, &["checkout", branch_name])
            .map_err(|e| GitError::GitCommandFailed {
                command: format!("git checkout {}", branch_name),
                stderr: e.to_string(),
            })?;
        !checkout_output.contains("error:")
    } else {
        false
    };

    // 获取分支哈希
    let hash = super::execute_git_command(cwd, &["rev-parse", "--short", branch_name])
        .ok()
        .map(|h| h.trim().to_string());

    Ok(BranchInfo {
        name: branch_name.clone(),
        from: from_point.to_string(),
        created,
        checked_out,
        hash,
    })
}

/// 格式化分支结果
fn format_branch_result(info: &BranchInfo, args: &BranchArgs) -> String {
    let mut output = String::new();

    output.push_str("## Branch Created\n\n");

    if info.created {
        output.push_str("✅ **Successfully created branch**\n\n");
    } else {
        output.push_str("⚠️ **Branch already existed**\n\n");
    }

    output.push_str(&format!("**Branch Name**: `{}`\n", info.name));
    output.push_str(&format!("**Created From**: `{}`\n", info.from));

    if let Some(hash) = &info.hash {
        output.push_str(&format!("**Commit Hash**: `{}`\n", hash));
    }

    if info.checked_out {
        output.push_str("\n✅ **Checked out to new branch**\n");
        output.push_str(&format!("You are now on branch `{}`\n", info.name));
    } else if args.checkout {
        output.push_str("\n❌ **Failed to checkout branch**\n");
        output.push_str("The branch was created but could not be checked out.\n");
        output.push_str(&format!("Use `git checkout {}` to switch to it.\n", info.name));
    } else {
        output.push_str(&format!("\n📌 **Branch created but not checked out**\n"));
        output.push_str(&format!("Use `git checkout {}` to switch to it.\n", info.name));
    }

    output.push_str("\n### Next Steps\n");
    output.push_str("1. Make your changes on this branch\n");
    output.push_str("2. Use `/commit` to commit your changes\n");
    output.push_str("3. Use `/diff` to review your changes\n");
    output.push_str("4. When ready, merge or push the branch\n");

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_branch_args() {
        let args = parse_branch_args("feature/new-feature --from main --checkout");
        assert_eq!(args.name, Some("feature/new-feature".to_string()));
        assert_eq!(args.from, Some("main".to_string()));
        assert!(args.checkout);
        assert!(!args.force);

        let args = parse_branch_args("--force");
        assert!(args.name.is_some());
        assert!(args.name.unwrap().starts_with("claude-branch-"));
        assert!(args.force);
    }

    #[test]
    fn test_generate_branch_name() {
        let name = generate_branch_name();
        assert!(name.starts_with("claude-branch-"));
        assert!(name.len() > "claude-branch-".len());
    }

    #[test]
    fn test_format_branch_result() {
        let info = BranchInfo {
            name: "test-branch".to_string(),
            from: "main".to_string(),
            created: true,
            checked_out: true,
            hash: Some("abc1234".to_string()),
        };

        let args = BranchArgs {
            name: Some("test-branch".to_string()),
            from: Some("main".to_string()),
            checkout: true,
            force: false,
        };

        let result = format_branch_result(&info, &args);
        assert!(result.contains("test-branch"));
        assert!(result.contains("main"));
        assert!(result.contains("abc1234"));
        assert!(result.contains("Next Steps"));
    }
}