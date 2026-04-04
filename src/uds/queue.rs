//! UDS Queue 模块
//! 
//! 实现消息队列管理和优先级处理

use super::message::*;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 消息队列项（用于优先级队列）
#[derive(Debug, Clone)]
pub struct QueueItem {
    /// 优先级（用于排序）
    pub priority: u8,
    /// 时间戳（用于相同优先级时的排序）
    pub timestamp: u128,
    /// 消息
    pub message: UdsMessage,
}

impl PartialEq for QueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl Eq for QueueItem {}

impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 优先级高的排在前面
        let priority_cmp = other.priority.cmp(&self.priority);
        if priority_cmp != std::cmp::Ordering::Equal {
            return priority_cmp;
        }
        // 时间戳小的排在前面（先到先处理）
        self.timestamp.cmp(&other.timestamp)
    }
}

/// 消息队列
pub struct MessageQueue {
    /// 优先级队列
    queue: Arc<RwLock<BinaryHeap<QueueItem>>>,
    /// 队列大小限制
    max_size: usize,
}

impl MessageQueue {
    /// 创建新的消息队列
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            max_size,
        }
    }
    
    /// 添加消息到队列
    pub async fn push(&self, message: UdsMessage) -> Result<()> {
        let mut queue = self.queue.write().await;
        
        // 检查队列大小
        if queue.len() >= self.max_size {
            // 移除最低优先级的消息
            queue.pop();
        }
        
        // 转换优先级为数值
        let priority = match message.priority {
            MessagePriority::Low => 0,
            MessagePriority::Normal => 1,
            MessagePriority::High => 2,
            MessagePriority::Urgent => 3,
        };
        
        // 解析时间戳
        let timestamp = message.timestamp.parse().unwrap_or(0);
        
        // 添加到队列
        queue.push(QueueItem {
            priority,
            timestamp,
            message,
        });
        
        Ok(())
    }
    
    /// 从队列中获取消息
    pub async fn pop(&self) -> Option<UdsMessage> {
        let mut queue = self.queue.write().await;
        
        // 弹出消息并检查是否过期
        while let Some(item) = queue.pop() {
            if !item.message.is_expired() {
                return Some(item.message);
            }
        }
        
        None
    }
    
    /// 获取队列大小
    pub async fn size(&self) -> usize {
        let queue = self.queue.read().await;
        queue.len()
    }
    
    /// 清空队列
    pub async fn clear(&self) {
        let mut queue = self.queue.write().await;
        queue.clear();
    }
    
    /// 检查队列是否为空
    pub async fn is_empty(&self) -> bool {
        let queue = self.queue.read().await;
        queue.is_empty()
    }
    
    /// 获取队列中的消息数量（按优先级）
    pub async fn get_count_by_priority(&self) -> std::collections::HashMap<MessagePriority, usize> {
        let queue = self.queue.read().await;
        let mut counts = std::collections::HashMap::new();
        
        for item in queue.iter() {
            *counts.entry(item.message.priority).or_insert(0) += 1;
        }
        
        counts
    }
}

/// 消息队列管理器
pub struct QueueManager {
    /// 消息队列
    queue: MessageQueue,
    /// 处理中的消息
    processing: Arc<RwLock<std::collections::HashMap<String, UdsMessage>>>,
    /// 已完成的消息
    completed: Arc<RwLock<std::collections::HashMap<String, MessageResult>>>,
}

impl QueueManager {
    /// 创建新的队列管理器
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            queue: MessageQueue::new(max_queue_size),
            processing: Arc::new(RwLock::new(std::collections::HashMap::new())),
            completed: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    /// 添加消息到队列
    pub async fn add_message(&self, message: UdsMessage) -> Result<()> {
        self.queue.push(message).await
    }
    
    /// 获取下一个要处理的消息
    pub async fn get_next_message(&self) -> Option<UdsMessage> {
        if let Some(message) = self.queue.pop().await {
            // 将消息标记为处理中
            let mut processing = self.processing.write().await;
            processing.insert(message.id.clone(), message.clone());
            Some(message)
        } else {
            None
        }
    }
    
    /// 标记消息为完成
    pub async fn complete_message(&self, message_id: &str, result: MessageResult) {
        // 从处理中移除
        let mut processing = self.processing.write().await;
        processing.remove(message_id);
        
        // 添加到已完成
        let mut completed = self.completed.write().await;
        completed.insert(message_id.to_string(), result);
        
        // 清理过期的完成消息（只保留最近的1000条）
        if completed.len() > 1000 {
            let keys: Vec<String> = completed.keys().cloned().collect();
            for key in keys.into_iter().take(completed.len() - 1000) {
                completed.remove(&key);
            }
        }
    }
    
    /// 获取队列状态
    pub async fn get_status(&self) -> QueueStatus {
        let queue_size = self.queue.size().await;
        let processing_size = self.processing.read().await.len();
        let completed_size = self.completed.read().await.len();
        let priority_counts = self.queue.get_count_by_priority().await;
        
        QueueStatus {
            queue_size,
            processing_size,
            completed_size,
            priority_counts,
        }
    }
    
    /// 清空所有队列
    pub async fn clear_all(&self) {
        self.queue.clear().await;
        let mut processing = self.processing.write().await;
        processing.clear();
        let mut completed = self.completed.write().await;
        completed.clear();
    }
}

/// 队列状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    /// 队列大小
    pub queue_size: usize,
    /// 处理中的消息数量
    pub processing_size: usize,
    /// 已完成的消息数量
    pub completed_size: usize,
    /// 按优先级统计的消息数量
    pub priority_counts: std::collections::HashMap<MessagePriority, usize>,
}
