//! Configuration Module
//!
//! 重构后的配置模块，提供：
//! - 配置验证机制
//! - 配置迁移系统
//! - System Prompt 组装
//! - 模块化配置管理

pub mod api_config;
pub mod mcp_config;
pub mod validation;
pub mod migration;
pub mod system_prompt;
pub mod plugin_marketplace_config;

pub use api_config::ApiConfig;
pub use mcp_config::{McpConfig, McpServerStatus};
pub use validation::{ValidationResult, ValidationError};
pub use migration::{MigrationManager, ConfigVersion, MigrationResult, create_standard_migration_manager};
pub use system_prompt::{SystemPromptBuilder, IdentityPrefix, SYSTEM_PROMPT_DYNAMIC_BOUNDARY};
pub use plugin_marketplace_config::PluginMarketplaceConfig;

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main configuration structure (alias for Settings)
pub type Config = Settings;

/// Permission mode for tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionMode {
    /// Use default permission
    Default,
    /// Always allow
    AlwaysAllow,
    /// Always deny
    AlwaysDeny,
    /// Always ask
    AlwaysAsk,
}

impl Default for PermissionMode {
    fn default() -> Self {
        PermissionMode::Default
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// 配置版本
    #[serde(default = "default_version")]
    pub version: String,
    
    /// API configuration
    pub api: ApiConfig,
    /// MCP server configurations
    pub mcp_servers: Vec<McpConfig>,
    /// Model selection
    pub model: String,
    /// Enable verbose logging
    pub verbose: bool,
    /// Working directory
    pub working_dir: PathBuf,
    /// Memory settings
    pub memory: MemorySettings,
    /// Voice settings
    pub voice: VoiceSettings,
    /// Plugin settings
    pub plugins: PluginSettings,
    /// Plugin marketplace configuration
    #[serde(default)]
    pub plugin_marketplace: PluginMarketplaceConfig,
    /// 特性标志
    #[serde(default)]
    pub feature_flags: FeatureFlags,
    /// 输出设置
    #[serde(default)]
    pub output: OutputSettings,
    /// Daemon settings
    #[serde(default)]
    pub daemon: DaemonSettings,
}

fn default_version() -> String {
    ConfigVersion::current().to_string()
}

/// Memory settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySettings {
    /// Enable memory persistence
    pub enabled: bool,
    /// Memory file path
    pub path: PathBuf,
    /// Auto-consolidation interval (hours)
    pub consolidation_interval: u64,
    /// Maximum memories to keep
    pub max_memories: usize,
}

/// Voice settings
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VoiceSettings {
    /// Enable voice input
    pub enabled: bool,
    /// Push-to-talk mode
    pub push_to_talk: bool,
    /// Silence detection threshold
    pub silence_threshold: f32,
    /// Sample rate
    pub sample_rate: u32,
}

/// Plugin settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettings {
    /// Enable plugin system
    pub enabled: bool,
    /// Plugin directory
    pub plugin_dir: PathBuf,
    /// Auto-update plugins
    pub auto_update: bool,
}

/// 特性标志
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// 主动模式
    #[serde(default)]
    pub proactive: bool,
    /// IDE 桥接模式
    #[serde(default)]
    pub bridge_mode: bool,
    /// 语音模式
    #[serde(default)]
    pub voice_mode: bool,
    /// 协调器模式
    #[serde(default)]
    pub coordinator_mode: bool,
    /// Fork 子智能体
    #[serde(default)]
    pub fork_subagent: bool,
    /// Buddy 伴侣精灵
    #[serde(default)]
    pub buddy: bool,
}

/// 输出设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSettings {
    /// 语言偏好
    #[serde(default = "default_language")]
    pub language: String,
    /// 输出样式
    #[serde(default = "default_output_style")]
    pub style: String,
    /// 简短模式
    #[serde(default)]
    pub brief_mode: bool,
    /// 启用 emoji
    #[serde(default)]
    pub emoji: bool,
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            language: default_language(),
            style: default_output_style(),
            brief_mode: false,
            emoji: false,
        }
    }
}

/// Daemon settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSettings {
    /// Enable daemon mode
    pub enabled: bool,
    /// Socket path
    pub socket_path: Option<String>,
    /// PID file path
    pub pid_file: Option<String>,
    /// Auto start on system boot
    pub auto_start: bool,
    /// Log file path
    pub log_file: Option<String>,
}

impl Default for DaemonSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            socket_path: None,
            pid_file: None,
            auto_start: false,
            log_file: None,
        }
    }
}

fn default_language() -> String {
    "en".to_string()
}

