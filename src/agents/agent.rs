//! 代理执行器
//!
//! 这个模块实现了代理执行和管理功能

use crate::error::Result;
use crate::tools::{ToolManager, ApiToolHandler};
use api_client::{ApiClient, ApiMessage, ApiRole, MessageContent, ApiTool};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::types::{AgentDefinition, AgentConfig, AgentResult, AgentId, AgentStatus, TokenUsage};

/// 代理执行器
pub struct AgentExecutor {
    /// 代理配置
    config: AgentConfig,

    /// 代理 ID
    agent_id: AgentId,

    /// 执行状态
    status: Arc<RwLock<AgentStatus>>,

    /// API 客户端
    api_client: Arc<ApiClient>,

    /// 工具管理器
    tool_manager: Arc<ToolManager>,
}

impl AgentExecutor {
    /// 创建新的代理执行器
    pub fn new(
        config: AgentConfig,
        api_client: Arc<ApiClient>,
        tool_manager: Arc<ToolManager>,
    ) -> Self {
        let agent_id = uuid::Uuid::new_v4().to_string();

        Self {
            config,
            agent_id,
            status: Arc::new(RwLock::new(AgentStatus::Idle)),
            api_client,
            tool_manager,
        }
    }
    
    /// 获取代理 ID
    pub fn id(&self) -> &str {
        &self.agent_id
    }
    
    /// 获取代理配置
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }
    
    /// 获取代理状态
    pub async fn status(&self) -> AgentStatus {
        *self.status.read().await
    }
    
    /// 执行代理
    pub async fn execute(&self, input: String) -> Result<AgentResult> {
        // 更新状态为运行中
        *self.status.write().await = AgentStatus::Running;

        let result = self.execute_inner(input).await;

        // 更新状态
        match &result {
            Ok(_) => *self.status.write().await = AgentStatus::Completed,
            Err(_) => *self.status.write().await = AgentStatus::Error,
        }

        result
    }

    /// 内部执行逻辑
    async fn execute_inner(&self, input: String) -> Result<AgentResult> {
        // 构建消息列表
        let mut messages = Vec::new();

        // 添加系统提示
        let system_prompt = self.config.definition.system_prompt.clone();
        if !system_prompt.is_empty() {
            messages.push(ApiMessage {
                role: ApiRole::User,
                content: MessageContent::Text(system_prompt),
            });
        }

        // 添加用户输入
        messages.push(ApiMessage {
            role: ApiRole::User,
            content: MessageContent::Text(input),
        });

        // 获取允许的工具列表
        let allowed_tools = self.config.definition.tools.as_ref().unwrap_or(&Vec::new());
        let mut api_tools = Vec::new();

        // 从工具管理器获取工具定义并转换为 API 格式
        let registry = self.tool_manager.registry();
        for tool_name in allowed_tools {
            if let Some(tool) = registry.get(tool_name).await {
                let api_tool = ApiTool {
                    name: tool.name().to_string(),
                    description: tool.description().map(|d: &str| d.to_string()),
                    input_schema: tool.input_schema().into(),
                };
                api_tools.push(api_tool);
            }
        }

        // 创建工具调用处理器
        let converter = Arc::new(api_client::integration::DefaultToolConverter::default());
        let tool_handler = Arc::new(api_client::integration::ToolRegistryAdapter::new(
            Arc::clone(self.tool_manager.registry()),
            converter,
        ));

        // 执行完整的工具调用循环
        let max_iterations = self.config.definition.max_turns.unwrap_or(10) as usize;
        let model = api_client::types::ApiModel::Claude35Sonnet20241022;

        let messages = self.api_client
            .execute_with_tools(
                messages,
                api_tools,
                tool_handler,
                Some(model),
                max_iterations,
            )
            .await?;

        // 收集输出消息
        let output_messages: Vec<String> = messages
            .iter()
            .filter(|msg| msg.role == ApiRole::Assistant)
            .filter_map(|msg| match &msg.content {
                MessageContent::Text(text) => Some(text.clone()),
                MessageContent::Blocks(blocks) => {
                    let texts: Vec<String> = blocks
                        .iter()
                        .filter_map(|block| match block {
                            api_client::types::ApiContentBlock::Text { text } => {
                                Some(text.clone())
                            }
                            _ => None,
                        })
                        .collect();
                    Some(texts.join("\n"))
                }
            })
            .collect();

        // 计算 token 使用量（TODO: 从 API 响应积累实际使用量）
        let usage = TokenUsage::default();

        Ok(AgentResult {
            agent_id: self.agent_id.clone(),
            messages: output_messages,
            usage,
            status: AgentStatus::Completed,
            error: None,
        })
    }
    
    /// 取消代理
    pub async fn cancel(&self) -> Result<()> {
        *self.status.write().await = AgentStatus::Cancelled;
        Ok(())
    }
}

