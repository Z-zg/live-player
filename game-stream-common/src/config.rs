use serde::{Deserialize, Serialize};
use crate::protocol::{StreamProtocol, VideoCodec, AudioCodec};

/// 客户端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server: ServerEndpoint,
    pub stream: StreamConfig,
    pub capture: CaptureConfig,
    pub encoding: EncodingConfig,
    pub network: NetworkConfig,
}

/// 服务器端点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEndpoint {
    pub protocol: StreamProtocol,
    pub host: String,
    pub port: u16,
    pub stream_key: String,
    pub app_name: Option<String>, // For RTMP
}

/// 流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    pub title: Option<String>,
    pub description: Option<String>,
    pub auto_reconnect: bool,
    pub reconnect_interval: u64, // seconds
    pub max_reconnect_attempts: u32,
}

/// 捕获配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub video_source: VideoSource,
    pub audio_source: AudioSource,
    pub capture_cursor: bool,
}

/// 视频源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoSource {
    Screen {
        display_index: u32,
    },
    Window {
        window_title: String,
    },
    Region {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
}

/// 音频源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioSource {
    Default,
    Device {
        device_name: String,
    },
    Disabled,
}

/// 编码配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingConfig {
    pub video: VideoEncodingConfig,
    pub audio: AudioEncodingConfig,
    pub hardware_acceleration: bool,
}

/// 视频编码配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoEncodingConfig {
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32, // kbps
    pub keyframe_interval: u32, // seconds
    pub preset: String, // e.g., "ultrafast", "fast", "medium", "slow"
}

/// 音频编码配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEncodingConfig {
    pub codec: AudioCodec,
    pub sample_rate: u32,
    pub channels: u32,
    pub bitrate: u32, // kbps
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub connection_timeout: u64, // seconds
    pub read_timeout: u64, // seconds
    pub write_timeout: u64, // seconds
    pub buffer_size: usize,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub rtmp: RtmpServerConfig,
    pub webrtc: WebRtcServerConfig,
    pub http: HttpServerConfig,
    pub auth: AuthConfig,
    pub storage: StorageConfig,
}

/// RTMP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtmpServerConfig {
    pub bind_addr: String,
    pub port: u16,
    pub chunk_size: u32,
    pub max_connections: u32,
}

/// WebRTC 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcServerConfig {
    pub ice_servers: Vec<IceServerConfig>,
    pub dtls_cert_path: Option<String>,
    pub dtls_key_path: Option<String>,
}

/// ICE 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceServerConfig {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

/// HTTP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpServerConfig {
    pub bind_addr: String,
    pub port: u16,
    pub static_dir: String,
    pub cors_enabled: bool,
}

/// 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub valid_stream_keys: Vec<String>,
    pub jwt_secret: Option<String>,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub hls_segment_dir: String,
    pub hls_segment_duration: u32, // seconds
    pub hls_playlist_length: u32, // number of segments
    pub dash_segment_dir: String,
    pub dash_segment_duration: u32, // seconds
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server: ServerEndpoint {
                protocol: StreamProtocol::Rtmp,
                host: "localhost".to_string(),
                port: 1935,
                stream_key: "test_stream".to_string(),
                app_name: Some("live".to_string()),
            },
            stream: StreamConfig {
                title: None,
                description: None,
                auto_reconnect: true,
                reconnect_interval: 5,
                max_reconnect_attempts: 10,
            },
            capture: CaptureConfig {
                video_source: VideoSource::Screen { display_index: 0 },
                audio_source: AudioSource::Default,
                capture_cursor: true,
            },
            encoding: EncodingConfig {
                video: VideoEncodingConfig {
                    codec: VideoCodec::H264,
                    width: 1920,
                    height: 1080,
                    fps: 30,
                    bitrate: 2500,
                    keyframe_interval: 2,
                    preset: "fast".to_string(),
                },
                audio: AudioEncodingConfig {
                    codec: AudioCodec::Aac,
                    sample_rate: 44100,
                    channels: 2,
                    bitrate: 128,
                },
                hardware_acceleration: true,
            },
            network: NetworkConfig {
                connection_timeout: 10,
                read_timeout: 30,
                write_timeout: 30,
                buffer_size: 65536,
            },
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            rtmp: RtmpServerConfig {
                bind_addr: "0.0.0.0".to_string(),
                port: 1935,
                chunk_size: 4096,
                max_connections: 100,
            },
            webrtc: WebRtcServerConfig {
                ice_servers: vec![
                    IceServerConfig {
                        urls: vec!["stun:stun.l.google.com:19302".to_string()],
                        username: None,
                        credential: None,
                    }
                ],
                dtls_cert_path: None,
                dtls_key_path: None,
            },
            http: HttpServerConfig {
                bind_addr: "0.0.0.0".to_string(),
                port: 8080,
                static_dir: "./web".to_string(),
                cors_enabled: true,
            },
            auth: AuthConfig {
                enabled: false,
                valid_stream_keys: vec!["test_stream".to_string()],
                jwt_secret: None,
            },
            storage: StorageConfig {
                hls_segment_dir: "./hls".to_string(),
                hls_segment_duration: 6,
                hls_playlist_length: 10,
                dash_segment_dir: "./dash".to_string(),
                dash_segment_duration: 6,
            },
        }
    }
}
