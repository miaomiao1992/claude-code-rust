//! 技能相关工具
//! 
//! 实现Skill Tool等技能执行工具

use crate::error::Result;
use async_trait::async_trait;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// Skill工具
/// 用于执行技能
pub struct SkillTool;

#[async_trait]
impl Tool for SkillTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("Skill", "Execute a skill within the main conversation")
            .category(ToolCategory::AgentSystem)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["skill".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("skill".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The skill name to execute"
                    })),
                    ("args".to_string(), serde_json::json!({ 
                        "type": "string",
                        "description": "Arguments to pass to the skill"
                    })),
                ])),
                required: Some(vec!["skill".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let skill = input["skill"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("skill is required".to_string()))?;
        
        let args = input["args"].as_str().unwrap_or("");
        
        // 注意：这里需要集成实际的技能系统
        // 由于技能系统的实现比较复杂，这里返回模拟结果
        // 实际实现时应该加载和执行相应的技能
        
        Ok(ToolResult::success(serde_json::json!({ 
            "skill": skill,
            "args": args,
            "result": "Skill executed successfully (mock implementation)",
            "note": "This is a mock implementation. In production, integrate with actual skill system.",
        })))
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["skill"].as_str().map(|s| format!("Executing skill '{}'", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_skill_metadata() {
        let tool = SkillTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "Skill");
        assert_eq!(metadata.category, ToolCategory::AgentSystem);
    }
}
