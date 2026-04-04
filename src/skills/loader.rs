//! 技能加载器
//!
//! 实现从不同来源加载技能的加载器

use std::path::PathBuf;
use std::fs;
use crate::error::Result;
use super::registry::SkillRegistry;
use super::types::{Skill, SkillMetadata, SkillCategory, SkillContext, SkillResult};
use super::registry::SkillLoader;

/// 内置技能加载器
pub struct BuiltinSkillLoader;

#[async_trait::async_trait]
impl SkillLoader for BuiltinSkillLoader {
    async fn load(&self, registry: &SkillRegistry) -> Result<()> {
        // 注册内置技能
        registry.register(HelpSkill).await;
        registry.register(ListSkillsSkill).await;
        registry.register(VersionSkill).await;
        registry.register(ConfigCheckSkill).await;

        tracing::debug!("Loaded {} builtin skills", 4);
        Ok(())
    }

    fn name(&self) -> &str {
        "builtin"
    }
}

/// 文件系统技能加载器
#[derive(Default)]
pub struct FileSystemSkillLoader {
    search_paths: Vec<PathBuf>,
}

impl FileSystemSkillLoader {
    /// 创建新的文件系统技能加载器
    pub fn new() -> Self {
        Self {
            search_paths: Self::default_search_paths(),
        }
    }

    /// 添加搜索路径
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// 获取默认搜索路径
    fn default_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 用户目录下的技能目录
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".claude-code").join("skills"));
        }

        // 当前目录下的技能目录
        paths.push(PathBuf::from("./.claude/skills"));
        paths.push(PathBuf::from("./skills"));

        paths
    }
}

#[async_trait::async_trait]
impl SkillLoader for FileSystemSkillLoader {
    async fn load(&self, registry: &SkillRegistry) -> Result<()> {
        let mut loaded_count = 0;

        for path in &self.search_paths {
            if path.exists() && path.is_dir() {
                if let Ok(count) = self.load_from_directory(registry, path).await {
                    loaded_count += count;
                }
            }
        }

        tracing::debug!("Loaded {} skills from filesystem", loaded_count);
        Ok(())
    }

    fn name(&self) -> &str {
        "filesystem"
    }
}

impl FileSystemSkillLoader {
    async fn load_from_directory(&self, _registry: &SkillRegistry, dir: &PathBuf) -> Result<usize> {
        let mut count = 0;

        // 扫描目录中的技能文件
        let entries = fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // 只处理 .rs 文件（Rust技能）和 .json 文件（技能定义）
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "rs" || ext == "json" {
                        // 这里可以添加技能文件解析逻辑
                        // 目前只记录找到了文件
                        tracing::debug!("Found skill file: {:?}", path);
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }
}

/// 内置技能：帮助技能
struct HelpSkill;

#[async_trait::async_trait]
impl Skill for HelpSkill {
    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            name: "help".to_string(),
            description: "显示帮助信息".to_string(),
            version: Some("1.0.0".to_string()),
            author: Some("Claude Code Team".to_string()),
            category: SkillCategory::Documentation,
            tags: vec!["help".to_string(), "manual".to_string(), "documentation".to_string()],
            input_schema: None,
            output_schema: None,
            required_permissions: Vec::new(),
            is_builtin: true,
            file_path: None,
            config: std::collections::HashMap::new(),
        }
    }

    async fn execute(&self, args: Option<&str>, _context: SkillContext) -> Result<SkillResult> {
        let help_text = match args {
            Some("skills") => "可用技能:\n- help: 显示帮助信息\n- list-skills: 列出所有技能\n- version: 显示版本信息\n- config-check: 检查配置",
            Some("commands") => "可用命令:\n- /help: 显示帮助\n- /skills: 列出技能\n- /version: 显示版本\n- /config: 检查配置",
            _ => "Claude Code Rust 技能系统\n\n用法:\n- /help [topic]: 显示帮助信息\n- /skills: 列出所有可用技能\n- /version: 显示版本信息\n- /config-check: 检查系统配置\n\n输入 /help skills 查看技能列表，输入 /help commands 查看命令列表",
        };

        Ok(SkillResult::success(
            "help".to_string(),
            serde_json::json!({
                "text": help_text,
                "format": "plain"
            }),
            0,
        ))
    }
}

/// 内置技能：列出技能
struct ListSkillsSkill;