fn default_output_style() -> String {
    "default".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home.join(".claude-code");

        Self {
            version: ConfigVersion::current().to_string(),
            api: ApiConfig::default(),
            mcp_servers: Vec::new(),
            model: "sonnet".to_string(),
            verbose: false,
            working_dir: PathBuf::from("."),
            memory: MemorySettings {
                enabled: true,
                path: config_dir.join("memory.json"),
                consolidation_interval: 24,
                max_memories: 1000,
            },
            voice: VoiceSettings {
                enabled: false,
                push_to_talk: false,
                silence_threshold: 0.01,
                sample_rate: 16000,
            },
            plugins: PluginSettings {
                enabled: true,
                plugin_dir: config_dir.join("plugins"),
                auto_update: true,
            },
            plugin_marketplace: PluginMarketplaceConfig::default(),
            feature_flags: FeatureFlags::default(),
            output: OutputSettings::default(),
            daemon: DaemonSettings::default(),
        }
    }
}

impl Settings {
    /// 加载配置（带迁移）
    pub fn load() -> Result<Self> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_path = home.join(".claude-code").join("settings.json");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let mut value: serde_json::Value = serde_json::from_str(&content)?;
            
            // 执行迁移
            let migration_manager = create_standard_migration_manager();
            let migration_result = migration_manager.migrate(value, None)?;
            
            if !migration_result.success {
                if let Some(error) = migration_result.error {
                    return Err(crate::error::ClaudeError::Config(error));
                }
            }
            
            // 重新加载迁移后的配置
            let content = std::fs::read_to_string(&config_path)?;
            let settings: Settings = serde_json::from_str(&content)?;
            
            Ok(settings)
        } else {
            let settings = Settings::default();
            settings.save()?;
            Ok(settings)
        }
    }

    /// 保存配置
    pub fn save(&self) -> Result<()> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home.join(".claude-code");
        std::fs::create_dir_all(&config_dir)?;
        
        let config_path = config_dir.join("settings.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        // 验证插件市场配置
        if let Err(e) = self.plugin_marketplace.validate() {
            return Err(crate::error::ClaudeError::Config(e));
        }

        // 简单验证，暂时总是返回Ok
        Ok(())
    }

    /// 设置配置值
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "model" => self.model = value.to_string(),
            "verbose" => self.verbose = value.parse().unwrap_or(false),
            "api_key" => self.api.api_key = Some(value.to_string()),
            "base_url" => self.api.base_url = value.to_string(),
            "max_tokens" => self.api.max_tokens = value.parse().unwrap_or(4096),
            "timeout" => self.api.timeout = value.parse().unwrap_or(120),
            "streaming" => self.api.streaming = value.parse().unwrap_or(true),
            "memory.enabled" => self.memory.enabled = value.parse().unwrap_or(true),
            "voice.enabled" => self.voice.enabled = value.parse().unwrap_or(false),
            "output.language" => self.output.language = value.to_string(),
            "output.style" => self.output.style = value.to_string(),
            "output.brief_mode" => self.output.brief_mode = value.parse().unwrap_or(false),
            "output.emoji" => self.output.emoji = value.parse().unwrap_or(false),
            "features.proactive" => self.feature_flags.proactive = value.parse().unwrap_or(false),
            "features.bridge_mode" => self.feature_flags.bridge_mode = value.parse().unwrap_or(false),
            "features.voice_mode" => self.feature_flags.voice_mode = value.parse().unwrap_or(false),
            "features.coordinator_mode" => self.feature_flags.coordinator_mode = value.parse().unwrap_or(false),
            "features.fork_subagent" => self.feature_flags.fork_subagent = value.parse().unwrap_or(false),
            "features.buddy" => self.feature_flags.buddy = value.parse().unwrap_or(false),
            // Daemon settings
            "daemon.enabled" => self.daemon.enabled = value.parse().unwrap_or(false),
            "daemon.socket_path" => self.daemon.socket_path = Some(value.to_string()),
            "daemon.pid_file" => self.daemon.pid_file = Some(value.to_string()),
            "daemon.auto_start" => self.daemon.auto_start = value.parse().unwrap_or(false),
            "daemon.log_file" => self.daemon.log_file = Some(value.to_string()),
            // Plugin marketplace settings
            "plugin_marketplace.base_url" => self.plugin_marketplace.base_url = value.to_string(),
            "plugin_marketplace.api_key" => self.plugin_marketplace.api_key = Some(value.to_string()),
            "plugin_marketplace.cache_ttl_seconds" => self.plugin_marketplace.cache_ttl_seconds = value.parse().unwrap_or(300),
            "plugin_marketplace.max_retries" => self.plugin_marketplace.max_retries = value.parse().unwrap_or(3),
            "plugin_marketplace.request_timeout_seconds" => self.plugin_marketplace.request_timeout_seconds = value.parse().unwrap_or(30),
            "plugin_marketplace.verify_signatures" => self.plugin_marketplace.verify_signatures = value.parse().unwrap_or(true),
            "plugin_marketplace.debug_logging" => self.plugin_marketplace.debug_logging = value.parse().unwrap_or(false),
            "plugin_marketplace.offline_mode" => self.plugin_marketplace.offline_mode = value.parse().unwrap_or(false),
            _ => return Err(crate::error::ClaudeError::Config(format!("Unknown setting: {}", key))),
        }
        
        Ok(())
    }

    /// 重置为默认配置
    pub fn reset() -> Result<()> {
        let settings = Settings::default();
        settings.save()?;
        Ok(())
    }
    
    /// 创建 System Prompt 构建器
    pub fn create_system_prompt_builder(&self) -> SystemPromptBuilder {
        let mut builder = SystemPromptBuilder::new(self.clone());
        
        // 设置语言
        builder.set_language(&self.output.language);
        
        // 设置输出样式
        builder.set_output_style(&self.output.style);
        
        // 设置简短模式
        builder.set_brief_mode(self.output.brief_mode);
        
        builder
    }
}

