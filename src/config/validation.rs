//! 配置验证模块
//!
//! 提供配置项的验证机制，确保配置的正确性和安全性。

use crate::error::ConfigError;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// 配置验证错误
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// 字段路径
    pub field: String,
    /// 错误消息
    pub message: String,
    /// 错误代码
    pub code: Option<String>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = &self.code {
            write!(f, "[{}] {}: {}", code, self.field, self.message)
        } else {
            write!(f, "{}: {}", self.field, self.message)
        }
    }
}

impl std::error::Error for ValidationError {}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 是否通过验证
    pub is_valid: bool,
    /// 验证错误列表
    pub errors: Vec<ValidationError>,
    /// 验证警告列表
    pub warnings: Vec<ValidationError>,
}

impl ValidationResult {
    /// 创建成功的验证结果
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// 添加错误
    pub fn add_error(&mut self, field: &str, message: &str, code: Option<&str>) {
        self.is_valid = false;
        self.errors.push(ValidationError {
            field: field.to_string(),
            message: message.to_string(),
            code: code.map(|s| s.to_string()),
        });
    }

    /// 添加警告
    pub fn add_warning(&mut self, field: &str, message: &str, code: Option<&str>) {
        self.warnings.push(ValidationError {
            field: field.to_string(),
            message: message.to_string(),
            code: code.map(|s| s.to_string()),
        });
    }

    /// 合并另一个验证结果
    pub fn merge(&mut self, other: ValidationResult) {
        self.is_valid = self.is_valid && other.is_valid;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }

    /// 转换为Result
    pub fn into_result(self) -> Result<()> {
        if self.is_valid {
            Ok(())
        } else {
            Err(ConfigError::ValidationFailed(self.errors))
        }
    }
}

/// 配置验证器 trait
pub trait ConfigValidator: Send + Sync {
    /// 验证配置
    fn validate(&self, value: &serde_json::Value, context: &ValidationContext) -> ValidationResult;
    
    /// 获取验证器名称
    fn name(&self) -> &'static str;
}

/// 验证上下文
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// 当前字段路径
    pub path: String,
    /// 完整配置
    pub full_config: serde_json::Value,
    /// 环境信息
    pub env: HashMap<String, String>,
}

impl ValidationContext {
    /// 创建新的验证上下文
    pub fn new(full_config: serde_json::Value) -> Self {
        Self {
            path: String::new(),
            full_config,
            env: std::env::vars().collect(),
        }
    }

    /// 嵌套字段
    pub fn nest(&self, field: &str) -> Self {
        let new_path = if self.path.is_empty() {
            field.to_string()
        } else {
            format!("{}.{}", self.path, field)
        };
        Self {
            path: new_path,
            full_config: self.full_config.clone(),
            env: self.env.clone(),
        }
    }
}

/// 必填验证器
pub struct RequiredValidator;

impl ConfigValidator for RequiredValidator {
    fn validate(&self, value: &serde_json::Value, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();
        
        if value.is_null() {
            result.add_error(
                &context.path,
                "此字段为必填项",
                Some("REQUIRED"),
            );
        }
        
        result
    }

    fn name(&self) -> &'static str {
        "required"
    }
}

/// 字符串范围验证器
pub struct StringRangeValidator {
    min_length: Option<usize>,
    max_length: Option<usize>,
}

impl StringRangeValidator {
    pub fn new(min_length: Option<usize>, max_length: Option<usize>) -> Self {
        Self {
            min_length,
            max_length,
        }
    }
}

impl ConfigValidator for StringRangeValidator {
    fn validate(&self, value: &serde_json::Value, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();
        
        if let Some(s) = value.as_str() {
            let len = s.len();
            
            if let Some(min) = self.min_length {
                if len < min {
                    result.add_error(
                        &context.path,
                        &format!("字符串长度不能少于 {} 个字符", min),
                        Some("STRING_TOO_SHORT"),
                    );
                }
            }
            
            if let Some(max) = self.max_length {
                if len > max {
                    result.add_error(
                        &context.path,
                        &format!("字符串长度不能超过 {} 个字符", max),
                        Some("STRING_TOO_LONG"),
                    );
                }
            }
        }
        
        result
    }

    fn name(&self) -> &'static str {
        "string_range"
    }
}

