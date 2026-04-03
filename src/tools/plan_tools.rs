//! 规划相关工具
//! 
//! 实现EnterPlanMode Tool等规划工具

use crate::error::Result;
use async_trait::async_trait;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// EnterPlanMode工具
/// 用于进入规划模式，处理非平凡的实现任务
pub struct EnterPlanModeTool;

#[async_trait]
impl Tool for EnterPlanModeTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("EnterPlanMode", "Enter plan mode for non-trivial implementation tasks")
            .category(ToolCategory::TaskManagement)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["enterplanmode".to_string(), "plan".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("reason".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Reason for entering plan mode"
                    })),
                ])),
                required: Some(vec![]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let reason = input["reason"].as_str().unwrap_or("Non-trivial implementation task");
        
        // 注意：这里需要集成实际的规划模式系统
        // 由于规划模式系统的实现比较复杂，这里返回模拟结果
        // 实际实现时应该引导用户通过规划流程
        
        Ok(ToolResult::success(serde_json::json!({ 
            "reason": reason,
            "result": "Entered plan mode successfully (mock implementation)",
            "steps": [
                "Thoroughly explore codebase using Glob, Grep, Read",
                "Understand existing patterns/architecture",
                "Design implementation approach",
                "Present plan to user for approval",
                "Use AskUserQuestion for clarifications",
                "Exit plan mode with ExitPlanMode"
            ],
            "note": "This is a mock implementation. In production, integrate with actual plan mode system.",
        })))
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["reason"].as_str().map(|r| format!("Entering plan mode: {}", r)).or_else(|| Some("Entering plan mode".to_string()))
    }
}

/// ExitPlanMode工具
/// 用于退出规划模式
pub struct ExitPlanModeTool;

#[async_trait]
impl Tool for ExitPlanModeTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("ExitPlanMode", "Exit plan mode")
            .category(ToolCategory::TaskManagement)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["exitplanmode".to_string(), "exitplan".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("plan".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Finalized plan"
                    })),
                ])),
                required: Some(vec![]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let plan = input["plan"].as_str().unwrap_or("No plan provided");
        
        // 注意：这里需要集成实际的规划模式系统
        // 由于规划模式系统的实现比较复杂，这里返回模拟结果
        // 实际实现时应该退出规划模式并保存计划
        
        Ok(ToolResult::success(serde_json::json!({ 
            "plan": plan,
            "result": "Exited plan mode successfully (mock implementation)",
            "note": "This is a mock implementation. In production, integrate with actual plan mode system.",
        })))
    }
    
    fn get_activity_description(&self, _input: &serde_json::Value) -> Option<String> {
        Some("Exiting plan mode".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_enterplanmode_metadata() {
        let tool = EnterPlanModeTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "EnterPlanMode");
        assert_eq!(metadata.category, ToolCategory::TaskManagement);
    }
    
    #[test]
    fn test_exitplanmode_metadata() {
        let tool = ExitPlanModeTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "ExitPlanMode");
        assert_eq!(metadata.category, ToolCategory::TaskManagement);
    }
}
