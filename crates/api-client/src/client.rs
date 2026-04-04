//! Claude API客户端实现
//!
//! 提供完整的API客户端功能，包括：
//! - 同步请求
//! - 流式响应
//! - 工具调用
//! - 重试和错误处理

use crate::error::{ApiError, Result};
use crate::tool_use::{ToolCall, ToolCallHandler, ToolResult};
use crate::types::{
    ApiContentBlock, ApiMessage, ApiModel, ApiRequest, ApiResponse, ApiTool,
    MessageContent, StreamEvent, ToolChoice,
};
use futures::{Stream, StreamExt};
use reqwest::{Client, ClientBuilder};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// API客户端配置
#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    /// 连接超时
    pub connect_timeout: Duration,
    /// 读取超时
    pub read_timeout: Duration,
    /// 启用压缩
    pub enable_compression: bool,
    /// 重试配置
    pub retry_config: RetryConfig,
    /// 基础URL
    pub base_url: String,
    /// 默认模型
    pub default_model: ApiModel,
    /// 默认最大令牌数
    pub default_max_tokens: u32,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(60),
            enable_compression: true,
            retry_config: RetryConfig::default(),
            base_url: "https://api.anthropic.com".to_string(),
            default_model: ApiModel::Claude35Sonnet20241022,
            default_max_tokens: 1024,
        }
    }
}

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始退避时间
    pub initial_backoff: Duration,
    /// 最大退避时间
    pub max_backoff: Duration,
    /// 退避因子
    pub backoff_factor: f32,
    /// 需要重试的状态码
    pub retry_status_codes: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
            backoff_factor: 2.0,
            retry_status_codes: vec![429, 500, 502, 503, 504],
        }
    }
}

/// API客户端
#[derive(Clone)]
pub struct ApiClient {
    /// 基础URL
    base_url: String,
    /// HTTP客户端
    client: Arc<Client>,
    /// 配置
    config: ApiClientConfig,
    /// API密钥
    api_key: Option<String>,
    /// 默认头信息
    default_headers: HashMap<String, String>,
}

impl ApiClient {
    /// 创建新的API客户端
    pub fn new(base_url: &str, config: ApiClientConfig) -> Self {
        let client = ClientBuilder::new()
            .connect_timeout(config.connect_timeout)
            .timeout(config.read_timeout)
            .build()
            .expect("Failed to create HTTP client");

        let mut default_headers = HashMap::new();
        default_headers.insert("Content-Type".to_string(), "application/json".to_string());
        default_headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
        default_headers.insert(
            "anthropic-beta".to_string(),
            "tools-2024-10-22".to_string(),
        );

        Self {
            base_url: base_url.to_string(),
            client: Arc::new(client),
            config,
            api_key: None,
            default_headers,
        }
    }

