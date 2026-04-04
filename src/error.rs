//! Error types for Claude Code

use std::fmt;
use libloading;

/// Result type for Claude Code operations
pub type Result<T> = std::result::Result<T, ClaudeError>;

/// Configuration specific error
#[derive(Debug)]
pub enum ConfigError {
    /// Invalid configuration setting
    InvalidSetting(String),
    /// Configuration validation failed
    ValidationFailed(Vec<crate::config::ValidationError>),
    /// Configuration migration failed
    MigrationFailed(String),
    /// Configuration file not found
    NotFound(String),
    /// Configuration version mismatch
    VersionMismatch {
        expected: String,
        found: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidSetting(msg) => write!(f, "Invalid configuration setting: {}", msg),
            ConfigError::ValidationFailed(errors) => {
                write!(f, "Configuration validation failed:")?;
                for err in errors {
                    write!(f, "\n  - {}", err)?;
                }
                Ok(())
            }
            ConfigError::MigrationFailed(msg) => write!(f, "Configuration migration failed: {}", msg),
            ConfigError::NotFound(msg) => write!(f, "Configuration not found: {}", msg),
            ConfigError::VersionMismatch { expected, found } => {
                write!(f, "Configuration version mismatch: expected {}, found {}", expected, found)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Main error type for Claude Code
#[derive(Debug)]
pub enum ClaudeError {
    /// Configuration error
    Config(String),
    
    /// Configuration specific error
    ConfigError(ConfigError),
    
    /// IO error
    Io(std::io::Error),
    
    /// File error
    File(String),
    
    /// Network error
    Network(reqwest::Error),
    
    /// Serialization/deserialization error
    Serialization(serde_json::Error),
    
    /// Tool execution error
    Tool(String),
    
    /// Command error
    Command(String),
    
    /// Authentication error
    Auth(String),
    
    /// Permission error
    Permission(String),
    
    /// Bridge error
    Bridge(String),
    
    /// MCP (Model Context Protocol) error
    Mcp(String),
    
    /// State error
    State(String),
    
    /// Agent error
    Agent(String),
    
    /// Not implemented
    NotImplemented(String),
    
    /// Any other error
    Other(String),
    
    /// Editor error
    Editor(String),
    
    /// Skill error
    Skill(String),
}

impl fmt::Display for ClaudeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClaudeError::Config(msg) => write!(f, "Configuration error: {}", msg),
            ClaudeError::ConfigError(err) => write!(f, "{}", err),
            ClaudeError::Io(err) => write!(f, "IO error: {}", err),
            ClaudeError::File(msg) => write!(f, "File error: {}", msg),
            ClaudeError::Network(err) => write!(f, "Network error: {}", err),
            ClaudeError::Serialization(err) => write!(f, "Serialization error: {}", err),
            ClaudeError::Tool(msg) => write!(f, "Tool error: {}", msg),
            ClaudeError::Command(msg) => write!(f, "Command error: {}", msg),
            ClaudeError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            ClaudeError::Permission(msg) => write!(f, "Permission error: {}", msg),
            ClaudeError::Bridge(msg) => write!(f, "Bridge error: {}", msg),
            ClaudeError::Mcp(msg) => write!(f, "MCP error: {}", msg),
            ClaudeError::State(msg) => write!(f, "State error: {}", msg),
            ClaudeError::Agent(msg) => write!(f, "Agent error: {}", msg),
            ClaudeError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            ClaudeError::Other(msg) => write!(f, "Error: {}", msg),
            ClaudeError::Editor(msg) => write!(f, "Editor error: {}", msg),
            ClaudeError::Skill(msg) => write!(f, "Skill error: {}", msg),
        }
    }
}

impl std::error::Error for ClaudeError {}

impl From<std::io::Error> for ClaudeError {
    fn from(err: std::io::Error) -> Self {
        ClaudeError::Io(err)
    }
}

impl From<reqwest::Error> for ClaudeError {
    fn from(err: reqwest::Error) -> Self {
        ClaudeError::Network(err)
    }
}

impl From<serde_json::Error> for ClaudeError {
    fn from(err: serde_json::Error) -> Self {
        ClaudeError::Serialization(err)
    }
}

impl From<anyhow::Error> for ClaudeError {
    fn from(err: anyhow::Error) -> Self {
        ClaudeError::Other(err.to_string())
    }
}

impl From<walkdir::Error> for ClaudeError {
    fn from(err: walkdir::Error) -> Self {
        ClaudeError::Io(err.into())
    }
}

impl From<regex::Error> for ClaudeError {
    fn from(err: regex::Error) -> Self {
        ClaudeError::Other(err.to_string())
    }
}

impl From<&str> for ClaudeError {
    fn from(msg: &str) -> Self {
        ClaudeError::Other(msg.to_string())
    }
}

impl From<String> for ClaudeError {
    fn from(msg: String) -> Self {
        ClaudeError::Other(msg)
    }
}

impl From<url::ParseError> for ClaudeError {
    fn from(err: url::ParseError) -> Self {
        ClaudeError::Other(err.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for ClaudeError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        ClaudeError::Mcp(err.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for ClaudeError {
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        ClaudeError::Other(err.to_string())
    }
}

impl<T> From<tokio::sync::broadcast::error::SendError<T>> for ClaudeError {
    fn from(err: tokio::sync::broadcast::error::SendError<T>) -> Self {
        ClaudeError::Other(err.to_string())
    }
}

impl From<libloading::Error> for ClaudeError {
    fn from(err: libloading::Error) -> Self {
        ClaudeError::Tool(err.to_string())
    }
}

impl From<ConfigError> for ClaudeError {
    fn from(err: ConfigError) -> Self {
        ClaudeError::ConfigError(err)
    }
}

impl From<std::ffi::NulError> for ClaudeError {
    fn from(err: std::ffi::NulError) -> Self {
        ClaudeError::Other(err.to_string())
    }
}

impl From<std::str::Utf8Error> for ClaudeError {
    fn from(err: std::str::Utf8Error) -> Self {
        ClaudeError::Other(err.to_string())
    }
}
