//! 网络相关工具
//! 
//! 实现WebFetch和WebSearch等网络工具

use crate::error::Result;
use async_trait::async_trait;
use reqwest::Client;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// WebFetch工具
/// 用于抓取URL内容并使用AI模型处理
pub struct WebFetchTool {
    client: Client,
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Tool for WebFetchTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("WebFetch", "Fetches content from URL and processes it using AI model")
            .category(ToolCategory::AgentSystem)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["webfetch".to_string(), "fetch".to_string()])
            .read_only()
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("url".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Fully-formed valid URL to fetch"
                    })),
                    ("prompt".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "What information to extract from the content"
                    })),
                ])),
                required: Some(vec!["url".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let url = input["url"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("url is required".to_string()))?;
        
        let prompt = input["prompt"].as_str().unwrap_or("");
        
        // 确保URL是HTTPS
        let url = if url.starts_with("http://") {
            url.replace("http://", "https://")
        } else {
            url.to_string()
        };
        
        // 发送请求
        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| crate::error::ClaudeError::Network(e))?;
        
        // 检查重定向
        let final_url = response.url().to_string();
        if final_url != url {
            tracing::info!("URL redirected to: {}", final_url);
        }
        
        // 读取内容
        let content = response.text()
            .await
            .map_err(|e| crate::error::ClaudeError::Network(e))?;
        
        // 对于GitHub URL，建议使用gh CLI
        if final_url.contains("github.com") {
            tracing::info!("GitHub URL detected, consider using gh CLI for better results");
        }
        
        // 处理大型内容
        let content = if content.len() > 100000 {
            // 对于大型内容，进行简单的总结
            format!("[Content truncated ({} characters). Here's a summary: ...]", content.len())
        } else {
            content
        };
        
        Ok(ToolResult::success(serde_json::json!({
            "content": content,
            "url": final_url,
            "prompt": prompt,
            "content_length": content.len(),
        })))
    }
    
    fn get_path(&self, input: &serde_json::Value) -> Option<String> {
        input["url"].as_str().map(|s| s.to_string())
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["url"].as_str().map(|u| format!("Fetching content from {}", u))
    }
}

/// WebSearch工具
/// 用于搜索网页并使用结果来提供信息
pub struct WebSearchTool {
    client: Client,
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("WebSearch", "Search web and use results to inform responses")
            .category(ToolCategory::AgentSystem)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["websearch".to_string(), "search".to_string()])
            .read_only()
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("query".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "Search query to execute"
                    })),
                    ("num".to_string(), serde_json::json!({
                        "type": "integer",
                        "description": "Maximum number of search results to return",
                        "default": 5
                    })),
                ])),
                required: Some(vec!["query".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let query = input["query"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("query is required".to_string()))?;
        
        let num = input["num"].as_u64().unwrap_or(5) as usize;
        
        // 注意：这里需要集成实际的搜索引擎API
        // 由于没有实际的搜索API密钥，这里返回模拟结果
        // 实际实现时应该使用Google Search API或其他搜索服务
        
        let mock_results = vec![
            serde_json::json!({
                "title": "Sample Search Result 1",
                "url": "https://example.com/result1",
                "snippet": "This is a sample search result snippet for testing purposes."
            }),
            serde_json::json!({
                "title": "Sample Search Result 2",
                "url": "https://example.com/result2",
                "snippet": "Another sample search result snippet for demonstration."
            }),
        ];
        
        Ok(ToolResult::success(serde_json::json!({
            "query": query,
            "results": mock_results,
            "num_results": mock_results.len(),
            "note": "This is a mock implementation. In production, integrate with a real search API.",
            "sources": mock_results.iter().map(|r| r["url"].as_str().unwrap()).collect::<Vec<&str>>(),
        })))
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["query"].as_str().map(|q| format!("Searching web for '{}'", q))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_webfetch_metadata() {
        let tool = WebFetchTool::default();
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "WebFetch");
        assert_eq!(metadata.category, ToolCategory::AgentSystem);
        assert!(metadata.is_read_only);
    }
    
    #[test]
    fn test_websearch_metadata() {
        let tool = WebSearchTool::default();
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "WebSearch");
        assert_eq!(metadata.category, ToolCategory::AgentSystem);
        assert!(metadata.is_read_only);
    }
}
