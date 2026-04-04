//! Teleport 跨机上下文传递系统
//! 
//! 提供跨机器的上下文传递和任务迁移功能，包括：
//! - 消息打包和序列化
//! - 传输协议
//! - 远程执行
//! - 上下文状态管理

pub mod packet;
pub mod protocol;
pub mod remote;
pub mod context;
pub mod executor;

pub use packet::*;
pub use protocol::*;
pub use remote::*;
pub use context::*;
pub use executor::*;

use crate::error::Result;

/// 初始化 Teleport 系统
pub async fn initialize() -> Result<()> {
    tracing::info!("Initializing Teleport system");
    Ok(())
}
