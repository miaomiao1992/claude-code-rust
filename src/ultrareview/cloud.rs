//! Ultrareview Cloud 模块
//! 
//! 实现云端代码审查功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 云端分析任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAnalysisTask {
    /// 任务ID
    pub task_id: String,
    /// 项目ID
    pub project_id: String,
    /// 分析配置
    pub config: CloudAnalysisConfig,
    /// 任务状态
    pub status: TaskStatus,
    /// 创建时间
    pub created_at: String,
    /// 开始时间
    pub started_at: Option<String>,
    /// 完成时间
    pub completed_at: Option<String>,
    /// 结果URL
    pub result_url: Option<String>,
    /// 错误信息
    pub error: Option<String>,
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 待处理
    Pending,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Canceled,
}

/// 云端分析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAnalysisConfig {
    /// 分析级别
    pub level: String,
    /// 包含的文件
    pub include_files: Vec<String>,
    /// 排除的文件
    pub exclude_files: Vec<String>,
    /// 分析规则
    pub rules: Vec<String>,
    /// 超时时间（秒）
    pub timeout: u32,
    /// 是否生成报告
    pub generate_report: bool,
    /// 报告格式
    pub report_format: String,
}

/// 云端项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProject {
    /// 项目ID
    pub project_id: String,
    /// 项目名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 创建者
    pub creator: String,
    /// 创建时间
    pub created_at: String,
    /// 最后更新时间
    pub updated_at: String,
    /// 分析历史
    pub analysis_history: Vec<String>, // 任务ID列表
    /// 项目设置
    pub settings: ProjectSettings,
}

/// 项目设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// 自动分析
    pub auto_analysis: bool,
    /// 分析频率（分钟）
    pub analysis_frequency: u32,
    /// 通知设置
    pub notifications: NotificationSettings,
    /// 权限设置
    pub permissions: PermissionSettings,
}

/// 通知设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// 邮件通知
    pub email: bool,
    /// Webhook通知
    pub webhook: Option<String>,
    /// 通知阈值
    pub threshold: u8,
}

/// 权限设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSettings {
    /// 可访问用户
    pub users: Vec<String>,
    /// 可编辑用户
    pub editors: Vec<String>,
    /// 管理员
    pub admins: Vec<String>,
}

/// 云端分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAnalysisResult {
    /// 结果ID
    pub result_id: String,
    /// 任务ID
    pub task_id: String,
    /// 项目ID
    pub project_id: String,
    /// 分析结果
    pub analysis_result: crate::ultrareview::AnalysisResult,
    /// 质量评估结果
    pub quality_result: crate::ultrareview::QualityResult,
    /// 生成的报告
    pub reports: Vec<crate::ultrareview::Report>,
    /// 分析时间
    pub analysis_time_ms: u64,
    /// 完成时间
    pub completed_at: String,
}

/// 云端管理器
pub struct CloudManager {
    /// 项目列表
    projects: Arc<RwLock<HashMap<String, CloudProject>>>,
    /// 任务列表
    tasks: Arc<RwLock<HashMap<String, CloudAnalysisTask>>>,
    /// 分析结果
    results: Arc<RwLock<HashMap<String, CloudAnalysisResult>>>,
}

