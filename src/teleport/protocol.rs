//! Teleport Protocol 模块
//! 
//! 实现传输协议，包括连接管理、数据传输和错误处理

use super::packet::*;
use crate::error::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

/// 传输协议
pub struct TeleportProtocol {
    /// 流
    stream: TcpStream,
    /// 发送通道
    send_channel: mpsc::Sender<TeleportPacket>,
    /// 接收通道
    recv_channel: mpsc::Receiver<TeleportPacket>,
}

impl TeleportProtocol {
    /// 创建新的传输协议
    pub async fn new(stream: TcpStream) -> Result<Self> {
        let (send_channel, send_receiver) = mpsc::channel(100);
        let (recv_sender, recv_channel) = mpsc::channel(100);
        
        let mut write_stream = stream.clone();
        tokio::spawn(async move {
            while let Some(packet) = send_receiver.recv().await {
                if let Ok(data) = packet.serialize() {
                    let len = data.len() as u32;
                    if write_stream.write_u32(len).await.is_err() {
                        break;
                    }
                    if write_stream.write_all(data.as_bytes()).await.is_err() {
                        break;
                    }
                }
            }
        });
        
        let mut read_stream = stream.clone();
        tokio::spawn(async move {
            loop {
                let mut len_buf = [0u8; 4];
                if read_stream.read_exact(&mut len_buf).await.is_err() {
                    break;
                }
                let len = u32::from_be_bytes(len_buf) as usize;
                
                let mut data_buf = vec![0u8; len];
                if read_stream.read_exact(&mut data_buf).await.is_err() {
                    break;
                }
                
                if let Ok(data_str) = String::from_utf8(data_buf) {
                    if let Ok(packet) = TeleportPacket::deserialize(&data_str) {
                        let _ = recv_sender.send(packet).await;
                    }
                }
            }
        });
        
        Ok(Self {
            stream,
            send_channel,
            recv_channel,
        })
    }
    
    /// 发送数据包
    pub async fn send_packet(&self, packet: TeleportPacket) -> Result<()> {
        self.send_channel.send(packet).await.map_err(|e| e.into())
    }
    
    /// 接收数据包
    pub async fn recv_packet(&mut self) -> Option<TeleportPacket> {
        self.recv_channel.recv().await
    }
    
    /// 关闭连接
    pub async fn close(&mut self) -> Result<()> {
        self.stream.shutdown().await.map_err(|e| e.into())
    }
}

/// 协议客户端
pub struct ProtocolClient {
    /// 协议
    protocol: TeleportProtocol,
    /// 服务器地址
    server_address: String,
}

impl ProtocolClient {
    /// 创建新的协议客户端
    pub async fn new(server_address: String) -> Result<Self> {
        let stream = TcpStream::connect(&server_address).await?;
        let protocol = TeleportProtocol::new(stream).await?;
        
        Ok(Self {
            protocol,
            server_address,
        })
    }
    
    /// 发送数据包
    pub async fn send_packet(&self, packet: TeleportPacket) -> Result<()> {
        self.protocol.send_packet(packet).await
    }
    
    /// 接收数据包
    pub async fn recv_packet(&mut self) -> Option<TeleportPacket> {
        self.protocol.recv_packet().await
    }
    
    /// 关闭连接
    pub async fn close(&mut self) -> Result<()> {
        self.protocol.close().await
    }
}

/// 协议服务器
pub struct ProtocolServer {
    /// 监听地址
    listen_address: String,
    /// 连接处理通道
    connection_sender: mpsc::Sender<TeleportProtocol>,
}

impl ProtocolServer {
    /// 创建新的协议服务器
    pub fn new(listen_address: String) -> Self {
        let (connection_sender, _) = mpsc::channel(100);
        
        Self {
            listen_address,
            connection_sender,
        }
    }
    
    /// 启动服务器
    pub async fn start(&self) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(&self.listen_address).await?;
        tracing::info!("Teleport server started on {}", self.listen_address);
        
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    tracing::info!("New connection from {}", addr);
                    let connection_sender = self.connection_sender.clone();
                    
                    tokio::spawn(async move {
                        if let Ok(protocol) = TeleportProtocol::new(stream).await {
                            let _ = connection_sender.send(protocol).await;
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Error accepting connection: {}", e);
                }
            }
        }
    }
}

/// 协议错误
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    /// 连接错误
    #[error("Connection error: {0}")]
    ConnectionError(#[from] std::io::Error),
    
    /// 序列化错误
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// 超时错误
    #[error("Timeout error")]
    TimeoutError,
    
    /// 认证错误
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    /// 数据包错误
    #[error("Packet error: {0}")]
    PacketError(String),
}