/// 配置管理器
pub struct ConfigManager {
    settings: Arc<RwLock<Settings>>,
    migration_manager: MigrationManager,
    change_listeners: Vec<Box<dyn Fn(&Settings) + Send + Sync>>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self> {
        let settings = Settings::load()?;
        let migration_manager = create_standard_migration_manager();
        
        Ok(Self {
            settings: Arc::new(RwLock::new(settings)),
            migration_manager,
            change_listeners: Vec::new(),
        })
    }
    
    /// 获取配置
    pub async fn settings(&self) -> Settings {
        self.settings.read().await.clone()
    }
    
    /// 更新配置
    pub async fn update<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut Settings) -> Result<()>,
    {
        let mut settings = self.settings.write().await;
        updater(&mut settings)?;
        settings.validate()?;
        settings.save()?;
        
        // 通知监听器
        let settings_clone = settings.clone();
        drop(settings);
        
        for listener in &self.change_listeners {
            listener(&settings_clone);
        }
        
        Ok(())
    }
    
    /// 重新加载配置
    pub async fn reload(&self) -> Result<()> {
        let settings = Settings::load()?;
        let mut write_guard = self.settings.write().await;
        *write_guard = settings.clone();
        drop(write_guard);
        
        // 通知监听器
        for listener in &self.change_listeners {
            listener(&settings);
        }
        
        Ok(())
    }
    
    /// 添加变更监听器
    pub fn add_change_listener<F>(&mut self, listener: F)
    where
        F: Fn(&Settings) + Send + Sync + 'static,
    {
        self.change_listeners.push(Box::new(listener));
    }
    
    /// 获取迁移管理器
    pub fn migration_manager(&self) -> &MigrationManager {
        &self.migration_manager
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create ConfigManager")
    }
}

/// Enable configuration system
pub fn enable_configs() -> Result<()> {
    tracing::debug!("Enabling configuration system");
    
    let _settings = Settings::load()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert_eq!(settings.model, "sonnet");
        assert!(!settings.verbose);
    }

    #[test]
    fn test_feature_flags() {
        let mut settings = Settings::default();
        settings.feature_flags.buddy = true;
        settings.feature_flags.proactive = true;
        
        assert!(settings.feature_flags.buddy);
        assert!(settings.feature_flags.proactive);
    }

    #[test]
    fn test_output_settings() {
        let mut settings = Settings::default();
        settings.output.language = "zh".to_string();
        settings.output.brief_mode = true;
        settings.output.emoji = true;
        
        assert_eq!(settings.output.language, "zh");
        assert!(settings.output.brief_mode);
        assert!(settings.output.emoji);
    }

    #[test]
    fn test_config_version() {
        let settings = Settings::default();
        assert!(!settings.version.is_empty());
    }

    #[test]
    fn test_plugin_marketplace_config() {
        let settings = Settings::default();
        assert_eq!(settings.plugin_marketplace.base_url, "https://plugins.claude.ai/api/v1");
        assert_eq!(settings.plugin_marketplace.cache_ttl_seconds, 300);
        assert!(settings.plugin_marketplace.verify_signatures);
        assert!(settings.plugin_marketplace.validate().is_ok());
    }

    #[test]
    fn test_plugin_marketplace_setting() {
        let mut settings = Settings::default();
        settings.set("plugin_marketplace.base_url", "https://custom.example.com").unwrap();
        settings.set("plugin_marketplace.debug_logging", "true").unwrap();

        assert_eq!(settings.plugin_marketplace.base_url, "https://custom.example.com");
        assert!(settings.plugin_marketplace.debug_logging);
    }

    #[test]
    fn test_plugin_marketplace_validation() {
        let mut settings = Settings::default();
        settings.plugin_marketplace.base_url = String::new();

        assert!(settings.validate().is_err());
    }
}
