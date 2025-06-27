use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tokio::fs;
use tracing::{info, error, debug, warn};

use game_stream_common::{StorageConfig, LiveStream, MediaPacket, StreamResult, StreamError};

/// HLS 管理器
pub struct HlsManager {
    config: StorageConfig,
    playlists: Arc<RwLock<HashMap<String, HlsPlaylist>>>,
    segments: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl HlsManager {
    pub async fn new(config: &StorageConfig) -> Result<Self> {
        info!("Initializing HLS manager...");
        
        // 创建 HLS 目录
        fs::create_dir_all(&config.hls_segment_dir).await?;
        
        Ok(Self {
            config: config.clone(),
            playlists: Arc::new(RwLock::new(HashMap::new())),
            segments: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// 处理流的 HLS 生成
    pub async fn process_stream(&self, stream_key: &str, stream: &LiveStream) -> StreamResult<()> {
        debug!("Processing HLS for stream: {}", stream_key);
        
        // 检查流是否为直播状态
        let status = stream.get_status().await;
        if !matches!(status, game_stream_common::StreamStatus::Live) {
            return Ok(());
        }
        
        // 获取或创建播放列表
        let mut playlists = self.playlists.write().await;
        let playlist = playlists.entry(stream_key.to_string())
            .or_insert_with(|| HlsPlaylist::new(stream_key.to_string(), &self.config));
        
        // 模拟生成新的片段
        if playlist.should_generate_segment().await {
            let segment_name = format!("segment_{}.ts", playlist.next_segment_number);
            let segment_data = self.generate_segment(stream_key, &segment_name).await?;
            
            // 存储片段
            {
                let mut segments = self.segments.write().await;
                let segment_key = format!("{}_{}", stream_key, segment_name);
                segments.insert(segment_key, segment_data);
            }
            
            // 更新播放列表
            playlist.add_segment(segment_name, self.config.hls_segment_duration).await;
            
            // 写入播放列表文件
            self.write_playlist_file(stream_key, playlist).await?;
        }
        
        Ok(())
    }
    
    /// 获取 HLS 播放列表
    pub async fn get_playlist(&self, stream_key: &str) -> StreamResult<String> {
        let playlists = self.playlists.read().await;
        let playlist = playlists.get(stream_key)
            .ok_or_else(|| StreamError::StreamNotFound(stream_key.to_string()))?;
        
        Ok(playlist.generate_m3u8().await)
    }
    
    /// 获取 HLS 片段
    pub async fn get_segment(&self, stream_key: &str, segment_name: &str) -> StreamResult<Vec<u8>> {
        let segments = self.segments.read().await;
        let segment_key = format!("{}_{}", stream_key, segment_name);
        let segment_data = segments.get(&segment_key)
            .ok_or_else(|| StreamError::StreamNotFound(format!("Segment not found: {}", segment_name)))?;
        
        Ok(segment_data.clone())
    }
    
    async fn generate_segment(&self, stream_key: &str, segment_name: &str) -> StreamResult<Vec<u8>> {
        debug!("Generating HLS segment: {} for stream: {}", segment_name, stream_key);
        
        // 实际实现中，这里需要：
        // 1. 从流中收集音视频数据
        // 2. 使用 FFmpeg 转码为 TS 格式
        // 3. 返回 TS 数据
        
        // 模拟生成 TS 片段数据
        let mock_ts_data = vec![0u8; 1024 * 1024]; // 1MB 模拟数据
        
        Ok(mock_ts_data)
    }
    
    async fn write_playlist_file(&self, stream_key: &str, playlist: &HlsPlaylist) -> StreamResult<()> {
        let playlist_path = PathBuf::from(&self.config.hls_segment_dir)
            .join(format!("{}.m3u8", stream_key));
        
        let playlist_content = playlist.generate_m3u8().await;
        
        fs::write(playlist_path, playlist_content).await
            .map_err(|e| StreamError::Io(e))?;
        
        Ok(())
    }
}

/// HLS 播放列表
struct HlsPlaylist {
    stream_key: String,
    segments: Vec<HlsSegment>,
    next_segment_number: u32,
    target_duration: u32,
    max_segments: u32,
    last_segment_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl HlsPlaylist {
    fn new(stream_key: String, config: &StorageConfig) -> Self {
        Self {
            stream_key,
            segments: Vec::new(),
            next_segment_number: 0,
            target_duration: config.hls_segment_duration,
            max_segments: config.hls_playlist_length,
            last_segment_time: None,
        }
    }
    
    async fn should_generate_segment(&self) -> bool {
        match self.last_segment_time {
            None => true, // 第一个片段
            Some(last_time) => {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(last_time);
                duration.num_seconds() >= self.target_duration as i64
            }
        }
    }
    
    async fn add_segment(&mut self, segment_name: String, duration: u32) {
        let segment = HlsSegment {
            name: segment_name,
            duration,
            sequence: self.next_segment_number,
        };
        
        self.segments.push(segment);
        self.next_segment_number += 1;
        self.last_segment_time = Some(chrono::Utc::now());
        
        // 保持播放列表长度
        while self.segments.len() > self.max_segments as usize {
            self.segments.remove(0);
        }
    }
    
    async fn generate_m3u8(&self) -> String {
        let mut m3u8 = String::new();
        
        // M3U8 头部
        m3u8.push_str("#EXTM3U\n");
        m3u8.push_str("#EXT-X-VERSION:3\n");
        m3u8.push_str(&format!("#EXT-X-TARGETDURATION:{}\n", self.target_duration));
        
        if let Some(first_segment) = self.segments.first() {
            m3u8.push_str(&format!("#EXT-X-MEDIA-SEQUENCE:{}\n", first_segment.sequence));
        }
        
        // 片段列表
        for segment in &self.segments {
            m3u8.push_str(&format!("#EXTINF:{}.0,\n", segment.duration));
            m3u8.push_str(&format!("{}\n", segment.name));
        }
        
        m3u8
    }
}

/// HLS 片段信息
#[derive(Debug, Clone)]
struct HlsSegment {
    name: String,
    duration: u32,
    sequence: u32,
}
