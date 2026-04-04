//! Claude Code - AI-powered coding assistant (Rust implementation)
//! 
//! This is a Rust port of the Claude Code project, providing the same
//! AI-assisted coding capabilities with improved performance and type safety.

#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]

pub mod config;
pub mod error;
pub mod state;
pub mod utils;

// 暂时禁用有编译错误的模块（只保留核心功能）
#[cfg(feature = "full")]
pub mod commands;
#[cfg(feature = "full")]
pub mod tools;
#[cfg(feature = "full")]
pub mod bridge;
#[cfg(feature = "full")]
pub mod mcp;
#[cfg(feature = "full")]
pub mod agents;
#[cfg(feature = "full")]
pub mod analytics;
#[cfg(feature = "full")]
pub mod voice;
#[cfg(unix)]
pub mod daemon;
#[cfg(feature = "full")]
pub mod features;
#[cfg(feature = "full")]
pub mod bootstrap;
#[cfg(feature = "full")]
pub mod services;
#[cfg(feature = "full")]
pub mod performance;
#[cfg(feature = "full")]
pub mod security;
#[cfg(feature = "full")]
pub mod api;
#[cfg(feature = "full")]
pub mod plugins;
#[cfg(feature = "full")]
pub mod query;
#[cfg(feature = "full")]
pub mod skills;
#[cfg(feature = "full")]
pub mod editor_compat;

// Re-export commonly used types
pub use error::{ClaudeError, Result};
pub use state::AppState;

// Re-export security types
#[cfg(feature = "full")]
pub use security::{SecurityManager, PermissionManager, SandboxManager, AuditLogger};
