//! 流式响应处理
//!
//! 提供Claude API流式响应的处理功能，包括：
//! - 流式事件处理
//! - 实时文本收集
//! - 工具调用检测
//! - 错误处理

use crate::error::Result;
use crate::types::{StreamEvent, ToolCall};
use async_stream::stream;
use futures::{Stream, StreamExt};
use serde_json::Value;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 流式响应处理器
pub trait StreamHandler: Send + Sync {
    /// 处理流式事件
    fn handle_event(&mut self, event: StreamEvent) -> Result<()>;

    /// 获取累积的文本
    fn get_text(&self) -> String;

    /// 获取检测到的工具调用
    fn get_tool_calls(&self) -> Vec<ToolCall>;

    /// 是否完成
    fn is_complete(&self) -> bool;
}

/// 默认流式处理器
pub struct DefaultStreamHandler {
    /// 累积的文本
    text: String,
    /// 检测到的工具调用
    tool_calls: Vec<ToolCall>,
    /// 是否完成
    complete: bool,
    /// 当前内容块索引
    current_block_index: Option<u32>,
    /// 当前工具调用（构建中）
    current_tool_call: Option<PartialToolCall>,
}

/// 部分工具调用（构建中）
struct PartialToolCall {
    id: String,
    name: String,
    input: Value,
}

impl DefaultStreamHandler {
    /// 创建新的默认处理器
    pub fn new() -> Self {
        Self {
            text: String::new(),
            tool_calls: Vec::new(),
            complete: false,
            current_block_index: None,
            current_tool_call: None,
        }
    }
}

impl Default for DefaultStreamHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamHandler for DefaultStreamHandler {
    fn handle_event(&mut self, event: StreamEvent) -> Result<()> {
        match event {
            StreamEvent::MessageStart { message: _ } => {
                // 消息开始，重置状态
                self.text.clear();
                self.tool_calls.clear();
                self.complete = false;
                self.current_block_index = None;
                self.current_tool_call = None;
            }
            StreamEvent::ContentBlockStart { index, content_block } => {
                self.current_block_index = Some(index);
                match content_block.content_block_type {
                    crate::types::ApiContentType::ToolUse => {
                        // 开始新的工具调用
                        self.current_tool_call = Some(PartialToolCall {
                            id: String::new(),
                            name: String::new(),
                            input: Value::Null,
                        });
                    }
                    _ => {}
                }
            }
            StreamEvent::ContentBlockDelta { index: _, delta } => match delta {
                crate::types::ContentBlockDelta::TextDelta { text } => {
                    self.text.push_str(&text);
                }
                crate::types::ContentBlockDelta::ToolUseDelta { id, name, input } => {
                    if let Some(ref mut tool_call) = self.current_tool_call {
                        // 更新工具调用信息
                        if !id.is_empty() {
                            tool_call.id = id;
                        }
                        if !name.is_empty() {
                            tool_call.name = name;
                        }
                        if input != Value::Null {
                            tool_call.input = input;
                        }
                    }
                }
            },
            StreamEvent::ContentBlockStop { index: _ } => {
                if let Some(tool_call) = self.current_tool_call.take() {
                    // 完成工具调用
                    self.tool_calls.push(ToolCall {
                        id: tool_call.id,
                        name: tool_call.name,
                        input: tool_call.input,
                        tool: None,
                    });
                }
                self.current_block_index = None;
            }
            StreamEvent::MessageDelta { delta: _, usage: _ } => {
                // 消息增量，暂无操作
            }
            StreamEvent::MessageStop => {
                self.complete = true;
            }
            StreamEvent::Error { error } => {
                return Err(crate::error::ApiError::stream(format!(
                    "Stream error: {} - {}",
                    error.error_type, error.message
                )));
            }
        }

        Ok(())
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn get_tool_calls(&self) -> Vec<ToolCall> {
        self.tool_calls.clone()
    }

    fn is_complete(&self) -> bool {
        self.complete
    }
}

/// 流式响应
pub struct StreamResponse {
    /// 流式事件流
    stream: Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>,
    /// 处理器
    handler: Arc<Mutex<Box<dyn StreamHandler>>>,
}

impl StreamResponse {
    /// 创建新的流式响应
    pub fn new(
        stream: impl Stream<Item = Result<StreamEvent>> + Send + 'static,
        handler: Box<dyn StreamHandler>,
    ) -> Self {
        Self {
            stream: Box::pin(stream),
            handler: Arc::new(Mutex::new(handler)),
        }
    }

    /// 处理流式响应
    pub async fn process(mut self) -> Result<ProcessedStream> {
        let mut text = String::new();
        let mut tool_calls = Vec::new();

        while let Some(event_result) = self.stream.next().await {
            let event = event_result?;

            // 更新处理器
            let mut handler = self.handler.lock().await;
            handler.handle_event(event.clone())?;

            // 更新结果
            text = handler.get_text();
            tool_calls = handler.get_tool_calls();

            if handler.is_complete() {
                break;
            }
        }

        Ok(ProcessedStream {
            text,
            tool_calls,
            complete: true,
        })
    }

    /// 获取原始事件流
    pub fn into_stream(self) -> Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>> {
        self.stream
    }
}

/// 处理后的流式结果
pub struct ProcessedStream {
    /// 累积的文本
    pub text: String,
    /// 检测到的工具调用
    pub tool_calls: Vec<ToolCall>,
    /// 是否完成
    pub complete: bool,
}

/// 创建简单的文本收集流
pub fn create_text_collector_stream(
    stream: impl Stream<Item = Result<StreamEvent>> + Send + 'static,
) -> impl Stream<Item = Result<String>> {
    stream! {
        let mut handler = DefaultStreamHandler::new();

        for await event_result in stream {
            let event = event_result?;
            handler.handle_event(event.clone())?;

            if !handler.get_text().is_empty() {
                yield Ok(handler.get_text());
            }

            if handler.is_complete() {
                break;
            }
        }
    }
}

/// 流式响应构建器
pub struct StreamResponseBuilder {
    /// 处理器
    handler: Box<dyn StreamHandler>,
}

impl StreamResponseBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            handler: Box::new(DefaultStreamHandler::new()),
        }
    }

    /// 设置自定义处理器
    pub fn with_handler(mut self, handler: Box<dyn StreamHandler>) -> Self {
        self.handler = handler;
        self
    }

    /// 构建流式响应
    pub fn build(
        self,
        stream: impl Stream<Item = Result<StreamEvent>> + Send + 'static,
    ) -> StreamResponse {
        StreamResponse::new(stream, self.handler)
    }
}

impl Default for StreamResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}