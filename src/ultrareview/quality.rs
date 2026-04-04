//! Ultrareview Quality 模块
//! 
//! 实现代码质量评估功能

use serde::{Deserialize, Serialize};

/// 代码质量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// 代码质量评分 (0-100)
    pub score: u8,
    /// 代码复杂度
    pub complexity: f64,
    /// 代码重复率
    pub duplication: f64,
    /// 代码覆盖率
    pub coverage: f64,
    /// 问题密度 (问题数/代码行数)
    pub issue_density: f64,
    /// 代码行数
    pub lines_of_code: usize,
    /// 注释率
    pub comment_ratio: f64,
    /// 函数平均长度
    pub avg_function_length: f64,
    /// 循环复杂度
    pub cyclomatic_complexity: f64,
}

/// 质量等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityGrade {
    /// 优秀
    A,
    /// 良好
    B,
    /// 一般
    C,
    /// 较差
    D,
    /// 差
    F,
}

/// 质量评估器
pub struct QualityEvaluator {
    /// 评估配置
    config: QualityConfig,
}

/// 质量评估配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// 评分权重
    pub weights: QualityWeights,
    /// 阈值
    pub thresholds: QualityThresholds,
}

/// 评分权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityWeights {
    /// 问题密度权重
    pub issue_density: f64,
    /// 代码复杂度权重
    pub complexity: f64,
    /// 代码重复率权重
    pub duplication: f64,
    /// 代码覆盖率权重
    pub coverage: f64,
    /// 注释率权重
    pub comment_ratio: f64,
}

/// 质量阈值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// 优秀阈值
    pub excellent: f64,
    /// 良好阈值
    pub good: f64,
    /// 一般阈值
    pub average: f64,
    /// 较差阈值
    pub poor: f64,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            weights: QualityWeights {
                issue_density: 0.3,
                complexity: 0.2,
                duplication: 0.2,
                coverage: 0.15,
                comment_ratio: 0.15,
            },
            thresholds: QualityThresholds {
                excellent: 90.0,
                good: 75.0,
                average: 60.0,
                poor: 40.0,
            },
        }
    }
}

impl QualityEvaluator {
    /// 创建新的质量评估器
    pub fn new() -> Self {
        Self {
            config: QualityConfig::default(),
        }
    }
    
    /// 用自定义配置创建质量评估器
    pub fn with_config(config: QualityConfig) -> Self {
        Self {
            config,
        }
    }
    
    /// 评估代码质量
    pub fn evaluate(&self, metrics: &QualityMetrics) -> QualityResult {
        let score = self.calculate_score(metrics);
        let grade = self.calculate_grade(score);
        let insights = self.generate_insights(metrics, score, grade);
        
        QualityResult {
            score,
            grade,
            metrics: metrics.clone(),
            insights,
            recommendations: self.generate_recommendations(metrics),
        }
    }
    
    /// 计算质量评分
    fn calculate_score(&self, metrics: &QualityMetrics) -> u8 {
        // 计算各项指标的得分
        let issue_density_score = self.calculate_issue_density_score(metrics.issue_density);
        let complexity_score = self.calculate_complexity_score(metrics.complexity);
        let duplication_score = self.calculate_duplication_score(metrics.duplication);
        let coverage_score = self.calculate_coverage_score(metrics.coverage);
        let comment_ratio_score = self.calculate_comment_ratio_score(metrics.comment_ratio);
        
        // 加权平均
        let total_score = (
            issue_density_score * self.config.weights.issue_density +
            complexity_score * self.config.weights.complexity +
            duplication_score * self.config.weights.duplication +
            coverage_score * self.config.weights.coverage +
            comment_ratio_score * self.config.weights.comment_ratio
        );
        
        total_score.round() as u8
    }
    
    /// 计算问题密度得分
    fn calculate_issue_density_score(&self, issue_density: f64) -> f64 {
        // 问题密度越低越好，0 表示没有问题
        if issue_density == 0.0 {
            100.0
        } else if issue_density < 0.01 {
            90.0
        } else if issue_density < 0.05 {
            70.0
        } else if issue_density < 0.1 {
            50.0
        } else {
            30.0
        }
    }
    
    /// 计算复杂度得分
    fn calculate_complexity_score(&self, complexity: f64) -> f64 {
        // 复杂度越低越好
        if complexity < 5.0 {
            100.0
        } else if complexity < 10.0 {
            80.0
        } else if complexity < 20.0 {
            60.0
        } else if complexity < 30.0 {
            40.0
        } else {
            20.0
        }
    }
    
    /// 计算重复率得分
    fn calculate_duplication_score(&self, duplication: f64) -> f64 {
        // 重复率越低越好
        if duplication == 0.0 {
            100.0
        } else if duplication < 5.0 {
            90.0
        } else if duplication < 10.0 {
            70.0
        } else if duplication < 20.0 {
            50.0
        } else {
            30.0
        }
    }
    
    /// 计算覆盖率得分
    fn calculate_coverage_score(&self, coverage: f64) -> f64 {
        // 覆盖率越高越好
        if coverage >= 90.0 {
            100.0
        } else if coverage >= 80.0 {
            90.0
        } else if coverage >= 70.0 {
            70.0
        } else if coverage >= 50.0 {
            50.0
        } else {
            30.0
        }
    }
    
