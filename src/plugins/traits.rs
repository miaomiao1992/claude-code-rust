//! 统一插件特质定义
//!
//! 整合 src/plugins/api.rs 和 src/plugins/plugin.rs 中的 Plugin trait，
//! 提供统一、安全的插件接口。

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::PluginSandbox;

/// 插件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// 未加载
    Unloaded,
    /// 加载中
    Loading,
    /// 已加载
    Loaded,
    /// 运行中
    Running,
    /// 错误
    Error,
    /// 正在卸载
    Unloading,
}

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件作者
    pub author: String,
    /// 插件描述
    pub description: String,
    /// 插件入口点函数名
    pub entry_point: String,
    /// 插件依赖
    pub dependencies: Vec<String>,
    /// 插件能力要求（可选，默认为空）
    pub capabilities: Vec<String>,
    /// 插件签名（base64编码的ed25519签名，可选）
    pub signature: Option<String>,
}

impl PluginMetadata {
    /// 创建新的插件元数据（向后兼容）
    pub fn new(name: String, version: String, author: String, description: String, entry_point: String) -> Self {
        Self {
            name,
            version,
            author,
            description,
            entry_point,
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            signature: None,
        }
    }
}

/// 插件API句柄
#[derive(Debug, Clone)]
pub struct PluginApi {
    /// API版本
    version: String,
    /// 插件名称
    plugin_name: String,
    /// 插件能力
    capabilities: Vec<String>,
    /// 沙箱实例（可选）
    sandbox: Option<Arc<PluginSandbox>>,
}

impl PluginApi {
    /// 创建新的插件API
    pub fn new(plugin_name: String, capabilities: Vec<String>) -> Self {
        Self {
            version: "1.0.0".to_string(),
            plugin_name,
            capabilities,
            sandbox: None,
        }
    }

    /// 创建带有沙箱的插件API
    pub fn with_sandbox(plugin_name: String, capabilities: Vec<String>, sandbox: Arc<PluginSandbox>) -> Self {
        Self {
            version: "1.0.0".to_string(),
            plugin_name,
            capabilities,
            sandbox: Some(sandbox),
        }
    }

    /// 获取API版本
    pub fn version(&self) -> &str {
        &self.version
    }

