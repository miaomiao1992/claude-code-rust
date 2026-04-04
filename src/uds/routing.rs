//! UDS Routing 模块
//! 
//! 实现消息的路由和地址解析功能

use super::message::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 路由表条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    /// 目标ID
    pub target_id: String,
    /// 目标类型
    pub target_type: String,
    /// 目标地址
    pub address: String,
    /// 优先级
    pub priority: u8,
    /// 过期时间（可选）
    pub expiration: Option<String>,
}

/// 路由管理器
pub struct RoutingManager {
    /// 路由表
    routes: Arc<RwLock<HashMap<String, RouteEntry>>>,
    /// 本地服务列表
    local_services: Arc<RwLock<HashMap<String, String>>>,
}

impl RoutingManager {
    /// 创建新的路由管理器
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
            local_services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 添加路由
    pub async fn add_route(&self, entry: RouteEntry) {
        let mut routes = self.routes.write().await;
        routes.insert(entry.target_id.clone(), entry);
    }
    
    /// 删除路由
    pub async fn remove_route(&self, target_id: &str) {
        let mut routes = self.routes.write().await;
        routes.remove(target_id);
    }
    
    /// 查找路由
    pub async fn find_route(&self, target_id: &str) -> Option<RouteEntry> {
        let routes = self.routes.read().await;
        routes.get(target_id).cloned()
    }
    
    /// 解析地址
    pub async fn resolve_address(&self, target: &MessageTarget) -> Option<String> {
        // 首先查找路由表
        if let Some(route) = self.find_route(&target.id).await {
            return Some(route.address);
        }
        
        // 查找本地服务
        let local_services = self.local_services.read().await;
        local_services.get(&target.id).cloned()
    }
    
    /// 添加本地服务
    pub async fn add_local_service(&self, service_id: String, address: String) {
        let mut local_services = self.local_services.write().await;
        local_services.insert(service_id, address);
    }
    
    /// 移除本地服务
    pub async fn remove_local_service(&self, service_id: &str) {
        let mut local_services = self.local_services.write().await;
        local_services.remove(service_id);
    }
    
    /// 获取所有路由
    pub async fn get_all_routes(&self) -> Vec<RouteEntry> {
        let routes = self.routes.read().await;
        routes.values().cloned().collect()
    }
    
    /// 清理过期路由
    pub async fn cleanup_expired_routes(&self) {
        let mut routes = self.routes.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        routes.retain(|_, entry| {
            if let Some(exp) = &entry.expiration {
                if let Ok(exp_time) = exp.parse::<u128>() {
                    return now <= exp_time;
                }
            }
            true
        });
    }
}

/// 地址解析器
pub struct AddressResolver {
    /// 路由管理器
    routing_manager: RoutingManager,
}

impl AddressResolver {
    /// 创建新的地址解析器
    pub fn new(routing_manager: RoutingManager) -> Self {
        Self {
            routing_manager,
        }
    }
    
    /// 解析消息目标地址
    pub async fn resolve_message_address(&self, message: &UdsMessage) -> Option<String> {
        self.routing_manager.resolve_address(&message.target).await
    }
    
    /// 验证地址
    pub async fn validate_address(&self, address: &str) -> bool {
        // 简单验证：检查地址格式
        address.starts_with("uds://") || Path::new(address).exists()
    }
}

use std::path::Path;
