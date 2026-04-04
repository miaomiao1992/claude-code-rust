//! Ultrareview Analysis 模块
//! 
//! 实现代码分析功能

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 分析级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisLevel {
    /// 基础分析
    Basic,
    /// 深度分析
    Deep,
    /// 全面分析
    Comprehensive,
}

/// 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// 分析ID
    pub analysis_id: String,
    /// 分析级别
    pub level: AnalysisLevel,
    /// 分析的文件
    pub files: Vec<String>,
    /// 发现的问题
    pub issues: Vec<Issue>,
    /// 代码质量评分
    pub quality_score: u8,
    /// 分析时间（毫秒）
    pub analysis_time_ms: u64,
    /// 分析时间戳
    pub timestamp: String,
    /// 分析摘要
    pub summary: String,
}

/// 问题级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IssueLevel {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重
    Critical,
}

/// 问题类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueType {
    /// 代码风格
    Style,
    /// 性能
    Performance,
    /// 安全
    Security,
    /// 内存
    Memory,
    /// 错误处理
    ErrorHandling,
    /// 并发
    Concurrency,
    /// 代码质量
    Quality,
    /// 其他
    Other,
}

/// 问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// 问题ID
    pub issue_id: String,
    /// 问题级别
    pub level: IssueLevel,
    /// 问题类型
    pub issue_type: IssueType,
    /// 问题描述
    pub description: String,
    /// 代码位置
    pub location: CodeLocation,
    /// 修复建议
    pub suggestion: String,
    /// 相关代码
    pub code_snippet: String,
    /// 严重程度（1-10）
    pub severity: u8,
}

/// 代码位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    /// 文件路径
    pub file_path: String,
    /// 开始行
    pub start_line: usize,
    /// 结束行
    pub end_line: usize,
    /// 开始列
    pub start_column: usize,
    /// 结束列
    pub end_column: usize,
}

/// 代码分析器
pub struct CodeAnalyzer {
    /// 分析级别
    level: AnalysisLevel,
    /// 分析规则
    rules: Vec<Box<dyn AnalysisRule>>,
    /// 分析历史
    analysis_history: Arc<RwLock<Vec<AnalysisResult>>>,
}

impl CodeAnalyzer {
    /// 创建新的代码分析器
    pub fn new(level: AnalysisLevel) -> Self {
        let mut rules = Vec::new();
        
        // 添加默认分析规则
        rules.push(Box::new(StyleRule));
        rules.push(Box::new(PerformanceRule));
        rules.push(Box::new(SecurityRule));
        rules.push(Box::new(ErrorHandlingRule));
        
        Self {
            level,
            rules,
            analysis_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// 分析文件
    pub async fn analyze_file(&self, file_path: &str) -> Result<AnalysisResult, String> {
        if !Path::new(file_path).exists() {
            return Err("File not found".to_string());
        }
        
        let start_time = std::time::Instant::now();
        let mut issues = Vec::new();
        
        // 读取文件内容
        let content = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        
        // 应用分析规则
        for rule in &self.rules {
            let rule_issues = rule.analyze(file_path, &content, self.level);
            issues.extend(rule_issues);
        }
        
        // 计算代码质量评分
        let quality_score = self.calculate_quality_score(&issues);
        
        // 生成分析结果
        let analysis_id = generate_analysis_id();
        let analysis_time = start_time.elapsed().as_millis() as u64;
        
        let result = AnalysisResult {
            analysis_id: analysis_id.clone(),
            level: self.level,
            files: vec![file_path.to_string()],
            issues,
            quality_score,
            analysis_time_ms: analysis_time,
            timestamp: chrono::Utc::now().to_rfc3339(),
            summary: self.generate_summary(quality_score),
        };
        
        // 保存到历史记录
        let mut history = self.analysis_history.write().await;
        history.push(result.clone());
        
        Ok(result)
    }
    
    /// 分析目录
    pub async fn analyze_directory(&self, directory: &str) -> Result<AnalysisResult, String> {
        if !Path::new(directory).exists() || !Path::new(directory).is_dir() {
            return Err("Directory not found".to_string());
        }
        
        let start_time = std::time::Instant::now();
        let mut all_issues = Vec::new();
        let mut files = Vec::new();
        
        // 遍历目录中的文件
        for entry in walkdir::WalkDir::new(directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path().to_string_lossy().to_string();
            
            // 只分析代码文件
            if is_code_file(&file_path) {
                files.push(file_path.clone());
                
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    for rule in &self.rules {
                        let rule_issues = rule.analyze(&file_path, &content, self.level);
                        all_issues.extend(rule_issues);
                    }
                }
            }
        }
        
        // 计算代码质量评分
        let quality_score = self.calculate_quality_score(&all_issues);
        
        // 生成分析结果
        let analysis_id = generate_analysis_id();
        let analysis_time = start_time.elapsed().as_millis() as u64;
        
        let result = AnalysisResult {
            analysis_id: analysis_id.clone(),
            level: self.level,
            files,
            issues: all_issues,
            quality_score,
            analysis_time_ms: analysis_time,
            timestamp: chrono::Utc::now().to_rfc3339(),
            summary: self.generate_summary(quality_score),
        };
        
        // 保存到历史记录
        let mut history = self.analysis_history.write().await;
        history.push(result.clone());
        
        Ok(result)
    }
    
    /// 计算代码质量评分
    fn calculate_quality_score(&self, issues: &[Issue]) -> u8 {
        if issues.is_empty() {
            return 100;
        }
        
        let total_severity: u32 = issues
            .iter()
            .map(|issue| issue.severity as u32)
            .sum();
        
        let avg_severity = total_severity as f64 / issues.len() as f64;
        let score = 100.0 - (avg_severity * 5.0);
        
        score.clamp(0.0, 100.0) as u8
    }
    
    /// 生成分析摘要
    fn generate_summary(&self, quality_score: u8) -> String {
        match quality_score {
            90..=100 => "代码质量优秀，几乎没有问题".to_string(),
            70..=89 => "代码质量良好，有少量需要改进的地方".to_string(),
            50..=69 => "代码质量一般，有一些需要修复的问题".to_string(),
            30..=49 => "代码质量较差，有多个严重问题需要修复".to_string(),
            _ => "代码质量差，存在大量严重问题".to_string(),
        }
    }
    
    /// 获取分析历史
    pub async fn get_analysis_history(&self) -> Vec<AnalysisResult> {
        let history = self.analysis_history.read().await;
        history.clone()
    }
}

/// 分析规则
pub trait AnalysisRule {
    /// 分析代码
    fn analyze(&self, file_path: &str, content: &str, level: AnalysisLevel) -> Vec<Issue>;
}

/// 风格规则
struct StyleRule;

impl AnalysisRule for StyleRule {
    fn analyze(&self, file_path: &str, content: &str, level: AnalysisLevel) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        // 检查缩进
        for (line_num, line) in content.lines().enumerate() {
            if line.starts_with(" ") && !line.starts_with("    ") && !line.trim().is_empty() {
                issues.push(Issue {
                    issue_id: format!("style_{}", generate_id()),
                    level: IssueLevel::Info,
                    issue_type: IssueType::Style,
                    description: "缩进使用了空格而不是制表符".to_string(),
                    location: CodeLocation {
                        file_path: file_path.to_string(),
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        start_column: 0,
                        end_column: line.len(),
                    },
                    suggestion: "使用制表符进行缩进".to_string(),
                    code_snippet: line.to_string(),
                    severity: 1,
                });
            }
        }
        
