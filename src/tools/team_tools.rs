//! 团队相关工具
//! 
//! 实现TeamCreate Tool等团队管理工具

use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// TeamCreate工具
/// 用于创建团队
pub struct TeamCreateTool;

#[async_trait]
impl Tool for TeamCreateTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("TeamCreate", "Create a new team")
            .category(ToolCategory::Collaboration)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["teamcreate".to_string(), "team".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("name".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Team name"
                    })),
                    ("members".to_string(), serde_json::json!({ 
                        "type": "array", 
                        "items": { "type": "string" },
                        "description": "List of team members"
                    })),
                    ("description".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Optional team description"
                    })),
                    ("project".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Optional project associated with the team"
                    })),
                ])),
                required: Some(vec!["name".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let name = input["name"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("name is required".to_string()))?;
        
        let members = if let Some(members_array) = input.get("members").and_then(|m| m.as_array()) {
            members_array.iter().filter_map(|m| m.as_str()).collect::<Vec<&str>>()
        } else {
            Vec::new()
        };
        
        let description = input["description"].as_str().unwrap_or("");
        let project = input["project"].as_str().unwrap_or("");
        
        // 模拟创建团队（实际实现中应该与团队管理服务交互）
        let result = serde_json::json!({ 
            "name": name,
            "members": members,
            "description": description,
            "project": project,
            "status": "created",
            "result": "Team created successfully",
        });
        
        Ok(ToolResult::success(result))
    }
    
    fn get_activity_description(&self, input: &Value) -> Option<String> {
        input["name"].as_str().map(|name| {
            format!("Creating team '{}'", name)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_teamcreate_metadata() {
        let tool = TeamCreateTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "TeamCreate");
        assert_eq!(metadata.category, ToolCategory::Collaboration);
    }
    
    #[tokio::test]
    async fn test_teamcreate_execute() {
        use crate::config::Config;
        use crate::state::AppState;
        
        let tool = TeamCreateTool;
        let input = serde_json::json!({ 
            "name": "Development Team",
            "members": ["user1", "user2", "user3"],
            "description": "Core development team",
            "project": "Claude Code"
        });
        
        let context = ToolUseContext::new(
            std::path::PathBuf::from("."),
            Config::default(),
            AppState::default()
        );
        let result = tool.execute(input, context).await.unwrap();
        assert!(result.error.is_none());
        let data = result.data;
        assert_eq!(data["name"], "Development Team");
        assert_eq!(data["members"].as_array().unwrap().len(), 3);
        assert_eq!(data["description"], "Core development team");
        assert_eq!(data["project"], "Claude Code");
    }
    
    #[tokio::test]
    async fn test_teamcreate_minimal() {
        use crate::config::Config;
        use crate::state::AppState;
        
        let tool = TeamCreateTool;
        let input = serde_json::json!({ 
            "name": "Test Team"
        });
        
        let context = ToolUseContext::new(
            std::path::PathBuf::from("."),
            Config::default(),
            AppState::default()
        );
        let result = tool.execute(input, context).await.unwrap();
        assert!(result.error.is_none());
        let data = result.data;
        assert_eq!(data["name"], "Test Team");
        assert_eq!(data["members"].as_array().unwrap().len(), 0);
    }
}
