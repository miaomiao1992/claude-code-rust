//! 技能注册表
//!
//! 实现技能注册和管理功能

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::Result;
use super::types::{Skill, SkillMetadata};

/// 技能注册表
#[derive(Clone)]
pub struct SkillRegistry {
    /// 技能映射（名称 -> 技能实例）
    skills: Arc<RwLock<HashMap<String, Arc<dyn Skill>>>>,

    /// 别名映射（别名 -> 技能名称）
    aliases: Arc<RwLock<HashMap<String, String>>>,
}

impl SkillRegistry {
    /// 创建新的技能注册表
    pub fn new() -> Self {
        Self {
            skills: Arc::new(RwLock::new(HashMap::new())),
            aliases: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册技能
    pub async fn register<S>(&self, skill: S)
    where
        S: Skill + 'static,
    {
        let metadata = skill.metadata();
        let name = metadata.name.clone();

        // 注册技能
        self.skills.write().await.insert(name.clone(), Arc::new(skill));

        // 注册别名（如果有）
        for tag in metadata.tags.iter() {
            self.aliases.write().await.insert(tag.clone(), name.clone());
        }
    }

    /// 注销技能
    pub async fn unregister(&self, name: &str) -> Option<Arc<dyn Skill>> {
        let skill = self.skills.write().await.remove(name);

        // 清理别名
        if skill.is_some() {
            let mut aliases = self.aliases.write().await;
            aliases.retain(|_, v| v != name);
        }

        skill
    }

    /// 查找技能
    pub async fn find(&self, name: &str) -> Option<Arc<dyn Skill>> {
        // 先查找技能名称
        if let Some(skill) = self.skills.read().await.get(name) {
            return Some(skill.clone());
        }

        // 再查找别名
        if let Some(real_name) = self.aliases.read().await.get(name) {
            return self.skills.read().await.get(real_name).cloned();
        }

        None
    }

    /// 检查技能是否存在
    pub async fn has(&self, name: &str) -> bool {
        self.find(name).await.is_some()
    }

    /// 获取所有技能
    pub async fn list(&self) -> Vec<Arc<dyn Skill>> {
        self.skills.read().await.values().cloned().collect()
    }

    /// 获取所有技能名称
    pub async fn names(&self) -> Vec<String> {
        self.skills.read().await.keys().cloned().collect()
    }

    /// 获取技能数量
    pub async fn len(&self) -> usize {
        self.skills.read().await.len()
    }

    /// 获取技能元数据
    pub async fn get_metadata(&self, name: &str) -> Option<SkillMetadata> {
        self.find(name).await.map(|skill| skill.metadata())
    }

    /// 搜索技能
    pub async fn search(&self, query: &str) -> Vec<SkillMetadata> {
        let skills = self.skills.read().await;
        let query_lower = query.to_lowercase();

        skills.values()
            .filter(|skill| {
                let metadata = skill.metadata();
                metadata.name.to_lowercase().contains(&query_lower) ||
                metadata.description.to_lowercase().contains(&query_lower) ||
                metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .map(|skill| skill.metadata())
            .collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 技能管理器
pub struct SkillManager {
    registry: SkillRegistry,
    loaders: Vec<Box<dyn SkillLoader>>,
}

impl SkillManager {
    /// 创建新的技能管理器
    pub fn new() -> Self {
        Self {
            registry: SkillRegistry::new(),
            loaders: Vec::new(),
        }
    }

    /// 添加技能加载器
    pub fn add_loader<L>(&mut self, loader: L)
    where
        L: SkillLoader + 'static,
    {
        self.loaders.push(Box::new(loader));
    }

    /// 加载所有技能
    pub async fn load_all(&mut self) -> Result<()> {
        for loader in &self.loaders {
            if let Err(e) = loader.load(&self.registry).await {
                tracing::error!("Failed to load skills from {}: {}", loader.name(), e);
            }
        }
        Ok(())
    }

    /// 获取技能注册表
    pub fn registry(&self) -> &SkillRegistry {
        &self.registry
    }

    /// 重新加载技能
    pub async fn reload(&mut self) -> Result<()> {
        // 清空当前注册表
        *self.registry.skills.write().await = HashMap::new();
        *self.registry.aliases.write().await = HashMap::new();

        // 重新加载
        self.load_all().await
    }
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 技能加载器 trait
#[async_trait::async_trait]
pub trait SkillLoader: Send + Sync {
    /// 加载技能到注册表
    async fn load(&self, registry: &SkillRegistry) -> Result<()>;

    /// 获取加载器名称
    fn name(&self) -> &str;
}