/// 数值范围验证器
pub struct NumberRangeValidator {
    min: Option<f64>,
    max: Option<f64>,
}

impl NumberRangeValidator {
    pub fn new(min: Option<f64>, max: Option<f64>) -> Self {
        Self { min, max }
    }
}

impl ConfigValidator for NumberRangeValidator {
    fn validate(&self, value: &serde_json::Value, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();
        
        if let Some(n) = value.as_f64() {
            if let Some(min) = self.min {
                if n < min {
                    result.add_error(
                        &context.path,
                        &format!("数值不能小于 {}", min),
                        Some("NUMBER_TOO_SMALL"),
                    );
                }
            }
            
            if let Some(max) = self.max {
                if n > max {
                    result.add_error(
                        &context.path,
                        &format!("数值不能大于 {}", max),
                        Some("NUMBER_TOO_LARGE"),
                    );
                }
            }
        }
        
        result
    }

    fn name(&self) -> &'static str {
        "number_range"
    }
}

/// 枚举值验证器
pub struct EnumValidator {
    allowed_values: Vec<String>,
}

impl EnumValidator {
    pub fn new(allowed_values: Vec<String>) -> Self {
        Self { allowed_values }
    }
}

impl ConfigValidator for EnumValidator {
    fn validate(&self, value: &serde_json::Value, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();
        
        if let Some(s) = value.as_str() {
            if !self.allowed_values.iter().any(|v| v == s) {
                result.add_error(
                    &context.path,
                    &format!(
                        "值 '{}' 不在允许的列表中: {}",
                        s,
                        self.allowed_values.join(", ")
                    ),
                    Some("INVALID_ENUM_VALUE"),
                );
            }
        }
        
        result
    }

    fn name(&self) -> &'static str {
        "enum"
    }
}

/// URL验证器
pub struct UrlValidator;

impl ConfigValidator for UrlValidator {
    fn validate(&self, value: &serde_json::Value, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();
        
        if let Some(s) = value.as_str() {
            if url::Url::parse(s).is_err() {
                result.add_error(
                    &context.path,
                    &format!("'{}' 不是一个有效的URL", s),
                    Some("INVALID_URL"),
                );
            }
        }
        
        result
    }

    fn name(&self) -> &'static str {
        "url"
    }
}

/// 路径验证器
pub struct PathValidator {
    must_exist: bool,
    must_be_dir: Option<bool>,
}

impl PathValidator {
    pub fn new(must_exist: bool, must_be_dir: Option<bool>) -> Self {
        Self {
            must_exist,
            must_be_dir,
        }
    }
}

impl ConfigValidator for PathValidator {
    fn validate(&self, value: &serde_json::Value, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();
        
        if let Some(s) = value.as_str() {
            let path = std::path::Path::new(s);
            
            if self.must_exist && !path.exists() {
                result.add_error(
                    &context.path,
                    &format!("路径 '{}' 不存在", s),
                    Some("PATH_NOT_FOUND"),
                );
                return result;
            }
            
            if let Some(must_be_dir) = self.must_be_dir {
                if path.exists() {
                    if must_be_dir && !path.is_dir() {
                        result.add_error(
                            &context.path,
                            &format!("'{}' 不是一个目录", s),
                            Some("PATH_NOT_DIRECTORY"),
                        );
                    } else if !must_be_dir && !path.is_file() {
                        result.add_error(
                            &context.path,
                            &format!("'{}' 不是一个文件", s),
                            Some("PATH_NOT_FILE"),
                        );
                    }
                }
            }
        }
        
        result
    }

    fn name(&self) -> &'static str {
        "path"
    }
}

/// 验证规则集合
#[derive(Debug, Default)]
pub struct ValidationSchema {
    rules: HashMap<String, Vec<Box<dyn ConfigValidator>>>,
}

impl ValidationSchema {
    /// 创建新的验证模式
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加验证规则
    pub fn add_rule(&mut self, field: &str, validator: Box<dyn ConfigValidator>) {
        self.rules
            .entry(field.to_string())
            .or_insert_with(Vec::new)
            .push(validator);
    }

    /// 添加必填规则
    pub fn required(&mut self, field: &str) {
        self.add_rule(field, Box::new(RequiredValidator));
    }

