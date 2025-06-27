use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{info, error, debug, warn};
use uuid::Uuid;

use game_stream_common::{
    RtmpServerConfig, StreamManager, StreamInfo, StreamStatus, MediaPacket,
    VideoConfig, AudioConfig, VideoCodec, AudioCodec, ClientConnection, StreamProtocol,
    StreamResult, StreamError
};
use crate::auth::AuthManager;

/// RTMP 服务器
#[derive(Clone)]
pub struct RtmpServer {
    config: RtmpServerConfig,
    stream_manager: Arc<StreamManager>,
    auth_manager: Arc<AuthManager>,
    connections: Arc<RwLock<HashMap<Uuid, RtmpConnection>>>,
}

impl RtmpServer {
    pub async fn new(
        config: &RtmpServerConfig,
        stream_manager: Arc<StreamManager>,
        auth_manager: Arc<AuthManager>,
    ) -> Result<Self> {
        info!("Initializing RTMP server...");
        
        Ok(Self {
            config: config.clone(),
            stream_manager,
            auth_manager,
            connections: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        let bind_addr = format!("{}:{}", self.config.bind_addr, self.config.port);
        let listener = TcpListener::bind(&bind_addr).await?;
        
        info!("RTMP server listening on {}", bind_addr);
        
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New RTMP connection from: {}", addr);
                    
                    let connection_id = Uuid::new_v4();
                    let connection = RtmpConnection::new(
                        connection_id,
                        stream,
                        addr,
                        self.stream_manager.clone(),
                        self.auth_manager.clone(),
                        self.config.clone(),
                    );
                    
                    // 存储连接
                    {
                        let mut connections = self.connections.write().await;
                        connections.insert(connection_id, connection.clone());
                    }
                    
                    // 处理连接
                    let connections_ref = self.connections.clone();
                    tokio::spawn(async move {
                        if let Err(e) = connection.handle().await {
                            error!("RTMP connection error: {}", e);
                        }
                        
                        // 清理连接
                        let mut connections = connections_ref.write().await;
                        connections.remove(&connection_id);
                        info!("RTMP connection {} closed", connection_id);
                    });
                }
                Err(e) => {
                    error!("Failed to accept RTMP connection: {}", e);
                }
            }
        }
    }
}

/// RTMP 连接处理器
#[derive(Clone)]
struct RtmpConnection {
    id: Uuid,
    stream: Arc<tokio::sync::Mutex<TcpStream>>,
    remote_addr: std::net::SocketAddr,
    stream_manager: Arc<StreamManager>,
    auth_manager: Arc<AuthManager>,
    config: RtmpServerConfig,
}

impl RtmpConnection {
    fn new(
        id: Uuid,
        stream: TcpStream,
        remote_addr: std::net::SocketAddr,
        stream_manager: Arc<StreamManager>,
        auth_manager: Arc<AuthManager>,
        config: RtmpServerConfig,
    ) -> Self {
        Self {
            id,
            stream: Arc::new(tokio::sync::Mutex::new(stream)),
            remote_addr,
            stream_manager,
            auth_manager,
            config,
        }
    }
    
    async fn handle(&self) -> StreamResult<()> {
        info!("Handling RTMP connection {}", self.id);
        
        // RTMP 握手
        self.perform_handshake().await?;
        
        // 处理 RTMP 消息
        self.process_messages().await?;
        
        Ok(())
    }
    
    async fn perform_handshake(&self) -> StreamResult<()> {
        debug!("Performing RTMP handshake for connection {}", self.id);
        
        // 实际的 RTMP 握手逻辑
        // 这里需要实现完整的 RTMP 握手协议
        // 包括 C0/S0, C1/S1, C2/S2 消息交换
        
        // 模拟握手过程
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        info!("RTMP handshake completed for connection {}", self.id);
        Ok(())
    }
    
