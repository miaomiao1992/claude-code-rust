//! OAuth认证模块
//!
//! 提供OAuth 2.0认证支持，包括：
//! - OAuth 2.0授权码流程
//! - 令牌刷新
//! - 令牌存储
//! - 自动令牌刷新

use crate::error::{ApiError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// OAuth客户端
#[derive(Debug, Clone)]
pub struct OAuthClient {
    /// 客户端配置
    config: OAuthClientConfig,
    /// 当前令牌
    token: Option<OAuthToken>,
    /// HTTP客户端
    http_client: reqwest::Client,
}

/// OAuth客户端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClientConfig {
    /// 客户端ID
    pub client_id: String,
    /// 客户端密钥
    pub client_secret: Option<String>,
    /// 授权URL
    pub auth_url: String,
    /// 令牌URL
    pub token_url: String,
    /// 重定向URL
    pub redirect_url: String,
    /// 作用域
    pub scopes: Vec<String>,
}

impl OAuthClientConfig {
    /// 创建新的OAuth配置
    pub fn new(
        client_id: impl Into<String>,
        auth_url: impl Into<String>,
        token_url: impl Into<String>,
        redirect_url: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: None,
            auth_url: auth_url.into(),
            token_url: token_url.into(),
            redirect_url: redirect_url.into(),
            scopes: Vec::new(),
        }
    }

    /// 设置客户端密钥
    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = Some(secret.into());
        self
    }

    /// 添加作用域
    pub fn add_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }

    /// 构建授权URL
    pub fn authorization_url(&self, state: &str) -> String {
        let scopes = self.scopes.join(" ");
        format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
            self.auth_url, self.client_id, self.redirect_url, scopes, state
        )
    }
}

/// OAuth令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    /// 访问令牌
    pub access_token: String,
    /// 刷新令牌
    pub refresh_token: Option<String>,
    /// 令牌类型
    pub token_type: String,
    /// 过期时间（秒）
    pub expires_in: u64,
    /// 作用域
    pub scope: Option<String>,
    /// 获取时间
    #[serde(default, skip_serializing)]
    pub obtained_at: Option<SystemTime>,
}

impl OAuthToken {
    /// 创建新的令牌
    pub fn new(
        access_token: impl Into<String>,
        token_type: impl Into<String>,
        expires_in: u64,
    ) -> Self {
        Self {
            access_token: access_token.into(),
            refresh_token: None,
            token_type: token_type.into(),
            expires_in,
            scope: None,
            obtained_at: Some(SystemTime::now()),
        }
    }

    /// 设置刷新令牌
    pub fn with_refresh_token(mut self, refresh_token: impl Into<String>) -> Self {
        self.refresh_token = Some(refresh_token.into());
        self
    }

    /// 设置作用域
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        // 如果没有获取时间，认为已过期
        let Some(obtained_at) = self.obtained_at else {
            return true;
        };
        let elapsed = SystemTime::now()
            .duration_since(obtained_at)
            .unwrap_or_default();
        elapsed.as_secs() >= self.expires_in.saturating_sub(60) // 提前60秒刷新
    }

    /// 获取授权头
    pub fn auth_header(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }

    /// 从响应创建
    pub fn from_response(response: TokenResponse) -> Self {
        Self {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            token_type: response.token_type.unwrap_or_else(|| "Bearer".to_string()),
            expires_in: response.expires_in.unwrap_or(3600),
            scope: response.scope,
            obtained_at: Some(SystemTime::now()),
        }
    }
}

/// 令牌响应
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: Option<String>,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

impl OAuthClient {
    /// 创建新的OAuth客户端
    pub fn new(config: OAuthClientConfig) -> Result<Self> {
        let http_client = reqwest::Client::new();
        Ok(Self {
            config,
            token: None,
            http_client,
        })
    }

    /// 获取授权URL
    pub fn authorization_url(&self, state: &str) -> String {
        self.config.authorization_url(state)
    }

    /// 交换授权码获取令牌
    pub async fn exchange_code(&mut self, code: &str) -> Result<&OAuthToken> {
        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code);
        params.insert("redirect_uri", &self.config.redirect_url);
        params.insert("client_id", &self.config.client_id);

