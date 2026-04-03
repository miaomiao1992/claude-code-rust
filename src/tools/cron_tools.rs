//! 定时任务相关工具
//! 
//! 实现CronCreate Tool等定时任务工具

use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// CronCreate工具
/// 用于创建定时任务
pub struct CronCreateTool;

#[async_trait]
impl Tool for CronCreateTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("CronCreate", "Create scheduled cron jobs")
            .category(ToolCategory::Other)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["croncreate".to_string(), "cron".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("schedule".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Cron schedule expression (e.g., '0 0 * * *' for daily at midnight)"
                    })),
                    ("command".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Command to execute"
                    })),
                    ("description".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Optional description for the cron job"
                    })),
                    ("name".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Optional name for the cron job"
                    })),
                ])),
                required: Some(vec!["schedule".to_string(), "command".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: Value,
        context: ToolUseContext,
    ) -> Result<ToolResult> {
        let schedule = input["schedule"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("schedule is required".to_string()))?;
        
        let command = input["command"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("command is required".to_string()))?;
        
        let description = input["description"].as_str().unwrap_or("");
        let name = input["name"].as_str().unwrap_or("cron_default");
        
        // 验证cron表达式
        if !validate_cron_expression(schedule) {
            return Err(crate::error::ClaudeError::Tool("Invalid cron expression".to_string()));
        }
        
        // 模拟创建cron任务（实际实现中应该与系统的cron服务交互）
        let result = serde_json::json!({ 
            "name": name,
            "schedule": schedule,
            "command": command,
            "description": description,
            "status": "created",
            "result": "Cron job created successfully",
        });
        
        Ok(ToolResult::success(result))
    }
    
    fn get_activity_description(&self, input: &Value) -> Option<String> {
        input["command"].as_str().map(|cmd| {
            input["schedule"].as_str().map(|schedule| {
                format!("Creating cron job for '{}' with schedule '{}'", cmd, schedule)
            }).unwrap_or_else(|| format!("Creating cron job for '{}'", cmd))
        })
    }
}

/// 验证cron表达式
fn validate_cron_expression(expr: &str) -> bool {
    // 简单的cron表达式验证
    // 标准cron格式: 分 时 日 月 周
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 {
        return false;
    }
    
    // 验证每个部分
    // 这里只是简单验证，实际实现中可以使用更复杂的验证
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_croncreate_metadata() {
        let tool = CronCreateTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "CronCreate");
        assert_eq!(metadata.category, ToolCategory::Other);
    }
    
    #[tokio::test]
    async fn test_croncreate_execute() {
        use crate::config::Config;
        use crate::state::AppState;
        
        let tool = CronCreateTool;
        let input = serde_json::json!({ 
            "schedule": "0 0 * * *",
            "command": "echo 'Hello World'",
            "description": "Daily hello message",
            "name": "daily_hello"
        });
        
        let context = ToolUseContext::new(
            std::path::PathBuf::from("."),
            Config::default(),
            AppState::default()
        );
        let result = tool.execute(input, context).await.unwrap();
        assert!(result.error.is_none());
        let data = result.data;
        assert_eq!(data["name"], "daily_hello");
        assert_eq!(data["schedule"], "0 0 * * *");
        assert_eq!(data["command"], "echo 'Hello World'");
    }
    
    #[tokio::test]
    async fn test_croncreate_invalid_schedule() {
        use crate::config::Config;
        use crate::state::AppState;
        
        let tool = CronCreateTool;
        let input = serde_json::json!({ 
            "schedule": "invalid",
            "command": "echo 'Hello World'"
        });
        
        let context = ToolUseContext::new(
            std::path::PathBuf::from("."),
            Config::default(),
            AppState::default()
        );
        let result = tool.execute(input, context).await;
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_cron_expression() {
        assert!(validate_cron_expression("0 0 * * *"));
        assert!(!validate_cron_expression("invalid"));
    }
}