        issues
    }
}

/// 性能规则
struct PerformanceRule;

impl AnalysisRule for PerformanceRule {
    fn analyze(&self, file_path: &str, content: &str, level: AnalysisLevel) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        // 检查循环中的字符串连接
        if content.contains("for") && content.contains("+") {
            issues.push(Issue {
                issue_id: format!("perf_{}", generate_id()),
                level: IssueLevel::Warning,
                issue_type: IssueType::Performance,
                description: "循环中可能存在字符串连接性能问题".to_string(),
                location: CodeLocation {
                    file_path: file_path.to_string(),
                    start_line: 1,
                    end_line: content.lines().count(),
                    start_column: 0,
                    end_column: 0,
                },
                suggestion: "使用String::with_capacity或StringBuilder".to_string(),
                code_snippet: "循环中字符串连接".to_string(),
                severity: 3,
            });
        }
        
        issues
    }
}

/// 安全规则
struct SecurityRule;

impl AnalysisRule for SecurityRule {
    fn analyze(&self, file_path: &str, content: &str, level: AnalysisLevel) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        // 检查硬编码的密码
        if content.contains("password") || content.contains("secret") {
            issues.push(Issue {
                issue_id: format!("sec_{}", generate_id()),
                level: IssueLevel::Critical,
                issue_type: IssueType::Security,
                description: "可能存在硬编码的敏感信息".to_string(),
                location: CodeLocation {
                    file_path: file_path.to_string(),
                    start_line: 1,
                    end_line: content.lines().count(),
                    start_column: 0,
                    end_column: 0,
                },
                suggestion: "使用环境变量或配置文件存储敏感信息".to_string(),
                code_snippet: "包含敏感信息的代码".to_string(),
                severity: 8,
            });
        }
        
        issues
    }
}

/// 错误处理规则
struct ErrorHandlingRule;

impl AnalysisRule for ErrorHandlingRule {
    fn analyze(&self, file_path: &str, content: &str, level: AnalysisLevel) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        // 检查未处理的错误
        if content.contains("unwrap()") || content.contains("expect(") {
            issues.push(Issue {
                issue_id: format!("err_{}", generate_id()),
                level: IssueLevel::Warning,
                issue_type: IssueType::ErrorHandling,
                description: "可能存在未处理的错误".to_string(),
                location: CodeLocation {
                    file_path: file_path.to_string(),
                    start_line: 1,
                    end_line: content.lines().count(),
                    start_column: 0,
                    end_column: 0,
                },
                suggestion: "使用proper错误处理".to_string(),
                code_snippet: "包含unwrap()或expect()的代码".to_string(),
                severity: 4,
            });
        }
        
        issues
    }
}

/// 生成分析ID
fn generate_analysis_id() -> String {
    format!("analysis_{}_{}", 
        chrono::Utc::now().timestamp(),
        generate_id()
    )
}

/// 生成ID
fn generate_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen::<u32>().to_string()
}

/// 检查是否是代码文件
fn is_code_file(file_path: &str) -> bool {
    let extensions = [
        ".rs", ".py", ".js", ".ts", ".jsx", ".tsx", 
        ".java", ".c", ".cpp", ".h", ".hpp", ".go",
        ".php", ".ruby", ".swift", ".kt", ".scala"
    ];
    
    extensions.iter().any(|ext| file_path.ends_with(ext))
}

use chrono;
use rand;
use walkdir;
