//! 工具调用处理
//!
//! 提供Claude API工具调用的处理功能，包括：
//! - 工具调用解析
//! - 工具结果生成
//! - 工具调用处理器
//! - 工具调用上下文

use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// 工具调用
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// 工具使用ID
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 输入参数
    pub input: Value,
    /// 工具定义（可选）
    pub tool: Option<ToolDefinition>,
}

/// 工具定义
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: Option<String>,
    /// 输入模式（JSON Schema）
    pub input_schema: Value,
}

/// 工具结果
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// 工具使用ID
    pub tool_use_id: String,
    /// 结果内容
    pub content: Value,
    /// 是否错误
    pub is_error: bool,
}

impl ToolResult {
    /// 创建成功的工具结果
    pub fn success(tool_use_id: impl Into<String>, content: impl Into<Value>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: content.into(),
            is_error: false,
        }
    }

    /// 创建失败的工具结果
    pub fn error(tool_use_id: impl Into<String>, error_message: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: Value::String(error_message.into()),
            is_error: true,
        }
    }

    /// 从字符串创建工具结果
    pub fn from_string(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: Value::String(content.into()),
            is_error: false,
        }
    }

    /// 从JSON创建工具结果
    pub fn from_json(tool_use_id: impl Into<String>, content: Value) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content,
            is_error: false,
        }
    }
}

/// 工具调用处理器
#[async_trait]
pub trait ToolCallHandler: Send + Sync {
    /// 处理工具调用
    async fn handle_tool_call(&self, tool_call: ToolCall) -> Result<ToolResult>;

    /// 获取支持的工具列表
    fn get_tools(&self) -> Vec<ToolDefinition>;
}

/// 组合工具调用处理器
pub struct CompositeToolHandler {
    /// 工具处理器映射
    handlers: HashMap<String, Box<dyn ToolCallHandler>>,
    /// 默认处理器
    default_handler: Option<Box<dyn ToolCallHandler>>,
}

impl CompositeToolHandler {
    /// 创建新的组合处理器
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            default_handler: None,
        }
    }

    /// 添加工具处理器
    pub fn add_handler(&mut self, name: impl Into<String>, handler: Box<dyn ToolCallHandler>) {
        self.handlers.insert(name.into(), handler);
    }

    /// 设置默认处理器
    pub fn set_default_handler(&mut self, handler: Box<dyn ToolCallHandler>) {
        self.default_handler = Some(handler);
    }

    /// 获取工具处理器
    fn get_handler(&self, tool_name: &str) -> Option<&dyn ToolCallHandler> {
        self.handlers
            .get(tool_name)
            .map(|h| h.as_ref())
            .or_else(|| self.default_handler.as_ref().map(|h| h.as_ref()))
    }
}

#[async_trait]
impl ToolCallHandler for CompositeToolHandler {
    async fn handle_tool_call(&self, tool_call: ToolCall) -> Result<ToolResult> {
        if let Some(handler) = self.get_handler(&tool_call.name) {
            handler.handle_tool_call(tool_call).await
        } else {
            Err(crate::error::ApiError::tool_call(format!(
                "No handler found for tool: {}",
                tool_call.name
            )))
        }
    }

    fn get_tools(&self) -> Vec<ToolDefinition> {
        let mut tools = Vec::new();

        for handler in self.handlers.values() {
            tools.extend(handler.get_tools());
        }

        if let Some(default_handler) = &self.default_handler {
            tools.extend(default_handler.get_tools());
        }

        tools
    }
}

impl Default for CompositeToolHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// 简单工具调用处理器
pub struct SimpleToolHandler<F>
where
    F: Fn(ToolCall) -> Result<ToolResult> + Send + Sync,
{
    /// 工具定义
    tool_definition: ToolDefinition,
    /// 处理函数
    handler: F,
}

impl<F> SimpleToolHandler<F>
where
    F: Fn(ToolCall) -> Result<ToolResult> + Send + Sync,
{
    /// 创建新的简单处理器
    pub fn new(
        name: impl Into<String>,
        description: Option<String>,
        input_schema: Value,
        handler: F,
    ) -> Self {
        Self {
            tool_definition: ToolDefinition {
                name: name.into(),
                description,
                input_schema,
            },
            handler,
        }
    }
}

