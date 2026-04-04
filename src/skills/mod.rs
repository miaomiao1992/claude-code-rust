//! 技能系统模块
//!
//! 这个模块实现了完整的技能发现、加载和执行系统，包括：
//! - 技能定义和注册
//! - 技能发现机制
//! - 技能加载器
//! - 技能执行器
//! - 技能元数据管理

pub mod types;
pub mod registry;
pub mod loader;
pub mod executor;
pub mod discovery;

// 重新导出主要类型
pub use types::{Skill, SkillMetadata, SkillResult, SkillContext, SkillInputSchema};
pub use registry::{SkillRegistry, SkillManager, SkillLoader};
pub use loader::{BuiltinSkillLoader, FileSystemSkillLoader};
pub use executor::SkillExecutor;
pub use discovery::SkillDiscovery;

use crate::error::Result;

/// 初始化技能系统
pub async fn init() -> Result<SkillManager> {
    let mut manager = SkillManager::new();

    // 注册内置技能加载器
    manager.add_loader(BuiltinSkillLoader);

    // 注册文件系统技能加载器
    manager.add_loader(FileSystemSkillLoader::default());

    // 加载所有技能
    manager.load_all().await?;

    tracing::info!("Skill system initialized with {} skills",
        manager.registry().len().await);

    Ok(manager)
}

/// 获取所有可用技能名称
pub async fn list_skills() -> Result<Vec<String>> {
    let manager = init().await?;
    Ok(manager.registry().names().await)
}

/// 执行技能
pub async fn execute_skill(name: &str, args: Option<&str>, context: SkillContext) -> Result<SkillResult> {
    let manager = init().await?;
    let executor = SkillExecutor::new(manager);
    executor.execute(name, args, context).await
}