//! Teleport Executor 模块
//! 
//! 实现任务执行和管理功能

use super::packet::*;
use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

/// 任务执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 任务执行信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    /// 任务ID
    pub task_id: String,
    /// 执行状态
    pub status: ExecutionStatus,
    /// 开始时间
    pub start_time: String,
    /// 结束时间
    pub end_time: Option<String>,
    /// 执行结果
    pub result: Option<ResultPacket>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行主机
    pub host: String,
    /// 执行线程
    pub thread_id: String,
}

/// 任务执行器
pub struct TaskExecutor {
    /// 执行线程池
    pool: tokio::runtime::Runtime,
    /// 执行中的任务
    running_tasks: Arc<RwLock<HashMap<String, ExecutionInfo>>>,
    /// 任务队列
    task_queue: Arc<RwLock<Vec<TaskPacket>>>,
    /// 最大并发任务数
    max_concurrent: usize,
}

impl TaskExecutor {
    /// 创建新的任务执行器
    pub fn new(max_concurrent: usize) -> Self {
        let pool = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(max_concurrent)
            .build()
            .unwrap();
        
        Self {
            pool,
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(Vec::new())),
            max_concurrent,
        }
    }
    
    /// 执行任务
    pub async fn execute_task(&self, task: TaskPacket) -> ResultPacket {
        let start_time = chrono::Utc::now().to_rfc3339();
        let start_instant = std::time::Instant::now();
        
        // 更新执行状态
        let mut running_tasks = self.running_tasks.write().await;
        running_tasks.insert(task.task_id.clone(), ExecutionInfo {
            task_id: task.task_id.clone(),
            status: ExecutionStatus::Running,
            start_time: start_time.clone(),
            end_time: None,
            result: None,
            error: None,
            host: hostname::get().unwrap_or_else(|_| "unknown".into()).to_string_lossy().to_string(),
            thread_id: std::thread::current().id().to_string(),
        });
        drop(running_tasks);
        
        // 执行任务
        let result = self.pool.block_on(async {
            self.execute_task_internal(task).await
        });
        
        let execution_time = start_instant.elapsed().as_millis() as u64;
        let end_time = chrono::Utc::now().to_rfc3339();
        
        // 更新执行状态
        let mut running_tasks = self.running_tasks.write().await;
        if let Some(info) = running_tasks.get_mut(&task.task_id) {
            info.status = if result.error.is_some() {
                ExecutionStatus::Failed
            } else {
                ExecutionStatus::Completed
            };
            info.end_time = Some(end_time);
            info.result = Some(result.clone());
            info.error = result.error.clone();
        }
        
        result
    }
    
    /// 内部执行任务
    async fn execute_task_internal(&self, task: TaskPacket) -> ResultPacket {
        // 模拟任务执行
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // 根据任务类型执行不同的操作
        match task.task_type.as_str() {
            "echo" => {
                let message = task.parameters.get("message").and_then(|v| v.as_str()).unwrap_or("Hello, World!");
                ResultPacket {
                    task_id: task.task_id,
                    status: "success".to_string(),
                    result: serde_json::Value::String(message.to_string()),
                    execution_time_ms: 1000,
                    error: None,
                    output: format!("Echo: {}", message),
                }
            }
            "sleep" => {
                let duration = task.parameters.get("duration").and_then(|v| v.as_u64()).unwrap_or(1);
                tokio::time::sleep(Duration::from_secs(duration)).await;
                ResultPacket {
                    task_id: task.task_id,
                    status: "success".to_string(),
                    result: serde_json::Value::Null,
                    execution_time_ms: (duration * 1000) as u64,
                    error: None,
                    output: format!("Slept for {} seconds", duration),
                }
            }
            "calculate" => {
                let a = task.parameters.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b = task.parameters.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let operation = task.parameters.get("operation").and_then(|v| v.as_str()).unwrap_or("add");
                
                let result = match operation {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => if b != 0.0 { a / b } else { return ResultPacket {
                        task_id: task.task_id,
                        status: "error".to_string(),
                        result: serde_json::Value::Null,
                        execution_time_ms: 100,
                        error: Some("Division by zero".to_string()),
                        output: "Error: Division by zero",
                    }},
                    _ => return ResultPacket {
                        task_id: task.task_id,
                        status: "error".to_string(),
                        result: serde_json::Value::Null,
                        execution_time_ms: 100,
                        error: Some("Unknown operation".to_string()),
                        output: "Error: Unknown operation",
                    },
                };
                
                ResultPacket {
                    task_id: task.task_id,
                    status: "success".to_string(),
                    result: serde_json::Value::Number(serde_json::Number::from_f64(result).unwrap()),
                    execution_time_ms: 100,
                    error: None,
                    output: format!("{} {} {} = {}", a, operation, b, result),
                }
            }
            _ => {
                ResultPacket {
                    task_id: task.task_id,
                    status: "error".to_string(),
                    result: serde_json::Value::Null,
                    execution_time_ms: 100,
                    error: Some("Unknown task type".to_string()),
                    output: "Error: Unknown task type",
                }
            }
        }
    }
    
    /// 添加任务到队列
    pub async fn queue_task(&self, task: TaskPacket) {
        let mut task_queue = self.task_queue.write().await;
        task_queue.push(task);
    }
    
    /// 处理队列中的任务
    pub async fn process_queue(&self) {
        loop {
            let mut task_queue = self.task_queue.write().await;
            let running_tasks = self.running_tasks.read().await;
            
            if task_queue.is_empty() || running_tasks.len() >= self.max_concurrent {
                drop(running_tasks);
                drop(task_queue);
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            
            let task = task_queue.remove(0);
            drop(task_queue);
            drop(running_tasks);
            
            // 执行任务
            let executor = self.clone();
            tokio::spawn(async move {
                executor.execute_task(task).await;
            });
        }
    }
    
    /// 获取任务执行状态
    pub async fn get_task_status(&self, task_id: &str) -> Option<ExecutionInfo> {
        let running_tasks = self.running_tasks.read().await;
        running_tasks.get(task_id).cloned()
    }
    
    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str) -> bool {
        let mut running_tasks = self.running_tasks.write().await;
        if let Some(info) = running_tasks.get_mut(task_id) {
            info.status = ExecutionStatus::Cancelled;
            info.end_time = Some(chrono::Utc::now().to_rfc3339());
            true
        } else {
            false
        }
    }
    
    /// 获取执行器状态
    pub async fn get_status(&self) -> ExecutorStatus {
        let running_tasks = self.running_tasks.read().await;
        let task_queue = self.task_queue.read().await;
        
        let running_count = running_tasks.len();
        let pending_count = task_queue.len();
        let completed_count = running_tasks
            .values()
            .filter(|info| info.status == ExecutionStatus::Completed)
            .count();
        let failed_count = running_tasks
            .values()
            .filter(|info| info.status == ExecutionStatus::Failed)
            .count();
        
        ExecutorStatus {
            running_tasks: running_count,
            pending_tasks: pending_count,
            completed_tasks: completed_count,
            failed_tasks: failed_count,
            max_concurrent: self.max_concurrent,
            utilization: (running_count as f64 / self.max_concurrent as f64) * 100.0,
        }
    }
}

