//! 技能执行器
//!
//! 实现技能的调度和执行功能

use std::sync::Arc;
use std::time::Instant;

use crate::error::{Result, ClaudeError};
use super::types::{SkillContext, SkillResult};
use super::registry::SkillManager;

/// 技能执行器
#[derive(Clone)]
pub struct SkillExecutor {
    manager: Arc<SkillManager>,
}

impl SkillExecutor {
    /// 创建新的技能执行器
    pub fn new(manager: SkillManager) -> Self {
        Self { manager: Arc::new(manager) }
    }

    /// 执行技能
    pub async fn execute(
        &self,
        name: &str,
        args: Option<&str>,
        context: SkillContext,
    ) -> Result<SkillResult> {
        let start_time = Instant::now();

        // 查找技能
        let skill = self.manager.registry().find(name).await
            .ok_or_else(|| ClaudeError::Skill(format!("技能未找到: {}", name)))?;

        // 获取技能元数据
        let metadata = skill.metadata();

        // 检查权限（简化实现）
        if !metadata.required_permissions.is_empty() {
            // 在实际实现中，这里应该检查用户权限
            tracing::debug!("技能 {} 需要权限: {:?}", name, metadata.required_permissions);
        }

        // 执行技能
        let result = match skill.execute(args, context).await {
            Ok(result) => result,
            Err(e) => SkillResult::error(
                name.to_string(),
                format!("技能执行失败: {}", e),
                start_time.elapsed().as_millis() as u64,
            ),
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // 记录执行日志
        tracing::debug!(
            "技能执行完成: {} ({}ms, 成功: {})",
            name,
            duration_ms,
            result.success
        );

        Ok(result)
    }

    /// 批量执行技能
    pub async fn execute_batch(
        &self,
        commands: Vec<(String, Option<String>)>,
        context: SkillContext,
    ) -> Result<Vec<SkillResult>> {
        let mut results = Vec::new();

        for (name, args) in commands {
            let result = self.execute(&name, args.as_deref(), context.clone()).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// 验证技能输入
    pub async fn validate_input(&self, name: &str, input: &str) -> Result<bool> {
        // 查找技能
        let skill = self.manager.registry().find(name).await
            .ok_or_else(|| ClaudeError::Skill(format!("技能未找到: {}", name)))?;

        let metadata = skill.metadata();

        // 检查输入模式（简化实现）
        if let Some(schema) = &metadata.input_schema {
            // 在实际实现中，这里应该根据JSON Schema验证输入
            tracing::debug!("验证技能 {} 输入: {}", name, input);
            Ok(true)
        } else {
            // 没有输入模式，接受任何输入
            Ok(true)
        }
    }

    /// 获取技能帮助
    pub async fn get_help(&self, name: &str) -> Result<String> {
        // 查找技能
        let skill = self.manager.registry().find(name).await
            .ok_or_else(|| ClaudeError::Skill(format!("技能未找到: {}", name)))?;

        let metadata = skill.metadata();

        let help_text = format!(
            "技能: {}\n\n描述: {}\n\n分类: {:?}\n\n版本: {}\n\n作者: {}\n\n标签: {:?}",
            metadata.name,
            metadata.description,
            metadata.category,
            metadata.version.as_deref().unwrap_or("未指定"),
            metadata.author.as_deref().unwrap_or("未指定"),
            metadata.tags
        );

        Ok(help_text)
    }
}

/// 技能执行上下文构建器
pub struct SkillContextBuilder {
    cwd: std::path::PathBuf,
    project_root: std::path::PathBuf,
    env: std::collections::HashMap<String, String>,
    config: std::collections::HashMap<String, serde_json::Value>,
    session_state: std::collections::HashMap<String, serde_json::Value>,
}

impl SkillContextBuilder {
    /// 创建新的上下文构建器
    pub fn new() -> Self {
        Self {
            cwd: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            project_root: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            env: std::env::vars().collect(),
            config: std::collections::HashMap::new(),
            session_state: std::collections::HashMap::new(),
        }
    }

    /// 设置当前工作目录
    pub fn with_cwd(mut self, cwd: std::path::PathBuf) -> Self {
        self.cwd = cwd;
        self
    }

    /// 设置项目根目录
    pub fn with_project_root(mut self, project_root: std::path::PathBuf) -> Self {
        self.project_root = project_root;
        self
    }

    /// 添加环境变量
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    /// 添加配置
    pub fn with_config(mut self, key: &str, value: serde_json::Value) -> Self {
        self.config.insert(key.to_string(), value);
        self
    }

    /// 添加会话状态
    pub fn with_session_state(mut self, key: &str, value: serde_json::Value) -> Self {
        self.session_state.insert(key.to_string(), value);
        self
    }

    /// 构建技能上下文
    pub fn build(self) -> SkillContext {
        SkillContext {
            cwd: self.cwd,
            project_root: self.project_root,
            env: self.env,
            config: self.config,
            session_state: self.session_state,
        }
    }
}

impl Default for SkillContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::loader::BuiltinSkillLoader;

    #[tokio::test]
    async fn test_skill_executor() {
        let mut manager = SkillManager::new();
        manager.add_loader(BuiltinSkillLoader);
        manager.load_all().await.unwrap();

        let executor = SkillExecutor::new(manager);
        let context = SkillContextBuilder::new().build();

        // 测试帮助技能
        let result = executor.execute("help", None, context.clone()).await.unwrap();
        assert!(result.success);
        assert_eq!(result.skill_name, "help");

        // 测试版本技能
        let result = executor.execute("version", None, context.clone()).await.unwrap();
        assert!(result.success);
        assert_eq!(result.skill_name, "version");

        // 测试配置检查技能
        let result = executor.execute("config-check", None, context).await.unwrap();
        assert!(result.success);
        assert_eq!(result.skill_name, "config-check");
    }

    #[tokio::test]
    async fn test_skill_not_found() {
        let manager = SkillManager::new();
        let executor = SkillExecutor::new(manager);
        let context = SkillContextBuilder::new().build();

        let result = executor.execute("nonexistent", None, context).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClaudeError::Skill(_)));
    }

    #[tokio::test]
    async fn test_skill_context_builder() {
        let context = SkillContextBuilder::new()
            .with_cwd(std::path::PathBuf::from("/test/cwd"))
            .with_project_root(std::path::PathBuf::from("/test/project"))
            .with_env("TEST_KEY", "TEST_VALUE")
            .with_config("test_config", serde_json::json!({"value": 123}))
            .with_session_state("test_state", serde_json::json!({"state": "active"}))
            .build();

        assert_eq!(context.cwd, std::path::PathBuf::from("/test/cwd"));
        assert_eq!(context.project_root, std::path::PathBuf::from("/test/project"));
        assert_eq!(context.get_env("TEST_KEY"), Some(&"TEST_VALUE".to_string()));
        assert_eq!(context.get_config("test_config"), Some(&serde_json::json!({"value": 123})));
    }
}