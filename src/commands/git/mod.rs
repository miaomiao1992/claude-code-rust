//! Git命令模块
//!
//! 这个模块实现了Git相关的命令，包括：
//! - `/commit` - 创建git提交
//! - `/diff` - 查看未提交的更改
//! - `/review` - 审查PR
//! - `/branch` - 创建对话分支
//! - `/pr-comments` - 获取PR评论
//! - `/commit-push-pr` - 提交+推送+创建PR
//! - `/security-review` - 安全审查

use std::path::Path;
use std::process::Command;

use crate::error::{ClaudeError, Result};
use crate::commands::registry::{CommandLoader, CommandRegistry};

pub mod commit;
pub mod diff;
pub mod review;
pub mod branch;
pub mod pr_comments;
pub mod commit_push_pr;
pub mod security_review;

pub use commit::CommitCommand;
pub use diff::DiffCommand;
pub use review::ReviewCommand;
pub use branch::BranchCommand;
pub use pr_comments::PrCommentsCommand;
pub use commit_push_pr::CommitPushPrCommand;
pub use security_review::SecurityReviewCommand;

/// Git命令加载器
pub struct GitCommandLoader;

#[async_trait::async_trait]
impl CommandLoader for GitCommandLoader {
    async fn load(&self, registry: &CommandRegistry) -> Result<()> {
        registry.register(CommitCommand).await;
        registry.register(DiffCommand).await;
        registry.register(ReviewCommand).await;
        registry.register(BranchCommand).await;
        registry.register(PrCommentsCommand).await;
        registry.register(CommitPushPrCommand).await;
        registry.register(SecurityReviewCommand).await;

        tracing::debug!("Loaded Git commands");

        Ok(())
    }

    fn name(&self) -> &str {
        "git"
    }
}

/// Git错误类型
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Git命令执行失败: {command} - {stderr}")]
    GitCommandFailed { command: String, stderr: String },

    #[error("当前目录不是Git仓库")]
    NotGitRepository,

    #[error("没有需要提交的更改")]
    NoChangesToCommit,

    #[error("GitHub API错误: {status} - {message}")]
    GitHubApiError { status: u16, message: String },

    #[error("需要认证: {message}")]
    AuthenticationRequired { message: String },

    #[error("缺少配置: {key}")]
    MissingConfiguration { key: String },

    #[error("参数错误: {message}")]
    InvalidArguments { message: String },
}

impl From<GitError> for ClaudeError {
    fn from(err: GitError) -> Self {
        ClaudeError::Command(format!("Git error: {}", err))
    }
}

/// 检查目录是否为Git仓库
pub fn is_git_repository(path: &Path) -> bool {
    Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .current_dir(path)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// 确保当前目录是Git仓库
pub fn ensure_git_repository(path: &Path) -> Result<()> {
    if !is_git_repository(path) {
        return Err(ClaudeError::Other("Not a git repository".to_string()));
    }
    Ok(())
}

/// 执行Git命令并返回输出
pub fn execute_git_command(path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .map_err(|e| ClaudeError::Other(format!("Git command failed: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(ClaudeError::Other(format!(
            "Git command failed: git {}",
            args.join(" ")
        )))
    }
}