impl Clone for TaskExecutor {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            running_tasks: self.running_tasks.clone(),
            task_queue: self.task_queue.clone(),
            max_concurrent: self.max_concurrent,
        }
    }
}

/// 执行器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorStatus {
    /// 运行中的任务数
    pub running_tasks: usize,
    /// 待执行的任务数
    pub pending_tasks: usize,
    /// 已完成的任务数
    pub completed_tasks: usize,
    /// 已失败的任务数
    pub failed_tasks: usize,
    /// 最大并发任务数
    pub max_concurrent: usize,
    /// 利用率（%）
    pub utilization: f64,
}

/// 执行器管理器
pub struct ExecutorManager {
    /// 执行器列表
    executors: Arc<RwLock<HashMap<String, TaskExecutor>>>,
    /// 默认执行器
    default_executor: String,
}

impl ExecutorManager {
    /// 创建新的执行器管理器
    pub fn new() -> Self {
        let mut executors = HashMap::new();
        let default_executor = "default".to_string();
        executors.insert(default_executor.clone(), TaskExecutor::new(4));
        
        Self {
            executors: Arc::new(RwLock::new(executors)),
            default_executor,
        }
    }
    
    /// 添加执行器
    pub async fn add_executor(&self, name: String, max_concurrent: usize) {
        let mut executors = self.executors.write().await;
        executors.insert(name, TaskExecutor::new(max_concurrent));
    }
    
    /// 获取执行器
    pub async fn get_executor(&self, name: &str) -> Option<TaskExecutor> {
        let executors = self.executors.read().await;
        executors.get(name).cloned()
    }
    
    /// 获取默认执行器
    pub async fn get_default_executor(&self) -> TaskExecutor {
        self.get_executor(&self.default_executor)
            .await
            .unwrap()
    }
    
    /// 执行任务
    pub async fn execute_task(&self, task: TaskPacket, executor_name: Option<&str>) -> ResultPacket {
        let executor = if let Some(name) = executor_name {
            self.get_executor(name)
                .await
                .unwrap_or_else(|| self.get_default_executor().await)
        } else {
            self.get_default_executor().await
        };
        
        executor.execute_task(task).await
    }
    
    /// 获取所有执行器状态
    pub async fn get_all_status(&self) -> HashMap<String, ExecutorStatus> {
        let executors = self.executors.read().await;
        let mut status_map = HashMap::new();
        
        for (name, executor) in executors.iter() {
            let status = executor.get_status().await;
            status_map.insert(name.clone(), status);
        }
        
        status_map
    }
}

use chrono;
use hostname;