    /// 获取插件名称
    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    /// 检查是否具有特定能力
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }

    /// 检查能力并在无权限时返回错误
    pub fn check_capability(&self, capability: &str) -> Result<()> {
        if self.has_capability(capability) {
            Ok(())
        } else {
            Err(crate::error::ClaudeError::Permission(
                format!("Plugin '{}' lacks capability '{}'", self.plugin_name, capability)
            ))
        }
    }

    /// 注册命令（存根实现，后续完善）
    pub fn register_command<F>(&self, name: &str, handler: F) -> Result<()>
    where
        F: Fn(&str) -> Result<String> + Send + Sync + 'static,
    {
        // 需要 command:register 能力
        self.check_capability(capabilities::COMMAND_REGISTER)?;
        // TODO: 实现命令注册逻辑
        Ok(())
    }

    /// 注册事件监听器（存根实现，后续完善）
    pub fn register_event_listener<F>(&self, event: &str, handler: F) -> Result<()>
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        // 需要 event:listen 能力
        self.check_capability(capabilities::EVENT_LISTEN)?;
        // TODO: 实现事件监听器注册逻辑
        Ok(())
    }

    /// 发送事件（存根实现，后续完善）
    pub async fn emit_event(&self, event: &str, data: &str) -> Result<()> {
        // 需要 event:emit 能力
        self.check_capability(capabilities::EVENT_EMIT)?;
        // TODO: 实现事件发送逻辑
        Ok(())
    }

    /// 读取文件内容
    pub async fn read_file(&self, path: impl AsRef<std::path::Path>) -> Result<String> {
        // 需要 file:read 能力
        self.check_capability(capabilities::FILE_READ)?;

        // 检查沙箱路径访问权限
        if let Some(sandbox) = &self.sandbox {
            let path_buf = PathBuf::from(path.as_ref());
            if !sandbox.check_path_access(&self.plugin_name, &path_buf).await? {
                return Err(crate::error::ClaudeError::Permission(
                    format!("Sandbox denied file read access to: {:?}", path.as_ref())
                ));
            }
        }

        // TODO: 实际实现文件读取
        tokio::fs::read_to_string(path).await
            .map_err(|e| crate::error::ClaudeError::Io(e))
    }

    /// 写入文件内容
    pub async fn write_file(&self, path: impl AsRef<std::path::Path>, content: &str) -> Result<()> {
        // 需要 file:write 能力
        self.check_capability(capabilities::FILE_WRITE)?;

        // 检查沙箱路径访问权限
        if let Some(sandbox) = &self.sandbox {
            let path_buf = PathBuf::from(path.as_ref());
            if !sandbox.check_path_access(&self.plugin_name, &path_buf).await? {
                return Err(crate::error::ClaudeError::Permission(
                    format!("Sandbox denied file write access to: {:?}", path.as_ref())
                ));
            }
        }

        // TODO: 实际实现文件写入
        tokio::fs::write(path, content).await
            .map_err(|e| crate::error::ClaudeError::Io(e))
    }

    /// 执行命令
    pub async fn execute_command(&self, command: &str) -> Result<String> {
        // 需要 command:exec 能力
        self.check_capability(capabilities::COMMAND_EXEC)?;

        // 检查沙箱命令权限
        if let Some(sandbox) = &self.sandbox {
            if !sandbox.check_command(&self.plugin_name, command).await? {
                return Err(crate::error::ClaudeError::Permission(
                    format!("Sandbox denied command execution: {}", command)
                ));
            }
        }

        // TODO: 实际实现命令执行
        use std::process::Command;
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .map_err(|e| crate::error::ClaudeError::Other(e.to_string()))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(crate::error::ClaudeError::Other(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }

    /// 获取环境变量
    pub async fn get_env(&self, key: &str) -> Result<Option<String>> {
        // 需要 env:access 能力
        self.check_capability(capabilities::ENV_ACCESS)?;

        // 检查沙箱环境变量权限
        if let Some(sandbox) = &self.sandbox {
            if !sandbox.check_env(&self.plugin_name, key).await? {
                return Err(crate::error::ClaudeError::Permission(
                    format!("Sandbox denied env var access: {}", key)
                ));
            }
        }

        Ok(std::env::var(key).ok())
    }

    /// 发送HTTP请求
    pub async fn http_request(&self, url: &str, method: &str) -> Result<String> {
        // 需要 network 能力
        self.check_capability(capabilities::NETWORK)?;

        // 检查沙箱网络访问权限
        if let Some(sandbox) = &self.sandbox {
            if !sandbox.check_network(&self.plugin_name).await? {
                return Err(crate::error::ClaudeError::Permission(
                    "Sandbox denied network access".to_string()
                ));
            }
        }

        // TODO: 实际实现HTTP请求
        let client = reqwest::Client::new();
        let response = match method.to_uppercase().as_str() {
            "GET" => client.get(url).send().await,
            "POST" => client.post(url).send().await,
            "PUT" => client.put(url).send().await,
            "DELETE" => client.delete(url).send().await,
            _ => return Err(crate::error::ClaudeError::Other(
                format!("Unsupported HTTP method: {}", method)
            )),
        }.map_err(|e| crate::error::ClaudeError::Other(e.to_string()))?;

        let text = response.text().await
            .map_err(|e| crate::error::ClaudeError::Other(e.to_string()))?;

        Ok(text)
    }

    /// 获取系统信息
    pub async fn get_system_info(&self) -> Result<String> {
        // 需要 system:info 能力
        self.check_capability(capabilities::SYSTEM_INFO)?;

        // TODO: 实际实现系统信息获取
        let info = format!(
            "System info for plugin: {}\n\
            OS: {:?}\n\
            Arch: {:?}\n\
            CPUs: {}",
            self.plugin_name,
            std::env::consts::OS,
            std::env::consts::ARCH,
            num_cpus::get()
        );

        Ok(info)
    }

    /// 获取配置值
    pub async fn get_config(&self, key: &str) -> Result<Option<String>> {
        // 不需要特殊能力，所有插件都可以读取自己的配置
        // TODO: 实际实现配置获取
        Ok(None)
    }

    /// 设置配置值
    pub async fn set_config(&self, key: &str, value: &str) -> Result<()> {
        // 需要 config:write 能力
        self.check_capability(capabilities::CONFIG_WRITE)?;
        // TODO: 实际实现配置设置
        Ok(())
    }

    /// 日志记录
    pub async fn log(&self, level: &str, message: &str) -> Result<()> {
        // 不需要特殊能力，所有插件都可以记录日志
        println!("[{}] {}: {}", level, self.plugin_name, message);
        Ok(())
    }
}

/// 统一的插件特质
#[async_trait::async_trait]
pub trait Plugin: Debug + Send + Sync {
    /// 获取插件元数据
    fn metadata(&self) -> &PluginMetadata;

    /// 获取插件状态
    fn state(&self) -> PluginState;

    /// 初始化插件（接收PluginAPI参数）
    async fn initialize(&mut self, api: PluginApi) -> Result<()>;

    /// 启动插件
    async fn start(&mut self) -> Result<()>;

    /// 停止插件
    async fn stop(&mut self) -> Result<()>;

    /// 卸载插件
    async fn unload(&mut self) -> Result<()>;

    /// 处理消息（可选）
    async fn handle_message(&mut self, message: &str) -> Result<Option<String>> {
        Ok(None)
    }
}

/// 插件导出函数类型
/// 动态库必须导出此函数作为入口点
pub type PluginEntryPoint = fn() -> *mut dyn Plugin;

/// 插件能力常量定义
pub mod capabilities {
    /// 文件读取能力
    pub const FILE_READ: &str = "file:read";
    /// 文件写入能力
    pub const FILE_WRITE: &str = "file:write";
    /// 网络访问能力
    pub const NETWORK: &str = "network";
    /// 命令执行能力
    pub const COMMAND_EXEC: &str = "command:exec";
    /// 命令注册能力
    pub const COMMAND_REGISTER: &str = "command:register";
    /// 环境变量访问能力
    pub const ENV_ACCESS: &str = "env:access";
    /// 系统信息读取能力
    pub const SYSTEM_INFO: &str = "system:info";
    /// 配置写入能力
    pub const CONFIG_WRITE: &str = "config:write";
    /// 事件监听能力
    pub const EVENT_LISTEN: &str = "event:listen";
    /// 事件发送能力
    pub const EVENT_EMIT: &str = "event:emit";
    /// 进程创建能力
    pub const PROCESS_CREATE: &str = "process:create";
    /// 插件间通信能力
    pub const IPC: &str = "ipc";
    /// 持久化存储能力
    pub const STORAGE: &str = "storage";
}

/// 插件配置
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// 是否启用插件
    pub enabled: bool,
    /// 插件能力白名单
    pub allowed_capabilities: Vec<String>,
    /// 沙箱配置
    pub sandbox: bool,
    /// 资源限制
    pub resource_limits: ResourceLimits,
}

/// 资源限制配置
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// 最大内存使用（MB）
    pub max_memory_mb: u64,
    /// 最大CPU使用率（%）
    pub max_cpu_percent: u32,
    /// 最大文件大小（MB）
    pub max_file_size_mb: u64,
    /// 最大网络带宽（KB/s）
    pub max_network_kbps: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 100,
            max_cpu_percent: 25,
            max_file_size_mb: 10,
            max_network_kbps: 100,
        }
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_capabilities: vec![capabilities::FILE_READ.to_string()],
            sandbox: true,
            resource_limits: ResourceLimits::default(),
        }
    }
}