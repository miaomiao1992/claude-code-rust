//! `/security-review` 命令实现
//!
//! 安全代码审查

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, PromptCommand, LoadedFrom, CommandSource,
};
use crate::commands::registry::CommandExecutor;
use super::GitError;

/// 安全审查命令
pub struct SecurityReviewCommand;

#[async_trait::async_trait]
impl CommandExecutor for SecurityReviewCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        // 解析参数
        let args = parse_security_review_args(&context.args);

        // 检查Git仓库
        super::ensure_git_repository(&context.cwd)?;

        // 获取要审查的代码
        let code_to_review = get_code_for_review(&context.cwd, &args).await?;

        // 构建安全审查提示
        let prompt = build_security_review_prompt(&code_to_review, &args);

        // 返回结果
        Ok(CommandResult {
            content: prompt,
            should_query: true,  // 需要AI进行安全分析
            ..Default::default()
        })
    }

    fn command(&self) -> Command {
        Command::Prompt(PromptCommand {
            base: CommandBase {
                name: "security-review".to_string(),
                description: "Security code review".to_string(),
                aliases: Some(vec!["sec".to_string()]),
                argument_hint: Some("[files] [--diff <range>] [--severity <level>]".to_string()),
                when_to_use: Some("Use when you want to perform a security-focused code review".to_string()),
                loaded_from: Some(LoadedFrom::Bundled),
                immediate: Some(false),
                ..Default::default()
            },
            progress_message: "Analyzing code for security vulnerabilities...".to_string(),
            content_length: 3500,
            arg_names: Some(vec!["files".to_string(), "diff_range".to_string(), "severity".to_string()]),
            allowed_tools: Some(vec!["git".to_string(), "read_file".to_string()]),
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

/// 安全审查参数
struct SecurityReviewArgs {
    files: Vec<String>,
    diff_range: Option<String>,
    severity: SecuritySeverity,
    include_false_positives: bool,
    limit_files: usize,
}

/// 安全严重级别
#[derive(Debug, Clone, Copy)]
enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl SecuritySeverity {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => SecuritySeverity::Critical,
            "high" => SecuritySeverity::High,
            "medium" => SecuritySeverity::Medium,
            "low" => SecuritySeverity::Low,
            "info" => SecuritySeverity::Info,
            _ => SecuritySeverity::Medium, // 默认
        }
    }
}

/// 解析安全审查命令参数
fn parse_security_review_args(args: &str) -> SecurityReviewArgs {
    let mut files = Vec::new();
    let mut diff_range = None;
    let mut severity = SecuritySeverity::Medium;
    let mut include_false_positives = false;
    let mut limit_files = 20; // 默认限制文件数

    let mut args_iter = args.split_whitespace().peekable();

    while let Some(arg) = args_iter.next() {
        match arg {
            "--diff" | "-d" => {
                if let Some(range) = args_iter.next() {
                    diff_range = Some(range.to_string());
                }
            }
            "--severity" | "-s" => {
                if let Some(sev) = args_iter.next() {
                    severity = SecuritySeverity::from_str(sev);
                }
            }
            "--include-fp" | "-f" => {
                include_false_positives = true;
            }
            "--limit" | "-l" => {
                if let Some(limit) = args_iter.next() {
                    if let Ok(num) = limit.parse() {
                        limit_files = num;
                    }
                }
            }
            arg if arg.starts_with("--") => {
                // 忽略其他选项
            }
            _ => {
                files.push(arg.to_string());
            }
        }
    }

    SecurityReviewArgs {
        files,
        diff_range,
        severity,
        include_false_positives,
        limit_files,
    }
}

/// 要审查的代码
struct CodeForReview {
    files: Vec<CodeFile>,
    diff: Option<String>,
    summary: String,
}

/// 代码文件
struct CodeFile {
    path: String,
    content: String,
    language: Option<String>,
}

/// 获取要审查的代码
async fn get_code_for_review(
    cwd: &std::path::Path,
    args: &SecurityReviewArgs,
) -> Result<CodeForReview> {
    let mut files = Vec::new();

    if args.files.is_empty() && args.diff_range.is_none() {
        // 如果没有指定文件或差异范围，获取所有更改的文件
        let changed_files = get_changed_files(cwd).await?;

        for file_path in changed_files.into_iter().take(args.limit_files) {
            if let Ok(content) = std::fs::read_to_string(cwd.join(&file_path)) {
                let language = detect_language(&file_path);
                files.push(CodeFile {
                    path: file_path,
                    content,
                    language,
                });
            }
        }
    } else if !args.files.is_empty() {
        // 获取指定文件
        for file_path in args.files.iter().take(args.limit_files) {
            if let Ok(content) = std::fs::read_to_string(cwd.join(file_path)) {
                let language = detect_language(file_path);
                files.push(CodeFile {
                    path: file_path.clone(),
                    content,
                    language,
                });
            }
        }
    }

    // 获取差异（如果指定）
    let diff = if let Some(range) = &args.diff_range {
        Some(get_git_diff_range(cwd, range).await?)
    } else {
        None
    };

    // 构建摘要
    let summary = build_code_summary(&files, diff.as_deref());

    Ok(CodeForReview {
        files,
        diff,
        summary,
    })
}

/// 获取更改的文件
async fn get_changed_files(cwd: &std::path::Path) -> Result<Vec<String>> {
    let status_output = super::execute_git_command(cwd, &["status", "--porcelain"])?;

    let files: Vec<String> = status_output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.len() > 3 {
                Some(line[3..].to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(files)
}

/// 检测语言
fn detect_language(file_path: &str) -> Option<String> {
    if file_path.ends_with(".rs") {
        Some("rust".to_string())
    } else if file_path.ends_with(".js") || file_path.ends_with(".ts") {
        Some("javascript".to_string())
    } else if file_path.ends_with(".py") {
        Some("python".to_string())
    } else if file_path.ends_with(".java") {
        Some("java".to_string())
    } else if file_path.ends_with(".go") {
        Some("go".to_string())
    } else if file_path.ends_with(".cpp") || file_path.ends_with(".h") {
        Some("cpp".to_string())
    } else {
        None
    }
}

/// 获取Git差异范围
async fn get_git_diff_range(cwd: &std::path::Path, range: &str) -> Result<String> {
    super::execute_git_command(cwd, &["diff", range]).map_err(Into::into)
}

/// 构建代码摘要
fn build_code_summary(files: &[CodeFile], diff: Option<&str>) -> String {
    let mut summary = String::new();

    summary.push_str(&format!("Files to review: {}\n", files.len()));

    let mut by_language = std::collections::HashMap::new();
    for file in files {
        let lang = file.language.as_deref().unwrap_or("unknown");
        *by_language.entry(lang).or_insert(0) += 1;
    }

    if !by_language.is_empty() {
        summary.push_str("Languages: ");
        let langs: Vec<String> = by_language
            .iter()
            .map(|(lang, count)| format!("{} ({})", lang, count))
            .collect();
        summary.push_str(&langs.join(", "));
        summary.push('\n');
    }

    if let Some(diff) = diff {
        let lines = diff.lines().count();
        let insertions = diff.lines().filter(|l| l.starts_with('+')).count();
        let deletions = diff.lines().filter(|l| l.starts_with('-')).count();
        summary.push_str(&format!(
            "Diff: {} lines ({} insertions, {} deletions)\n",
            lines, insertions, deletions
        ));
    }

    summary
}

/// 构建安全审查提示
fn build_security_review_prompt(code: &CodeForReview, args: &SecurityReviewArgs) -> String {
    let mut prompt = String::new();

    prompt.push_str("# Security Code Review\n\n");

    // 审查参数
    prompt.push_str("## Review Parameters\n\n");
    prompt.push_str(&format!("**Severity Level**: {:?}\n", args.severity));
    prompt.push_str(&format!("**Include False Positives**: {}\n", args.include_false_positives));
    prompt.push_str(&format!("**File Limit**: {}\n", args.limit_files));

    // 代码摘要
    prompt.push_str("\n## Code Summary\n\n");
    prompt.push_str(&code.summary);

    // 显示代码内容
    if !code.files.is_empty() {
        prompt.push_str("\n## Code to Review\n\n");

        for file in &code.files {
            prompt.push_str(&format!("### File: {}\n", file.path));

            if let Some(lang) = &file.language {
                prompt.push_str(&format!("```{}\n", lang));
            } else {
                prompt.push_str("```\n");
            }

            // 限制代码行数
            let lines: Vec<&str> = file.content.lines().collect();
            if lines.len() > 100 {
                prompt.push_str("// File is large, showing first 100 lines only\n");
                for line in lines.iter().take(100) {
                    prompt.push_str(line);
                    prompt.push('\n');
                }
                prompt.push_str(&format!("// ... and {} more lines\n", lines.len() - 100));
            } else {
                prompt.push_str(&file.content);
                if !file.content.ends_with('\n') {
                    prompt.push('\n');
                }
            }

            prompt.push_str("```\n\n");
        }
    }

    // 显示差异
    if let Some(diff) = &code.diff {
        prompt.push_str("\n## Git Diff\n\n");

        if diff.lines().count() > 200 {
            prompt.push_str("```diff\n");
            // 显示差异摘要
            let file_changes = diff.lines().filter(|l| l.starts_with("diff --git")).count();
            let insertions = diff.lines().filter(|l| l.starts_with('+')).count();
            let deletions = diff.lines().filter(|l| l.starts_with('-')).count();

            prompt.push_str(&format!(
                "# Diff summary: {} files changed, {} insertions(+), {} deletions(-)\n",
                file_changes, insertions, deletions
            ));

            // 显示关键部分
            let mut lines_shown = 0;
            for line in diff.lines().take(100) {
                if line.starts_with("diff --git") || line.starts_with("@") || line.starts_with('+') || line.starts_with('-') {
                    prompt.push_str(line);
                    prompt.push('\n');
                    lines_shown += 1;
                }
                if lines_shown >= 50 {
                    break;
                }
            }

            if diff.lines().count() > 100 {
                prompt.push_str("# ... diff truncated for brevity\n");
            }
            prompt.push_str("```\n");
        } else {
            prompt.push_str("```diff\n");
            prompt.push_str(diff);
            if !diff.ends_with('\n') {
                prompt.push('\n');
            }
            prompt.push_str("```\n");
        }
    }

    // 安全审查指导
    prompt.push_str("\n## Security Review Guidelines\n\n");

    prompt.push_str("Please perform a thorough security review focusing on:\n\n");

    match args.severity {
        SecuritySeverity::Critical => {
            prompt.push_str("### Critical Severity Focus\n");
            prompt.push_str("1. **Remote Code Execution (RCE)** vulnerabilities\n");
            prompt.push_str("2. **SQL Injection** and NoSQL injection\n");
            prompt.push_str("3. **Authentication/Authorization** bypasses\n");
            prompt.push_str("4. **Sensitive data exposure** (tokens, keys, PII)\n");
            prompt.push_str("5. **Path traversal** and file inclusion vulnerabilities\n");
        }
        SecuritySeverity::High => {
            prompt.push_str("### High Severity Focus\n");
            prompt.push_str("1. **Cross-Site Scripting (XSS)** vulnerabilities\n");
            prompt.push_str("2. **CSRF** and SSRF vulnerabilities\n");
            prompt.push_str("3. **Insecure deserialization**\n");
            prompt.push_str("4. **XXE** (XML External Entity) attacks\n");
            prompt.push_str("5. **Broken access control**\n");
        }
        SecuritySeverity::Medium => {
            prompt.push_str("### Medium Severity Focus\n");
            prompt.push_str("1. **Information disclosure** through error messages\n");
            prompt.push_str("2. **Insecure dependencies** (check versions)\n");
            prompt.push_str("3. **Weak cryptography** or improper usage\n");
            prompt.push_str("4. **Business logic flaws**\n");
            prompt.push_str("5. **Insufficient logging/monitoring**\n");
        }
        SecuritySeverity::Low => {
            prompt.push_str("### Low Severity Focus\n");
            prompt.push_str("1. **Best practice violations**\n");
            prompt.push_str("2. **Code quality issues** with security implications\n");
            prompt.push_str("3. **Defense in depth** improvements\n");
            prompt.push_str("4. **Security headers** and configurations\n");
        }
        SecuritySeverity::Info => {
            prompt.push_str("### Informational Review\n");
            prompt.push_str("1. **General security posture**\n");
            prompt.push_str("2. **Architectural security considerations**\n");
            prompt.push_str("3. **Future security improvements**\n");
        }
    }

    prompt.push_str("\n## Output Format\n\n");
    prompt.push_str("Please provide findings in this format:\n");
    prompt.push_str("1. **Severity**: Critical/High/Medium/Low/Info\n");
    prompt.push_str("2. **Vulnerability Type**: (e.g., XSS, SQLi, etc.)\n");
    prompt.push_str("3. **Location**: File and line number\n");
    prompt.push_str("4. **Description**: What the vulnerability is\n");
    prompt.push_str("5. **Impact**: Potential consequences\n");
    prompt.push_str("6. **Remediation**: How to fix it\n");
    prompt.push_str("7. **Confidence**: High/Medium/Low\n");

    if args.include_false_positives {
        prompt.push_str("\n**Note**: Include potential false positives with lower confidence ratings.\n");
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_security_review_args() {
        let args = parse_security_review_args("file1.rs file2.rs --severity high --limit 10");
        assert_eq!(args.files, vec!["file1.rs", "file2.rs"]);
        match args.severity {
            SecuritySeverity::High => (),
            _ => panic!("Expected High severity"),
        }
        assert_eq!(args.limit_files, 10);
        assert!(!args.include_false_positives);
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("file.rs"), Some("rust".to_string()));
        assert_eq!(detect_language("script.js"), Some("javascript".to_string()));
        assert_eq!(detect_language("unknown.txt"), None);
    }

    #[test]
    fn test_build_security_review_prompt() {
        let code = CodeForReview {
            files: vec![CodeFile {
                path: "test.rs".to_string(),
                content: "fn main() {}".to_string(),
                language: Some("rust".to_string()),
            }],
            diff: None,
            summary: "Summary".to_string(),
        };

        let args = SecurityReviewArgs {
            files: vec!["test.rs".to_string()],
            diff_range: None,
            severity: SecuritySeverity::Medium,
            include_false_positives: false,
            limit_files: 20,
        };

        let prompt = build_security_review_prompt(&code, &args);
        assert!(prompt.contains("Security Code Review"));
        assert!(prompt.contains("test.rs"));
        assert!(prompt.contains("fn main() {}"));
        assert!(prompt.contains("Security Review Guidelines"));
        assert!(prompt.contains("Output Format"));
    }
}