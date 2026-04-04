//! 单次查询命令
//!
//! 这个模块实现了单次查询功能，使用 QueryEngine 处理查询。

#[cfg(feature = "full")]
use crate::config::Settings;
#[cfg(feature = "full")]
use crate::error::Result;
#[cfg(feature = "full")]
use crate::state::AppState;
#[cfg(feature = "full")]
use crate::tools;
#[cfg(feature = "full")]
use std::sync::Arc;

/// 运行单次查询
#[cfg(feature = "full")]
pub async fn run(query: String, settings: Settings, state: AppState) -> Result<()> {
    tracing::info!("Running query: {}", query);

    // 初始化工具系统
    let tool_manager = Arc::new(tools::init().await?);

    // 创建查询引擎
    let query_engine = crate::query::engine::QueryEngine::new(
        settings,
        state,
        tool_manager,
    ).await?;

    // 提交查询
    let result = query_engine.submit_message(&query).await?;

    // 显示结果
    if let Some(response) = result.response {
        if let Some(text) = response.text_content() {
            println!("{}", text);
        } else {
            println!("(Received non-text response)");
        }
    } else {
        println!("(No response generated)");
    }

    Ok(())
}

/// 运行单次查询（简化版本）
#[cfg(not(feature = "full"))]
pub async fn run(_query: String, _settings: crate::config::Settings, _state: crate::state::AppState) -> crate::error::Result<()> {
    tracing::info!("Query command is not available in this build. Use --features full to enable it.");
    Ok(())
}
