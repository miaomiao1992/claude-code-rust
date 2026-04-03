//! Git相关工具
//! 
//! 实现EnterWorktree Tool等Git操作工具

use crate::error::Result;
use async_trait::async_trait;
use git2::{Repository, BranchType};
use std::path::{Path, PathBuf};
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// EnterWorktree工具
/// 用于创建隔离的git worktree并切换到其中
pub struct EnterWorktreeTool;

#[async_trait]
impl Tool for EnterWorktreeTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("EnterWorktree", "Create isolated git worktree and switch current session into it")
            .category(ToolCategory::FileOperation)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["enterworktree".to_string(), "worktree".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("branch".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Branch name for the worktree"
                    })),
                    ("path".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Optional path for the worktree"
                    })),
                ])),
                required: Some(vec!["branch".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        context: ToolUseContext,
    ) -> Result<ToolResult> {
        let branch = input["branch"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("branch is required".to_string()))?;
        
        let path = input["path"].as_str().unwrap_or(".claude/worktrees/default");
        
        // 检查当前目录是否是git仓库
        let repo = Repository::open(&context.cwd)
            .map_err(|e| crate::error::ClaudeError::Other(format!("Not a git repository: {}", e)))?;
        
        // 构建worktree路径
        let worktree_path = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            context.cwd.join(path)
        };
        
        // 确保worktree目录存在
        if let Some(parent) = worktree_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::error::ClaudeError::File(format!("Failed to create directory: {}", e)))?;
        }
        
        // 检查分支是否存在
        let branch_ref = format!("refs/heads/{}", branch);
        let branch_exists = repo.find_reference(&branch_ref).is_ok();
        
        // 创建worktree
        let worktree = repo.worktree(branch, &worktree_path, None)
            .map_err(|e| crate::error::ClaudeError::Other(format!("Failed to create worktree: {}", e)))?;
        
        Ok(ToolResult::success(serde_json::json!({ 
            "branch": branch,
            "path": worktree_path.to_string_lossy(),
            "result": "Entered worktree successfully",
        })))
    }
    
    fn get_path(&self, input: &serde_json::Value) -> Option<String> {
        input["path"].as_str().map(|s| s.to_string())
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["branch"].as_str().map(|b| format!("Entering worktree for branch '{}'", b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use git2::Repository;
    
    #[test]
    fn test_enterworktree_metadata() {
        let tool = EnterWorktreeTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "EnterWorktree");
        assert_eq!(metadata.category, ToolCategory::FileOperation);
    }
    
    #[tokio::test]
    async fn test_enterworktree_execute() {
        use crate::config::Config;
        use crate::state::AppState;
        
        // 创建临时目录作为测试仓库
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        
        // 配置git
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
        
        // 创建初始提交
        let mut index = repo.index().unwrap();
        let file_path = temp_dir.path().join("README.md");
        std::fs::write(&file_path, "# Test Repository").unwrap();
        index.add_path(Path::new("README.md")).unwrap();
        let oid = index.write_tree().unwrap();
        let tree = repo.find_tree(oid).unwrap();
        let sig = repo.signature().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
        
        // 创建工具上下文
        let context = ToolUseContext::new(
            temp_dir.path().to_path_buf(),
            Config::default(),
            AppState::default()
        );
        
        // 测试创建新worktree
        let tool = EnterWorktreeTool;
        let input = serde_json::json!({
            "branch": "test-branch"
        });
        
        let result = tool.execute(input, context).await.unwrap();
        assert!(result.error.is_none());
        let data = result.data;
        assert_eq!(data["branch"], "test-branch");
        // 路径应该包含worktree目录
        assert!(data["path"].as_str().unwrap().contains("worktrees"));
    }
}
