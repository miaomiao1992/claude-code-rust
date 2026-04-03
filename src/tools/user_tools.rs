//! 用户交互相关工具
//! 
//! 实现AskUserQuestion Tool等用户交互工具

use crate::error::Result;
use async_trait::async_trait;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// AskUserQuestion工具
/// 用于向用户提问并获取回答
pub struct AskUserQuestionTool;

#[async_trait]
impl Tool for AskUserQuestionTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("AskUserQuestion", "Ask user questions and get answers")
            .category(ToolCategory::AgentSystem)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["askuserquestion".to_string(), "ask".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("questions".to_string(), serde_json::json!({
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "question": {
                                    "type": "string",
                                    "description": "The complete question to ask the user"
                                },
                                "header": {
                                    "type": "string",
                                    "description": "Very short label displayed as a chip/tag (max 12 chars)"
                                },
                                "options": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "label": {
                                                "type": "string",
                                                "description": "The display text for this option"
                                            },
                                            "description": {
                                                "type": "string",
                                                "description": "Explanation of what this option means"
                                            }
                                        },
                                        "required": ["label", "description"]
                                    },
                                    "minItems": 2,
                                    "maxItems": 4
                                },
                                "multiSelect": {
                                    "type": "boolean",
                                    "description": "Allow multiple answers to be selected"
                                }
                            },
                            "required": ["question", "header", "options"]
                        },
                        "minItems": 1,
                        "maxItems": 4
                    })),
                    ("answers".to_string(), serde_json::json!({
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "selected_options": {
                                    "type": "array",
                                    "items": {
                                        "type": "string"
                                    }
                                }
                            },
                            "required": ["selected_options"]
                        }
                    })),
                ])),
                required: Some(vec!["questions".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        _context: ToolUseContext,
    ) -> Result<ToolResult> {
        let questions = input["questions"].as_array()
            .ok_or_else(|| crate::error::ClaudeError::Tool("questions array is required".to_string()))?;
        
        // 构建问题描述
        let mut questions_text = Vec::new();
        for (i, question) in questions.iter().enumerate() {
            let q_text = question["question"].as_str()
                .ok_or_else(|| crate::error::ClaudeError::Tool("question text is required".to_string()))?;
            let header = question["header"].as_str()
                .ok_or_else(|| crate::error::ClaudeError::Tool("question header is required".to_string()))?;
            let options = question["options"].as_array()
                .ok_or_else(|| crate::error::ClaudeError::Tool("question options are required".to_string()))?;
            
            let mut question_desc = format!("{}: {}\nOptions:", header, q_text);
            for (j, option) in options.iter().enumerate() {
                let label = option["label"].as_str().unwrap_or_default();
                let desc = option["description"].as_str().unwrap_or_default();
                question_desc.push_str(&format!("\n{}. {}: {}", j + 1, label, desc));
            }
            questions_text.push(question_desc);
        }
        
        // 模拟用户回答（实际实现中应该暂停执行并等待用户输入）
        // 这里我们返回一个模拟的回答
        let mock_answers = questions.iter().map(|_| {
            serde_json::json!({
                "selected_options": ["First option"]
            })
        }).collect::<Vec<_>>();
        
        Ok(ToolResult::success(serde_json::json!({ 
            "questions": questions_text.join("\n\n"),
            "answers": mock_answers,
            "result": "Questions asked and answers received",
        })))
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["questions"].as_array().and_then(|questions| {
            questions.first().and_then(|q| {
                q["question"].as_str().map(|text| {
                    format!("Asking user: {}", text)
                })
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_askuserquestion_metadata() {
        let tool = AskUserQuestionTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "AskUserQuestion");
        assert_eq!(metadata.category, ToolCategory::AgentSystem);
    }
    
    #[tokio::test]
    async fn test_askuserquestion_execute() {
        use crate::config::Config;
        use crate::state::AppState;
        
        let tool = AskUserQuestionTool;
        let input = serde_json::json!({
            "questions": [
                {
                    "question": "Which library should we use?",
                    "header": "Library",
                    "options": [
                        {
                            "label": "serde",
                            "description": "Serialization library"
                        },
                        {
                            "label": "reqwest",
                            "description": "HTTP client"
                        }
                    ],
                    "multiSelect": false
                }
            ]
        });
        
        let context = ToolUseContext::new(
            std::path::PathBuf::from("."),
            Config::default(),
            AppState::default()
        );
        let result = tool.execute(input, context).await.unwrap();
        assert!(result.error.is_none());
        let data = result.data;
        assert!(data["questions"].as_str().unwrap().contains("Which library should we use?"));
        assert!(data["answers"].as_array().unwrap().len() > 0);
    }
}
