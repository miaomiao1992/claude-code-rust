//! LSP相关工具
//! 
//! 实现LSP Tool等语言服务器交互工具

use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// LSP工具
/// 用于与语言服务器进行交互，提供代码补全、错误检查等功能
pub struct LSPTool;

#[async_trait]
impl Tool for LSPTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("LSP", "Language Server Protocol interaction for code intelligence")
            .category(ToolCategory::CodeSearch)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["lsp".to_string(), "language".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("action".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "LSP action to perform: 'complete', 'hover', 'definition', 'references', 'diagnostics'"
                    })),
                    ("file_path".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Path to the file to analyze"
                    })),
                    ("position".to_string(), serde_json::json!({ 
                        "type": "object", 
                        "properties": {
                            "line": { "type": "integer" },
                            "character": { "type": "integer" }
                        },
                        "description": "Position in the file (line and character)"
                    })),
                    ("text".to_string(), serde_json::json!({ 
                        "type": "string", 
                        "description": "Optional text content for analysis"
                    })),
                ])),
                required: Some(vec!["action".to_string(), "file_path".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: Value,
        context: ToolUseContext,
    ) -> Result<ToolResult> {
        let action = input["action"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("action is required".to_string()))?;
        
        let file_path = input["file_path"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("file_path is required".to_string()))?;
        
        let full_path = if file_path.starts_with('/') {
            file_path.to_string()
        } else {
            context.cwd.join(file_path).to_string_lossy().to_string()
        };
        
        let position = input.get("position");
        let text = input.get("text").and_then(|t| t.as_str());
        
        // 模拟LSP响应
        let result = match action {
            "complete" => {
                serde_json::json!({ 
                    "action": "complete",
                    "file_path": full_path,
                    "position": position,
                    "completions": [
                        { "label": "function", "kind": "function", "detail": "Function declaration" },
                        { "label": "let", "kind": "keyword", "detail": "Variable declaration" },
                        { "label": "if", "kind": "keyword", "detail": "Conditional statement" }
                    ]
                })
            },
            "hover" => {
                serde_json::json!({ 
                    "action": "hover",
                    "file_path": full_path,
                    "position": position,
                    "contents": "Function: example()\n\nDescription: This is an example function" 
                })
            },
            "definition" => {
                serde_json::json!({ 
                    "action": "definition",
                    "file_path": full_path,
                    "position": position,
                    "definitions": [
                        { "file": full_path, "line": 10, "character": 5 }
                    ]
                })
            },
            "references" => {
                serde_json::json!({ 
                    "action": "references",
                    "file_path": full_path,
                    "position": position,
                    "references": [
                        { "file": full_path, "line": 20, "character": 10 },
                        { "file": full_path, "line": 30, "character": 15 }
                    ]
                })
            },
            "diagnostics" => {
                serde_json::json!({ 
                    "action": "diagnostics",
                    "file_path": full_path,
                    "diagnostics": [
                        { "severity": "error", "message": "Undefined variable", "line": 5, "character": 10 },
                        { "severity": "warning", "message": "Unused variable", "line": 15, "character": 5 }
                    ]
                })
            },
            _ => {
                return Err(crate::error::ClaudeError::Tool(format!("Unknown LSP action: {}", action)));
            }
        };
        
        Ok(ToolResult::success(result))
    }
    
    fn get_path(&self, input: &Value) -> Option<String> {
        input["file_path"].as_str().map(|s| s.to_string())
    }
    
    fn get_activity_description(&self, input: &Value) -> Option<String> {
        input["action"].as_str().map(|action| {
            input["file_path"].as_str().map(|path| {
                format!("Performing LSP {} on {}", action, path)
            }).unwrap_or_else(|| format!("Performing LSP action: {}", action))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lsp_metadata() {
        let tool = LSPTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "LSP");
        assert_eq!(metadata.category, ToolCategory::CodeSearch);
    }
    
    #[tokio::test]
    async fn test_lsp_execute() {
        use crate::config::Config;
        use crate::state::AppState;
        
        let tool = LSPTool;
        let input = serde_json::json!({ 
            "action": "complete",
            "file_path": "src/main.rs",
            "position": { "line": 10, "character": 5 }
        });
        
        let context = ToolUseContext::new(
            std::path::PathBuf::from("."),
            Config::default(),
            AppState::default()
        );
        let result = tool.execute(input, context).await.unwrap();
        assert!(result.error.is_none());
        let data = result.data;
        assert_eq!(data["action"], "complete");
        assert!(data["completions"].as_array().unwrap().len() > 0);
    }
    
    #[tokio::test]
    async fn test_lsp_unknown_action() {
        use crate::config::Config;
        use crate::state::AppState;
        
        let tool = LSPTool;
        let input = serde_json::json!({ 
            "action": "unknown",
            "file_path": "src/main.rs"
        });
        
        let context = ToolUseContext::new(
            std::path::PathBuf::from("."),
            Config::default(),
            AppState::default()
        );
        let result = tool.execute(input, context).await;
        assert!(result.is_err());
    }
}
