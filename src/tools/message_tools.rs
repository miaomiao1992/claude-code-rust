//! 消息相关工具
//! 
//! 实现SendMessage Tool等消息传递工具

use crate::error::Result;
use async_trait::async_trait;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// SendMessage工具
/// 用于向其他智能体发送消息
pub struct SendMessageTool;

#[async_trait]
impl Tool for SendMessageTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("SendMessage", "Send message to another agent")
            .category(ToolCategory::AgentSystem)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["sendmessage".to_string(), "send".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("to".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Recipient type: 'researcher' (by name), '*' (broadcast), 'uds:/path/to.sock' (local session), 'bridge:session_...' (remote control)"
                    })),
                    ("summary".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Brief summary of the message"
                    })),
                    ("message".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The message content"
                    })),
                ])),
                required: Some(vec!["to".to_string(), "message".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let to = input["to"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("to is required".to_string()))?;
        
        let summary = input["summary"].as_str().unwrap_or("");
        let message = input["message"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("message is required".to_string()))?;
        
        // 注意：这里需要集成实际的智能体通信系统
        // 由于智能体通信系统的实现比较复杂，这里返回模拟结果
        // 实际实现时应该根据接收者类型发送消息
        
        Ok(ToolResult::success(serde_json::json!({ 
            "to": to,
            "summary": summary,
            "message": message,
            "result": "Message sent successfully (mock implementation)",
            "note": "This is a mock implementation. In production, integrate with actual agent communication system.",
        })))
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["to"].as_str().map(|t| format!("Sending message to '{}'", t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sendmessage_metadata() {
        let tool = SendMessageTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "SendMessage");
        assert_eq!(metadata.category, ToolCategory::AgentSystem);
    }
}
