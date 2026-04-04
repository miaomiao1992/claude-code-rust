//! API客户端与工具系统集成层
//!
//! 提供本地工具系统与Claude API工具调用之间的桥梁。

use crate::error::Result;
use crate::tool_use::{ToolCall, ToolCallHandler, ToolDefinition, ToolResult};
use crate::types::{ApiTool, ApiContentBlock};
use serde_json::Value;
use std::sync::Arc;

/// 工具到API工具的转换器
pub trait ToolToApiConverter: Send + Sync {
    /// 将本地工具转换为API工具定义
    fn tool_to_api(&self, tool_name: &str, tool_description: &str, input_schema: Value) -> ApiTool;

    /// 将API工具调用转换为本地工具调用
    fn api_to_tool_call(&self, api_tool_call: &ApiContentBlock) -> Option<ToolCall>;

    /// 将本地工具结果转换为API工具结果
    fn tool_result_to_api(&self, tool_result: &ToolResult) -> ApiContentBlock;
}

/// 默认工具转换器
pub struct DefaultToolConverter;

impl ToolToApiConverter for DefaultToolConverter {
    fn tool_to_api(&self, tool_name: &str, tool_description: &str, input_schema: Value) -> ApiTool {
        ApiTool {
            name: tool_name.to_string(),
            description: Some(tool_description.to_string()),
            input_schema: input_schema,
        }
    }

    fn api_to_tool_call(&self, api_tool_call: &ApiContentBlock) -> Option<ToolCall> {
        match api_tool_call {
            ApiContentBlock::ToolUse { id, name, input } => Some(ToolCall {
                id: id.clone(),
                name: name.clone(),
                input: input.clone(),
                tool: None,
            }),
            _ => None,
        }
    }

    fn tool_result_to_api(&self, tool_result: &ToolResult) -> ApiContentBlock {
        ApiContentBlock::ToolResult {
            tool_use_id: tool_result.tool_use_id.clone(),
            content: tool_result.content.clone(),
            is_error: Some(tool_result.is_error),
        }
    }
}

/// 工具注册表适配器
///
/// 将`ToolRegistry`适配为`ToolCallHandler`
/// 注意：此实现为存根，实际集成请使用`tools`crate的`api_integration`模块
pub struct ToolRegistryAdapter;

impl ToolRegistryAdapter {
    /// 创建新的工具注册表适配器
    pub fn new(
        _registry: Arc<dyn std::any::Any + Send + Sync>,
        _converter: Arc<dyn ToolToApiConverter>,
    ) -> Self {
        Self
    }

    /// 获取所有工具的API定义
    pub async fn get_tools_as_api(&self) -> Result<Vec<ApiTool>> {
        Ok(Vec::new())
    }
}

#[async_trait::async_trait]
impl ToolCallHandler for ToolRegistryAdapter {
    async fn handle_tool_call(&self, _tool_call: ToolCall) -> Result<ToolResult> {
        Err(crate::error::ApiError::tool_call(
            "ToolRegistryAdapter is a stub. Enable 'api-tool-use' feature in tools crate for integration."
        ))
    }

    fn get_tools(&self) -> Vec<ToolDefinition> {
        Vec::new()
    }
}

/// API工具处理器
///
/// 管理工具调用执行和结果处理
pub struct ApiToolHandler {
    /// 工具处理器
    tool_handler: Arc<dyn ToolCallHandler>,
    /// 工具转换器
    converter: Arc<dyn ToolToApiConverter>,
}

impl ApiToolHandler {
    /// 创建新的API工具处理器
    pub fn new(
        tool_handler: Arc<dyn ToolCallHandler>,
        converter: Arc<dyn ToolToApiConverter>,
    ) -> Self {
        Self { tool_handler, converter }
    }

    /// 处理API响应中的工具调用
    pub async fn handle_api_response(
        &self,
        response: crate::types::ApiResponse,
    ) -> Result<Vec<ApiContentBlock>> {
        let mut tool_results = Vec::new();

        for content_block in response.content {
            if let Some(tool_call) = self.converter.api_to_tool_call(&content_block) {
                match self.tool_handler.handle_tool_call(tool_call).await {
                    Ok(tool_result) => {
                        let api_result = self.converter.tool_result_to_api(&tool_result);
                        tool_results.push(api_result);
                    }
                    Err(e) => {
                        // 创建错误结果
                        let error_result = ToolResult::error(
                            "unknown".to_string(),
                            format!("Tool execution failed: {}", e),
                        );
                        let api_result = self.converter.tool_result_to_api(&error_result);
                        tool_results.push(api_result);
                    }
                }
            }
        }

        Ok(tool_results)
    }

    /// 获取所有工具的API定义
    pub async fn get_api_tools(&self) -> Result<Vec<ApiTool>> {
        let tool_definitions = self.tool_handler.get_tools();
        let mut api_tools = Vec::new();

        for tool_def in tool_definitions {
            let api_tool = self.converter.tool_to_api(
                &tool_def.name,
                tool_def.description.as_deref().unwrap_or(""),
                tool_def.input_schema,
            );
            api_tools.push(api_tool);
        }

        Ok(api_tools)
    }
}