use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 支持的推流协议类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StreamProtocol {
    Rtmp,
    Srt,
    Custom,
}

/// 支持的观看协议类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViewProtocol {
    Rtmp,
    Hls,
    Dash,
    WebRtc,
}

/// 流媒体信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub stream_id: Uuid,
    pub stream_key: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_live: bool,
    pub viewer_count: u32,
    pub video_config: VideoConfig,
    pub audio_config: AudioConfig,
}

/// 视频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32, // kbps
    pub codec: VideoCodec,
}

/// 音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u32,
    pub bitrate: u32, // kbps
    pub codec: AudioCodec,
}

/// 视频编码格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    H265,
    Vp8,
    Vp9,
    Av1,
}

/// 音频编码格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioCodec {
    Aac,
    Opus,
    Mp3,
    Pcm,
}

/// WebRTC 信令消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebRtcSignal {
    Offer {
        stream_key: String,
        sdp: String,
    },
    Answer {
        sdp: String,
    },
    IceCandidate {
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
    },
    Error {
        message: String,
    },
}

/// 流状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamStatus {
    Starting,
    Live,
    Stopping,
    Stopped,
    Error(String),
}

/// 客户端连接信息
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub id: Uuid,
    pub remote_addr: std::net::SocketAddr,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub protocol: StreamProtocol,
}

/// 观看者连接信息
#[derive(Debug, Clone)]
pub struct ViewerConnection {
    pub id: Uuid,
    pub remote_addr: std::net::SocketAddr,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub protocol: ViewProtocol,
    pub stream_key: String,
}
