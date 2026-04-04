//! Plugin Marketplace Error Handling
//!
//! 插件市场专门的错误类型和处理。

use crate::error::ClaudeError;
use std::fmt;

/// 插件市场错误类型
#[derive(Debug, thiserror::Error)]
pub enum PluginMarketplaceError {
    /// API请求失败
    #[error("API请求失败: {0}")]
    ApiError(String),

    /// 网络错误
    #[error("网络错误: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// 插件未找到
    #[error("插件未找到: {0}")]
    PluginNotFound(String),

    /// 签名验证失败
    #[error("签名验证失败: {0}")]
    SignatureVerificationFailed(String),

    /// 资源限制超出
    #[error("资源限制超出: {0}")]
    ResourceLimitExceeded(String),

    /// 缓存错误
    #[error("缓存错误: {0}")]
    CacheError(String),

    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// 下载错误
    #[error("下载错误: {0}")]
    DownloadError(String),

    /// 安装错误
    #[error("安装错误: {0}")]
    InstallationError(String),

    /// 解析错误
    #[error("解析错误: {0}")]
    ParseError(String),

    /// 存储错误
    #[error("存储错误: {0}")]
    StorageError(String),

    /// 版本不兼容
    #[error("版本不兼容: {0}")]
    VersionIncompatible(String),

    /// 权限错误
    #[error("权限错误: {0}")]
    PermissionError(String),

    /// 超时错误
    #[error("操作超时: {0}")]
    TimeoutError(String),

    /// 限流错误
    #[error("API限流: {0}")]
    RateLimitError(String),

    /// 其他错误
    #[error("其他错误: {0}")]
    Other(String),
}

/// 插件市场结果类型
pub type Result<T> = std::result::Result<T, PluginMarketplaceError>;

impl From<PluginMarketplaceError> for ClaudeError {
    fn from(err: PluginMarketplaceError) -> Self {
        match err {
            PluginMarketplaceError::ApiError(msg) => ClaudeError::Other(format!("API错误: {}", msg)),
            PluginMarketplaceError::NetworkError(e) => ClaudeError::Network(e),
            PluginMarketplaceError::PluginNotFound(msg) => ClaudeError::Other(format!("插件未找到: {}", msg)),
            PluginMarketplaceError::SignatureVerificationFailed(msg) => ClaudeError::Permission(format!("签名验证失败: {}", msg)),
            PluginMarketplaceError::ResourceLimitExceeded(msg) => ClaudeError::Other(format!("资源限制超出: {}", msg)),
            PluginMarketplaceError::CacheError(msg) => ClaudeError::Other(format!("缓存错误: {}", msg)),
            PluginMarketplaceError::ConfigError(msg) => ClaudeError::Config(msg),
            PluginMarketplaceError::DownloadError(msg) => ClaudeError::Other(format!("下载错误: {}", msg)),
            PluginMarketplaceError::InstallationError(msg) => ClaudeError::Other(format!("安装错误: {}", msg)),
            PluginMarketplaceError::ParseError(msg) => ClaudeError::Other(format!("解析错误: {}", msg)),
            PluginMarketplaceError::StorageError(msg) => ClaudeError::Other(format!("存储错误: {}", msg)),
            PluginMarketplaceError::VersionIncompatible(msg) => ClaudeError::Other(format!("版本不兼容: {}", msg)),
            PluginMarketplaceError::PermissionError(msg) => ClaudeError::Permission(msg),
            PluginMarketplaceError::TimeoutError(msg) => ClaudeError::Other(format!("超时错误: {}", msg)),
            PluginMarketplaceError::RateLimitError(msg) => ClaudeError::Other(format!("API限流: {}", msg)),
            PluginMarketplaceError::Other(msg) => ClaudeError::Other(msg),
        }
    }
}

impl From<serde_json::Error> for PluginMarketplaceError {
    fn from(err: serde_json::Error) -> Self {
        PluginMarketplaceError::ParseError(err.to_string())
    }
}

impl From<std::io::Error> for PluginMarketplaceError {
    fn from(err: std::io::Error) -> Self {
        PluginMarketplaceError::StorageError(err.to_string())
    }
}

impl From<anyhow::Error> for PluginMarketplaceError {
    fn from(err: anyhow::Error) -> Self {
        PluginMarketplaceError::Other(err.to_string())
    }
}

