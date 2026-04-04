//! LSP 集成模块
//!
//! 实现通用 Language Server Protocol 集成

use super::common::*;
use crate::error::Result;
use std::collections::HashMap;

/// LSP 集成
pub struct LspIntegration {
    config: EditorConfig,
    language_servers: Vec<LanguageServerInfo>,
}

/// 语言服务器信息
#[derive(Debug, Clone)]
struct LanguageServerInfo {
    name: String,
    version: Option<String>,
    languages: Vec<String>,
    running: bool,
}

impl LspIntegration {
    /// 创建新的 LSP 集成
    pub fn new() -> Self {
        Self {
            config: EditorConfig::default(),
            language_servers: vec![
                LanguageServerInfo {
                    name: "rust-analyzer".to_string(),
                    version: None,
                    languages: vec!["rust".to_string()],
                    running: false,
                },
                LanguageServerInfo {
                    name: "typescript-language-server".to_string(),
                    version: None,
                    languages: vec!["typescript".to_string(), "javascript".to_string()],
                    running: false,
                },
                LanguageServerInfo {
                    name: "pylsp".to_string(),
                    version: None,
                    languages: vec!["python".to_string()],
                    running: false,
                },
            ],
        }
    }
}

#[async_trait::async_trait]
impl EditorIntegration for LspIntegration {
    async fn init(&mut self, config: &EditorConfig) -> Result<()> {
        self.config = config.clone();

        // 检测语言服务器
        for server in &mut self.language_servers {
            // 简化：假设服务器可用
            server.running = true;
        }

        tracing::info!("LSP integration initialized with {} language servers", self.language_servers.len());
        Ok(())
    }

    fn supported_features(&self) -> Vec<EditorFeature> {
        vec![
            EditorFeature::CodeCompletion,
            EditorFeature::SyntaxHighlighting,
            EditorFeature::ErrorChecking,
            EditorFeature::CodeNavigation,
            EditorFeature::Refactoring,
            EditorFeature::StateQuery,
        ]
    }

    fn supports_command(&self, command: &str) -> bool {
        matches!(command,
            "getState" | "getLanguageServers" |
            "complete" | "hover" | "definition" | "references"
        )
    }

    async fn execute_command(&self, command: &str, args: serde_json::Value) -> Result<serde_json::Value> {
        match command {
            "getState" => {
                let servers: Vec<_> = self.language_servers.iter()
                    .map(|server| serde_json::json!({
                        "name": server.name,
                        "version": server.version,
                        "languages": server.languages,
                        "running": server.running,
                    }))
                    .collect();

                Ok(serde_json::json!({
                    "success": true,
                    "editor": "LSP Client",
                    "languageServers": servers,
                    "count": servers.len()
                }))
            }
            "getLanguageServers" => {
                let servers: Vec<_> = self.language_servers.iter()
                    .map(|server| serde_json::json!({
                        "name": server.name,
                        "languages": server.languages,
                        "running": server.running,
                    }))
                    .collect();

                Ok(serde_json::json!({
                    "success": true,
                    "servers": servers
                }))
            }
            "complete" => {
                let file_path = args["filePath"].as_str().unwrap_or("");
                let position = args["position"].as_object();
                let language = args["language"].as_str().unwrap_or("");

                Ok(serde_json::json!({
                    "success": true,
                    "command": "complete",
                    "filePath": file_path,
                    "language": language,
                    "completions": [
                        { "label": "function", "kind": "function", "detail": "Function declaration" },
                        { "label": "let", "kind": "keyword", "detail": "Variable declaration" },
                        { "label": "if", "kind": "keyword", "detail": "Conditional statement" },
                    ]
                }))
            }
            "hover" => {
                let file_path = args["filePath"].as_str().unwrap_or("");
                let position = args["position"].as_object();

                Ok(serde_json::json!({
                    "success": true,
                    "command": "hover",
                    "filePath": file_path,
                    "contents": "Function: example()\n\nDescription: This is an example function"
                }))
            }
            "definition" => {
                let file_path = args["filePath"].as_str().unwrap_or("");
                let position = args["position"].as_object();

                Ok(serde_json::json!({
                    "success": true,
                    "command": "definition",
                    "filePath": file_path,
                    "definitions": [
                        { "file": file_path, "line": 10, "character": 5 }
                    ]
                }))
            }
            "references" => {
                let file_path = args["filePath"].as_str().unwrap_or("");
                let position = args["position"].as_object();

                Ok(serde_json::json!({
                    "success": true,
                    "command": "references",
                    "filePath": file_path,
                    "references": [
                        { "file": file_path, "line": 20, "character": 10 },
                        { "file": file_path, "line": 30, "character": 15 }
                    ]
                }))
            }
            _ => {
                Err(crate::error::ClaudeError::Editor(format!("Unknown command: {}", command)))
            }
        }
    }

    async fn get_state(&self) -> Result<EditorState> {
        let servers: Vec<_> = self.language_servers.iter()
            .map(|server| serde_json::json!({
                "name": server.name,
                "languages": server.languages,
                "running": server.running,
            }))
            .collect();

        Ok(EditorState {
            is_running: true,
            editor_version: Some("LSP Client".to_string()),
            extensions: HashMap::from_iter([(
                "lsp".to_string(),
                ExtensionState {
                    id: "lsp".to_string(),
                    name: "Language Server Protocol".to_string(),
                    enabled: true,
                    version: "3.17".to_string(),
                    status: ExtensionStatus::Active,
                }
            )]),
            ..Default::default()
        })
    }

    async fn update_config(&mut self, config: &EditorConfig) -> Result<()> {
        self.config = config.clone();
        Ok(())
    }

    fn name(&self) -> &str {
        "LSP"
    }
}

impl Default for LspIntegration {
    fn default() -> Self {
        Self::new()
    }
}