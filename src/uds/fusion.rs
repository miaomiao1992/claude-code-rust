//! UDS Fusion 模块
//! 
//! 实现消息融合算法，将多个相关消息融合成一个更有意义的消息

use super::message::*;
use std::collections::HashMap;
use std::time::SystemTime;

/// 消息融合器
pub struct MessageFusion {
    /// 融合规则
    fusion_rules: Vec<Box<dyn FusionRule>>,
    /// 消息分组器
    message_grouper: MessageGrouper,
}

impl MessageFusion {
    /// 创建新的消息融合器
    pub fn new() -> Self {
        let mut fusion_rules = Vec::new();
        
        // 添加默认的融合规则
        fusion_rules.push(Box::new(TextMessageFusion));
        fusion_rules.push(Box::new(CodeMessageFusion));
        fusion_rules.push(Box::new(NotificationMessageFusion));
        
        Self {
            fusion_rules,
            message_grouper: MessageGrouper::new(),
        }
    }
    
    /// 融合消息
    pub async fn fuse_messages(&self, messages: Vec<UdsMessage>) -> Vec<UdsMessage> {
        // 分组消息
        let grouped_messages = self.message_grouper.group_messages(messages).await;
        
        let mut fused_messages = Vec::new();
        
        // 对每个组应用融合规则
        for (group_key, group_messages) in grouped_messages {
            if group_messages.len() == 1 {
                // 只有一条消息，不需要融合
                fused_messages.push(group_messages[0].clone());
            } else {
                // 尝试应用融合规则
                let mut fused = false;
                
                for rule in &self.fusion_rules {
                    if rule.can_fuse(&group_messages) {
                        if let Some(fused_message) = rule.fuse(&group_messages) {
                            fused_messages.push(fused_message);
                            fused = true;
                            break;
                        }
                    }
                }
                
                if !fused {
                    // 没有适用的融合规则，使用第一条消息
                    fused_messages.push(group_messages[0].clone());
                }
            }
        }
        
        fused_messages
    }
}

/// 消息分组器
struct MessageGrouper {
    /// 分组时间窗口（毫秒）
    time_window: u128,
}

impl MessageGrouper {
    /// 创建新的消息分组器
    pub fn new() -> Self {
        Self {
            time_window: 5000, // 5秒
        }
    }
    
    /// 分组消息
    pub async fn group_messages(&self, messages: Vec<UdsMessage>) -> HashMap<String, Vec<UdsMessage>> {
        let mut groups = HashMap::new();
        
        for message in messages {
            // 生成分组键
            let group_key = self.generate_group_key(&message);
            
            // 查找或创建组
            let group = groups.entry(group_key).or_insert_with(Vec::new);
            
            // 检查是否在时间窗口内
            if let Some(last_message) = group.last() {
                let last_time = last_message.timestamp.parse().unwrap_or(0);
                let current_time = message.timestamp.parse().unwrap_or(0);
                
                if current_time - last_time <= self.time_window {
                    group.push(message);
                } else {
                    // 创建新组
                    let new_group_key = format!("{}:{}", group_key, current_time);
                    groups.entry(new_group_key).or_insert_with(Vec::new).push(message);
                }
            } else {
                group.push(message);
            }
        }
        
        groups
    }
    
    /// 生成分组键
    fn generate_group_key(&self, message: &UdsMessage) -> String {
        format!("{}_{}_{}", 
            message.source.id, 
            message.target.id, 
            message.message_type as u8
        )
    }
}

/// 融合规则 trait
pub trait FusionRule {
    /// 检查是否可以融合这些消息
    fn can_fuse(&self, messages: &[UdsMessage]) -> bool;
    
    /// 融合消息
    fn fuse(&self, messages: &[UdsMessage]) -> Option<UdsMessage>;
}

/// 文本消息融合规则
struct TextMessageFusion;

impl FusionRule for TextMessageFusion {
    fn can_fuse(&self, messages: &[UdsMessage]) -> bool {
        messages.len() >= 2 && 
        messages.iter().all(|m| m.message_type == MessageType::Text)
    }
    
    fn fuse(&self, messages: &[UdsMessage]) -> Option<UdsMessage> {
        if messages.is_empty() {
            return None;
        }
        
        // 合并文本内容
        let content = messages
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<String>>()
            .join("\n");
        
        // 使用第一条消息的元数据
        let first_message = &messages[0];
        
        Some(UdsMessage::new(
            MessageType::Text,
            first_message.priority,
            first_message.source.clone(),
            first_message.target.clone(),
            content,
        ).with_metadata(first_message.metadata.clone()))
    }
}

/// 代码消息融合规则
struct CodeMessageFusion;

impl FusionRule for CodeMessageFusion {
    fn can_fuse(&self, messages: &[UdsMessage]) -> bool {
        messages.len() >= 2 && 
        messages.iter().all(|m| m.message_type == MessageType::Code)
    }
    
    fn fuse(&self, messages: &[UdsMessage]) -> Option<UdsMessage> {
        if messages.is_empty() {
            return None;
        }
        
        // 合并代码内容
        let content = messages
            .iter()
            .map(|m| format!("// Message from {} at {}\n{}", 
                m.source.id, m.timestamp, m.content))
            .collect::<Vec<String>>()
            .join("\n\n");
        
        let first_message = &messages[0];
        
        Some(UdsMessage::new(
            MessageType::Code,
            first_message.priority,
            first_message.source.clone(),
            first_message.target.clone(),
            content,
        ).with_metadata(first_message.metadata.clone()))
    }
}

/// 通知消息融合规则
struct NotificationMessageFusion;

impl FusionRule for NotificationMessageFusion {
    fn can_fuse(&self, messages: &[UdsMessage]) -> bool {
        messages.len() >= 2 && 
        messages.iter().all(|m| m.message_type == MessageType::Notification)
    }
    
    fn fuse(&self, messages: &[UdsMessage]) -> Option<UdsMessage> {
        if messages.is_empty() {
            return None;
        }
        
        // 合并通知内容
        let content = format!(
            "Multiple notifications ({})\n{}",
            messages.len(),
            messages
                .iter()
                .map(|m| format!("- {}", m.content))
                .collect::<Vec<String>>()
                .join("\n")
        );
        
        let first_message = &messages[0];
        
        Some(UdsMessage::new(
            MessageType::Notification,
            first_message.priority,
            first_message.source.clone(),
            first_message.target.clone(),
            content,
        ).with_metadata(first_message.metadata.clone()))
    }
}

/// 融合策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FusionStrategy {
    /// 不融合
    None,
    /// 简单融合（基于时间窗口）
    Simple,
    /// 智能融合（基于内容相似性）
    Smart,
    /// 完全融合（所有相关消息）
    Complete,
}
