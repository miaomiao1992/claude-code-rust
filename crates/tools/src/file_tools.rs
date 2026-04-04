//! 文件操作工具

use crate::base::{Tool, ToolBuilder};
use crate::error::Result;
use crate::types::{ToolCategory, ToolPermissionLevel, ToolMetadata, ToolResult, ToolUseContext};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;

/// 文件读取工具
pub struct FileReadTool;

#[async_trait]
impl Tool for FileReadTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("Read", "Read file contents")
            .category(ToolCategory::FileOperation)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["read".to_string(), "cat".to_string()])
            .read_only()
            .build_metadata()
    }

    async fn execute(
        &self,
        input: Value,
        context: ToolUseContext,
    ) -> Result<ToolResult<Value>> {
        // 解析输入参数
        let file_path = input.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::ToolError::ValidationError("Missing file_path".to_string()))?;

        // 构建完整路径（相对于当前工作目录）
        let full_path = if Path::new(file_path).is_absolute() {
            file_path.to_string()
        } else {
            let cwd = context.cwd.to_string_lossy().to_string();
            format!("{}/{}", cwd, file_path)
        };

        // 读取文件内容
        let content = std::fs::read_to_string(&full_path)
            .map_err(crate::error::ToolError::IoError)?;

        // 返回结果
        Ok(ToolResult::success(Value::String(content)))
    }
}

/// 文件编辑工具
pub struct FileEditTool;

#[async_trait]
impl Tool for FileEditTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("Edit", "Edit file contents by replacing text")
            .category(ToolCategory::FileOperation)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["edit".to_string()])
            .build_metadata()
    }

    async fn execute(
        &self,
        input: Value,
        context: ToolUseContext,
    ) -> Result<ToolResult<Value>> {
        // 解析输入参数
        let file_path = input.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::ToolError::ValidationError("Missing file_path".to_string()))?;

        let old_string = input.get("old_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::ToolError::ValidationError("Missing old_string".to_string()))?;

        let new_string = input.get("new_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::ToolError::ValidationError("Missing new_string".to_string()))?;

        // 构建完整路径
        let full_path = if Path::new(file_path).is_absolute() {
            file_path.to_string()
        } else {
            let cwd = context.cwd.to_string_lossy().to_string();
            format!("{}/{}", cwd, file_path)
        };

        // 读取文件内容
        let content = std::fs::read_to_string(&full_path)
            .map_err(crate::error::ToolError::IoError)?;

        // 替换文本
        if !content.contains(old_string) {
            return Err(crate::error::ToolError::ValidationError(
                format!("Pattern not found in file: {}", old_string)
            ));
        }

        let new_content = content.replace(old_string, new_string);

        // 写入文件
        std::fs::write(&full_path, new_content)
            .map_err(crate::error::ToolError::IoError)?;

        // 返回结果
        Ok(ToolResult::success(Value::String("File edited successfully".to_string())))
    }
}

/// 文件写入工具
pub struct FileWriteTool;

#[async_trait]
impl Tool for FileWriteTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("Write", "Write content to a file")
            .category(ToolCategory::FileOperation)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["write".to_string()])
            .build_metadata()
    }

    async fn execute(
        &self,
        input: Value,
        context: ToolUseContext,
    ) -> Result<ToolResult<Value>> {
        // 解析输入参数
        let file_path = input.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::ToolError::ValidationError("Missing file_path".to_string()))?;

        let content = input.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::ToolError::ValidationError("Missing content".to_string()))?;

        // 构建完整路径
        let full_path = if Path::new(file_path).is_absolute() {
            file_path.to_string()
        } else {
            let cwd = context.cwd.to_string_lossy().to_string();
            format!("{}/{}", cwd, file_path)
        };

        // 写入文件
        std::fs::write(&full_path, content)
            .map_err(crate::error::ToolError::IoError)?;

        // 返回结果
        Ok(ToolResult::success(Value::String("File written successfully".to_string())))
    }
}