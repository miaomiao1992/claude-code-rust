//! 提示词缓存
//!
//! 提供提示词缓存功能，减少重复 API 调用，
//! 提高响应速度并降低成本。

use api_client::types::{ApiRequest, ApiResponse};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// 提示词缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCacheConfig {
    /// 最大缓存条目
    pub max_size: usize,
    /// TTL（存活时间）秒
    pub ttl_seconds: u64,
    /// 是否启用缓存
    pub enabled: bool,
}

impl Default for PromptCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 100,
            ttl_seconds: 3600, // 1 hour
            enabled: true,
        }
    }
}

/// 缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntry {
    /// 缓存键哈希
    pub key_hash: u64,
    /// 响应
    pub response: ApiResponse,
    /// 创建时间
    pub created_at: SystemTime,
    /// 过期时间
    pub expires_at: SystemTime,
    /// 命中次数
    pub hit_count: u32,
}

/// 提示词缓存
pub struct PromptCache {
    /// LRU 缓存
    cache: Mutex<LruCache<u64, CachedEntry>>,
    /// 配置
    config: PromptCacheConfig,
}

impl PromptCache {
    /// 创建新的提示词缓存
    pub fn new(config: PromptCacheConfig) -> Self {
        let cache = Mutex::new(LruCache::new(config.max_size));
        Self { cache, config }
    }

    /// 计算缓存键
    pub fn compute_key(request: &ApiRequest) -> u64 {
        let mut hasher = DefaultHasher::new();
        serde_json::to_string(request).unwrap_or_default().hash(&mut hasher);
        hasher.finish()
    }

    /// 获取缓存条目
    pub fn get(&self, key: u64) -> Option<ApiResponse> {
        if !self.config.enabled {
            return None;
        }

        let mut cache = self.cache.lock().unwrap();
        let entry = cache.get(&key)?;

        // 检查是否过期
        if SystemTime::now() > entry.expires_at {
            cache.pop(&key);
            return None;
        }

        // 更新命中计数
        // 需要可变访问，LruCache 已经处理了
        Some(entry.response.clone())
    }

    /// 放入缓存
    pub fn put(&self, key: u64, response: ApiResponse) {
        if !self.config.enabled {
            return;
        }

        let now = SystemTime::now();
        let ttl = Duration::from_secs(self.config.ttl_seconds);
        let expires_at = now + ttl;

        let entry = CachedEntry {
            key_hash: key,
            response,
            created_at: now,
            expires_at,
            hit_count: 0,
        };

        let mut cache = self.cache.lock().unwrap();
        cache.put(key, entry);
    }

    /// 清除过期条目
    pub fn cleanup(&self) -> usize {
        let mut cache = self.cache.lock().unwrap();
        let now = SystemTime::now();
        let mut expired = 0;

        // 因为 LruCache 不支持迭代器移除，我们需要重建
        // 这不高效，但对于缓存大小来说可以接受
        let mut entries: Vec<(u64, CachedEntry)> = cache
            .iter()
            .filter(|(_, entry)| now < entry.expires_at)
            .map(|(k, v)| (*k, v.clone()))
            .collect();

        expired = cache.len() - entries.len();

        cache.clear();
        for (k, v) in entries {
            cache.put(k, v);
        }

        expired
    }

    /// 获取缓存统计
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.lock().unwrap();
        CacheStats {
            size: cache.len(),
            max_size: cache.cap(),
        }
    }

    /// 清空缓存
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

/// 缓存统计
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 当前大小
    pub size: usize,
    /// 最大大小
    pub max_size: usize,
}

/// 带缓存的 API 请求包装器
pub struct CachedApiRequest {
    /// API 客户端
    api_client: Arc<api_client::ApiClient>,
    /// 缓存
    cache: Arc<PromptCache>,
}

impl CachedApiRequest {
    /// 创建新的带缓存请求包装器
    pub fn new(
        api_client: Arc<api_client::ApiClient>,
        cache: Arc<PromptCache>,
    ) -> Self {
        Self { api_client, cache }
    }

    /// 发送请求（优先缓存）
    pub async fn send_request(
        &self,
        request: ApiRequest,
    ) -> Result<ApiResponse, api_client::error::ApiError> {
        let key = PromptCache::compute_key(&request);

        // 尝试从缓存获取
        if let Some(cached) = self.cache.get(key) {
            tracing::debug!("Prompt cache hit for key {}", key);
            return Ok(cached);
        }

        // 缓存未命中，发送实际请求
        tracing::debug!("Prompt cache miss for key {}", key);
        let response = self.api_client.send_request(request).await?;

        // 存入缓存
        self.cache.put(key, response.clone());

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use api_client::types::{ApiRequest, ApiMessage, ApiRole};

    #[test]
    fn test_cache_basic() {
        let config = PromptCacheConfig {
            max_size: 5,
            ttl_seconds: 3600,
            enabled: true,
        };
        let cache = PromptCache::new(config);

        let request = ApiRequest {
            model: api_client::types::ApiModel::Claude35Sonnet20241022,
            messages: vec![ApiMessage {
                role: ApiRole::User,
                content: api_client::types::MessageContent::Text("test".to_string()),
            }],
            max_tokens: Some(100),
            ..Default::default()
        };

        let key = PromptCache::compute_key(&request);
        println!("Computed key: {}", key);
        assert!(cache.get(key).is_none());
    }

    #[test]
    fn test_cache_put_get() {
        let config = PromptCacheConfig {
            max_size: 5,
            ttl_seconds: 3600,
            enabled: true,
        };
        let cache = PromptCache::new(config);

        let request = ApiRequest {
            model: api_client::types::ApiModel::Claude35Sonnet20241022,
            messages: vec![ApiMessage {
                role: ApiRole::User,
                content: api_client::types::MessageContent::Text("test".to_string()),
            }],
            max_tokens: Some(100),
            ..Default::default()
        };

        let key = PromptCache::compute_key(&request);
        // 创建一个空响应用于测试
        let response = ApiResponse {
            id: "test".to_string(),
            content: vec![],
            model: "test".to_string(),
            usage: None,
        };

        cache.put(key, response);
        assert!(cache.get(key).is_some());
        assert_eq!(cache.stats().size, 1);
    }

    #[test]
    fn test_cache_disabled() {
        let config = PromptCacheConfig {
            max_size: 5,
            ttl_seconds: 3600,
            enabled: false,
        };
        let cache = PromptCache::new(config);

        let request = ApiRequest {
            model: api_client::types::ApiModel::Claude35Sonnet20241022,
            messages: vec![ApiMessage {
                role: ApiRole::User,
                content: api_client::types::MessageContent::Text("test".to_string()),
            }],
            max_tokens: Some(100),
            ..Default::default()
        };

        let key = PromptCache::compute_key(&request);
        let response = ApiResponse {
            id: "test".to_string(),
            content: vec![],
            model: "test".to_string(),
            usage: None,
        };

        cache.put(key, response);
        assert!(cache.get(key).is_none());
    }
}
