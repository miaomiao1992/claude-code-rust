//! UDS Server 模块
//! 
//! 实现 Unix Domain Socket 服务端，处理消息的接收和发送

use super::message::*;
use super::queue::*;
use super::fusion::*;
use crate::error::Result;
use serde_json::json;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::mpsc;

/// UDS 服务器
pub struct UdsServer {
    /// 监听地址
    socket_path: String,
    /// 队列管理器
    queue_manager: QueueManager,
    /// 消息融合器
    message_fusion: MessageFusion,
    /// 处理消息的通道
    message_sender: mpsc::Sender<UdsMessage>,
    /// 停止信号
    shutdown_sender: mpsc::Sender<()>,
}

impl UdsServer {
    /// 创建新的 UDS 服务器
    pub fn new(socket_path: String) -> Self {
        let (message_sender, _) = mpsc::channel(100);
        let (shutdown_sender, _) = mpsc::channel(1);
        
        Self {
            socket_path,
            queue_manager: QueueManager::new(1000),
            message_fusion: MessageFusion::new(),
            message_sender,
            shutdown_sender,
        }
    }
    
    /// 启动服务器
    pub async fn start(&self) -> Result<()> {
        // 移除现有的 socket 文件
        if Path::new(&self.socket_path).exists() {
            std::fs::remove_file(&self.socket_path)?;
        }
        
        // 创建监听器
        let listener = UnixListener::bind(&self.socket_path)?;
        tracing::info!("UDS server started on {}", self.socket_path);
        
        // 处理连接
        loop {
            tokio::select! {
                Ok((stream, _)) = listener.accept() => {
                    let message_sender = self.message_sender.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, message_sender).await {
                            tracing::error!("Error handling connection: {}", e);
                        }
                    });
                }
                _ = self.shutdown_sender.clone().recv() => {
                    tracing::info!("Shutting down UDS server");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// 停止服务器
    pub async fn stop(&self) -> Result<()> {
        let _ = self.shutdown_sender.send(()).await;
        
        // 移除 socket 文件
        if Path::new(&self.socket_path).exists() {
            std::fs::remove_file(&self.socket_path)?;
        }
        
        Ok(())
    }
    
    /// 获取队列状态
    pub async fn get_queue_status(&self) -> QueueStatus {
        self.queue_manager.get_status().await
    }
    
    /// 处理消息
    pub async fn process_messages(&self) -> Result<()> {
        let mut message_receiver = self.message_sender.subscribe();
        
        while let Some(message) = message_receiver.recv().await {
            // 添加到队列
            self.queue_manager.add_message(message).await?;
        }
        
        Ok(())
    }
}

/// 处理连接
async fn handle_connection(
    mut stream: tokio::net::UnixStream,
    message_sender: mpsc::Sender<UdsMessage>,
) -> Result<()> {
    let mut buffer = vec![0; 4096];
    
    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        
        // 解析消息
        let message_str = String::from_utf8_lossy(&buffer[..n]);
        if let Ok(message) = serde_json::from_str::<UdsMessage>(&message_str) {
            // 发送消息到处理通道
            message_sender.send(message).await?;
            
            // 发送确认
            let response = json!({"status": "ok"});
            stream.write_all(response.to_string().as_bytes()).await?;
        } else {
            // 发送错误响应
            let response = json!({"status": "error", "message": "Invalid message format"});
            stream.write_all(response.to_string().as_bytes()).await?;
        }
    }
    
    Ok(())
}

/// UDS 客户端
pub struct UdsClient {
    /// Socket 路径
    socket_path: String,
}

impl UdsClient {
    /// 创建新的 UDS 客户端
    pub fn new(socket_path: String) -> Self {
        Self { socket_path }
    }
    
    /// 发送消息
    pub async fn send_message(&self, message: UdsMessage) -> Result<MessageResult> {
        // 连接到服务器
        let mut stream = tokio::net::UnixStream::connect(&self.socket_path).await?;
        
        // 发送消息
        let message_str = serde_json::to_string(&message)?;
        stream.write_all(message_str.as_bytes()).await?;
        
        // 读取响应
        let mut buffer = vec![0; 1024];
        let n = stream.read(&mut buffer).await?;
        let response_str = String::from_utf8_lossy(&buffer[..n]);
        
        // 解析响应
        let response: serde_json::Value = serde_json::from_str(&response_str)?;
        
        if response["status"] == "ok" {
            Ok(MessageResult {
                message_id: message.id,
                status: MessageStatus::Completed,
                result: Some("Message sent successfully".to_string()),
                error: None,
                processing_time_ms: 0,
            })
        } else {
            Ok(MessageResult {
                message_id: message.id,
                status: MessageStatus::Failed,
                result: None,
                error: response["message"].as_str().map(|s| s.to_string()),
                processing_time_ms: 0,
            })
        }
    }
}
