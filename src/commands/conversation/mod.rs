//! 对话管理命令模块
//!
//! 这个模块实现了对话相关的命令，包括：
//! - `/resume` - 恢复之前的对话
//! - `/compact` - 压缩对话 (保留摘要)
//! - `/export` - 导出对话
//! - `/rename` - 重命名当前对话
//! - `/rewind` - 回退到之前的点

use crate::error::{ClaudeError, Result};
use crate::commands::registry::{CommandLoader, CommandRegistry};

pub mod resume;
pub mod compact;
pub mod export;
pub mod rename;
pub mod rewind;

pub use resume::ResumeCommand;
pub use compact::CompactCommand;
pub use export::ExportCommand;
pub use rename::RenameCommand;
pub use rewind::RewindCommand;

/// 对话管理命令加载器
pub struct ConversationCommandLoader;

#[async_trait::async_trait]
impl CommandLoader for ConversationCommandLoader {
    async fn load(&self, registry: &CommandRegistry) -> Result<()> {
        registry.register(ResumeCommand).await?;
        registry.register(CompactCommand).await?;
        registry.register(ExportCommand).await?;
        registry.register(RenameCommand).await?;
        registry.register(RewindCommand).await?;

        tracing::debug!("Loaded conversation management commands");

        Ok(())
    }

    fn name(&self) -> &str {
        "conversation"
    }
}

/// 对话错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConversationError {
    #[error("对话未找到: {name}")]
    ConversationNotFound { name: String },

    #[error("会话历史为空")]
    EmptyHistory,

    #[error("导出失败: {message}")]
    ExportFailed { message: String },

    #[error("压缩失败: {message}")]
    CompactFailed { message: String },

    #[error("无效的时间点: {point}")]
    InvalidRewindPoint { point: String },

    #[error("参数错误: {message}")]
    InvalidArguments { message: String },
}

impl From<ConversationError> for ClaudeError {
    fn from(err: ConversationError) -> Self {
        ClaudeError::Command(format!("Conversation error: {}", err))
    }
}

/// 对话信息
#[derive(Debug, Clone)]
pub struct ConversationInfo {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub message_count: usize,
    pub token_count: usize,
}

/// 对话存储接口
pub trait ConversationStorage: Send + Sync {
    /// 保存对话
    fn save(&self, conversation: &ConversationInfo, messages: &[ConversationMessage]) -> Result<()>;

    /// 加载对话
    fn load(&self, id: &str) -> Result<(Option<ConversationInfo>, Vec<ConversationMessage>)>;

    /// 列出所有对话
    fn list(&self) -> Result<Vec<ConversationInfo>>;

    /// 删除对话
    fn delete(&self, id: &str) -> Result<()>;

    /// 重命名对话
    fn rename(&self, id: &str, new_name: &str) -> Result<()>;
}

/// 对话消息
#[derive(Debug, Clone)]
pub struct ConversationMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 文件系统对话存储实现
pub struct FileConversationStorage {
    storage_dir: std::path::PathBuf,
}

impl FileConversationStorage {
    pub fn new(storage_dir: std::path::PathBuf) -> Self {
        Self { storage_dir }
    }

    fn conversation_path(&self, id: &str) -> std::path::PathBuf {
        self.storage_dir.join(format!("{}.json", id))
    }
}

impl ConversationStorage for FileConversationStorage {
    fn save(
        &self,
        conversation: &ConversationInfo,
        messages: &[ConversationMessage],
    ) -> Result<()> {
        use std::fs;
        use std::io::Write;

        // 确保存储目录存在
        fs::create_dir_all(&self.storage_dir)?;

        // 序列化对话数据
        let data = serde_json::json!({
            "info": {
                "id": conversation.id,
                "name": conversation.name,
                "created_at": conversation.created_at,
                "updated_at": conversation.updated_at,
                "message_count": conversation.message_count,
                "token_count": conversation.token_count,
            },
            "messages": messages.iter().map(|m| serde_json::json!({
                "id": m.id,
                "role": m.role,
                "content": m.content,
                "timestamp": m.timestamp,
            })).collect::<Vec<_>(),
        });

        let path = self.conversation_path(&conversation.id);
        let mut file = fs::File::create(path)?;
        file.write_all(data.to_string().as_bytes())?;

        Ok(())
    }

    fn load(
        &self,
        id: &str,
    ) -> Result<(Option<ConversationInfo>, Vec<ConversationMessage>)> {
        use std::fs;

        let path = self.conversation_path(id);
        if !path.exists() {
            return Ok((None, Vec::new()));
        }

        let content = fs::read_to_string(path)?;
        let data: serde_json::Value = serde_json::from_str(&content)?;

        let info = ConversationInfo {
            id: data["info"]["id"].as_str().unwrap_or(id).to_string(),
            name: data["info"]["name"].as_str().unwrap_or("Unnamed").to_string(),
            created_at: chrono::DateTime::parse_from_rfc3339(
                data["info"]["created_at"].as_str().unwrap_or("1970-01-01T00:00:00Z")
            )
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(
                data["info"]["updated_at"].as_str().unwrap_or("1970-01-01T00:00:00Z")
            )
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
            message_count: data["info"]["message_count"].as_u64().unwrap_or(0) as usize,
            token_count: data["info"]["token_count"].as_u64().unwrap_or(0) as usize,
        };

        let messages: Vec<ConversationMessage> = data["messages"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|m| ConversationMessage {
                id: m["id"].as_str().unwrap_or("").to_string(),
                role: m["role"].as_str().unwrap_or("user").to_string(),
                content: m["content"].as_str().unwrap_or("").to_string(),
                timestamp: chrono::DateTime::parse_from_rfc3339(
                    m["timestamp"].as_str().unwrap_or("1970-01-01T00:00:00Z")
                )
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            })
            .collect();

        Ok((Some(info), messages))
    }

    fn list(&self) -> Result<Vec<ConversationInfo>> {
        use std::fs;

        let mut conversations = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.storage_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") {
                        let id = name.trim_end_matches(".json");
                        if let Ok((Some(info), _)) = self.load(id) {
                            conversations.push(info);
                        }
                    }
                }
            }
        }

        // 按更新时间排序
        conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(conversations)
    }

    fn delete(&self, id: &str) -> Result<()> {
        use std::fs;

        let path = self.conversation_path(id);
        if path.exists() {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    fn rename(&self, id: &str, new_name: &str) -> Result<()> {
        use std::fs;

        let path = self.conversation_path(id);
        if !path.exists() {
            return Err(ConversationError::ConversationNotFound {
                name: id.to_string(),
            }
            .into());
        }

        let content = fs::read_to_string(&path)?;
        let mut data: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(info) = data.get_mut("info") {
            info["name"] = serde_json::Value::String(new_name.to_string());
            info["updated_at"] = serde_json::Value::String(chrono::Utc::now().to_rfc3339());
        }

        fs::write(path, data.to_string())?;

        Ok(())
    }
}
