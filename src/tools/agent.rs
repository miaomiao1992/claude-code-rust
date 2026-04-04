//! Agent tools - call subagents to perform tasks
//!
//! This tool allows the main agent to fork subagents for parallel
//! or specialized task execution.

use crate::error::Result;
use crate::agents::{AgentManager, AgentExecutor};
use crate::tools::{Tool, ToolInputSchema, ToolResult, ToolUseContext};
use api_client::ApiClient;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Agent tool
#[derive(Debug, Clone)]
pub struct AgentTool {
    /// Agent manager
    agent_manager: Arc<AgentManager>,
    /// API client
    api_client: Arc<ApiClient>,
    /// Tool manager
    tool_manager: Arc<crate::tools::ToolManager>,
}

impl AgentTool {
    /// Create a new agent tool
    pub fn new(
        agent_manager: Arc<AgentManager>,
        api_client: Arc<ApiClient>,
        tool_manager: Arc<crate::tools::ToolManager>,
    ) -> Self {
        Self {
            agent_manager,
            api_client,
            tool_manager,
        }
    }
}

#[async_trait]
impl Tool for AgentTool {
    fn name(&self) -> &str {
        "agent"
    }
    
    fn description(&self) -> &str {
        "Call an AI agent to perform a task"
    }
    
    fn input_schema(&self) -> ToolInputSchema {
        let mut properties = HashMap::new();
        properties.insert(
            "agent_name".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Name of the agent to call",
            }),
        );
        properties.insert(
            "task".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Task for the agent to perform",
            }),
        );
        
        ToolInputSchema {
            r#type: "object".to_string(),
            properties,
            required: vec!["agent_name".to_string(), "task".to_string()],
        }
    }
    
    async fn execute(&self, input: serde_json::Value, _context: ToolUseContext) -> Result<ToolResult> {
        // Parse input
        let agent_name = input["agent_name"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("agent_name is required".to_string()))?;
        let task = input["task"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("task is required".to_string()))?;

        // Create executor
        let executor = self.agent_manager
            .create_executor(
                agent_name,
                self.api_client.clone(),
                self.tool_manager.clone(),
            )
            .await
            .ok_or_else(|| crate::error::ClaudeError::Tool(
                format!("Agent '{}' not found", agent_name)
            ))?;

        // Execute
        let result = executor.execute(task.to_string()).await?;

        // Format output
        let output = result.messages.join("\n\n");

        if output.is_empty() {
            Ok(ToolResult::success(
                "Agent executed successfully with no output".to_string()
            ))
        } else {
            Ok(ToolResult::success(output))
        }
    }
}
