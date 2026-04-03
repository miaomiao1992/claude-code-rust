//! 时间相关工具
//! 
//! 实现Sleep Tool等时间操作工具

use crate::error::Result;
use async_trait::async_trait;
use tokio::time;
use serde_json::Value;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// Sleep工具
/// 用于在执行过程中添加延迟
pub struct SleepTool;

#[async_trait]
impl Tool for SleepTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("Sleep", "Wait for a specified amount of time")
            .category(ToolCategory::Other)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["sleep".to_string(), "wait".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("seconds".to_string(), serde_json::json!({ 
                        "type": "number", 
                        "description": "Number of seconds to sleep"
                    })),
                    ("reason".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Optional reason for sleeping"
                    })),
                ])),
                required: Some(vec!["seconds".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let seconds = input["seconds"].as_f64()
            .ok_or_else(|| crate::error::ClaudeError::Tool("seconds is required".to_string()))?;
        
        let reason = input["reason"].as_str().unwrap_or("");
        
        // 验证seconds值
        if seconds < 0.0 {
            return Err(crate::error::ClaudeError::Tool("seconds must be non-negative".to_string()));
        }
        
        if seconds > 3600.0 { // 限制最大睡眠时间为1小时
            return Err(crate::error::ClaudeError::Tool("seconds must be less than 3600".to_string()));
        }
        
        // 执行睡眠
        let duration = time::Duration::from_secs_f64(seconds);
        time::sleep(duration).await;
        
        let result = serde_json::json!({ 
            "seconds": seconds,
            "reason": reason,
            "result": "Slept successfully",
        });
        
        Ok(ToolResult::success(result))
    }
    
    fn get_activity_description(&self, input: &Value) -> Option<String> {
        input["seconds"].as_f64().map(|secs| {
            input["reason"].as_str().map(|reason| {
                if !reason.is_empty() {
                    format!("Sleeping for {} seconds: {}", secs, reason)
                } else {
                    format!("Sleeping for {} seconds", secs)
                }
            }).unwrap_or_else(|| format!("Sleeping for {} seconds", secs))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;
    
    #[test]
    fn test_sleep_metadata() {
        let tool = SleepTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "Sleep");
        assert_eq!(metadata.category, ToolCategory::Other);
    }
    
    #[tokio::test]
    async fn test_sleep_execute() {
        use crate::config::Config;
        use crate::state::AppState;
        
        let tool = SleepTool;
        let input = serde_json::json!({ 
            "seconds": 0.1,
            "reason": "Testing sleep functionality"
        });
        
        let start = Instant::now();
        let context = ToolUseContext::new(
            std::path::PathBuf::from("."),
            Config::default(),
            AppState::default()
        );
        let result = tool.execute(input, context).await.unwrap();
        let elapsed = start.elapsed();
        
        assert!(result.error.is_none());
        let data = result.data;
        assert_eq!(data["seconds"], 0.1);
        assert!(elapsed.as_secs_f64() >= 0.1);
    }
    
    #[tokio::test]
    async fn test_sleep_negative_seconds() {
        use crate::config::Config;
        use crate::state::AppState;
        
        let tool = SleepTool;
        let input = serde_json::json!({ 
            "seconds": -1.0
        });
        
        let context = ToolUseContext::new(
            std::path::PathBuf::from("."),
            Config::default(),
            AppState::default()
        );
        let result = tool.execute(input, context).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_sleep_too_long() {
        use crate::config::Config;
        use crate::state::AppState;
        
        let tool = SleepTool;
        let input = serde_json::json!({ 
            "seconds": 3601.0
        });
        
        let context = ToolUseContext::new(
            std::path::PathBuf::from("."),
            Config::default(),
            AppState::default()
        );
        let result = tool.execute(input, context).await;
        assert!(result.is_err());
    }
}
