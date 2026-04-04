//! 上下文压缩模块
//!
//! 提供上下文压缩功能，包括：
//! - 消息摘要生成
//! - 上下文截断
//! - 智能压缩策略
//! - 压缩历史管理

use api_client::{ApiClient, ApiMessage, ApiRole, MessageContent};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

/// 压缩配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactConfig {
    /// 启用压缩
    pub enabled: bool,
    /// 压缩策略
    pub strategy: CompressionStrategy,
    /// 保留消息数
    pub keep_messages: usize,
    /// 最大上下文长度
    pub max_context_length: usize,
    /// 压缩阈值
    pub compression_threshold: usize,
}

impl Default for CompactConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategy: CompressionStrategy::Smart,
            keep_messages: 10,
            max_context_length: 8000,
            compression_threshold: 6000,
        }
    }
}

/// 压缩策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// 截断：直接截断旧消息
    Truncate,
    /// 摘要：生成消息摘要
    Summarize,
    /// 智能：根据内容智能选择
    Smart,
    /// 保留最近：只保留最近的消息
    KeepRecent,
}

/// 压缩结果
#[derive(Debug, Clone)]
pub struct CompactResult {
    /// 原始消息数
    pub original_count: usize,
    /// 压缩后消息数
    pub compacted_count: usize,
    /// 移除的消息数
    pub removed_count: usize,
    /// 生成的摘要
    pub summary: Option<String>,
}

/// 压缩器
pub struct Compactor {
    config: CompactConfig,
}

impl Compactor {
    /// 创建新的压缩器
    pub fn new(config: CompactConfig) -> Self {
        Self { config }
    }

    /// 压缩消息（同步版本，不使用AI摘要）
    pub fn compact(&self, messages: &mut VecDeque<CompactMessage>) -> CompactResult {
        if !self.config.enabled || messages.len() <= self.config.keep_messages {
            return CompactResult {
                original_count: messages.len(),
                compacted_count: messages.len(),
                removed_count: 0,
                summary: None,
            };
        }

        let original_count = messages.len();

        match self.config.strategy {
            CompressionStrategy::Truncate => self.truncate(messages),
            CompressionStrategy::Summarize => self.summarize(messages),
            CompressionStrategy::Smart => self.smart_compact(messages),
            CompressionStrategy::KeepRecent => self.keep_recent(messages),
        };

        CompactResult {
            original_count,
            compacted_count: messages.len(),
            removed_count: original_count - messages.len(),
            summary: None,
        }
    }

    /// 截断策略
    fn truncate(&self, messages: &mut VecDeque<CompactMessage>) {
        while messages.len() > self.config.keep_messages {
            messages.pop_front();
        }
    }

    /// 摘要策略
    fn summarize(&self, messages: &mut VecDeque<CompactMessage>) {
        // 保留最近的keep_messages/2条消息
        let keep_recent = self.config.keep_messages / 2;

        // 将旧消息转换为摘要
        let old_messages: Vec<_> = messages
            .drain(0..messages.len().saturating_sub(keep_recent))
            .collect();

        if !old_messages.is_empty() {
            let summary = self.generate_summary(&old_messages);
            messages.push_front(CompactMessage {
                role: "system".to_string(),
                content: format!("[Previous conversation summary]: {}", summary),
                timestamp: old_messages.last().map(|m| m.timestamp).unwrap_or_default(),
            });
        }
    }

    /// 智能压缩策略
    fn smart_compact(&self, messages: &mut VecDeque<CompactMessage>) {
        let total_length: usize = messages.iter().map(|m| m.content.len()).sum();

        if total_length > self.config.max_context_length {
            // 如果上下文过长，使用摘要策略
            self.summarize(messages);
        } else if messages.len() > self.config.keep_messages * 2 {
            // 如果消息数过多，使用保留最近策略
            self.keep_recent(messages);
        }
    }

    /// 保留最近策略
    fn keep_recent(&self, messages: &mut VecDeque<CompactMessage>) {
        let keep = self.config.keep_messages;
        while messages.len() > keep {
            messages.pop_front();
        }
    }

    /// 生成摘要
    fn generate_summary(&self, messages: &[CompactMessage]) -> String {
        // 简化实现：返回消息数量和内容概要
        let user_messages: Vec<_> = messages
            .iter()
            .filter(|m| m.role == "user")
            .map(|m| &m.content)
            .collect();

        if user_messages.is_empty() {
            return "No user messages to summarize".to_string();
        }

        // 实际应用中应该调用LLM生成摘要
        format!(
            "{} messages including {} user queries",
            messages.len(),
            user_messages.len()
        )
    }

