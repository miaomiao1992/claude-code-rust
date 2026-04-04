//! UDS Message 模块
//! 
//! 定义 UDS 消息的结构和类型

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// 文本消息
    Text,
    /// 代码消息
    Code,
    /// 命令消息
    Command,
    /// 通知消息
    Notification,
    /// 错误消息
    Error,
    /// 状态消息
    Status,
    /// 配置消息
    Config,
}

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    /// 低优先级
    Low,
    /// 普通优先级
    Normal,
    /// 高优先级
    High,
    /// 紧急优先级
    Urgent,
}

/// 消息来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSource {
    /// 来源ID
    pub id: String,
    /// 来源类型
    pub source_type: String,
    /// 来源地址
    pub address: String,
}

/// 消息目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTarget {
    /// 目标ID
    pub id: String,
    /// 目标类型
    pub target_type: String,
    /// 目标地址
    pub address: String,
}

/// UDS 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdsMessage {
    /// 消息ID
    pub id: String,
    /// 消息类型
    pub message_type: MessageType,
    /// 优先级
    pub priority: MessagePriority,
    /// 来源
    pub source: MessageSource,
    /// 目标
    pub target: MessageTarget,
    /// 消息内容
    pub content: String,
    /// 元数据
    pub metadata: serde_json::Value,
    /// 时间戳
    pub timestamp: String,
    /// 过期时间（可选）
    pub expiration: Option<String>,
}

impl UdsMessage {
    /// 创建新消息
    pub fn new(
        message_type: MessageType,
        priority: MessagePriority,
        source: MessageSource,
        target: MessageTarget,
        content: String,
    ) -> Self {
        Self {
            id: generate_message_id(),
            message_type,
            priority,
            source,
            target,
            content,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string(),
            expiration: None,
        }
    }
    
    /// 设置元数据
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// 设置过期时间
    pub fn with_expiration(mut self, expiration: String) -> Self {
        self.expiration = Some(expiration);
        self
    }
    
    /// 检查消息是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = &self.expiration {
            if let Ok(exp_time) = exp.parse::<u128>() {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                return now > exp_time;
            }
        }
        false
    }
}

/// 生成消息ID
fn generate_message_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("msg_{}_{}", 
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis(),
        rng.gen::<u32>()
    )
}

/// 消息状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    /// 待处理
    Pending,
    /// 处理中
    Processing,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已融合
    Fused,
    /// 已丢弃
    Dropped,
}

/// 消息处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResult {
    /// 消息ID
    pub message_id: String,
    /// 处理状态
    pub status: MessageStatus,
    /// 结果内容
    pub result: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 处理时间
    pub processing_time_ms: u64,
}