    /// 添加字符串范围规则
    pub fn string_range(&mut self, field: &str, min: Option<usize>, max: Option<usize>) {
        self.add_rule(field, Box::new(StringRangeValidator::new(min, max)));
    }

    /// 添加数值范围规则
    pub fn number_range(&mut self, field: &str, min: Option<f64>, max: Option<f64>) {
        self.add_rule(field, Box::new(NumberRangeValidator::new(min, max)));
    }

    /// 添加枚举规则
    pub fn enum_values(&mut self, field: &str, values: Vec<String>) {
        self.add_rule(field, Box::new(EnumValidator::new(values)));
    }

    /// 添加URL规则
    pub fn url(&mut self, field: &str) {
        self.add_rule(field, Box::new(UrlValidator));
    }

    /// 添加路径规则
    pub fn path(&mut self, field: &str, must_exist: bool, must_be_dir: Option<bool>) {
        self.add_rule(
            field,
            Box::new(PathValidator::new(must_exist, must_be_dir)),
        );
    }

    /// 验证配置
    pub fn validate(&self, config: &serde_json::Value) -> ValidationResult {
        let mut result = ValidationResult::success();
        let context = ValidationContext::new(config.clone());

        for (field, validators) in &self.rules {
            let field_context = if field.is_empty() {
                context.clone()
            } else {
                context.nest(field)
            };

            let field_value = get_value_by_path(config, field);

            for validator in validators {
                let field_result = validator.validate(&field_value, &field_context);
                result.merge(field_result);
            }
        }

        result
    }
}

/// 根据路径获取值
fn get_value_by_path(value: &serde_json::Value, path: &str) -> serde_json::Value {
    if path.is_empty() {
        return value.clone();
    }

    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        match current {
            serde_json::Value::Object(map) => {
                if let Some(v) = map.get(part) {
                    current = v;
                } else {
                    return serde_json::Value::Null;
                }
            }
            serde_json::Value::Array(arr) => {
                if let Ok(index) = part.parse::<usize>() {
                    if let Some(v) = arr.get(index) {
                        current = v;
                    } else {
                        return serde_json::Value::Null;
                    }
                } else {
                    return serde_json::Value::Null;
                }
            }
            _ => return serde_json::Value::Null,
        }
    }

    current.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_required_validator() {
        let validator = RequiredValidator;
        let context = ValidationContext::new(json!({}));

        let result = validator.validate(&json!(null), &context);
        assert!(!result.is_valid);

        let result = validator.validate(&json!("value"), &context);
        assert!(result.is_valid);
    }

    #[test]
    fn test_string_range_validator() {
        let validator = StringRangeValidator::new(Some(3), Some(10));
        let context = ValidationContext::new(json!({}));

        let result = validator.validate(&json!("ab"), &context);
        assert!(!result.is_valid);

        let result = validator.validate(&json!("abcde"), &context);
        assert!(result.is_valid);

        let result = validator.validate(&json!("abcdefghijk"), &context);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_number_range_validator() {
        let validator = NumberRangeValidator::new(Some(1.0), Some(10.0));
        let context = ValidationContext::new(json!({}));

        let result = validator.validate(&json!(0.5), &context);
        assert!(!result.is_valid);

        let result = validator.validate(&json!(5.0), &context);
        assert!(result.is_valid);

        let result = validator.validate(&json!(11.0), &context);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_enum_validator() {
        let validator = EnumValidator::new(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        let context = ValidationContext::new(json!({}));

        let result = validator.validate(&json!("a"), &context);
        assert!(result.is_valid);

        let result = validator.validate(&json!("d"), &context);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_url_validator() {
        let validator = UrlValidator;
        let context = ValidationContext::new(json!({}));

        let result = validator.validate(&json!("https://example.com"), &context);
        assert!(result.is_valid);

        let result = validator.validate(&json!("not a url"), &context);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validation_schema() {
        let mut schema = ValidationSchema::new();
        schema.required("name");
        schema.string_range("name", Some(2), Some(50));
        schema.number_range("age", Some(0.0), Some(150.0));

        let config = json!({
            "name": "Test User",
            "age": 25
        });

        let result = schema.validate(&config);
        assert!(result.is_valid);

        let config = json!({
            "name": null,
            "age": 200
        });

        let result = schema.validate(&config);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 2);
    }
}
