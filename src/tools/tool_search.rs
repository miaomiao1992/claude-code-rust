//! 工具搜索相关工具
//! 
//! 实现ToolSearch Tool等工具搜索功能

use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// ToolSearch工具
/// 用于延迟工具搜索，帮助用户发现可用的工具
pub struct ToolSearchTool;

#[async_trait]
impl Tool for ToolSearchTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("ToolSearch", "Search for available tools")
            .category(ToolCategory::AgentSystem)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["toolsearch".to_string(), "tools".to_string(), "searchtools".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("query".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Optional search query to filter tools"
                    })),
                    ("category".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Optional category to filter tools"
                    })),
                    ("permission_level".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Optional permission level to filter tools"
                    })),
                ])),
                required: Some(vec![]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let query = input["query"].as_str().unwrap_or("");
        let category = input["category"].as_str().unwrap_or("");
        let permission_level = input["permission_level"].as_str().unwrap_or("");
        
        // 模拟工具搜索结果
        // 实际实现中应该从ToolRegistry中获取真实的工具列表
        let mut tools = vec![
            serde_json::json!({ 
                "name": "Read",
                "category": "File",
                "permission_level": "Standard",
                "description": "Read a file"
            }),
            serde_json::json!({ 
                "name": "Edit",
                "category": "File",
                "permission_level": "Standard",
                "description": "Edit a file"
            }),
            serde_json::json!({ 
                "name": "Write",
                "category": "File",
                "permission_level": "Standard",
                "description": "Write to a file"
            }),
            serde_json::json!({ 
                "name": "Bash",
                "category": "Command",
                "permission_level": "Standard",
                "description": "Execute bash command"
            }),
            serde_json::json!({ 
                "name": "WebFetch",
                "category": "Web",
                "permission_level": "Standard",
                "description": "Fetch URL content"
            }),
            serde_json::json!({ 
                "name": "WebSearch",
                "category": "Web",
                "permission_level": "Standard",
                "description": "Search the web"
            }),
            serde_json::json!({ 
                "name": "Skill",
                "category": "System",
                "permission_level": "Standard",
                "description": "Execute a skill"
            }),
            serde_json::json!({ 
                "name": "SendMessage",
                "category": "System",
                "permission_level": "Standard",
                "description": "Send message to another agent"
            }),
            serde_json::json!({ 
                "name": "TaskCreate",
                "category": "System",
                "permission_level": "Standard",
                "description": "Create a task list"
            }),
            serde_json::json!({ 
                "name": "EnterPlanMode",
                "category": "System",
                "permission_level": "Standard",
                "description": "Enter plan mode"
            }),
            serde_json::json!({ 
                "name": "ExitPlanMode",
                "category": "System",
                "permission_level": "Standard",
                "description": "Exit plan mode"
            }),
            serde_json::json!({ 
                "name": "EnterWorktree",
                "category": "Git",
                "permission_level": "Standard",
                "description": "Enter git worktree"
            }),
            serde_json::json!({ 
                "name": "AskUserQuestion",
                "category": "UserInteraction",
                "permission_level": "Standard",
                "description": "Ask user a question"
            }),
            serde_json::json!({ 
                "name": "LSP",
                "category": "Development",
                "permission_level": "Standard",
                "description": "Language Server Protocol interaction"
            }),
            serde_json::json!({ 
                "name": "Sleep",
                "category": "System",
                "permission_level": "Standard",
                "description": "Wait for a specified amount of time"
            }),
            serde_json::json!({ 
                "name": "CronCreate",
                "category": "System",
                "permission_level": "Standard",
                "description": "Create scheduled cron jobs"
            }),
            serde_json::json!({ 
                "name": "TeamCreate",
                "category": "Team",
                "permission_level": "Standard",
                "description": "Create a new team"
            }),
            serde_json::json!({ 
                "name": "ToolSearch",
                "category": "System",
                "permission_level": "Standard",
                "description": "Search for available tools"
            }),
        ];
        
        // 过滤工具
        if !query.is_empty() {
            tools.retain(|tool| {
                tool["name"].as_str().unwrap_or("").to_lowercase().contains(&query.to_lowercase()) ||
                tool["description"].as_str().unwrap_or("").to_lowercase().contains(&query.to_lowercase())
            });
        }
        
        if !category.is_empty() {
            tools.retain(|tool| {
                tool["category"].as_str().unwrap_or("").to_lowercase() == category.to_lowercase()
            });
        }
        
        if !permission_level.is_empty() {
            tools.retain(|tool| {
                tool["permission_level"].as_str().unwrap_or("").to_lowercase() == permission_level.to_lowercase()
            });
        }
        
        let result = serde_json::json!({ 
            "query": query,
            "category": category,
            "permission_level": permission_level,
            "tools": tools,
            "total": tools.len(),
            "result": "Tool search completed successfully",
        });
        
        Ok(ToolResult::success(result))
    }
    
    fn get_activity_description(&self, input: &Value) -> Option<String> {
        Some(input["query"].as_str().map(|query| {
            if !query.is_empty() {
                format!("Searching for tools matching '{}'", query)
            } else {
                "Searching for all available tools".to_string()
            }
        }).unwrap_or_else(|| "Searching for all available tools".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_toolsearch_metadata() {
        let tool = ToolSearchTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "ToolSearch");
        assert_eq!(metadata.category, ToolCategory::AgentSystem);
    }
    
    #[tokio::test]
    async fn test_toolsearch_execute() {
        use crate::config::Config;
        use crate::state::AppState;
        
        let tool = ToolSearchTool;
        let input = serde_json::json!({ 
            "query": "file"
        });
        
        let context = ToolUseContext::new(
            std::path::PathBuf::from("."),
            Config::default(),
            AppState::default()
        );
        let result = tool.execute(input, context).await.unwrap();
        assert!(result.error.is_none());
        let data = result.data;
        assert_eq!(data["query"], "file");
        assert!(data["tools"].as_array().unwrap().len() > 0);
    }
    
    #[tokio::test]
    async fn test_toolsearch_execute_all() {
        use crate::config::Config;
        use crate::state::AppState;
        
        let tool = ToolSearchTool;
        let input = serde_json::json!({});
        
        let context = ToolUseContext::new(
            std::path::PathBuf::from("."),
            Config::default(),
            AppState::default()
        );
        let result = tool.execute(input, context).await.unwrap();
        assert!(result.error.is_none());
        let data = result.data;
        assert_eq!(data["query"], "");
        assert!(data["tools"].as_array().unwrap().len() > 0);
    }
}