    /// 计算注释率得分
    fn calculate_comment_ratio_score(&self, comment_ratio: f64) -> f64 {
        // 注释率在 20-30% 之间最佳
        if comment_ratio >= 20.0 && comment_ratio <= 30.0 {
            100.0
        } else if comment_ratio >= 15.0 && comment_ratio < 20.0 {
            90.0
        } else if comment_ratio >= 30.0 && comment_ratio <= 40.0 {
            90.0
        } else if comment_ratio >= 10.0 && comment_ratio < 15.0 {
            70.0
        } else if comment_ratio > 40.0 && comment_ratio <= 50.0 {
            70.0
        } else if comment_ratio >= 5.0 && comment_ratio < 10.0 {
            50.0
        } else {
            30.0
        }
    }
    
    /// 计算质量等级
    fn calculate_grade(&self, score: u8) -> QualityGrade {
        let score_f = score as f64;
        if score_f >= self.config.thresholds.excellent {
            QualityGrade::A
        } else if score_f >= self.config.thresholds.good {
            QualityGrade::B
        } else if score_f >= self.config.thresholds.average {
            QualityGrade::C
        } else if score_f >= self.config.thresholds.poor {
            QualityGrade::D
        } else {
            QualityGrade::F
        }
    }
    
    /// 生成质量洞察
    fn generate_insights(&self, metrics: &QualityMetrics, score: u8, grade: QualityGrade) -> Vec<String> {
        let mut insights = Vec::new();
        
        insights.push(format!("代码质量评分: {}/100 (等级: {:?})", score, grade));
        
        if metrics.issue_density > 0.05 {
            insights.push("问题密度较高，建议修复现有问题并加强代码审查".to_string());
        }
        
        if metrics.complexity > 15.0 {
            insights.push("代码复杂度较高，建议重构以提高可维护性".to_string());
        }
        
        if metrics.duplication > 10.0 {
            insights.push("代码重复率较高，建议提取公共代码以减少重复".to_string());
        }
        
        if metrics.coverage < 70.0 {
            insights.push("代码覆盖率较低，建议增加测试用例".to_string());
        }
        
        if metrics.comment_ratio < 10.0 {
            insights.push("注释率较低，建议增加代码注释以提高可读性".to_string());
        }
        
        if metrics.avg_function_length > 30.0 {
            insights.push("函数平均长度较长，建议拆分为更小的函数".to_string());
        }
        
        if metrics.cyclomatic_complexity > 10.0 {
            insights.push("循环复杂度较高，建议简化条件逻辑".to_string());
        }
        
        insights
    }
    
    /// 生成改进建议
    fn generate_recommendations(&self, metrics: &QualityMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // 基于各项指标生成建议
        if metrics.issue_density > 0.01 {
            recommendations.push("定期运行代码分析工具，及时发现并修复问题".to_string());
            recommendations.push("建立代码审查流程，确保代码质量".to_string());
        }
        
        if metrics.complexity > 10.0 {
            recommendations.push("采用模块化设计，降低代码复杂度".to_string());
            recommendations.push("使用设计模式简化代码结构".to_string());
        }
        
        if metrics.duplication > 5.0 {
            recommendations.push("使用代码重构工具检测并消除重复代码".to_string());
            recommendations.push("提取公共函数和模块，提高代码复用性".to_string());
        }
        
        if metrics.coverage < 80.0 {
            recommendations.push("编写单元测试，提高代码覆盖率".to_string());
            recommendations.push("使用测试覆盖率工具监控测试效果".to_string());
        }
        
        if metrics.comment_ratio < 15.0 {
            recommendations.push("为关键函数和复杂逻辑添加注释".to_string());
            recommendations.push("遵循团队的代码注释规范".to_string());
        }
        
        if metrics.avg_function_length > 20.0 {
            recommendations.push("将长函数拆分为多个小函数，每个函数只负责一个功能".to_string());
            recommendations.push("使用有意义的函数名，提高代码可读性".to_string());
        }
        
        if metrics.cyclomatic_complexity > 8.0 {
            recommendations.push("简化条件语句，减少嵌套层级".to_string());
            recommendations.push("使用早期返回（guard clauses）减少代码复杂度".to_string());
        }
        
        recommendations
    }
}

/// 质量评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityResult {
    /// 质量评分
    pub score: u8,
    /// 质量等级
    pub grade: QualityGrade,
    /// 质量指标
    pub metrics: QualityMetrics,
    /// 质量洞察
    pub insights: Vec<String>,
    /// 改进建议
    pub recommendations: Vec<String>,
}

/// 从分析结果生成质量指标
pub fn generate_metrics_from_analysis(analysis_result: &crate::ultrareview::AnalysisResult) -> QualityMetrics {
    // 计算各项指标
    let lines_of_code = analysis_result.files.len() * 100; // 简化计算，实际应该统计代码行数
    let issue_density = if lines_of_code > 0 {
        (analysis_result.issues.len() as f64) / (lines_of_code as f64) * 1000.0 // 每千行代码的问题数
    } else {
        0.0
    };
    
    // 简化计算，实际应该基于真实的代码分析
    let complexity = 5.0; // 假设平均复杂度
    let duplication = 3.0; // 假设重复率
    let coverage = 75.0; // 假设覆盖率
    let comment_ratio = 15.0; // 假设注释率
    let avg_function_length = 25.0; // 假设平均函数长度
    let cyclomatic_complexity = 7.0; // 假设循环复杂度
    
    QualityMetrics {
        score: analysis_result.quality_score,
        complexity,
        duplication,
        coverage,
        issue_density,
        lines_of_code,
        comment_ratio,
        avg_function_length,
        cyclomatic_complexity,
    }
}
