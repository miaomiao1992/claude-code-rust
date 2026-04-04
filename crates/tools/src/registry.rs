//! 工具注册表模块
//!
//! 管理工具的注册、加载和检索，支持API工具调用集成

use crate::base::Tool;
use crate::types::{ApiToolDefinition, ApiToolCall, ApiToolResult, ToolCallResponse, ToolExecutionOptions, ToolMetadata, ToolUseContext};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 工具加载器 trait
#[async_trait::async_trait]
pub trait ToolLoader: Send + Sync {
    /// 加载工具到注册表
    async fn load(&self, registry: &ToolRegistry) -> Result<()>;

    /// 获取加载器名称
    fn name(&self) -> &str;
}

/// 工具注册表
///
/// 负责存储和管理所有注册的工具
#[derive(Clone, Default)]
pub struct ToolRegistry {
    /// 工具映射（名称 -> 工具）
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    /// 别名映射（别名 -> 工具名称）
    aliases: Arc<RwLock<HashMap<String, String>>>,
}

impl ToolRegistry {
    /// 创建新的工具注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查注册表是否为空
    pub async fn is_empty(&self) -> bool {
        self.tools.read().await.is_empty()
    }

    /// 注册工具
    pub async fn register<T: Tool + 'static>(&self, tool: T) {
        let metadata = tool.metadata();
        let tool_arc = Arc::new(tool);

        let mut tools = self.tools.write().await;
        tools.insert(metadata.name.clone(), tool_arc);

        let mut aliases = self.aliases.write().await;
        if let Some(tool_aliases) = metadata.aliases {
            for alias in tool_aliases {
                aliases.insert(alias, metadata.name.clone());
            }
        }

        // 注册小写别名
        let lowercase_name = metadata.name.to_lowercase();
        if lowercase_name != metadata.name {
            aliases.insert(lowercase_name, metadata.name.clone());
        }
    }

    /// 获取工具
    pub async fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().await;
        let aliases = self.aliases.read().await;

        // 先尝试直接查找
        if let Some(tool) = tools.get(name) {
            return Some(tool.clone());
        }

        // 尝试通过别名查找
        if let Some(real_name) = aliases.get(name) {
            if let Some(tool) = tools.get(real_name) {
                return Some(tool.clone());
            }
        }

        None
    }

    /// 检查工具是否存在
    pub async fn has(&self, name: &str) -> bool {
        self.get(name).await.is_some()
    }

    /// 获取工具数量
    pub async fn len(&self) -> usize {
        self.tools.read().await.len()
    }

    /// 获取所有工具名称
    pub async fn tool_names(&self) -> Vec<String> {
        self.tools.read().await.keys().cloned().collect()
    }

    /// 获取所有工具元数据
    pub async fn tool_metadata(&self) -> Vec<ToolMetadata> {
        self.tools.read().await.values()
            .map(|tool| tool.metadata())
            .collect()
    }

    /// 获取所有API工具定义
    pub async fn api_tool_definitions(&self) -> Vec<ApiToolDefinition> {
        self.tools.read().await.values()
            .map(|tool| tool.api_tool_definition())
            .collect()
    }

    /// 执行工具
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
        context: ToolUseContext,
        options: &ToolExecutionOptions,
    ) -> Result<ToolCallResponse> {
        let tool = self.get(tool_name).await
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;

        tool.execute_with_options(input, context, options).await.map_err(|e| anyhow::anyhow!("{:?}", e))
    }

    /// 处理API工具调用
    pub async fn handle_api_tool_call(
        &self,
        tool_call: ApiToolCall,
        context: ToolUseContext,
    ) -> Result<ApiToolResult> {
        let tool = self.get(&tool_call.name).await
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_call.name))?;

        // 验证工具调用
        self.validate_tool_call(&tool_call, &tool).await?;

        // 执行工具
        let tool_result = tool.handle_api_tool_result(
            ApiToolResult::success(&tool_call.id, serde_json::Value::Null),
            context,
        ).await?;

        // 转换为API工具结果
        Ok(ApiToolResult::success(
            tool_call.id,
            tool_result.data,
        ))
    }

    /// 验证工具调用
    async fn validate_tool_call(
        &self,
        tool_call: &ApiToolCall,
        tool: &Arc<dyn Tool>,
    ) -> Result<()> {
        // 检查工具名称是否匹配
        if tool.name() != tool_call.name {
            return Err(anyhow::anyhow!(
                "Tool name mismatch: expected {}, got {}",
                tool.name(),
                tool_call.name
            ));
        }

        // TODO: 验证输入schema
        // TODO: 检查权限

        Ok(())
    }
}

/// 工具管理器
///
/// 负责管理工具加载器和加载工具，支持API工具调用集成
#[derive(Default)]
pub struct ToolManager {
    /// 工具注册表
    registry: ToolRegistry,
    /// 工具加载器
    loaders: Arc<RwLock<Vec<Box<dyn ToolLoader>>>>,
    /// 执行选项
    execution_options: ToolExecutionOptions,
}

impl ToolManager {
    /// 创建新的工具管理器
    pub fn new(execution_options: ToolExecutionOptions) -> Self {
        Self {
            registry: ToolRegistry::new(),
            loaders: Arc::new(RwLock::new(Vec::new())),
            execution_options,
        }
    }

    /// 创建新的工具管理器使用默认执行选项
    pub fn default() -> Self {
        Self {
            registry: ToolRegistry::new(),
            loaders: Arc::new(RwLock::new(Vec::new())),
            execution_options: ToolExecutionOptions::default(),
        }
    }

    /// 添加工具加载器
    pub async fn add_loader(&self, loader: Box<dyn ToolLoader>) {
        let mut loaders = self.loaders.write().await;
        loaders.push(loader);
    }