#[async_trait::async_trait]
impl Skill for ListSkillsSkill {
    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            name: "list-skills".to_string(),
            description: "列出所有可用技能".to_string(),
            version: Some("1.0.0".to_string()),
            author: Some("Claude Code Team".to_string()),
            category: SkillCategory::Documentation,
            tags: vec!["list".to_string(), "skills".to_string(), "discovery".to_string()],
            input_schema: None,
            output_schema: None,
            required_permissions: Vec::new(),
            is_builtin: true,
            file_path: None,
            config: std::collections::HashMap::new(),
        }
    }

    async fn execute(&self, _args: Option<&str>, context: SkillContext) -> Result<SkillResult, crate::error::ClaudeError> {
        // 在实际实现中，这里应该查询技能注册表
        // 目前返回模拟数据
        let skills = vec![
            serde_json::json!({
                "name": "help",
                "description": "显示帮助信息",
                "category": "Documentation",
                "builtin": true
            }),
            serde_json::json!({
                "name": "list-skills",
                "description": "列出所有可用技能",
                "category": "Documentation",
                "builtin": true
            }),
            serde_json::json!({
                "name": "version",
                "description": "显示版本信息",
                "category": "Documentation",
                "builtin": true
            }),
            serde_json::json!({
                "name": "config-check",
                "description": "检查系统配置",
                "category": "ToolIntegration",
                "builtin": true
            }),
        ];

        Ok(SkillResult::success(
            "list-skills".to_string(),
            serde_json::json!({
                "skills": skills,
                "count": skills.len()
            }),
            0,
        ))
    }
}

/// 内置技能：版本信息
struct VersionSkill;

#[async_trait::async_trait]
impl Skill for VersionSkill {
    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            name: "version".to_string(),
            description: "显示版本信息".to_string(),
            version: Some("1.0.0".to_string()),
            author: Some("Claude Code Team".to_string()),
            category: SkillCategory::Documentation,
            tags: vec!["version".to_string(), "info".to_string()],
            input_schema: None,
            output_schema: None,
            required_permissions: Vec::new(),
            is_builtin: true,
            file_path: None,
            config: std::collections::HashMap::new(),
        }
    }

    async fn execute(&self, _args: Option<&str>, _context: SkillContext) -> Result<SkillResult, crate::error::ClaudeError> {
        let version_info = serde_json::json!({
            "claude_code_rust": env!("CARGO_PKG_VERSION"),
            "rust_version": option_env!("RUSTC_VERSION").unwrap_or(env!("CARGO_PKG_VERSION")),
            "build_time": option_env!("BUILD_TIMESTAMP").unwrap_or(env!("CARGO_PKG_VERSION")),
            "skill_system": "1.0.0"
        });

        Ok(SkillResult::success(
            "version".to_string(),
            version_info,
            0,
        ))
    }
}

/// 内置技能：配置检查
struct ConfigCheckSkill;

#[async_trait::async_trait]
impl Skill for ConfigCheckSkill {
    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            name: "config-check".to_string(),
            description: "检查系统配置".to_string(),
            version: Some("1.0.0".to_string()),
            author: Some("Claude Code Team".to_string()),
            category: SkillCategory::ToolIntegration,
            tags: vec!["config".to_string(), "check".to_string(), "diagnostics".to_string()],
            input_schema: None,
            output_schema: None,
            required_permissions: Vec::new(),
            is_builtin: true,
            file_path: None,
            config: std::collections::HashMap::new(),
        }
    }

    async fn execute(&self, _args: Option<&str>, context: SkillContext) -> Result<SkillResult, crate::error::ClaudeError> {
        let mut checks = Vec::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // 检查环境变量
        if context.get_env("HOME").is_some() {
            checks.push(serde_json::json!({
                "check": "HOME环境变量",
                "status": "ok",
                "value": "已设置"
            }));
        } else {
            warnings.push(serde_json::json!({
                "check": "HOME环境变量",
                "status": "warning",
                "message": "HOME环境变量未设置"
            }));
        }

        // 检查当前目录
        if context.cwd.exists() {
            checks.push(serde_json::json!({
                "check": "当前工作目录",
                "status": "ok",
                "value": context.cwd.display().to_string()
            }));
        } else {
            errors.push(serde_json::json!({
                "check": "当前工作目录",
                "status": "error",
                "message": "当前工作目录不存在"
            }));
        }

        // 检查项目根目录
        if context.project_root.exists() {
            checks.push(serde_json::json!({
                "check": "项目根目录",
                "status": "ok",
                "value": context.project_root.display().to_string()
            }));
        } else {
            warnings.push(serde_json::json!({
                "check": "项目根目录",
                "status": "warning",
                "message": "项目根目录不存在"
            }));
        }

        let result = serde_json::json!({
            "checks": checks,
            "warnings": warnings,
            "errors": errors,
            "summary": {
                "total": checks.len() + warnings.len() + errors.len(),
                "ok": checks.len(),
                "warnings": warnings.len(),
                "errors": errors.len()
            }
        });

        Ok(SkillResult::success(
            "config-check".to_string(),
            result,
            0,
        ))
    }
}