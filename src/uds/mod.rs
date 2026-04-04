//! UDS Inbox 多消息融合系统
//! 
//! 提供基于 Unix Domain Socket 的消息融合功能，包括：
//! - 消息队列管理
//! - 消息融合算法
//! - 优先级管理
//! - 路由和地址解析
//! - Socket 服务端

pub mod message;
pub mod queue;
pub mod fusion;
pub mod server;
pub mod routing;

pub use message::*;
pub use queue::*;
pub use fusion::*;
pub use server::*;
pub use routing::*;

use crate::error::Result;

/// 初始化 UDS Inbox 系统
pub async fn initialize() -> Result<()> {
    tracing::info!("Initializing UDS Inbox system");
    Ok(())
}
