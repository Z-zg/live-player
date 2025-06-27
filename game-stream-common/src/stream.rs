use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use bytes::Bytes;
use crate::{StreamInfo, StreamStatus, StreamResult, ViewerConnection};

/// 媒体数据包类型
#[derive(Debug, Clone)]
pub enum MediaPacket {
    Video {
        data: Bytes,
        timestamp: u64,
        is_keyframe: bool,
    },
    Audio {
        data: Bytes,
        timestamp: u64,
    },
    Metadata {
        data: Bytes,
    },
}

/// 流管理器 - 管理所有活跃的直播流
#[derive(Debug)]
pub struct StreamManager {
    streams: Arc<RwLock<HashMap<String, Arc<LiveStream>>>>,
}

impl StreamManager {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建新的直播流
    pub async fn create_stream(&self, stream_key: String, info: StreamInfo) -> StreamResult<Arc<LiveStream>> {
        let stream = Arc::new(LiveStream::new(stream_key.clone(), info));
        
        let mut streams = self.streams.write().await;
        streams.insert(stream_key, stream.clone());
        
        Ok(stream)
    }

    /// 获取直播流
    pub async fn get_stream(&self, stream_key: &str) -> Option<Arc<LiveStream>> {
        let streams = self.streams.read().await;
        streams.get(stream_key).cloned()
    }

    /// 移除直播流
    pub async fn remove_stream(&self, stream_key: &str) -> Option<Arc<LiveStream>> {
        let mut streams = self.streams.write().await;
        streams.remove(stream_key)
    }

    /// 获取所有活跃的流
    pub async fn list_streams(&self) -> Vec<(String, Arc<LiveStream>)> {
        let streams = self.streams.read().await;
        streams.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}

/// 单个直播流
#[derive(Debug)]
pub struct LiveStream {
    pub stream_key: String,
    pub info: Arc<RwLock<StreamInfo>>,
    pub status: Arc<RwLock<StreamStatus>>,
    pub viewers: Arc<RwLock<HashMap<Uuid, ViewerConnection>>>,
    
    // 媒体数据分发通道
    media_sender: mpsc::UnboundedSender<MediaPacket>,
    media_receivers: Arc<RwLock<Vec<mpsc::UnboundedReceiver<MediaPacket>>>>,
}

impl LiveStream {
    pub fn new(stream_key: String, info: StreamInfo) -> Self {
        let (media_sender, _) = mpsc::unbounded_channel();
        
        Self {
            stream_key,
            info: Arc::new(RwLock::new(info)),
            status: Arc::new(RwLock::new(StreamStatus::Starting)),
            viewers: Arc::new(RwLock::new(HashMap::new())),
            media_sender,
            media_receivers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 发送媒体数据包
    pub async fn send_media_packet(&self, packet: MediaPacket) -> StreamResult<()> {
        self.media_sender.send(packet)
            .map_err(|_| crate::StreamError::Internal("Failed to send media packet".to_string()))?;
        Ok(())
    }

    /// 添加观看者
    pub async fn add_viewer(&self, viewer: ViewerConnection) -> mpsc::UnboundedReceiver<MediaPacket> {
        let (_sender, receiver) = mpsc::unbounded_channel();

        // 添加观看者信息
        {
            let mut viewers = self.viewers.write().await;
            viewers.insert(viewer.id, viewer);
        }

        // 更新观看者数量
        {
            let mut info = self.info.write().await;
            info.viewer_count = self.viewers.read().await.len() as u32;
        }

        receiver
    }

    /// 移除观看者
    pub async fn remove_viewer(&self, viewer_id: Uuid) {
        let mut viewers = self.viewers.write().await;
        viewers.remove(&viewer_id);
        
        // 更新观看者数量
        let mut info = self.info.write().await;
        info.viewer_count = viewers.len() as u32;
    }

    /// 设置流状态
    pub async fn set_status(&self, status: StreamStatus) {
        let mut current_status = self.status.write().await;
        *current_status = status;
        
        // 如果流状态变为 Live，更新信息中的 is_live 字段
        if matches!(*current_status, StreamStatus::Live) {
            let mut info = self.info.write().await;
            info.is_live = true;
        } else if matches!(*current_status, StreamStatus::Stopped | StreamStatus::Error(_)) {
            let mut info = self.info.write().await;
            info.is_live = false;
        }
    }

    /// 获取流状态
    pub async fn get_status(&self) -> StreamStatus {
        self.status.read().await.clone()
    }

    /// 获取流信息
    pub async fn get_info(&self) -> StreamInfo {
        self.info.read().await.clone()
    }

    /// 获取观看者数量
    pub async fn get_viewer_count(&self) -> u32 {
        self.viewers.read().await.len() as u32
    }
}

/// 媒体数据缓冲区 - 用于缓存关键帧等
#[derive(Debug)]
pub struct MediaBuffer {
    video_keyframe: Option<MediaPacket>,
    audio_config: Option<MediaPacket>,
    metadata: Option<MediaPacket>,
}

impl MediaBuffer {
    pub fn new() -> Self {
        Self {
            video_keyframe: None,
            audio_config: None,
            metadata: None,
        }
    }

    /// 添加媒体包到缓冲区
    pub fn add_packet(&mut self, packet: MediaPacket) {
        match &packet {
            MediaPacket::Video { is_keyframe, .. } => {
                if *is_keyframe {
                    self.video_keyframe = Some(packet);
                }
            }
            MediaPacket::Audio { .. } => {
                // 可以在这里缓存音频配置包
            }
            MediaPacket::Metadata { .. } => {
                self.metadata = Some(packet);
            }
        }
    }

    /// 获取初始化包（给新连接的观看者）
    pub fn get_init_packets(&self) -> Vec<MediaPacket> {
        let mut packets = Vec::new();
        
        if let Some(metadata) = &self.metadata {
            packets.push(metadata.clone());
        }
        
        if let Some(audio_config) = &self.audio_config {
            packets.push(audio_config.clone());
        }
        
        if let Some(keyframe) = &self.video_keyframe {
            packets.push(keyframe.clone());
        }
        
        packets
    }
}
