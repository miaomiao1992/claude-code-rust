//! Teleport Remote 模块
//! 
//! 实现远程执行和上下文传递功能

use super::packet::*;
use super::protocol::*;
use crate::error::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 远程执行器
pub struct RemoteExecutor {
    /// 协议客户端
    client: ProtocolClient,
    /// 执行超时时间（秒）
    timeout: u32,
}

impl RemoteExecutor {
    /// 创建新的远程执行器
    pub async fn new(server_address: String, timeout: u32) -> Result<Self> {
        let client = ProtocolClient::new(server_address).await?;
        
        Ok(Self {
            client,
            timeout,
        })
    }
    
    /// 执行远程任务
    pub async fn execute_task(&mut self, task: TaskPacket) -> Result<ResultPacket> {
        // 创建任务数据包
        let task_data = serde_json::to_value(task)?;
        let packet = TeleportPacket::new(
            PacketType::Task,
            "local".to_string(),
            "remote".to_string(),
            task_data,
        );
        
        // 发送任务
        self.client.send_packet(packet).await?;
        
        // 等待结果
        tokio::time::timeout(tokio::time::Duration::from_secs(self.timeout as u64), async {
            while let Some(packet) = self.client.recv_packet().await {
                if packet.packet_type == PacketType::Result {
                    return serde_json::from_value::<ResultPacket>(packet.data);
                }
            }
            Err(serde_json::Error::custom("No result received"))
        }).await
        .map_err(|_| crate::error::ClaudeError::Other("Execution timeout".to_string()))?
        .map_err(|e| e.into())
    }
    
    /// 传递上下文
    pub async fn transfer_context(&mut self, context: ContextPacket) -> Result<bool> {
        // 创建上下文数据包
        let context_data = serde_json::to_value(context)?;
        let packet = TeleportPacket::new(
            PacketType::Context,
            "local".to_string(),
            "remote".to_string(),
            context_data,
        );
        
        // 发送上下文
        self.client.send_packet(packet).await?;
        
        // 等待确认
        tokio::time::timeout(tokio::time::Duration::from_secs(30), async {
            while let Some(packet) = self.client.recv_packet().await {
                if packet.packet_type == PacketType::Result {
                    let result: ResultPacket = serde_json::from_value(packet.data)?;
                    return Ok(result.status == "success");
                }
            }
            Ok(false)
        }).await
        .map_err(|_| crate::error::ClaudeError::Other("Transfer timeout".to_string()))?
    }
    
    /// 关闭连接
    pub async fn close(&mut self) -> Result<()> {
        self.client.close().await
    }
}

/// 远程服务器
pub struct RemoteServer {
    /// 协议服务器
    server: ProtocolServer,
    /// 上下文管理器
    context_manager: Arc<RwLock<ContextManager>>,
    /// 任务执行器
    task_executor: Arc<RwLock<TaskExecutor>>,
}

impl RemoteServer {
    /// 创建新的远程服务器
    pub fn new(listen_address: String) -> Self {
        let server = ProtocolServer::new(listen_address);
        let context_manager = Arc::new(RwLock::new(ContextManager::new()));
        let task_executor = Arc::new(RwLock::new(TaskExecutor::new()));
        
        Self {
            server,
            context_manager,
            task_executor,
        }
    }
    
    /// 启动服务器
    pub async fn start(&self) -> Result<()> {
        self.server.start().await
    }
    
    /// 处理数据包
    pub async fn handle_packet(&self, packet: TeleportPacket) -> Result<TeleportPacket> {
        match packet.packet_type {
            PacketType::Context => {
                let context: ContextPacket = serde_json::from_value(packet.data)?;
                let mut context_manager = self.context_manager.write().await;
                context_manager.add_context(context).await;
                
                let result = ResultPacket {
                    task_id: "".to_string(),
                    status: "success".to_string(),
                    result: serde_json::Value::Null,
                    execution_time_ms: 0,
                    error: None,
                    output: "Context received".to_string(),
                };
                
                let result_data = serde_json::to_value(result)?;
                Ok(TeleportPacket::new(
                    PacketType::Result,
                    "remote".to_string(),
                    packet.source,
                    result_data,
                ))
            }
            PacketType::Task => {
                let task: TaskPacket = serde_json::from_value(packet.data)?;
                let mut task_executor = self.task_executor.write().await;
                let result = task_executor.execute_task(task).await;
                
                let result_data = serde_json::to_value(result)?;
                Ok(TeleportPacket::new(
                    PacketType::Result,
                    "remote".to_string(),
                    packet.source,
                    result_data,
                ))
            }
            _ => {
                let error = ErrorPacket {
                    error_code: 400,
                    error_message: "Unsupported packet type".to_string(),
                    error_details: None,
                    related_packet_id: Some(packet.id),
                };
                
                let error_data = serde_json::to_value(error)?;
                Ok(TeleportPacket::new(
                    PacketType::Error,
                    "remote".to_string(),
                    packet.source,
                    error_data,
                ))
            }
        }
    }
}

/// 上下文管理器
pub struct ContextManager {
    /// 上下文存储
    contexts: std::collections::HashMap<String, ContextPacket>,
}

impl ContextManager {
    /// 创建新的上下文管理器
    pub fn new() -> Self {
        Self {
            contexts: std::collections::HashMap::new(),
        }
    }
    
    /// 添加上下文
    pub async fn add_context(&mut self, context: ContextPacket) {
        self.contexts.insert(context.context_id.clone(), context);
    }
    
    /// 获取上下文
    pub async fn get_context(&self, context_id: &str) -> Option<ContextPacket> {
        self.contexts.get(context_id).cloned()
    }
    
    /// 删除上下文
    pub async fn remove_context(&mut self, context_id: &str) {
        self.contexts.remove(context_id);
    }
    
    /// 获取所有上下文
    pub async fn get_all_contexts(&self) -> Vec<ContextPacket> {
        self.contexts.values().cloned().collect()
    }
}

/// 任务执行器
pub struct TaskExecutor {
    /// 执行线程池
    executor: tokio::runtime::Runtime,
}

impl TaskExecutor {
    /// 创建新的任务执行器
    pub fn new() -> Self {
        let executor = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .build()
            .unwrap();
        
        Self {
            executor,
        }
    }
    
    /// 执行任务
    pub async fn execute_task(&self, task: TaskPacket) -> ResultPacket {
        let start_time = std::time::Instant::now();
        
        let result = self.executor.block_on(async {
            // 模拟任务执行
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            Ok(serde_json::Value::String("Task executed successfully"))
        });
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        match result {
            Ok(result_value) => ResultPacket {
                task_id: task.task_id,
                status: "success".to_string(),
                result: result_value,
                execution_time_ms: execution_time,
                error: None,
                output: "Task completed".to_string(),
            },
            Err(e) => ResultPacket {
                task_id: task.task_id,
                status: "error".to_string(),
                result: serde_json::Value::Null,
                execution_time_ms: execution_time,
                error: Some(e.to_string()),
                output: "Task failed".to_string(),
            },
        }
    }
}