#[async_trait]
impl<F> ToolCallHandler for SimpleToolHandler<F>
where
    F: Fn(ToolCall) -> Result<ToolResult> + Send + Sync,
{
    async fn handle_tool_call(&self, tool_call: ToolCall) -> Result<ToolResult> {
        (self.handler)(tool_call)
    }

    fn get_tools(&self) -> Vec<ToolDefinition> {
        vec![self.tool_definition.clone()]
    }
}

/// 工具调用上下文
pub struct ToolCallContext {
    /// 工具调用
    pub tool_call: ToolCall,
    /// 原始消息
    pub message: Option<Value>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 用户ID
    pub user_id: Option<String>,
    /// 自定义上下文数据
    pub custom_data: HashMap<String, Value>,
}

impl ToolCallContext {
    /// 创建新的工具调用上下文
    pub fn new(tool_call: ToolCall) -> Self {
        Self {
            tool_call,
            message: None,
            session_id: None,
            user_id: None,
            custom_data: HashMap::new(),
        }
    }

    /// 设置消息
    pub fn with_message(mut self, message: Value) -> Self {
        self.message = Some(message);
        self
    }

    /// 设置会话ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// 设置用户ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// 设置自定义数据
    pub fn with_custom_data(mut self, key: impl Into<String>, value: Value) -> Self {
        self.custom_data.insert(key.into(), value);
        self
    }

    /// 获取输入参数作为指定类型
    pub fn get_input_as<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_value(self.tool_call.input.clone())
            .map_err(|e| crate::error::ApiError::Serialization(e))
    }

    /// 获取输入参数作为字符串
    pub fn get_input_as_string(&self) -> Result<String> {
        match &self.tool_call.input {
            Value::String(s) => Ok(s.clone()),
            _ => serde_json::to_string(&self.tool_call.input)
                .map_err(|e| crate::error::ApiError::Serialization(e)),
        }
    }
}

/// 工具调用结果构建器
pub struct ToolResultBuilder {
    /// 工具使用ID
    tool_use_id: String,
    /// 结果内容
    content: Value,
    /// 是否错误
    is_error: bool,
}

impl ToolResultBuilder {
    /// 创建新的构建器
    pub fn new(tool_use_id: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: Value::Null,
            is_error: false,
        }
    }

    /// 设置内容为字符串
    pub fn with_string_content(mut self, content: impl Into<String>) -> Self {
        self.content = Value::String(content.into());
        self
    }

    /// 设置内容为JSON
    pub fn with_json_content(mut self, content: Value) -> Self {
        self.content = content;
        self
    }

    /// 标记为错误
    pub fn as_error(mut self) -> Self {
        self.is_error = true;
        self
    }

    /// 构建工具结果
    pub fn build(self) -> ToolResult {
        ToolResult {
            tool_use_id: self.tool_use_id,
            content: self.content,
            is_error: self.is_error,
        }
    }
}

/// 工具调用工具函数
pub mod utils {
    use super::*;

    /// 将工具定义转换为API工具格式
    pub fn tool_definition_to_api(tool: &ToolDefinition) -> crate::types::ApiTool {
        crate::types::ApiTool {
            name: tool.name.clone(),
            description: tool.description.clone(),
            input_schema: tool.input_schema.clone(),
        }
    }

    /// 从工具调用创建内容块
    pub fn tool_call_to_content_block(tool_call: &ToolCall) -> crate::types::ApiContentBlock {
        crate::types::ApiContentBlock::ToolUse {
            id: tool_call.id.clone(),
            name: tool_call.name.clone(),
            input: tool_call.input.clone(),
        }
    }

    /// 从工具结果创建内容块
    pub fn tool_result_to_content_block(tool_result: &ToolResult) -> crate::types::ApiContentBlock {
        crate::types::ApiContentBlock::ToolResult {
            tool_use_id: tool_result.tool_use_id.clone(),
            content: tool_result.content.clone(),
            is_error: Some(tool_result.is_error),
        }
    }

    /// 验证工具调用输入
    pub fn validate_tool_input(
        tool_call: &ToolCall,
        _schema: &Value,
    ) -> Result<()> {
        // 这里可以添加JSON Schema验证
        // 暂时只检查是否为有效JSON
        if tool_call.input.is_null() {
            return Err(crate::error::ApiError::tool_call(
                "Tool input is null or missing",
            ));
        }
        Ok(())
    }
}