    /// 设置API密钥
    pub fn with_api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    /// 设置基础URL
    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.to_string();
        self
    }

    /// 添加默认头信息
    pub fn add_default_header(mut self, key: &str, value: &str) -> Self {
        self.default_headers
            .insert(key.to_string(), value.to_string());
        self
    }

    /// 发送同步请求
    pub async fn send_request(&self, request: ApiRequest) -> Result<ApiResponse> {
        let url = format!("{}/v1/messages", self.base_url);
        let mut headers = self.default_headers.clone();

        if let Some(api_key) = &self.api_key {
            headers.insert("x-api-key".to_string(), api_key.clone());
        }

        // 添加Beta头信息
        if let Some(betas) = &request.betas {
            for beta in betas {
                headers.insert("anthropic-beta".to_string(), beta.clone());
            }
        }

        let mut req_builder = self.client.post(&url);

        for (key, value) in headers {
            req_builder = req_builder.header(key, value);
        }

        let response = req_builder.json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::http(status, text));
        }

        let api_response: ApiResponse = response.json().await?;
        Ok(api_response)
    }

    /// 发送流式请求
    pub async fn send_stream_request(
        &self,
        mut request: ApiRequest,
    ) -> Result<impl Stream<Item = Result<StreamEvent>>> {
        // 确保启用流式
        request.stream = Some(true);

        let url = format!("{}/v1/messages", self.base_url);
        let mut headers = self.default_headers.clone();

        if let Some(api_key) = &self.api_key {
            headers.insert("x-api-key".to_string(), api_key.clone());
        }

        // 添加Beta头信息
        if let Some(betas) = &request.betas {
            for beta in betas {
                headers.insert("anthropic-beta".to_string(), beta.clone());
            }
        }

        let mut req_builder = self.client.post(&url);

        for (key, value) in headers {
            req_builder = req_builder.header(key, value);
        }

        let response = req_builder.json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::http(status, text));
        }

        // 创建流式响应处理器
        let stream = response.bytes_stream();
        let event_stream = stream.map(|chunk_result| {
            let chunk = chunk_result.map_err(|e| ApiError::Network(e))?;
            let chunk_str = String::from_utf8_lossy(&chunk);

            // 解析SSE事件
            for line in chunk_str.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        continue;
                    }

                    let event: StreamEvent =
                        serde_json::from_str(data).map_err(|e| ApiError::Serialization(e))?;
                    return Ok(event);
                }
            }

            Err(ApiError::stream("No valid SSE data found in chunk"))
        });

        Ok(event_stream)
    }

    /// 发送消息（简化接口）
    pub async fn send_message(&self, message: &str, model: Option<ApiModel>) -> Result<String> {
        let model = model.unwrap_or_else(|| self.config.default_model.clone());
        let request = ApiRequest {
            model,
            messages: vec![ApiMessage {
                role: crate::types::ApiRole::User,
                content: MessageContent::Text(message.to_string()),
            }],
            max_tokens: Some(self.config.default_max_tokens),
            ..Default::default()
        };

        let response = self.send_request(request).await?;

        // 提取文本内容
        let text = response
            .content
            .iter()
            .filter_map(|block| match block {
                ApiContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }

    /// 发送带工具调用的消息
    pub async fn send_message_with_tools(
        &self,
        messages: Vec<ApiMessage>,
        tools: Vec<ApiTool>,
        tool_choice: Option<ToolChoice>,
        model: Option<ApiModel>,
    ) -> Result<ApiResponse> {
        let model = model.unwrap_or_else(|| self.config.default_model.clone());
        let request = ApiRequest {
            model,
            messages,
            tools: Some(tools),
            tool_choice,
            max_tokens: Some(self.config.default_max_tokens),
            ..Default::default()
        };

        self.send_request(request).await
    }

    /// 处理工具调用
    pub async fn handle_tool_calls(
        &self,
        response: ApiResponse,
        tool_handler: &dyn ToolCallHandler,
    ) -> Result<Vec<ToolResult>> {
        let mut tool_results = Vec::new();

        for content in response.content {
            if let ApiContentBlock::ToolUse { id, name, input } = content {
                let tool_call = ToolCall {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                    tool: None, // 需要从工具列表中查找
                };

                match tool_handler.handle_tool_call(tool_call).await {
                    Ok(tool_result) => {
                        tool_results.push(tool_result);
                    }
                    Err(e) => {
                        return Err(ApiError::tool_call(format!(
                            "Failed to handle tool call {}: {}",
                            name, e
                        )));
                    }
                }
            }
        }

        Ok(tool_results)
    }

    /// 发送工具结果并获取后续响应
    pub async fn send_tool_results(
        &self,
        mut messages: Vec<ApiMessage>,
        tool_results: Vec<ToolResult>,
        model: Option<ApiModel>,
    ) -> Result<ApiResponse> {
        // 将工具结果添加到消息中
        let mut tool_result_blocks = Vec::new();
        for result in tool_results {
            tool_result_blocks.push(ApiContentBlock::ToolResult {
                tool_use_id: result.tool_use_id,
                content: result.content,
                is_error: Some(result.is_error),
            });
        }

        if !tool_result_blocks.is_empty() {
            messages.push(ApiMessage {
                role: crate::types::ApiRole::User,
                content: MessageContent::Blocks(tool_result_blocks),
            });
        }

        let model = model.unwrap_or_else(|| self.config.default_model.clone());
        let request = ApiRequest {
            model,
            messages,
            max_tokens: Some(self.config.default_max_tokens),
            ..Default::default()
        };

        self.send_request(request).await
    }

    /// 执行完整的工具调用循环
    pub async fn execute_with_tools<H>(
        &self,
        initial_messages: Vec<ApiMessage>,
        tools: Vec<ApiTool>,
        tool_handler: H,
        model: Option<ApiModel>,
        max_iterations: usize,
    ) -> Result<Vec<ApiMessage>>
    where
        H: ToolCallHandler,
    {
        let model = model.unwrap_or_else(|| self.config.default_model.clone());
        let mut messages = initial_messages;
        let mut iterations = 0;

        loop {
            if iterations >= max_iterations {
                return Err(ApiError::tool_call(
                    "Maximum tool call iterations exceeded",
                ));
            }

            // 发送请求
            let response = self
                .send_message_with_tools(messages.clone(), tools.clone(), None, Some(model.clone()))
                .await?;

            // 将助手响应添加到消息历史
            messages.push(ApiMessage {
                role: crate::types::ApiRole::Assistant,
                content: MessageContent::Blocks(response.content.clone()),
            });

            // 检查是否有工具调用
            let has_tool_calls = response.content.iter().any(|block| match block {
                ApiContentBlock::ToolUse { .. } => true,
                _ => false,
            });

            if !has_tool_calls {
                // 没有工具调用，完成
                break;
            }

            // 处理工具调用
            let tool_results = self.handle_tool_calls(response, &tool_handler).await?;

            // 发送工具结果
            let next_response = self
                .send_tool_results(messages.clone(), tool_results, Some(model.clone()))
                .await?;

            // 将用户响应（工具结果）添加到消息历史
            messages.push(ApiMessage {
                role: crate::types::ApiRole::User,
                content: MessageContent::Blocks(next_response.content.clone()),
            });

            iterations += 1;
        }

        Ok(messages)
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        let config = ApiClientConfig::default();
        let base_url = config.base_url.clone();
        Self::new(&base_url, config)
    }
}

/// 为ApiRequest实现Default
impl Default for ApiRequest {
    fn default() -> Self {
        Self {
            model: ApiModel::Claude35Sonnet20241022,
            messages: Vec::new(),
            system: None,
            max_tokens: Some(1024),
            tools: None,
            tool_choice: None,
            thinking: None,
            stream: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            metadata: None,
            betas: None,
        }
    }
}