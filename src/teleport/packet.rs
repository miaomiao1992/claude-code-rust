//! Teleport Packet 模块
//! 
//! 实现消息打包和序列化功能

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// 数据包类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketType {
    /// 上下文传递
    Context,
    /// 任务执行
    Task,
    /// 执行结果
    Result,
    /// 错误信息
    Error,
    /// 心跳包
    Heartbeat,
    /// 认证信息
    Auth,
    /// 控制命令
    Control,
}

/// 数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeleportPacket {
    /// 包ID
    pub id: String,
    /// 包类型
    pub packet_type: PacketType,
    /// 源地址
    pub source: String,
    /// 目标地址
    pub destination: String,
    /// 数据内容
    pub data: serde_json::Value,
    /// 元数据
    pub metadata: serde_json::Value,
    /// 时间戳
    pub timestamp: String,
    /// 过期时间（可选）
    pub expiration: Option<String>,
    /// 签名（可选）
    pub signature: Option<String>,
}

impl TeleportPacket {
    /// 创建新数据包
    pub fn new(
        packet_type: PacketType,
        source: String,
        destination: String,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: generate_packet_id(),
            packet_type,
            source,
            destination,
            data,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string(),
            expiration: None,
            signature: None,
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
    
    /// 设置签名
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }
    
    /// 检查数据包是否过期
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
    
    /// 序列化数据包
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    /// 反序列化数据包
    pub fn deserialize(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

/// 生成数据包ID
fn generate_packet_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("packet_{}_{}", 
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis(),
        rng.gen::<u32>()
    )
}

/// 上下文数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacket {
    /// 上下文ID
    pub context_id: String,
    /// 上下文类型
    pub context_type: String,
    /// 上下文内容
    pub content: serde_json::Value,
    /// 上下文版本
    pub version: String,
    /// 相关文件
    pub files: Vec<String>,
    /// 环境变量
    pub environment: std::collections::HashMap<String, String>,
}

/// 任务数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPacket {
    /// 任务ID
    pub task_id: String,
    /// 任务类型
    pub task_type: String,
    /// 任务参数
    pub parameters: serde_json::Value,
    /// 执行超时（秒）
    pub timeout: u32,
    /// 优先级
    pub priority: u8,
    /// 依赖任务
    pub dependencies: Vec<String>,
}

/// 结果数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultPacket {
    /// 任务ID
    pub task_id: String,
    /// 执行状态
    pub status: String,
    /// 执行结果
    pub result: serde_json::Value,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 错误信息（可选）
    pub error: Option<String>,
    /// 输出日志
    pub output: String,
}

/// 错误数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPacket {
    /// 错误代码
    pub error_code: u32,
    /// 错误消息
    pub error_message: String,
    /// 错误详情
    pub error_details: Option<String>,
    /// 相关数据包ID
    pub related_packet_id: Option<String>,
}

/// 心跳数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPacket {
    /// 节点ID
    pub node_id: String,
    /// 节点状态
    pub status: String,
    /// 负载信息
    pub load: f32,
    /// 可用内存
    pub memory_available: u64,
    /// 上次活动时间
    pub last_activity: String,
}

/// 认证数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthPacket {
    /// 认证类型
    pub auth_type: String,
    /// 认证数据
    pub auth_data: serde_json::Value,
    /// 过期时间
    pub expires_at: String,
}

/// 控制命令数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPacket {
    /// 命令类型
    pub command: String,
    /// 命令参数
    pub arguments: serde_json::Value,
    /// 执行目标
    pub target: String,
}
