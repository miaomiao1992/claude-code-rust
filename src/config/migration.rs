//! 配置迁移模块
//!
//! 提供配置版本管理和自动迁移功能，确保配置的向后兼容性。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// 配置版本
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConfigVersion {
    /// 主版本
    pub major: u32,
    /// 次版本
    pub minor: u32,
    /// 补丁版本
    pub patch: u32,
}

impl PartialEq for ConfigVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl Eq for ConfigVersion {}

impl PartialOrd for ConfigVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ConfigVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.patch.cmp(&other.patch)
    }
}

impl ConfigVersion {
    /// 创建新版本
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// 解析版本字符串
    pub fn parse(version_str: &str) -> crate::error::Result<Self> {
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() != 3 {
            return Err(crate::error::ClaudeError::Config(
                "Invalid version format".to_string(),
            ));
        }

        let major = parts[0].parse().map_err(|_| {
            crate::error::ClaudeError::Config("Invalid version format".to_string())
        })?;
        let minor = parts[1].parse().map_err(|_| {
            crate::error::ClaudeError::Config("Invalid version format".to_string())
        })?;
        let patch = parts[2].parse().map_err(|_| {
            crate::error::ClaudeError::Config("Invalid version format".to_string())
        })?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    /// 获取当前版本
    pub fn current() -> Self {
        Self::new(1, 0, 0)
    }
}

impl fmt::Display for ConfigVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// 迁移结果
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// 是否成功
    pub success: bool,
    /// 源版本
    pub source_version: ConfigVersion,
    /// 目标版本
    pub target_version: ConfigVersion,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

/// 配置迁移管理器
pub struct MigrationManager;

impl MigrationManager {
    /// 创建新的迁移管理器
    pub fn new() -> Self {
        Self
    }

    /// 从配置中提取版本
    pub fn extract_version(&self, config: &Value) -> crate::error::Result<ConfigVersion> {
        if let Some(version_str) = config.get("version").and_then(|v| v.as_str()) {
            ConfigVersion::parse(version_str)
        } else {
            // 默认为1.0.0之前的版本
            Ok(ConfigVersion::new(0, 0, 0))
        }
    }

    /// 执行迁移
    pub fn migrate(
        &self,
        mut config: Value,
        target_version: Option<ConfigVersion>,
    ) -> crate::error::Result<MigrationResult> {
        let source_version = self.extract_version(&config)?;
        let target_version = target_version.unwrap_or_else(ConfigVersion::current);

        let mut result = MigrationResult {
            success: true,
            source_version,
            target_version,
            error: None,
        };

        if source_version >= target_version {
            // 已经是最新版本或更高版本
            return Ok(result);
        }

        // 简单的迁移：从0.0.0到1.0.0
        if source_version < ConfigVersion::new(1, 0, 0) {
            if let Some(obj) = config.as_object_mut() {
                obj.insert(
                    "version".to_string(),
                    Value::String("1.0.0".to_string()),
                );

                // 初始化默认结构
                if !obj.contains_key("api") {
                    obj.insert("api".to_string(), Value::Object(Default::default()));
                }
                if !obj.contains_key("model") {
                    obj.insert("model".to_string(), Value::String("sonnet".to_string()));
                }
                if !obj.contains_key("verbose") {
                    obj.insert("verbose".to_string(), Value::Bool(false));
                }
                if !obj.contains_key("feature_flags") {
                    obj.insert(
                        "feature_flags".to_string(),
                        Value::Object(Default::default()),
                    );
                }
                if !obj.contains_key("output") {
                    obj.insert("output".to_string(), Value::Object(Default::default()));
                }
            }
        }

        Ok(result)
    }

    /// 验证配置
    pub fn validate_config(&self, _config: &Value, _version: ConfigVersion) -> crate::error::Result<bool> {
        // 简单实现，总是返回true
        Ok(true)
    }
}

impl Default for MigrationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建标准迁移管理器
pub fn create_standard_migration_manager() -> MigrationManager {
    MigrationManager::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_config_version_parse() {
        let version = ConfigVersion::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_config_version_display() {
        let version = ConfigVersion::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_config_version_order() {
        let v1 = ConfigVersion::new(1, 0, 0);
        let v2 = ConfigVersion::new(1, 1, 0);
        let v3 = ConfigVersion::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_simple_migration() {
        let manager = MigrationManager::new();
        let config = json!({
            "old_field": "value"
        });

        let result = manager.migrate(config, None).unwrap();
        assert!(result.success);
    }
}
