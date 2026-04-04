//! Ultrareview Report 模块
//! 
//! 实现报告生成功能

use super::analysis::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// 报告类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    /// 摘要报告
    Summary,
    /// 详细报告
    Detailed,
    /// 问题报告
    Issues,
    /// 趋势报告
    Trends,
}

/// 报告格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    /// 文本格式
    Text,
    /// JSON格式
    Json,
    /// HTML格式
    Html,
    /// Markdown格式
    Markdown,
}

/// 报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// 报告ID
    pub report_id: String,
    /// 报告类型
    pub report_type: ReportType,
    /// 报告格式
    pub format: ReportFormat,
    /// 分析ID
    pub analysis_id: String,
    /// 报告标题
    pub title: String,
    /// 报告内容
    pub content: String,
    /// 生成时间
    pub generated_at: String,
    /// 报告元数据
    pub metadata: serde_json::Value,
}

/// 报告生成器
pub struct ReportGenerator {
    /// 分析结果
    analysis_result: AnalysisResult,
}

impl ReportGenerator {
    /// 创建新的报告生成器
    pub fn new(analysis_result: AnalysisResult) -> Self {
        Self {
            analysis_result,
        }
    }
    
    /// 生成报告
    pub fn generate(&self, report_type: ReportType, format: ReportFormat) -> Result<Report, String> {
        let report_id = generate_report_id();
        let title = self.generate_title(report_type);
        let content = self.generate_content(report_type, format);
        let metadata = self.generate_metadata(report_type, format);
        
        Ok(Report {
            report_id,
            report_type,
            format,
            analysis_id: self.analysis_result.analysis_id.clone(),
            title,
            content,
            generated_at: chrono::Utc::now().to_rfc3339(),
            metadata,
        })
    }
    
    /// 生成报告标题
    fn generate_title(&self, report_type: ReportType) -> String {
        match report_type {
            ReportType::Summary => "代码分析摘要报告".to_string(),
            ReportType::Detailed => "代码分析详细报告".to_string(),
            ReportType::Issues => "代码问题报告".to_string(),
            ReportType::Trends => "代码质量趋势报告".to_string(),
        }
    }
    
    /// 生成报告内容
    fn generate_content(&self, report_type: ReportType, format: ReportFormat) -> String {
        match format {
            ReportFormat::Text => self.generate_text_content(report_type),
            ReportFormat::Json => self.generate_json_content(report_type),
            ReportFormat::Html => self.generate_html_content(report_type),
            ReportFormat::Markdown => self.generate_markdown_content(report_type),
        }
    }
    