    async fn process_messages(&self) -> StreamResult<()> {
        debug!("Processing RTMP messages for connection {}", self.id);
        
        let mut stream_key: Option<String> = None;
        let mut live_stream: Option<Arc<game_stream_common::LiveStream>> = None;
        
        // 模拟 RTMP 消息处理循环
        loop {
            // 读取 RTMP 消息
            match self.read_rtmp_message().await {
                Ok(message) => {
                    match message {
                        RtmpMessage::Connect { app_name } => {
                            info!("RTMP connect to app: {}", app_name);
                            self.send_connect_response().await?;
                        }
                        RtmpMessage::Publish { stream_key: key } => {
                            info!("RTMP publish stream: {}", key);
                            
                            // 验证流密钥
                            if !self.auth_manager.validate_stream_key(&key).await {
                                warn!("Invalid stream key: {}", key);
                                return Err(StreamError::Auth(format!("Invalid stream key: {}", key)));
                            }
                            
                            // 创建直播流
                            let stream_info = StreamInfo {
                                stream_id: Uuid::new_v4(),
                                stream_key: key.clone(),
                                title: None,
                                description: None,
                                created_at: chrono::Utc::now(),
                                is_live: false,
                                viewer_count: 0,
                                video_config: VideoConfig {
                                    width: 1920,
                                    height: 1080,
                                    fps: 30,
                                    bitrate: 2500,
                                    codec: VideoCodec::H264,
                                },
                                audio_config: AudioConfig {
                                    sample_rate: 44100,
                                    channels: 2,
                                    bitrate: 128,
                                    codec: AudioCodec::Aac,
                                },
                            };
                            
                            let stream = self.stream_manager.create_stream(key.clone(), stream_info).await?;
                            stream.set_status(StreamStatus::Live).await;
                            
                            stream_key = Some(key);
                            live_stream = Some(stream);
                            
                            self.send_publish_response().await?;
                        }
                        RtmpMessage::VideoData { data, timestamp } => {
                            if let Some(stream) = &live_stream {
                                let is_keyframe = self.is_keyframe(&data);
                                let packet = MediaPacket::Video {
                                    data,
                                    timestamp,
                                    is_keyframe,
                                };
                                stream.send_media_packet(packet).await?;
                            }
                        }
                        RtmpMessage::AudioData { data, timestamp } => {
                            if let Some(stream) = &live_stream {
                                let packet = MediaPacket::Audio {
                                    data,
                                    timestamp,
                                };
                                stream.send_media_packet(packet).await?;
                            }
                        }
                        RtmpMessage::Disconnect => {
                            info!("RTMP client disconnected");
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read RTMP message: {}", e);
                    break;
                }
            }
        }
        
        // 清理流
        if let Some(key) = stream_key {
            if let Some(stream) = self.stream_manager.get_stream(&key).await {
                stream.set_status(StreamStatus::Stopped).await;
            }
            self.stream_manager.remove_stream(&key).await;
            info!("Stream {} stopped", key);
        }
        
        Ok(())
    }
    
    async fn read_rtmp_message(&self) -> StreamResult<RtmpMessage> {
        // 实际的 RTMP 消息读取逻辑
        // 这里需要解析 RTMP 协议的各种消息类型
        
        // 模拟消息读取
        tokio::time::sleep(tokio::time::Duration::from_millis(33)).await; // ~30fps
        
        // 模拟不同类型的消息
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let message_type = rng.gen_range(0..4);
        
        match message_type {
            0 => Ok(RtmpMessage::Connect { app_name: "live".to_string() }),
            1 => Ok(RtmpMessage::Publish { stream_key: "test_stream".to_string() }),
            2 => Ok(RtmpMessage::VideoData { 
                data: bytes::Bytes::from(vec![0u8; 1024]), 
                timestamp: chrono::Utc::now().timestamp_millis() as u64 
            }),
            3 => Ok(RtmpMessage::AudioData { 
                data: bytes::Bytes::from(vec![0u8; 256]), 
                timestamp: chrono::Utc::now().timestamp_millis() as u64 
            }),
            _ => Ok(RtmpMessage::Disconnect),
        }
    }
    
    async fn send_connect_response(&self) -> StreamResult<()> {
        debug!("Sending RTMP connect response");
        // 实际的响应发送逻辑
        Ok(())
    }
    
    async fn send_publish_response(&self) -> StreamResult<()> {
        debug!("Sending RTMP publish response");
        // 实际的响应发送逻辑
        Ok(())
    }
    
    fn is_keyframe(&self, data: &bytes::Bytes) -> bool {
        // 简单的关键帧检测逻辑
        // 实际实现需要解析视频数据格式
        data.len() > 1000 // 简单假设大包是关键帧
    }
}

/// RTMP 消息类型
#[derive(Debug)]
enum RtmpMessage {
    Connect { app_name: String },
    Publish { stream_key: String },
    VideoData { data: bytes::Bytes, timestamp: u64 },
    AudioData { data: bytes::Bytes, timestamp: u64 },
    Disconnect,
}
