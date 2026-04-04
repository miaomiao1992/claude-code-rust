//! 工具权限系统
//!
//! 这个模块实现了工具权限检查机制，支持API工具调用权限检查

use crate::types::{PermissionMode, PermissionResult, ToolPermissionContext, ToolPermissionRule};
use serde_json::Value;

/// 权限检查器
pub struct PermissionChecker;

impl PermissionChecker {
    /// 检查工具权限
    pub fn check(
        tool_name: &str,
        input: &Value,
        context: &ToolPermissionContext,
    ) -> PermissionResult {
        // 简化实现：根据模式决定
        match context.mode {
            PermissionMode::Default => {
                // 默认模式：检查规则
                Self::check_rules(tool_name, input, context)
            }
            PermissionMode::Bypass => PermissionResult::allow(),
            PermissionMode::Plan => PermissionResult::allow(),
        }
    }

    /// 检查规则
    fn check_rules(
        tool_name: &str,
        _input: &Value,
        context: &ToolPermissionContext,
    ) -> PermissionResult {
        // 检查总是允许规则
        for rules in context.always_allow_rules.values() {
            for rule in rules {
                if Self::matches_tool_name(&rule.name, tool_name) {
                    return PermissionResult::allow();
                }
            }
        }

        // 检查总是拒绝规则
        for rules in context.always_deny_rules.values() {
            for rule in rules {
                if Self::matches_tool_name(&rule.name, tool_name) {
                    return PermissionResult::deny(format!("Tool {} is denied by rule", tool_name));
                }
            }
        }

        // 检查总是询问规则
        for rules in context.always_ask_rules.values() {
            for rule in rules {
                if Self::matches_tool_name(&rule.name, tool_name) {
                    return PermissionResult::ask();
                }
            }
        }

        // 默认允许
        PermissionResult::allow()
    }

    /// 检查工具名称是否匹配
    fn matches_tool_name(rule_name: &str, tool_name: &str) -> bool {
        // 完全匹配
        if rule_name == tool_name {
            return true;
        }

        // 支持通配符匹配
        if rule_name.contains('*') {
            let pattern = rule_name.replace('*', ".*");
            if let Ok(regex) = regex::Regex::new(&pattern) {
                if regex.is_match(tool_name) {
                    return true;
                }
            }
        }

        // 支持 MCP 格式: mcp__server__tool
        if rule_name.starts_with("mcp__") {
            let rule_parts: Vec<&str> = rule_name.split("__").collect();
            let tool_parts: Vec<&str> = tool_name.split("__").collect();

            // 如果规则是 mcp__server，匹配该服务器的所有工具
            if rule_parts.len() == 2 && tool_parts.len() >= 2 {
                return rule_parts[1] == tool_parts[1];
            }

            // 如果规则是 mcp__server__tool，完全匹配
            if rule_parts.len() >= 3 && tool_parts.len() >= 3 {
                return rule_parts[1] == tool_parts[1] && rule_parts[2] == tool_parts[2];
            }
        }

        false
    }

    /// 添加允许规则
    pub fn add_allow_rule(
        context: &mut ToolPermissionContext,
        source: impl Into<String>,
        rule: ToolPermissionRule,
    ) {
        let source = source.into();
        context.always_allow_rules
            .entry(source)
            .or_default()
            .push(rule);
    }

    /// 添加拒绝规则
    pub fn add_deny_rule(
        context: &mut ToolPermissionContext,
        source: impl Into<String>,
        rule: ToolPermissionRule,
    ) {
        let source = source.into();
        context.always_deny_rules
            .entry(source)
            .or_default()
            .push(rule);
    }

    /// 添加询问规则
    pub fn add_ask_rule(
        context: &mut ToolPermissionContext,
        source: impl Into<String>,
        rule: ToolPermissionRule,
    ) {
        let source = source.into();
        context.always_ask_rules
            .entry(source)
            .or_default()
            .push(rule);
    }

    /// 创建简单的允许规则
    pub fn allow_tool(tool_name: impl Into<String>) -> ToolPermissionRule {
        ToolPermissionRule {
            name: tool_name.into(),
            content: None,
        }
    }

    /// 创建带模式的允许规则
    pub fn allow_tool_pattern(tool_name: impl Into<String>, pattern: impl Into<String>) -> ToolPermissionRule {
        ToolPermissionRule {
            name: tool_name.into(),
            content: Some(pattern.into()),
        }
    }

    /// 创建拒绝规则
    pub fn deny_tool(tool_name: impl Into<String>) -> ToolPermissionRule {
        ToolPermissionRule {
            name: tool_name.into(),
            content: None,
        }
    }
}

/// 模式检查器
pub struct ModeChecker;

impl ModeChecker {
    /// 检查是否允许在当前模式下执行
    pub fn check_mode(mode: PermissionMode, _context: &ToolPermissionContext) -> bool {
        match mode {
            PermissionMode::Default => true,
            PermissionMode::Bypass => true,
            PermissionMode::Plan => true,
        }
    }

    /// 检查是否可以绕过权限
    pub fn can_bypass(context: &ToolPermissionContext) -> bool {
        context.is_bypass_permissions_mode_available
    }

    /// 检查是否应该自动允许
    pub fn should_auto_allow(context: &ToolPermissionContext) -> bool {
        matches!(context.mode, PermissionMode::Bypass | PermissionMode::Plan)
    }

    /// 检查是否应该询问用户
    pub fn should_ask(context: &ToolPermissionContext) -> bool {
        matches!(context.mode, PermissionMode::Default)
    }
}