    /// 生成文本格式内容
    fn generate_text_content(&self, report_type: ReportType) -> String {
        let mut content = String::new();
        
        match report_type {
            ReportType::Summary => {
                content.push_str(&format!("# 代码分析摘要报告\n"));
                content.push_str(&format!("分析ID: {}\n", self.analysis_result.analysis_id));
                content.push_str(&format!("分析时间: {}\n", self.analysis_result.timestamp));
                content.push_str(&format!("代码质量评分: {}/100\n", self.analysis_result.quality_score));
                content.push_str(&format!("分析文件数: {}\n", self.analysis_result.files.len()));
                content.push_str(&format!("发现问题数: {}\n", self.analysis_result.issues.len()));
                content.push_str(&format!("分析时间: {}ms\n", self.analysis_result.analysis_time_ms));
                content.push_str(&format!("\n## 摘要\n{}\n", self.analysis_result.summary));
            }
            ReportType::Detailed => {
                content.push_str(&format!("# 代码分析详细报告\n"));
                content.push_str(&format!("分析ID: {}\n", self.analysis_result.analysis_id));
                content.push_str(&format!("分析时间: {}\n", self.analysis_result.timestamp));
                content.push_str(&format!("代码质量评分: {}/100\n", self.analysis_result.quality_score));
                content.push_str(&format!("分析文件数: {}\n", self.analysis_result.files.len()));
                content.push_str(&format!("发现问题数: {}\n", self.analysis_result.issues.len()));
                content.push_str(&format!("分析时间: {}ms\n", self.analysis_result.analysis_time_ms));
                content.push_str(&format!("\n## 分析的文件\n"));
                for file in &self.analysis_result.files {
                    content.push_str(&format!("- {}\n", file));
                }
                content.push_str(&format!("\n## 发现的问题\n"));
                for issue in &self.analysis_result.issues {
                    content.push_str(&format!("[{}] {}: {}\n", 
                        self.level_to_string(issue.level),
                        self.type_to_string(issue.issue_type),
                        issue.description
                    ));
                    if let Some(location) = &issue.location {
                        content.push_str(&format!("  位置: {}:{}:{}\n", 
                            location.file_path,
                            location.start_line,
                            location.start_column
                        ));
                    }
                    content.push_str(&format!("  建议: {}\n\n", issue.suggestion));
                }
                content.push_str(&format!("\n## 摘要\n{}\n", self.analysis_result.summary));
            }
            ReportType::Issues => {
                content.push_str(&format!("# 代码问题报告\n"));
                content.push_str(&format!("分析ID: {}\n", self.analysis_result.analysis_id));
                content.push_str(&format!("分析时间: {}\n", self.analysis_result.timestamp));
                content.push_str(&format!("总问题数: {}\n\n", self.analysis_result.issues.len()));
                
                // 按级别分组
                content.push_str(&format!("## 问题级别分布\n"));
                let level_counts = self.count_issues_by_level();
                for (level, count) in level_counts {
                    content.push_str(&format!("- {}: {}\n", self.level_to_string(level), count));
                }
                
                // 按类型分组
                content.push_str(&format!("\n## 问题类型分布\n"));
                let type_counts = self.count_issues_by_type();
                for (issue_type, count) in type_counts {
                    content.push_str(&format!("- {}: {}\n", self.type_to_string(issue_type), count));
                }
                
                // 详细问题列表
                content.push_str(&format!("\n## 详细问题列表\n"));
                for issue in &self.analysis_result.issues {
                    content.push_str(&format!("[{}] {}: {}\n", 
                        self.level_to_string(issue.level),
                        self.type_to_string(issue.issue_type),
                        issue.description
                    ));
                    if let Some(location) = &issue.location {
                        content.push_str(&format!("  位置: {}:{}:{}\n", 
                            location.file_path,
                            location.start_line,
                            location.start_column
                        ));
                    }
                    content.push_str(&format!("  建议: {}\n\n", issue.suggestion));
                }
            }
            ReportType::Trends => {
                content.push_str(&format!("# 代码质量趋势报告\n"));
                content.push_str(&format!("分析ID: {}\n", self.analysis_result.analysis_id));
                content.push_str(&format!("分析时间: {}\n", self.analysis_result.timestamp));
                content.push_str(&format!("代码质量评分: {}/100\n\n", self.analysis_result.quality_score));
                content.push_str(&format!("## 质量趋势分析\n"));
                content.push_str(&format!("由于这是单次分析，无法提供趋势数据。\n"));
                content.push_str(&format!("建议定期运行分析以跟踪代码质量变化。\n"));
                content.push_str(&format!("\n## 当前质量状态\n"));
                content.push_str(&format!("- 代码质量评分: {}/100\n", self.analysis_result.quality_score));
                content.push_str(&format!("- 问题总数: {}\n", self.analysis_result.issues.len()));
                content.push_str(&format!("- 分析文件数: {}\n", self.analysis_result.files.len()));
            }
        }
        
        content
    }
    
    /// 生成JSON格式内容
    fn generate_json_content(&self, report_type: ReportType) -> String {
        let report = serde_json::json!({
            "report_id": generate_report_id(),
            "report_type": format!("{:?}", report_type),
            "analysis_id": self.analysis_result.analysis_id,
            "title": self.generate_title(report_type),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "quality_score": self.analysis_result.quality_score,
            "files_analyzed": self.analysis_result.files,
            "issues": self.analysis_result.issues,
            "analysis_time_ms": self.analysis_result.analysis_time_ms,
            "summary": self.analysis_result.summary,
            "metadata": {
                "level_counts": self.count_issues_by_level(),
                "type_counts": self.count_issues_by_type(),
            }
        });
        
        serde_json::to_string_pretty(&report).unwrap()
    }
    