    /// 加载所有工具
    pub async fn load_all(&self) -> Result<()> {
        let loaders = self.loaders.read().await;

        for loader in loaders.iter() {
            tracing::debug!("Loading tools from {}", loader.name());
            loader.load(&self.registry).await?;
        }

        Ok(())
    }

    /// 获取工具注册表
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    /// 获取工具
    pub async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.registry.get(name).await
    }

    /// 检查工具是否存在
    pub async fn has_tool(&self, name: &str) -> bool {
        self.registry.has(name).await
    }

    /// 获取工具数量
    pub async fn tool_count(&self) -> usize {
        self.registry.len().await
    }

    /// 获取所有工具名称
    pub async fn tool_names(&self) -> Vec<String> {
        self.registry.tool_names().await
    }

    /// 获取所有工具元数据
    pub async fn tool_metadata(&self) -> Vec<ToolMetadata> {
        self.registry.tool_metadata().await
    }

    /// 获取所有API工具定义
    pub async fn api_tool_definitions(&self) -> Vec<ApiToolDefinition> {
        self.registry.api_tool_definitions().await
    }

    /// 执行工具
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
        context: ToolUseContext,
    ) -> Result<ToolCallResponse> {
        self.registry.execute_tool(tool_name, input, context, &self.execution_options).await
    }

    /// 处理API工具调用
    pub async fn handle_api_tool_call(
        &self,
        tool_call: ApiToolCall,
        context: ToolUseContext,
    ) -> Result<ApiToolResult> {
        self.registry.handle_api_tool_call(tool_call, context).await
    }

    /// 批量处理API工具调用
    pub async fn handle_api_tool_calls(
        &self,
        tool_calls: Vec<ApiToolCall>,
        context: ToolUseContext,
    ) -> Result<Vec<ApiToolResult>> {
        let mut results = Vec::new();

        for tool_call in tool_calls {
            let result = self.handle_api_tool_call(tool_call, context.clone()).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// 获取执行选项
    pub fn execution_options(&self) -> &ToolExecutionOptions {
        &self.execution_options
    }

    /// 更新执行选项
    pub fn update_execution_options(&mut self, options: ToolExecutionOptions) {
        self.execution_options = options;
    }
}

/// API工具调用处理器
#[derive(Clone)]
pub struct ApiToolCallHandler {
    /// 工具管理器
    tool_manager: Arc<ToolManager>,
}

impl ApiToolCallHandler {
    /// 创建新的API工具调用处理器
    pub fn new(tool_manager: Arc<ToolManager>) -> Self {
        Self { tool_manager }
    }

    /// 处理API工具调用
    pub async fn handle_tool_call(
        &self,
        tool_call: ApiToolCall,
        context: ToolUseContext,
    ) -> Result<ApiToolResult> {
        self.tool_manager.handle_api_tool_call(tool_call, context).await
    }

    /// 获取工具管理器
    pub fn tool_manager(&self) -> &Arc<ToolManager> {
        &self.tool_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::base::ToolBuilder;
    use crate::types::{ToolCategory, ToolPermissionLevel, ToolResult};

    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn metadata(&self) -> ToolMetadata {
            ToolBuilder::new("test", "Test tool")
                .category(ToolCategory::Other)
                .aliases(vec!["t".to_string()])
                .build_metadata()
        }

        async fn execute(
            &self,
            _input: serde_json::Value,
            _context: ToolUseContext,
        ) -> Result<ToolResult<serde_json::Value>> {
            Ok(ToolResult::success(serde_json::json!("ok")))
        }
    }

    #[tokio::test]
    async fn test_register_tool() {
        let registry = ToolRegistry::new();
        registry.register(TestTool).await;

        assert!(registry.has("test").await);
        assert!(registry.has("t").await);
        assert_eq!(registry.len().await, 1);
    }

    #[tokio::test]
    async fn test_get_tool() {
        let registry = ToolRegistry::new();
        registry.register(TestTool).await;

        let tool = registry.get("test").await;
        assert!(tool.is_some());

        let tool_by_alias = registry.get("t").await;
        assert!(tool_by_alias.is_some());
    }

    #[tokio::test]
    async fn test_api_tool_definitions() {
        let registry = ToolRegistry::new();
        registry.register(TestTool).await;

        let definitions = registry.api_tool_definitions().await;
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].name, "test");
    }

    #[tokio::test]
    async fn test_tool_manager() {
        let options = ToolExecutionOptions::default();
        let manager = ToolManager::new(options);

        struct TestLoader;

        #[async_trait]
        impl ToolLoader for TestLoader {
            async fn load(&self, registry: &ToolRegistry) -> Result<()> {
                registry.register(TestTool).await;
                Ok(())
            }

            fn name(&self) -> &str {
                "test"
            }
        }

        manager.add_loader(Box::new(TestLoader)).await;
        manager.load_all().await.unwrap();

        assert!(manager.has_tool("test").await);
        assert_eq!(manager.tool_count().await, 1);
        assert_eq!(manager.api_tool_definitions().await.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_tool() {
        let options = ToolExecutionOptions::default();
        let manager = ToolManager::new(options);

        // 直接注册工具，不通过加载器
        manager.registry().register(TestTool).await;

        let context = ToolUseContext::new(std::path::PathBuf::from("."));
        let result = manager.execute_tool("test", serde_json::json!({}), context).await;

        assert!(result.is_ok());
        match result.unwrap() {
            ToolCallResponse::Direct(result) => {
                assert!(result.error.is_none());
                assert_eq!(result.data, serde_json::json!("ok"));
            }
            _ => panic!("Expected direct tool result"),
        }
    }
}