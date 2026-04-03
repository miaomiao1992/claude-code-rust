//! 任务相关工具
//! 
//! 实现TaskCreate Tool等任务管理工具

use crate::error::Result;
use async_trait::async_trait;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// TaskCreate工具
/// 用于创建结构化任务列表
pub struct TaskCreateTool;

#[async_trait]
impl Tool for TaskCreateTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("TaskCreate", "Create structured task list for current coding session")
            .category(ToolCategory::TaskManagement)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["taskcreate".to_string(), "task".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("subject".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Brief, actionable title in imperative form"
                    })),
                    ("description".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Detailed description"
                    })),
                    ("activeForm".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Present continuous form for spinner (e.g., \"Fixing authentication bug\")"
                    })),
                ])),
                required: Some(vec!["subject".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let subject = input["subject"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("subject is required".to_string()))?;
        
        let description = input["description"].as_str().unwrap_or("");
        let active_form = if let Some(form) = input["activeForm"].as_str() {
            form
        } else {
            subject
        };
        
        // 注意：这里需要集成实际的任务管理系统
        // 由于任务管理系统的实现比较复杂，这里返回模拟结果
        // 实际实现时应该创建并存储任务
        
        Ok(ToolResult::success(serde_json::json!({ 
            "subject": subject,
            "description": description,
            "activeForm": active_form,
            "result": "Task created successfully (mock implementation)",
            "note": "This is a mock implementation. In production, integrate with actual task management system.",
        })))
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["subject"].as_str().map(|s| format!("Creating task '{}'", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_taskcreate_metadata() {
        let tool = TaskCreateTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "TaskCreate");
        assert_eq!(metadata.category, ToolCategory::TaskManagement);
    }
}