impl PluginMarketplaceError {
    /// 是否为网络错误
    pub fn is_network_error(&self) -> bool {
        matches!(self, PluginMarketplaceError::NetworkError(_) | PluginMarketplaceError::ApiError(_))
    }

    /// 是否为可恢复错误
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            PluginMarketplaceError::NetworkError(_)
                | PluginMarketplaceError::ApiError(_)
                | PluginMarketplaceError::TimeoutError(_)
                | PluginMarketplaceError::RateLimitError(_)
        )
    }

    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            PluginMarketplaceError::ApiError(_) => "API_ERROR",
            PluginMarketplaceError::NetworkError(_) => "NETWORK_ERROR",
            PluginMarketplaceError::PluginNotFound(_) => "PLUGIN_NOT_FOUND",
            PluginMarketplaceError::SignatureVerificationFailed(_) => "SIGNATURE_VERIFICATION_FAILED",
            PluginMarketplaceError::ResourceLimitExceeded(_) => "RESOURCE_LIMIT_EXCEEDED",
            PluginMarketplaceError::CacheError(_) => "CACHE_ERROR",
            PluginMarketplaceError::ConfigError(_) => "CONFIG_ERROR",
            PluginMarketplaceError::DownloadError(_) => "DOWNLOAD_ERROR",
            PluginMarketplaceError::InstallationError(_) => "INSTALLATION_ERROR",
            PluginMarketplaceError::ParseError(_) => "PARSE_ERROR",
            PluginMarketplaceError::StorageError(_) => "STORAGE_ERROR",
            PluginMarketplaceError::VersionIncompatible(_) => "VERSION_INCOMPATIBLE",
            PluginMarketplaceError::PermissionError(_) => "PERMISSION_ERROR",
            PluginMarketplaceError::TimeoutError(_) => "TIMEOUT_ERROR",
            PluginMarketplaceError::RateLimitError(_) => "RATE_LIMIT_ERROR",
            PluginMarketplaceError::Other(_) => "OTHER_ERROR",
        }
    }

    /// 从HTTP状态码创建错误
    pub fn from_http_status(status: u16, message: String) -> Self {
        match status {
            404 => PluginMarketplaceError::PluginNotFound(message),
            429 => PluginMarketplaceError::RateLimitError(message),
            400..=499 => PluginMarketplaceError::ApiError(format!("客户端错误 {}: {}", status, message)),
            500..=599 => PluginMarketplaceError::ApiError(format!("服务器错误 {}: {}", status, message)),
            _ => PluginMarketplaceError::ApiError(format!("HTTP错误 {}: {}", status, message)),
        }
    }
}

// 为anyhow::Error添加一个辅助转换
impl PluginMarketplaceError {
    /// 从字符串创建其他错误
    pub fn other(msg: impl Into<String>) -> Self {
        PluginMarketplaceError::Other(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let api_error = PluginMarketplaceError::ApiError("test".to_string());
        let claude_error: ClaudeError = api_error.into();

        match claude_error {
            ClaudeError::Other(msg) => assert!(msg.contains("API错误")),
            _ => panic!("Expected ClaudeError::Other"),
        }
    }

    #[test]
    fn test_network_error_detection() {
        let network_error = PluginMarketplaceError::NetworkError(
            reqwest::Error::builder().build()
        );
        assert!(network_error.is_network_error());
        assert!(network_error.is_recoverable());
    }

    #[test]
    fn test_error_codes() {
        let errors = vec![
            (PluginMarketplaceError::ApiError("".to_string()), "API_ERROR"),
            (PluginMarketplaceError::PluginNotFound("".to_string()), "PLUGIN_NOT_FOUND"),
            (PluginMarketplaceError::SignatureVerificationFailed("".to_string()), "SIGNATURE_VERIFICATION_FAILED"),
        ];

        for (error, expected_code) in errors {
            assert_eq!(error.error_code(), expected_code);
        }
    }

    #[test]
    fn test_from_http_status() {
        let not_found = PluginMarketplaceError::from_http_status(404, "Not found".to_string());
        assert!(matches!(not_found, PluginMarketplaceError::PluginNotFound(_)));

        let rate_limit = PluginMarketplaceError::from_http_status(429, "Too many requests".to_string());
        assert!(matches!(rate_limit, PluginMarketplaceError::RateLimitError(_)));
    }
}