    /// 生成HTML格式内容
    fn generate_html_content(&self, report_type: ReportType) -> String {
        let mut content = String::new();
        content.push_str(&format!("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n"));
        content.push_str(&format!("  <meta charset=\"UTF-8\">\n"));
        content.push_str(&format!("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n"));
        content.push_str(&format!("  <title>{}</title>\n", self.generate_title(report_type)));
        content.push_str(&format!("  <style>\n"));
        content.push_str(&format!("    body {{ font-family: Arial, sans-serif; margin: 20px; }} \n"));
        content.push_str(&format!("    h1, h2 {{ color: #333; }} \n"));
        content.push_str(&format!("    .info {{ background-color: #f0f8ff; padding: 10px; border-radius: 5px; margin: 10px 0; }} \n"));
        content.push_str(&format!("    .warning {{ background-color: #fff3cd; padding: 10px; border-radius: 5px; margin: 10px 0; }} \n"));
        content.push_str(&format!("    .error {{ background-color: #f8d7da; padding: 10px; border-radius: 5px; margin: 10px 0; }} \n"));
        content.push_str(&format!("    .critical {{ background-color: #dc3545; color: white; padding: 10px; border-radius: 5px; margin: 10px 0; }} \n"));
        content.push_str(&format!("    table {{ border-collapse: collapse; width: 100%; margin: 10px 0; }} \n"));
        content.push_str(&format!("    th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }} \n"));
        content.push_str(&format!("    th {{ background-color: #f2f2f2; }} \n"));
        content.push_str(&format!("  </style>\n"));
        content.push_str(&format!("</head>\n<body>\n"));
        
        content.push_str(&format!("<h1>{}</h1>\n", self.generate_title(report_type)));
        content.push_str(&format!("<div class=\"info\">\n"));
        content.push_str(&format!("  <p><strong>分析ID:</strong> {}</p>\n", self.analysis_result.analysis_id));
        content.push_str(&format!("  <p><strong>分析时间:</strong> {}</p>\n", self.analysis_result.timestamp));
        content.push_str(&format!("  <p><strong>代码质量评分:</strong> {}/100</p>\n", self.analysis_result.quality_score));
        content.push_str(&format!("  <p><strong>分析文件数:</strong> {}</p>\n", self.analysis_result.files.len()));
        content.push_str(&format!("  <p><strong>发现问题数:</strong> {}</p>\n", self.analysis_result.issues.len()));
        content.push_str(&format!("  <p><strong>分析时间:</strong> {}ms</p>\n", self.analysis_result.analysis_time_ms));
        content.push_str(&format!("</div>\n"));
        
        if report_type == ReportType::Summary || report_type == ReportType::Detailed {
            content.push_str(&format!("<h2>摘要</h2>\n"));
            content.push_str(&format!("<p>{}</p>\n", self.analysis_result.summary));
        }
        
        if report_type == ReportType::Detailed {
            content.push_str(&format!("<h2>分析的文件</h2>\n"));
            content.push_str(&format!("<ul>\n"));
            for file in &self.analysis_result.files {
                content.push_str(&format!("  <li>{}</li>\n", file));
            }
            content.push_str(&format!("</ul>\n"));
        }
        
        if report_type == ReportType::Detailed || report_type == ReportType::Issues {
            content.push_str(&format!("<h2>发现的问题</h2>\n"));
            for issue in &self.analysis_result.issues {
                let class = match issue.level {
                    IssueLevel::Info => "info",
                    IssueLevel::Warning => "warning",
                    IssueLevel::Error => "error",
                    IssueLevel::Critical => "critical",
                };
                content.push_str(&format!("<div class=\"{}">\n", class));
                content.push_str(&format!("  <p><strong>[{}] {}:</strong> {}</p>\n", 
                    self.level_to_string(issue.level),
                    self.type_to_string(issue.issue_type),
                    issue.description
                ));
                if let Some(location) = &issue.location {
                    content.push_str(&format!("  <p><strong>位置:</strong> {}:{}:{}</p>\n", 
                        location.file_path,
                        location.start_line,
                        location.start_column
                    ));
                }
                content.push_str(&format!("  <p><strong>建议:</strong> {}</p>\n", issue.suggestion));
                content.push_str(&format!("</div>\n"));
            }
        }
        
        content.push_str(&format!("</body>\n</html>\n"));
        content
    }
    
    /// 生成Markdown格式内容
    fn generate_markdown_content(&self, report_type: ReportType) -> String {
        let mut content = String::new();
        
        content.push_str(&format!("# {}\n\n", self.generate_title(report_type)));
        content.push_str(&format!("## 分析信息\n\n"));
        content.push_str(&format!("| 项目 | 值 |\n"));
        content.push_str(&format!("| --- | --- |\n"));
        content.push_str(&format!("| 分析ID | {} |\n", self.analysis_result.analysis_id));
        content.push_str(&format!("| 分析时间 | {} |\n", self.analysis_result.timestamp));
        content.push_str(&format!("| 代码质量评分 | {}/100 |\n", self.analysis_result.quality_score));
        content.push_str(&format!("| 分析文件数 | {} |\n", self.analysis_result.files.len()));
        content.push_str(&format!("| 发现问题数 | {} |\n", self.analysis_result.issues.len()));
        content.push_str(&format!("| 分析时间 | {}ms |\n\n", self.analysis_result.analysis_time_ms));
        
        if report_type == ReportType::Summary || report_type == ReportType::Detailed {
            content.push_str(&format!("## 摘要\n\n{}\n\n", self.analysis_result.summary));
        }
        
        if report_type == ReportType::Detailed {
            content.push_str(&format!("## 分析的文件\n\n"));
            for file in &self.analysis_result.files {
                content.push_str(&format!("- {}\n", file));
            }
            content.push_str(&format!("\n"));
        }
        
        if report_type == ReportType::Detailed || report_type == ReportType::Issues {
            content.push_str(&format!("## 发现的问题\n\n"));
            for issue in &self.analysis_result.issues {
                content.push_str(&format!("### [{}] {}\n\n", 
                    self.level_to_string(issue.level),
                    self.type_to_string(issue.issue_type)
                ));
                content.push_str(&format!("**描述:** {}\n\n", issue.description));
                if let Some(location) = &issue.location {
                    content.push_str(&format!("**位置:** {}:{}:{}\n\n", 
                        location.file_path,
                        location.start_line,
                        location.start_column
                    ));
                }
                content.push_str(&format!("**建议:** {}\n\n", issue.suggestion));
            }
        }
        
        content
    }
    
    /// 生成报告元数据
    fn generate_metadata(&self, report_type: ReportType, format: ReportFormat) -> serde_json::Value {
        serde_json::json!({
            "report_type": format!("{:?}", report_type),
            "format": format!("{:?}", format),
            "analysis_level": format!("{:?}", self.analysis_result.level),
            "issue_counts": self.count_issues_by_type(),
            "level_counts": self.count_issues_by_level(),
            "files_analyzed": self.analysis_result.files.len(),
        })
    }
    
    /// 按类型统计问题
    fn count_issues_by_type(&self) -> std::collections::HashMap<IssueType, usize> {
        let mut counts = std::collections::HashMap::new();
        for issue in &self.analysis_result.issues {
            *counts.entry(issue.issue_type).or_insert(0) += 1;
        }
        counts
    }
    
    /// 按级别统计问题
    fn count_issues_by_level(&self) -> std::collections::HashMap<IssueLevel, usize> {
        let mut counts = std::collections::HashMap::new();
        for issue in &self.analysis_result.issues {
            *counts.entry(issue.level).or_insert(0) += 1;
        }
        counts
    }
    
    /// 将级别转换为字符串
    fn level_to_string(&self, level: IssueLevel) -> String {
        match level {
            IssueLevel::Info => "INFO",
            IssueLevel::Warning => "WARNING",
            IssueLevel::Error => "ERROR",
            IssueLevel::Critical => "CRITICAL",
        }.to_string()
    }
    
    /// 将类型转换为字符串
    fn type_to_string(&self, issue_type: IssueType) -> String {
        match issue_type {
            IssueType::Style => "代码风格",
            IssueType::Performance => "性能",
            IssueType::Security => "安全",
            IssueType::Memory => "内存",
            IssueType::ErrorHandling => "错误处理",
            IssueType::Concurrency => "并发",
            IssueType::Quality => "代码质量",
            IssueType::Other => "其他",
        }.to_string()
    }
    
    /// 保存报告到文件
    pub fn save_report(&self, report: &Report, output_path: &str) -> Result<(), String> {
        let path = Path::new(output_path);
        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(report.content.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// 生成报告ID
fn generate_report_id() -> String {
    format!("report_{}_{}", 
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

use chrono;
use rand;
