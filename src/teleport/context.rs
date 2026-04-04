//! Teleport Context 模块
//! 
//! 实现上下文状态管理功能

use super::packet::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 上下文状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextStatus {
    /// 活跃
    Active,
    /// 已冻结
    Frozen,
    /// 已过期
    Expired,
    /// 已销毁
    Destroyed,
}

/// 上下文元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// 创建时间
    pub created_at: String,
    /// 最后访问时间
    pub last_accessed: String,
    /// 状态
    pub status: ContextStatus,
    /// 访问次数
    pub access_count: u32,
    /// 大小（字节）
    pub size: u64,
    /// 标签
    pub tags: Vec<String>,
}

/// 上下文存储
pub struct ContextStorage {
    /// 上下文数据
    contexts: Arc<RwLock<HashMap<String, (ContextPacket, ContextMetadata)>>>,
    /// 最大存储大小（字节）
    max_size: u64,
    /// 当前存储大小（字节）
    current_size: Arc<RwLock<u64>>,
}

impl ContextStorage {
    /// 创建新的上下文存储
    pub fn new(max_size: u64) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            current_size: Arc::new(RwLock::new(0)),
        }
    }
    
    /// 添加上下文
    pub async fn add_context(&self, context: ContextPacket) -> Result<(), String> {
        let context_size = serde_json::to_string(&context)
            .map_err(|e| e.to_string())?
            .len() as u64;
        
        let mut current_size = self.current_size.write().await;
        if *current_size + context_size > self.max_size {
            // 清理过期上下文
            self.cleanup_expired().await;
            
            // 再次检查
            if *current_size + context_size > self.max_size {
                return Err("Context storage full".to_string());
            }
        }
        
        let metadata = ContextMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            last_accessed: chrono::Utc::now().to_rfc3339(),
            status: ContextStatus::Active,
            access_count: 0,
            size: context_size,
            tags: vec![],
        };
        
        let mut contexts = self.contexts.write().await;
        contexts.insert(context.context_id.clone(), (context, metadata));
        *current_size += context_size;
        
        Ok(())
    }
    
    /// 获取上下文
    pub async fn get_context(&self, context_id: &str) -> Option<ContextPacket> {
        let mut contexts = self.contexts.write().await;
        if let Some((context, metadata)) = contexts.get_mut(context_id) {
            // 更新元数据
            metadata.last_accessed = chrono::Utc::now().to_rfc3339();
            metadata.access_count += 1;
            
            Some(context.clone())
        } else {
            None
        }
    }
    
    /// 删除上下文
    pub async fn remove_context(&self, context_id: &str) -> bool {
        let mut contexts = self.contexts.write().await;
        if let Some((_, metadata)) = contexts.remove(context_id) {
            let mut current_size = self.current_size.write().await;
            *current_size -= metadata.size;
            true
        } else {
            false
        }
    }
    
    /// 冻结上下文
    pub async fn freeze_context(&self, context_id: &str) -> bool {
        let mut contexts = self.contexts.write().await;
        if let Some((_, metadata)) = contexts.get_mut(context_id) {
            metadata.status = ContextStatus::Frozen;
            true
        } else {
            false
        }
    }
    
    /// 解冻上下文
    pub async fn thaw_context(&self, context_id: &str) -> bool {
        let mut contexts = self.contexts.write().await;
        if let Some((_, metadata)) = contexts.get_mut(context_id) {
            metadata.status = ContextStatus::Active;
            true
        } else {
            false
        }
    }
    
    /// 清理过期上下文
    pub async fn cleanup_expired(&self) {
        let mut contexts = self.contexts.write().await;
        let mut current_size = self.current_size.write().await;
        
        let now = chrono::Utc::now();
        let expired_ids: Vec<String> = contexts
            .iter()
            .filter(|(_, (_, metadata))| {
                if let ContextStatus::Expired = metadata.status {
                    true
                } else {
                    // 检查是否超过30天未访问
                    if let Ok(last_accessed) = chrono::DateTime::parse_from_rfc3339(&metadata.last_accessed) {
                        now.signed_duration_since(last_accessed).num_days() > 30
                    } else {
                        false
                    }
                }
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in expired_ids {
            if let Some((_, metadata)) = contexts.remove(&id) {
                *current_size -= metadata.size;
            }
        }
    }
    
    /// 获取存储状态
    pub async fn get_status(&self) -> StorageStatus {
        let contexts = self.contexts.read().await;
        let current_size = *self.current_size.read().await;
        
        let active_count = contexts
            .values()
            .filter(|(_, metadata)| metadata.status == ContextStatus::Active)
            .count();
        
        let frozen_count = contexts
            .values()
            .filter(|(_, metadata)| metadata.status == ContextStatus::Frozen)
            .count();
        
        let expired_count = contexts
            .values()
            .filter(|(_, metadata)| metadata.status == ContextStatus::Expired)
            .count();
        
        StorageStatus {
            total_contexts: contexts.len(),
            active_contexts: active_count,
            frozen_contexts: frozen_count,
            expired_contexts: expired_count,
            current_size,
            max_size: self.max_size,
            utilization: (current_size as f64 / self.max_size as f64) * 100.0,
        }
    }
}

/// 存储状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatus {
    /// 总上下文数
    pub total_contexts: usize,
    /// 活跃上下文数
    pub active_contexts: usize,
    /// 冻结上下文数
    pub frozen_contexts: usize,
    /// 过期上下文数
    pub expired_contexts: usize,
    /// 当前存储大小（字节）
    pub current_size: u64,
    /// 最大存储大小（字节）
    pub max_size: u64,
    /// 利用率（%）
    pub utilization: f64,
}

/// 上下文同步器
pub struct ContextSynchronizer {
    /// 本地存储
    local_storage: Arc<ContextStorage>,
    /// 远程地址
    remote_address: String,
}

impl ContextSynchronizer {
    /// 创建新的上下文同步器
    pub fn new(local_storage: Arc<ContextStorage>, remote_address: String) -> Self {
        Self {
            local_storage,
            remote_address,
        }
    }
    
    /// 同步上下文到远程
    pub async fn sync_to_remote(&self, context_id: &str) -> Result<bool, String> {
        if let Some(context) = self.local_storage.get_context(context_id).await {
            // 创建远程执行器
            let mut executor = match RemoteExecutor::new(self.remote_address.clone(), 30).await {
                Ok(executor) => executor,
                Err(e) => return Err(e.to_string()),
            };
            
            // 传递上下文
            let result = executor.transfer_context(context).await;
            executor.close().await.ok();
            
            result.map_err(|e| e.to_string())
        } else {
            Err("Context not found".to_string())
        }
    }
    
    /// 从远程同步上下文
    pub async fn sync_from_remote(&self, context_id: &str) -> Result<bool, String> {
        // TODO: 实现从远程同步上下文
        Ok(false)
    }
}

use super::remote::RemoteExecutor;
use chrono;
