use std::collections::HashSet;
use tracing::{info, debug};

use game_stream_common::AuthConfig;

/// 认证管理器
pub struct AuthManager {
    config: AuthConfig,
    valid_stream_keys: HashSet<String>,
}

impl AuthManager {
    pub fn new(config: &AuthConfig) -> Self {
        info!("Initializing auth manager...");
        
        let valid_stream_keys = config.valid_stream_keys.iter().cloned().collect();
        
        Self {
            config: config.clone(),
            valid_stream_keys,
        }
    }
    
    /// 验证流密钥
    pub async fn validate_stream_key(&self, stream_key: &str) -> bool {
        if !self.config.enabled {
            debug!("Authentication disabled, allowing stream key: {}", stream_key);
            return true;
        }
        
        let is_valid = self.valid_stream_keys.contains(stream_key);
        
        if is_valid {
            debug!("Stream key validated: {}", stream_key);
        } else {
            debug!("Invalid stream key: {}", stream_key);
        }
        
        is_valid
    }
    
    /// 验证观看者权限
    pub async fn validate_viewer(&self, stream_key: &str, _viewer_token: Option<&str>) -> bool {
        // 简单实现：如果流存在且有效，则允许观看
        self.validate_stream_key(stream_key).await
    }
    
    /// 添加有效的流密钥
    pub async fn add_stream_key(&mut self, stream_key: String) {
        self.valid_stream_keys.insert(stream_key.clone());
        info!("Added stream key: {}", stream_key);
    }
    
    /// 移除流密钥
    pub async fn remove_stream_key(&mut self, stream_key: &str) {
        self.valid_stream_keys.remove(stream_key);
        info!("Removed stream key: {}", stream_key);
    }
    
    /// 获取所有有效的流密钥
    pub async fn get_valid_stream_keys(&self) -> Vec<String> {
        self.valid_stream_keys.iter().cloned().collect()
    }
}