impl std::fmt::Debug for AgentExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentExecutor")
            .field("agent_id", &self.agent_id)
            .field("status", &self.status.try_read())
            .finish_non_exhaustive()
    }
}

/// 代理管理器
pub struct AgentManager {
    /// 已注册的代理定义
    agents: Arc<RwLock<HashMap<String, AgentDefinition>>>,
    
    /// 活动的代理执行器
    executors: Arc<RwLock<HashMap<AgentId, Arc<AgentExecutor>>>>,
}

impl AgentManager {
    /// 创建新的代理管理器
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            executors: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册代理
    pub async fn register(&self, definition: AgentDefinition) {
        self.agents.write().await.insert(definition.name.clone(), definition);
    }
    
    /// 注销代理
    pub async fn unregister(&self, name: &str) -> Option<AgentDefinition> {
        self.agents.write().await.remove(name)
    }
    
    /// 获取代理定义
    pub async fn get(&self, name: &str) -> Option<AgentDefinition> {
        self.agents.read().await.get(name).cloned()
    }
    
    /// 列出所有代理
    pub async fn list(&self) -> Vec<AgentDefinition> {
        self.agents.read().await.values().cloned().collect()
    }
    
    /// 创建代理执行器
    pub async fn create_executor(
        &self,
        name: &str,
        api_client: Arc<ApiClient>,
        tool_manager: Arc<ToolManager>,
    ) -> Option<Arc<AgentExecutor>> {
        let definition = self.get(name).await?;
        let config = AgentConfig::new(definition);
        let executor = Arc::new(AgentExecutor::new(
            config,
            api_client,
            tool_manager,
        ));

        let agent_id = executor.id().to_string();
        self.executors.write().await.insert(agent_id, executor.clone());

        Some(executor)
    }
    
    /// 获取执行器
    pub async fn get_executor(&self, agent_id: &str) -> Option<Arc<AgentExecutor>> {
        self.executors.read().await.get(agent_id).cloned()
    }
    
    /// 移除执行器
    pub async fn remove_executor(&self, agent_id: &str) -> Option<Arc<AgentExecutor>> {
        self.executors.write().await.remove(agent_id)
    }
    
    /// 清理已完成的执行器
    pub async fn cleanup(&self) {
        let mut executors = self.executors.write().await;
        executors.retain(|_, executor| {
            executor.status.try_read().map(|s| *s != AgentStatus::Completed && *s != AgentStatus::Cancelled && *s != AgentStatus::Error).unwrap_or(false)
        });
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AgentManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentManager")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::AgentType;
    
    #[tokio::test]
    async fn test_agent_manager() {
        let manager = AgentManager::new();
        
        let definition = AgentDefinition::new(
            "test".to_string(),
            AgentType::GeneralPurpose,
            "Test agent".to_string(),
        );
        
        manager.register(definition).await;
        
        let retrieved = manager.get("test").await;
        assert!(retrieved.is_some());
    }
}