    /// 异步压缩消息（使用 AI 生成摘要）
    pub async fn compact_async(
        &self,
        messages: &mut VecDeque<CompactMessage>,
        api_client: Arc<ApiClient>,
    ) -> crate::error::Result<CompactResult> {
        if !self.config.enabled || messages.len() <= self.config.keep_messages {
            return Ok(CompactResult {
                original_count: messages.len(),
                compacted_count: messages.len(),
                removed_count: 0,
                summary: None,
            });
        }

        let original_count = messages.len();

        match self.config.strategy {
            CompressionStrategy::Truncate => self.truncate(messages),
            CompressionStrategy::Summarize => {
                // 使用 AI 生成摘要
                let keep_recent = self.config.keep_messages / 2;
                let old_messages: Vec<_> = messages
                    .drain(0..messages.len().saturating_sub(keep_recent))
                    .collect();

                if !old_messages.is_empty() {
                    let summary = self.generate_summary_async(&old_messages, api_client).await?;
                    messages.push_front(CompactMessage {
                        role: "system".to_string(),
                        content: format!("[Previous conversation summary]: {}", summary),
                        timestamp: old_messages.last().map(|m| m.timestamp).unwrap_or_default(),
                    });
                }
            }
            CompressionStrategy::Smart => {
                let total_length: usize = messages.iter().map(|m| m.content.len()).sum();

                if total_length > self.config.max_context_length {
                    // 如果上下文过长，使用摘要策略
                    let keep_recent = self.config.keep_messages / 2;
                    let old_messages: Vec<_> = messages
                        .drain(0..messages.len().saturating_sub(keep_recent))
                        .collect();

                    if !old_messages.is_empty() {
                        let summary = self.generate_summary_async(&old_messages, api_client).await?;
                        messages.push_front(CompactMessage {
                            role: "system".to_string(),
                            content: format!("[Previous conversation summary]: {}", summary),
                            timestamp: old_messages.last().map(|m| m.timestamp).unwrap_or_default(),
                        });
                    }
                } else if messages.len() > self.config.keep_messages * 2 {
                    // 如果消息数过多，使用保留最近策略
                    self.keep_recent(messages);
                }
            }
            CompressionStrategy::KeepRecent => self.keep_recent(messages),
        }

        Ok(CompactResult {
            original_count,
            compacted_count: messages.len(),
            removed_count: original_count - messages.len(),
            summary: None,
        })
    }

    /// 异步生成摘要（使用 AI）
    async fn generate_summary_async(
        &self,
        messages: &[CompactMessage],
        api_client: Arc<ApiClient>,
    ) -> crate::error::Result<String> {
        // 构建压缩提示
        let mut message_content = String::new();
        message_content.push_str("Please compress the following conversation history into a concise summary.\n");
        message_content.push_str("Preserve all key information, decisions, and important context.\n");
        message_content.push_str("Remove redundant details but keep all critical information.\n\n");
        message_content.push_str("--- Conversation to compress ---\n");

        for msg in messages {
            message_content.push_str(&format!("[{}]: {}\n\n", msg.role, msg.content));
        }

        message_content.push_str("--- End of conversation ---\n\n");
        message_content.push_str("Provide a concise summary:");

        // 调用 API 生成摘要
        let request = ApiMessage {
            role: ApiRole::User,
            content: MessageContent::Text(message_content),
        };

        let model = api_client::types::ApiModel::Claude35Haiku20241022;
        let response = api_client
            .send_message(&message_content, Some(model))
            .await
            .map_err(|e| crate::error::Error::Other(format!("Failed to generate summary: {}", e)))?;

        Ok(response.trim().to_string())
    }

    /// 检查是否需要压缩
    pub fn needs_compaction(&self, messages: &[CompactMessage]) -> bool {
        if !self.config.enabled {
            return false;
        }

        let total_length: usize = messages.iter().map(|m| m.content.len()).sum();
        messages.len() > self.config.keep_messages * 2
            || total_length > self.config.compression_threshold
    }
}

/// 可压缩消息
#[derive(Debug, Clone)]
pub struct CompactMessage {
    /// 角色
    pub role: String,
    /// 内容
    pub content: String,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 压缩历史
pub struct CompactHistory {
    /// 压缩记录
    history: Vec<CompactResult>,
    /// 最大历史记录数
    max_history: usize,
}

impl CompactHistory {
    /// 创建新的压缩历史
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::new(),
            max_history,
        }
    }

    /// 添加压缩记录
    pub fn add(&mut self, result: CompactResult) {
        self.history.push(result);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// 获取压缩历史
    pub fn history(&self) -> &[CompactResult] {
        &self.history
    }

    /// 获取压缩统计
    pub fn stats(&self) -> CompactStats {
        let total_removed: usize = self.history.iter().map(|r| r.removed_count).sum();
        CompactStats {
            total_compactions: self.history.len(),
            total_messages_removed: total_removed,
            average_removed: if self.history.is_empty() {
                0.0
            } else {
                total_removed as f64 / self.history.len() as f64
            },
        }
    }
}

/// 压缩统计
#[derive(Debug, Clone)]
pub struct CompactStats {
    /// 总压缩次数
    pub total_compactions: usize,
    /// 总移除消息数
    pub total_messages_removed: usize,
    /// 平均每次移除消息数
    pub average_removed: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_strategy() {
        let config = CompactConfig {
            enabled: true,
            strategy: CompressionStrategy::Truncate,
            keep_messages: 3,
            max_context_length: 1000,
            compression_threshold: 800,
        };

        let compactor = Compactor::new(config);
        let mut messages: VecDeque<CompactMessage> = (0..10)
            .map(|i| CompactMessage {
                role: "user".to_string(),
                content: format!("Message {}", i),
                timestamp: chrono::Utc::now(),
            })
            .collect();

        let result = compactor.compact(&mut messages);
        assert_eq!(result.original_count, 10);
        assert_eq!(result.compacted_count, 3);
        assert_eq!(result.removed_count, 7);
    }

    #[test]
    fn test_needs_compaction() {
        let config = CompactConfig::default();
        let compactor = Compactor::new(config);

        let long_messages: Vec<CompactMessage> = (0..100)
            .map(|_| CompactMessage {
                role: "user".to_string(),
                content: "x".repeat(100),
                timestamp: chrono::Utc::now(),
            })
            .collect();

        assert!(compactor.needs_compaction(&long_messages));

        let short_messages: Vec<CompactMessage> = (0..5)
            .map(|i| CompactMessage {
                role: "user".to_string(),
                content: format!("Short {}", i),
                timestamp: chrono::Utc::now(),
            })
            .collect();

        assert!(!compactor.needs_compaction(&short_messages));
    }
}
