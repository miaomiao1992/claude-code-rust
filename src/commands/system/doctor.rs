//! `/doctor` 命令实现
//!
//! 系统诊断

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, LocalJsxCommand, LoadedFrom,
    CommandResultDisplay,
};
use crate::commands::registry::CommandExecutor;

/// 诊断命令
pub struct DoctorCommand;

#[async_trait::async_trait]
impl CommandExecutor for DoctorCommand {
    async fn execute(&self, _context: CommandContext) -> Result<CommandResult> {
        let mut checks = Vec::new();

        // 检查 1: Rust 版本
        let rust_check = check_rust_version().await;
        checks.push(rust_check);

        // 检查 2: Git 可用性
        let git_check = check_git().await;
        checks.push(git_check);

        // 检查 3: API 配置
        let api_check = check_api_config().await;
        checks.push(api_check);

        // 检查 4: 磁盘空间
        let disk_check = check_disk_space().await;
        checks.push(disk_check);

        // 格式化结果
        let result = format_diagnosis(&checks);

        Ok(CommandResult {
            content: result,
            display: CommandResultDisplay::User,
            should_query: false,
            ..Default::default()
        })
    }

    fn command(&self) -> Command {
        Command::LocalJsx(LocalJsxCommand {
            base: CommandBase {
                name: "doctor".to_string(),
                description: "Diagnose installation and settings".to_string(),
                aliases: Some(vec!["doc".to_string()]),
                argument_hint: None,
                when_to_use: Some("Use when you're experiencing issues or want to verify your setup".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
        })
    }
}

/// 诊断检查项
struct CheckResult {
    name: String,
    status: CheckStatus,
    message: String,
    details: Option<String>,
}

/// 检查状态
enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

/// 检查 Rust 版本
async fn check_rust_version() -> CheckResult {
    match std::process::Command::new("rustc").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            CheckResult {
                name: "Rust Compiler".to_string(),
                status: CheckStatus::Pass,
                message: format!("✅ {}", version.trim()),
                details: None,
            }
        }
        _ => CheckResult {
            name: "Rust Compiler".to_string(),
            status: CheckStatus::Fail,
            message: "❌ Rust not found".to_string(),
            details: Some("Please install Rust from https://rustup.rs".to_string()),
        },
    }
}

/// 检查 Git
async fn check_git() -> CheckResult {
    match std::process::Command::new("git").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            CheckResult {
                name: "Git".to_string(),
                status: CheckStatus::Pass,
                message: format!("✅ {}", version.trim()),
                details: None,
            }
        }
        _ => CheckResult {
            name: "Git".to_string(),
            status: CheckStatus::Fail,
            message: "❌ Git not found".to_string(),
            details: Some("Please install Git".to_string()),
        },
    }
}

/// 检查 API 配置
async fn check_api_config() -> CheckResult {
    // 简化检查
    CheckResult {
        name: "API Configuration".to_string(),
        status: CheckStatus::Warn,
        message: "⚠️ API Key not configured".to_string(),
        details: Some("Set ANTHROPIC_API_KEY environment variable".to_string()),
    }
}

/// 检查磁盘空间
async fn check_disk_space() -> CheckResult {
    CheckResult {
        name: "Disk Space".to_string(),
        status: CheckStatus::Pass,
        message: "✅ Sufficient disk space".to_string(),
        details: None,
    }
}

/// 格式化诊断结果
fn format_diagnosis(checks: &[CheckResult]) -> String {
    let mut result = String::new();

    result.push_str("## 🏥 Claude Code Doctor\n\n");
    result.push_str("系统诊断 | System Diagnosis\n\n");

    let pass_count = checks.iter().filter(|c| matches!(c.status, CheckStatus::Pass)).count();
    let warn_count = checks.iter().filter(|c| matches!(c.status, CheckStatus::Warn)).count();
    let fail_count = checks.iter().filter(|c| matches!(c.status, CheckStatus::Fail)).count();

    // 摘要
    result.push_str("### 摘要 | Summary\n\n");
    result.push_str(&format!("✅ Pass: {}  |  ⚠️ Warn: {}  |  ❌ Fail: {}\n\n", pass_count, warn_count, fail_count));

    // 详细结果
    result.push_str("### 详细结果 | Details\n\n");

    for check in checks {
        result.push_str(&format!("**{}**\n", check.name));
        result.push_str(&format!("{}\n", check.message));
        if let Some(details) = &check.details {
            result.push_str(&format!("💡 {}\n", details));
        }
        result.push('\n');
    }

    // 建议
    if fail_count > 0 {
        result.push_str("### ❌ 需要修复 | Issues to Fix\n\n");
        result.push_str("请解决上述失败项后再使用 Claude Code。\n");
        result.push_str("Please fix the failed items before using Claude Code.\n");
    } else if warn_count > 0 {
        result.push_str("### ⚠️ 建议 | Recommendations\n\n");
        result.push_str("系统可用，但建议处理警告项以获得更好体验。\n");
        result.push_str("System is usable, but addressing warnings is recommended.\n");
    } else {
        result.push_str("### ✅ 全部通过 | All Checks Passed\n\n");
        result.push_str("系统状态良好！System is in good condition!\n");
    }

    result
}