        if let Some(ref secret) = self.config.client_secret {
            params.insert("client_secret", secret);
        }

        let response = self
            .http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(ApiError::network)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::auth(format!(
                "Token exchange failed ({}): {}",
                status, text
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(ApiError::network)?;

        let token = OAuthToken::from_response(token_response);
        self.token = Some(token);

        Ok(self.token.as_ref().unwrap())
    }

    /// 刷新令牌
    pub async fn refresh_token(&mut self) -> Result<&OAuthToken> {
        let current_token = self
            .token
            .as_ref()
            .ok_or_else(|| ApiError::auth("No token to refresh"))?;

        let refresh_token = current_token
            .refresh_token
            .as_ref()
            .ok_or_else(|| ApiError::auth("No refresh token available"))?;

        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", refresh_token);
        params.insert("client_id", &self.config.client_id);

        if let Some(ref secret) = self.config.client_secret {
            params.insert("client_secret", secret);
        }

        let response = self
            .http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(ApiError::network)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::auth(format!(
                "Token refresh failed ({}): {}",
                status, text
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(ApiError::network)?;

        let token = OAuthToken::from_response(token_response);
        self.token = Some(token);

        Ok(self.token.as_ref().unwrap())
    }

    /// 获取当前令牌
    pub fn token(&self) -> Option<&OAuthToken> {
        self.token.as_ref()
    }

    /// 获取有效的访问令牌（自动刷新）
    pub async fn access_token(&mut self) -> Result<String> {
        if let Some(ref token) = self.token {
            if token.is_expired() {
                if token.refresh_token.is_some() {
                    self.refresh_token().await?;
                } else {
                    return Err(ApiError::auth("Token expired and no refresh token available"));
                }
            }
            Ok(self.token.as_ref().unwrap().access_token.clone())
        } else {
            Err(ApiError::auth("No token available"))
        }
    }

    /// 设置令牌
    pub fn set_token(&mut self, token: OAuthToken) {
        self.token = Some(token);
    }

    /// 清除令牌
    pub fn clear_token(&mut self) {
        self.token = None;
    }

    /// 检查是否已认证
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }
}

/// 简单的内存令牌存储
pub struct InMemoryTokenStore {
    tokens: HashMap<String, OAuthToken>,
}

impl InMemoryTokenStore {
    /// 创建新的存储
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    /// 存储令牌
    pub fn store(&mut self, key: impl Into<String>, token: OAuthToken) {
        self.tokens.insert(key.into(), token);
    }

    /// 获取令牌
    pub fn get(&self, key: &str) -> Option<&OAuthToken> {
        self.tokens.get(key)
    }

    /// 删除令牌
    pub fn remove(&mut self, key: &str) -> Option<OAuthToken> {
        self.tokens.remove(key)
    }
}

impl Default for InMemoryTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_config() {
        let config = OAuthClientConfig::new(
            "client-id",
            "https://auth.example.com/authorize",
            "https://auth.example.com/token",
            "https://app.example.com/callback",
        )
        .add_scope("read")
        .add_scope("write");

        let auth_url = config.authorization_url("random-state");
        assert!(auth_url.contains("client_id=client-id"));
        assert!(auth_url.contains("state=random-state"));
        assert!(auth_url.contains("scope=read+write"));
    }

    #[test]
    fn test_oauth_token() {
        let token = OAuthToken::new("access123", "Bearer", 3600)
            .with_refresh_token("refresh456")
            .with_scope("read write");

        assert_eq!(token.access_token, "access123");
        assert_eq!(token.token_type, "Bearer");
        assert_eq!(token.refresh_token, Some("refresh456".to_string()));
        assert_eq!(token.scope, Some("read write".to_string()));
        assert!(!token.is_expired());

        let header = token.auth_header();
        assert_eq!(header, "Bearer access123");
    }

    #[test]
    fn test_token_store() {
        let mut store = InMemoryTokenStore::new();
        let token = OAuthToken::new("test", "Bearer", 3600);

        store.store("user1", token.clone());
        assert!(store.get("user1").is_some());
        assert!(store.get("user2").is_none());

        let removed = store.remove("user1");
        assert!(removed.is_some());
        assert!(store.get("user1").is_none());
    }
}
