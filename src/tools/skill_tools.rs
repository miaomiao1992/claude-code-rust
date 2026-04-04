//! 技能相关工具
//!
//! 实现Skill Tool等技能执行工具，集成真正的技能系统
//!
//! 注意：此模块需要启用 "full" feature 才能使用完整功能

// 当 full feature 未启用时，提供一个空的实现
#[cfg(not(feature = "full"))]
pub struct SkillTool;

#[cfg(not(feature = "full"))]
impl SkillTool {
    pub fn new() -> Self { Self }
}

// 当 full feature 启用时，提供完整的技能工具功能
#[cfg(feature = "full")]
use crate::error::Result;
#[cfg(feature = "full")]
use async_trait::async_trait;
#[cfg(feature = "full")]
use crate::skills;
#[cfg(feature = "full")]
use crate::skills::executor::SkillContextBuilder;
#[cfg(feature = "full")]
use super::base::{Tool, ToolBuilder};
#[cfg(feature = "full")]
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// Skill工具
/// 用于执行技能
#[cfg(feature = "full")]
pub struct SkillTool;

#[cfg(feature = "full")]
impl Default for SkillTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "full")]
impl SkillTool {
    /// 创建新的SkillTool实例
    pub fn new() -> Self {
        Self
    }

    /// 获取工具元数据
    pub fn metadata() -> ToolMetadata {
        ToolMetadata {
            name: "Skill".to_string(),
            description: "Execute skills to perform specialized tasks".to_string(),
            input_schema: ToolInputSchema::Object {
                properties: vec![
                    ("name".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Name of the skill to execute"
                    })),
                    ("args".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Arguments for the skill (optional)"
                    }))
                ],
                required: vec!["name".to_string()],
            },
            category: ToolCategory::Utility,
            permission_level: ToolPermissionLevel::Normal,
            requires_approval: false,
            is_deprecated: false,
        }
    }
}

#[cfg(feature = "full")]
#[async_trait]
impl Tool for SkillTool {
    fn metadata(&self) -> ToolMetadata {
        Self::metadata()
    }

    async fn execute(
        &self,
        input: serde_json::Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let skill_name = input["name"].as_str().unwrap_or("");
        let args = input["args"].as_str();

        if skill_name.is_empty() {
            return Ok(ToolResult::error("Skill name is required"));
        }

        // 初始化技能系统
        let manager = match skills::init().await {
            Ok(m) => m,
            Err(e) => return Ok(ToolResult::error(format!("Failed to initialize skill system: {}", e))),
        };

        // 查找技能
        let registry = manager.registry();
        let all_skills = registry.list().await;

        let skill = all_skills.iter()
            .find(|s| s.metadata().name == skill_name);

        match skill {
            Some(s) => {
                // 构建上下文
                let context_builder = SkillContextBuilder::new();
                let context = context_builder.build();

                // 执行技能
                match s.execute(args, context).await {
                    Ok(result) => Ok(ToolResult::success(serde_json::Value::String(
                        format!("Skill '{}' executed successfully", skill_name)
                    ))),
                    Err(e) => Ok(ToolResult::error(format!("Failed to execute skill: {}", e))),
                }
            }
            None => Ok(ToolResult::error(format!("Skill '{}' not found", skill_name))),
        }
    }

    async fn execute_with_options(
        &self,
        input: serde_json::Value,
        _context: ToolUseContext,
        options: super::types::ToolExecutionOptions,
    ) -> Result<ToolResult> {
        if options.dry_run {
            return Ok(ToolResult::success(serde_json::Value::String(
                format!("Would execute skill: {}", input.get("name").and_then(|v| v.as_str()).unwrap_or("unknown"))
            )));
        }

        self.execute(input, _context).await
    }
}

/// 技能列表工具
/// 用于列出可用的技能

#[cfg(not(feature = "full"))]
pub struct ListSkillsTool;

#[cfg(not(feature = "full"))]
impl ListSkillsTool {
    pub fn new() -> Self { Self }
}

#[cfg(feature = "full")]
pub struct ListSkillsTool;

#[cfg(feature = "full")]
impl Default for ListSkillsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "full")]
impl ListSkillsTool {
    /// 创建新的ListSkillsTool实例
    pub fn new() -> Self {
        Self
    }

    /// 获取工具元数据
    pub fn metadata() -> ToolMetadata {
        ToolMetadata {
            name: "list_skills".to_string(),
            description: "List all available skills with their descriptions and categories".to_string(),
            input_schema: ToolInputSchema::Object {
                properties: vec![
                    ("category".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Filter by category (optional)"
                    })),
                    ("query".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Search query (optional)"
                    }))
                ],
                required: vec![],
            },
            category: ToolCategory::Utility,
            permission_level: ToolPermissionLevel::Normal,
            requires_approval: false,
            is_deprecated: false,
        }
    }
}

#[cfg(feature = "full")]
#[async_trait]
impl Tool for ListSkillsTool {
    fn metadata(&self) -> ToolMetadata {
        Self::metadata()
    }

    async fn execute(
        &self,
        input: serde_json::Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let query = input["query"].as_str();
        let category = input["category"].as_str();

        // 初始化技能系统
        let manager = match skills::init().await {
            Ok(m) => m,
            Err(e) => return Ok(ToolResult::error(format!("Failed to initialize skill system: {}", e))),
        };

        // 获取所有技能
        let registry = manager.registry();
        let all_skills = registry.list().await;

        // 过滤技能
        let filtered_skills: Vec<_> = all_skills.iter()
            .filter(|skill| {
                let metadata = skill.metadata();

                // 应用查询过滤
                let query_match = query.map(|q| {
                    metadata.name.to_lowercase().contains(&q.to_lowercase()) ||
                    metadata.description.to_lowercase().contains(&q.to_lowercase()) ||
                    metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&q.to_lowercase()))
                }).unwrap_or(true);

                // 应用分类过滤
                let category_match = category.map(|c| {
                    metadata.category == c || c.is_empty()
                }).unwrap_or(true);

                query_match && category_match
            })
            .collect();

        // 格式化结果
        let mut result = String::new();
        result.push_str("Available Skills:\n");
        result.push_str("================\n\n");

        if filtered_skills.is_empty() {
            result.push_str("No skills found matching your criteria.\n");
        } else {
            for skill in &filtered_skills {
                let metadata = skill.metadata();

                result.push_str(&format!("**{}** ({})\n", metadata.name, metadata.category));
                result.push_str(&format!("  {}\n\n", metadata.description));

                if !metadata.tags.is_empty() {
                    result.push_str(&format!("  Tags: {}\n\n", metadata.tags.join(", ")));
                }
            }
        }

        result.push_str(&format!("\nTotal: {} skill(s)\n", filtered_skills.len()));

        Ok(ToolResult::success(serde_json::Value::String(result)))
    }

    async fn execute_with_options(
        &self,
        input: serde_json::Value,
        _context: ToolUseContext,
        options: super::types::ToolExecutionOptions,
    ) -> Result<ToolResult> {
        if options.dry_run {
            return Ok(ToolResult::success(serde_json::Value::String(
                "Would list available skills".to_string()
            )));
        }

        self.execute(input, _context).await
    }
}