impl CloudManager {
    /// 创建新的云端管理器
    pub fn new() -> Self {
        Self {
            projects: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 创建项目
    pub async fn create_project(
        &self,
        name: String,
        description: String,
        creator: String,
    ) -> Result<CloudProject, String> {
        let project_id = generate_project_id();
        let now = chrono::Utc::now().to_rfc3339();
        
        let project = CloudProject {
            project_id: project_id.clone(),
            name,
            description,
            creator,
            created_at: now.clone(),
            updated_at: now,
            analysis_history: Vec::new(),
            settings: ProjectSettings {
                auto_analysis: false,
                analysis_frequency: 60, // 1小时
                notifications: NotificationSettings {
                    email: true,
                    webhook: None,
                    threshold: 70,
                },
                permissions: PermissionSettings {
                    users: vec![creator.clone()],
                    editors: vec![creator.clone()],
                    admins: vec![creator.clone()],
                },
            },
        };
        
        let mut projects = self.projects.write().await;
        projects.insert(project_id, project.clone());
        
        Ok(project)
    }
    
    /// 创建分析任务
    pub async fn create_analysis_task(
        &self,
        project_id: String,
        config: CloudAnalysisConfig,
    ) -> Result<CloudAnalysisTask, String> {
        // 检查项目是否存在
        let projects = self.projects.read().await;
        if !projects.contains_key(&project_id) {
            return Err("Project not found".to_string());
        }
        drop(projects);
        
        let task_id = generate_task_id();
        let now = chrono::Utc::now().to_rfc3339();
        
        let task = CloudAnalysisTask {
            task_id: task_id.clone(),
            project_id: project_id.clone(),
            config,
            status: TaskStatus::Pending,
            created_at: now,
            started_at: None,
            completed_at: None,
            result_url: None,
            error: None,
        };
        
        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id, task.clone());
        
        // 更新项目的分析历史
        let mut projects = self.projects.write().await;
        if let Some(project) = projects.get_mut(&project_id) {
            project.analysis_history.push(task.task_id.clone());
            project.updated_at = chrono::Utc::now().to_rfc3339();
        }
        
        Ok(task)
    }
    
    /// 开始分析任务
    pub async fn start_task(&self, task_id: &str) -> Result<bool, String> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            if task.status == TaskStatus::Pending {
                task.status = TaskStatus::Running;
                task.started_at = Some(chrono::Utc::now().to_rfc3339());
                Ok(true)
            } else {
                Err("Task is not in pending status".to_string())
            }
        } else {
            Err("Task not found".to_string())
        }
    }
    
    /// 完成分析任务
    pub async fn complete_task(
        &self,
        task_id: &str,
        result: CloudAnalysisResult,
    ) -> Result<bool, String> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            if task.status == TaskStatus::Running {
                task.status = TaskStatus::Completed;
                task.completed_at = Some(chrono::Utc::now().to_rfc3339());
                task.result_url = Some(format!("/api/results/{}", result.result_id));
                
                // 保存结果
                let mut results = self.results.write().await;
                results.insert(result.result_id.clone(), result);
                
                Ok(true)
            } else {
                Err("Task is not in running status".to_string())
            }
        } else {
            Err("Task not found".to_string())
        }
    }
    
    /// 失败分析任务
    pub async fn fail_task(
        &self,
        task_id: &str,
        error: String,
    ) -> Result<bool, String> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            if task.status == TaskStatus::Running {
                task.status = TaskStatus::Failed;
                task.completed_at = Some(chrono::Utc::now().to_rfc3339());
                task.error = Some(error);
                Ok(true)
            } else {
                Err("Task is not in running status".to_string())
            }
        } else {
            Err("Task not found".to_string())
        }
    }
    
    /// 取消分析任务
    pub async fn cancel_task(&self, task_id: &str) -> Result<bool, String> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            if task.status == TaskStatus::Pending || task.status == TaskStatus::Running {
                task.status = TaskStatus::Canceled;
                task.completed_at = Some(chrono::Utc::now().to_rfc3339());
                Ok(true)
            } else {
                Err("Task cannot be canceled".to_string())
            }
        } else {
            Err("Task not found".to_string())
        }
    }
    
    /// 获取项目
    pub async fn get_project(&self, project_id: &str) -> Option<CloudProject> {
        let projects = self.projects.read().await;
        projects.get(project_id).cloned()
    }
    
    /// 获取任务
    pub async fn get_task(&self, task_id: &str) -> Option<CloudAnalysisTask> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }
    
    /// 获取分析结果
    pub async fn get_result(&self, result_id: &str) -> Option<CloudAnalysisResult> {
        let results = self.results.read().await;
        results.get(result_id).cloned()
    }
    
    /// 获取项目的分析历史
    pub async fn get_project_analysis_history(
        &self,
        project_id: &str,
    ) -> Result<Vec<CloudAnalysisResult>, String> {
        let projects = self.projects.read().await;
        let project = projects.get(project_id).ok_or("Project not found".to_string())?;
        
        let results = self.results.read().await;
        let mut history = Vec::new();
        
        for task_id in &project.analysis_history {
            // 查找对应的结果
            for result in results.values() {
                if result.task_id == *task_id {
                    history.push(result.clone());
                    break;
                }
            }
        }
        
        Ok(history)
    }
    
    /// 更新项目设置
    pub async fn update_project_settings(
        &self,
        project_id: &str,
        settings: ProjectSettings,
    ) -> Result<bool, String> {
        let mut projects = self.projects.write().await;
        if let Some(project) = projects.get_mut(project_id) {
            project.settings = settings;
            project.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(true)
        } else {
            Err("Project not found".to_string())
        }
    }
    
    /// 删除项目
    pub async fn delete_project(&self, project_id: &str) -> Result<bool, String> {
        let mut projects = self.projects.write().await;
        if projects.remove(project_id).is_some() {
            // 同时删除相关的任务和结果
            let mut tasks = self.tasks.write().await;
            let mut results = self.results.write().await;
            
            // 查找并删除相关的任务和结果
            let task_ids_to_remove: Vec<String> = tasks
                .iter()
                .filter(|(_, task)| task.project_id == *project_id)
                .map(|(id, _)| id.clone())
                .collect();
            
            for task_id in task_ids_to_remove {
                tasks.remove(&task_id);
                // 删除相关的结果
                let result_ids_to_remove: Vec<String> = results
                    .iter()
                    .filter(|(_, result)| result.task_id == task_id)
                    .map(|(id, _)| id.clone())
                    .collect();
                
                for result_id in result_ids_to_remove {
                    results.remove(&result_id);
                }
            }
            
            Ok(true)
        } else {
            Err("Project not found".to_string())
        }
    }
    
    /// 获取所有项目
    pub async fn get_all_projects(&self) -> Vec<CloudProject> {
        let projects = self.projects.read().await;
        projects.values().cloned().collect()
    }
    
    /// 获取所有任务
    pub async fn get_all_tasks(&self) -> Vec<CloudAnalysisTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }
}

/// 生成项目ID
fn generate_project_id() -> String {
    format!("project_{}_{}", 
        chrono::Utc::now().timestamp(),
        generate_id()
    )
}

/// 生成任务ID
fn generate_task_id() -> String {
    format!("task_{}_{}", 
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
