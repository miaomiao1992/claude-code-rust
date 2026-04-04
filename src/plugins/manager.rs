//! 插件管理器
//!
//! 实现插件的加载、卸载和管理功能

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use libloading;
use serde_json;
use crate::error::Result;
use super::traits::{Plugin, PluginMetadata, PluginState, PluginApi};
use super::plugin::DynamicPlugin;
use super::message_bus::MessageBus;
use super::lifecycle::PluginLifecycle;
use super::dependency::DependencyManager;
use super::security::PluginSignatureVerifier;

/// 插件管理器
#[derive(Debug, Clone)]
pub struct PluginManager {
    /// 插件映射
    plugins: Arc<RwLock<HashMap<String, Arc<RwLock<Box<dyn Plugin>>>>>>,
    /// 消息总线
    message_bus: Arc<MessageBus>,
    /// 依赖管理器
    dependency_manager: Arc<RwLock<DependencyManager>>,
    /// 插件目录
    plugin_dirs: Vec<PathBuf>,
    /// 签名验证器
    signature_verifier: Arc<PluginSignatureVerifier>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            message_bus: Arc::new(MessageBus::new()),
            dependency_manager: Arc::new(RwLock::new(DependencyManager::new())),
            plugin_dirs: Vec::new(),
            signature_verifier: Arc::new(super::security::create_default_verifier()),
        }
    }
    
    /// 添加插件目录
    pub fn add_plugin_dir(&mut self, path: PathBuf) {
        self.plugin_dirs.push(path);
    }
    
    /// 加载插件
    pub async fn load_plugin(&self, path: PathBuf) -> Result<()>
    {
        // 解析插件元数据
        let metadata = self.parse_plugin_metadata(&path)?;

        // 验证插件签名
        if !self.signature_verifier.verify_plugin(&path, &metadata).await? {
            return Err(crate::error::ClaudeError::Permission(
                format!("Plugin signature verification failed for: {}", path.display())
            ));
        }

        // 检查插件是否已加载
        let mut plugins = self.plugins.write().await;
        if plugins.contains_key(&metadata.name) {
            return Err("Plugin already loaded".into());
        }

        // 创建插件实例
        let mut plugin = Box::new(DynamicPlugin::new(path, metadata.clone()));

        // 创建插件API（使用元数据中的能力）
        let api = PluginApi::new(metadata.name.clone(), metadata.capabilities.clone());

        // 初始化插件
        plugin.initialize(api).await?;
        plugin.start().await?;

        // 添加到插件映射
        plugins.insert(metadata.name, Arc::new(RwLock::new(plugin)));

        Ok(())
    }
    
    /// 卸载插件
    pub async fn unload_plugin(&self, name: &str) -> Result<()>
    {
        let mut plugins = self.plugins.write().await;
        if let Some(plugin) = plugins.remove(name) {
            let mut plugin_mut = plugin.write().await;
            plugin_mut.stop().await?;
            plugin_mut.unload().await?;
        }
        Ok(())
    }
    
    /// 启动插件
    pub async fn start_plugin(&self, name: &str) -> Result<()>
    {
        let plugins = self.plugins.read().await;
        if let Some(plugin) = plugins.get(name) {
            let mut plugin_mut = plugin.write().await;
            plugin_mut.start().await?;
        }
        Ok(())
    }
    
    /// 停止插件
    pub async fn stop_plugin(&self, name: &str) -> Result<()>
    {
        let plugins = self.plugins.read().await;
        if let Some(plugin) = plugins.get(name) {
            let mut plugin_mut = plugin.write().await;
            plugin_mut.stop().await?;
        }
        Ok(())
    }
    
    /// 获取插件
    pub async fn get_plugin(&self, name: &str) -> Option<Arc<RwLock<Box<dyn Plugin>>>> {
        let plugins = self.plugins.read().await;
        plugins.get(name).cloned()
    }
    
    /// 获取所有插件
    pub async fn get_all_plugins(&self) -> HashMap<String, Arc<RwLock<Box<dyn Plugin>>>> {
        self.plugins.read().await.clone()
    }
    
    /// 扫描插件目录
    pub async fn scan_plugins(&self) -> Result<Vec<PathBuf>>
    {
        let mut plugins = Vec::new();
        for dir in &self.plugin_dirs {
            if dir.exists() && dir.is_dir() {
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() && path.extension().map(|ext| ext == "so").unwrap_or(false) {
                        plugins.push(path);
                    }
                }
            }
        }
        Ok(plugins)
    }
    
    /// 解析插件元数据
    fn parse_plugin_metadata(&self, path: &PathBuf) -> Result<PluginMetadata>
    {
        // 尝试查找同名的.json文件
        let json_path = path.with_extension("json");
        if json_path.exists() {
            match std::fs::read_to_string(&json_path) {
                Ok(content) => {
                    // 解析JSON元数据
                    let mut metadata: PluginMetadata = serde_json::from_str(&content)?;
                    // 确保entry_point字段有值（默认为"plugin_entry"）
                    if metadata.entry_point.is_empty() {
                        metadata.entry_point = "plugin_entry".to_string();
                    }
                    return Ok(metadata);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read plugin metadata from {}: {}", json_path.display(), e);
                }
            }
        }

        // 如果没有.json文件，尝试从共享库中读取元数据符号
        // 注意：这里我们只是加载库来读取元数据，然后立即卸载
        unsafe {
            let lib = libloading::Library::new(path)?;
            if let Ok(metadata_symbol) = lib.get::<libloading::Symbol<*const u8>>(b"plugin_metadata") {
                let metadata_ptr = *metadata_symbol;
                if !metadata_ptr.is_null() {
                    let c_str = std::ffi::CStr::from_ptr(metadata_ptr as *const std::os::raw::c_char);
                    let json_str = c_str.to_str()?;
                    let mut metadata: PluginMetadata = serde_json::from_str(json_str)?;
                    if metadata.entry_point.is_empty() {
                        metadata.entry_point = "plugin_entry".to_string();
                    }
                    return Ok(metadata);
                }
            }
        }

        // 回退到默认元数据（基于文件名）
        let file_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown-plugin");

        Ok(PluginMetadata::new(
            file_name.to_string(),
            "1.0.0".to_string(),
            "Unknown Author".to_string(),
            format!("Plugin loaded from {}", path.display()),
            "plugin_entry".to_string(),
        ))
    }
    
    /// 获取消息总线
    pub fn message_bus(&self) -> Arc<MessageBus> {
        self.message_bus.clone()
    }
    
    /// 获取依赖管理器
    pub fn dependency_manager(&self) -> Arc<RwLock<DependencyManager>> {
        self.dependency_manager.clone()
    }
}
