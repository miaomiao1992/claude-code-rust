//! Ultrareview Collaboration 模块
//! 
//! 实现多人协作功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 协作会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    /// 会话ID
    pub session_id: String,
    /// 会话名称
    pub name: String,
    /// 创建者
    pub creator: String,
    /// 参与者
    pub participants: Vec<String>,
    /// 分析ID
    pub analysis_id: String,
    /// 创建时间
    pub created_at: String,
    /// 最后活动时间
    pub last_activity: String,
    /// 状态
    pub status: SessionStatus,
}

/// 会话状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// 活跃
    Active,
    /// 已关闭
    Closed,
    /// 已归档
    Archived,
}

/// 协作评论
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// 评论ID
    pub comment_id: String,
    /// 会话ID
    pub session_id: String,
    /// 评论者
    pub author: String,
    /// 评论内容
    pub content: String,
    /// 评论位置
    pub location: Option<CodeLocation>,
    /// 创建时间
    pub created_at: String,
    /// 回复
    pub replies: Vec<Reply>,
    /// 状态
    pub status: CommentStatus,
}

/// 评论状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommentStatus {
    /// 待处理
    Pending,
    /// 已解决
    Resolved,
    /// 已关闭
    Closed,
}

/// 回复
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
    /// 回复ID
    pub reply_id: String,
    /// 回复者
    pub author: String,
    /// 回复内容
    pub content: String,
    /// 创建时间
    pub created_at: String,
}

/// 代码位置（与analysis模块相同）
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

/// 协作管理器
pub struct CollaborationManager {
    /// 会话列表
    sessions: Arc<RwLock<HashMap<String, CollaborationSession>>>,
    /// 评论列表
    comments: Arc<RwLock<HashMap<String, Comment>>>,
    /// 用户列表
    users: Arc<RwLock<HashMap<String, User>>>,
}

/// 用户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 用户ID
    pub user_id: String,
    /// 用户名
    pub username: String,
    /// 电子邮件
    pub email: String,
    /// 角色
    pub role: UserRole,
}

/// 用户角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    /// 所有者
    Owner,
    /// 参与者
    Participant,
    /// 观察者
    Observer,
}

impl CollaborationManager {
    /// 创建新的协作管理器
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            comments: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 创建协作会话
    pub async fn create_session(
        &self,
        name: String,
        creator: String,
        analysis_id: String,
    ) -> Result<CollaborationSession, String> {
        let session_id = generate_session_id();
        let now = chrono::Utc::now().to_rfc3339();
        
        let session = CollaborationSession {
            session_id: session_id.clone(),
            name,
            creator: creator.clone(),
            participants: vec![creator],
            analysis_id,
            created_at: now.clone(),
            last_activity: now,
            status: SessionStatus::Active,
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session.clone());
        
        Ok(session)
    }
    
    /// 添加参与者
    pub async fn add_participant(
        &self,
        session_id: &str,
        user_id: String,
    ) -> Result<bool, String> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if !session.participants.contains(&user_id) {
                session.participants.push(user_id);
                session.last_activity = chrono::Utc::now().to_rfc3339();
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err("Session not found".to_string())
        }
    }
    
    /// 移除参与者
    pub async fn remove_participant(
        &self,
        session_id: &str,
        user_id: &str,
    ) -> Result<bool, String> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(index) = session.participants.iter().position(|id| id == user_id) {
                session.participants.remove(index);
                session.last_activity = chrono::Utc::now().to_rfc3339();
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err("Session not found".to_string())
        }
    }
    
    /// 添加评论
    pub async fn add_comment(
        &self,
        session_id: &str,
        author: String,
        content: String,
        location: Option<CodeLocation>,
    ) -> Result<Comment, String> {
        // 检查会话是否存在
        let sessions = self.sessions.read().await;
        if !sessions.contains_key(session_id) {
            return Err("Session not found".to_string());
        }
        drop(sessions);
        
        let comment_id = generate_comment_id();
        let now = chrono::Utc::now().to_rfc3339();
        
        let comment = Comment {
            comment_id: comment_id.clone(),
            session_id: session_id.to_string(),
            author,
            content,
            location,
            created_at: now,
            replies: Vec::new(),
            status: CommentStatus::Pending,
        };
        
        let mut comments = self.comments.write().await;
        comments.insert(comment_id, comment.clone());
        
        // 更新会话活动时间
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity = chrono::Utc::now().to_rfc3339();
        }
        
        Ok(comment)
    }
    
    /// 添加回复
    pub async fn add_reply(
        &self,
        comment_id: &str,
        author: String,
        content: String,
    ) -> Result<Reply, String> {
        let mut comments = self.comments.write().await;
        if let Some(comment) = comments.get_mut(comment_id) {
            let reply_id = generate_reply_id();
            let now = chrono::Utc::now().to_rfc3339();
            
            let reply = Reply {
                reply_id: reply_id.clone(),
                author,
                content,
                created_at: now,
            };
            
            comment.replies.push(reply.clone());
            
            // 更新会话活动时间
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&comment.session_id) {
                session.last_activity = chrono::Utc::now().to_rfc3339();
            }
            
            Ok(reply)
        } else {
            Err("Comment not found".to_string())
        }
    }
    
    /// 更新评论状态
    pub async fn update_comment_status(
        &self,
        comment_id: &str,
        status: CommentStatus,
    ) -> Result<bool, String> {
        let mut comments = self.comments.write().await;
        if let Some(comment) = comments.get_mut(comment_id) {
            comment.status = status;
            
            // 更新会话活动时间
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&comment.session_id) {
                session.last_activity = chrono::Utc::now().to_rfc3339();
            }
            
            Ok(true)
        } else {
            Err("Comment not found".to_string())
        }
    }
    
    /// 获取会话
    pub async fn get_session(&self, session_id: &str) -> Option<CollaborationSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }
    
    /// 获取会话的评论
    pub async fn get_session_comments(&self, session_id: &str) -> Vec<Comment> {
        let comments = self.comments.read().await;
        comments
            .values()
            .filter(|comment| comment.session_id == session_id)
            .cloned()
            .collect()
    }
    
    /// 关闭会话
    pub async fn close_session(&self, session_id: &str) -> Result<bool, String> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = SessionStatus::Closed;
            session.last_activity = chrono::Utc::now().to_rfc3339();
            Ok(true)
        } else {
            Err("Session not found".to_string())
        }
    }
    
    /// 归档会话
    pub async fn archive_session(&self, session_id: &str) -> Result<bool, String> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = SessionStatus::Archived;
            session.last_activity = chrono::Utc::now().to_rfc3339();
            Ok(true)
        } else {
            Err("Session not found".to_string())
        }
    }
    
    /// 获取所有会话
    pub async fn get_all_sessions(&self) -> Vec<CollaborationSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }
}

/// 生成会话ID
fn generate_session_id() -> String {
    format!("session_{}_{}", 
        chrono::Utc::now().timestamp(),
        generate_id()
    )
}

/// 生成评论ID
fn generate_comment_id() -> String {
    format!("comment_{}_{}", 
        chrono::Utc::now().timestamp(),
        generate_id()
    )
}

/// 生成回复ID
fn generate_reply_id() -> String {
    format!("reply_{}_{}", 
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
