use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, error, debug, warn};
use std::time::Duration;

use game_stream_common::{
    ServerEndpoint, NetworkConfig, StreamProtocol, MediaPacket,
    StreamResult, StreamError
};

/// 推流管理器
pub struct PusherManager {
    server_config: ServerEndpoint,
    network_config: NetworkConfig,
    pusher: Option<StreamPusherEnum>,
}

/// 推流器枚举
#[derive(Clone)]
pub enum StreamPusherEnum {
    Rtmp(RtmpPusher),
    Srt(SrtPusher),
}

impl PusherManager {
    pub async fn new(server_config: &ServerEndpoint, network_config: &NetworkConfig) -> Result<Self> {
        info!("Initializing pusher manager...");

        let pusher = create_pusher(server_config, network_config).await?;

        Ok(Self {
            server_config: server_config.clone(),
            network_config: network_config.clone(),
            pusher: Some(pusher),
        })
    }
    
    pub async fn start_pushing(
        &mut self,
        mut packet_receiver: mpsc::UnboundedReceiver<MediaPacket>,
    ) -> StreamResult<()> {
        info!("Starting pushing...");
        
        // 连接到服务器
        if let Some(pusher) = &mut self.pusher {
            pusher.connect().await?;
            info!("Connected to streaming server");

            // 开始推流
            while let Some(packet) = packet_receiver.recv().await {
                match pusher.push_packet(packet).await {
                    Ok(_) => {
                        debug!("Packet pushed successfully");
                    }
                    Err(e) => {
                        error!("Failed to push packet: {}", e);

                        // 尝试重连
                        if let Err(reconnect_err) = pusher.reconnect().await {
                            error!("Failed to reconnect: {}", reconnect_err);
                            return Err(e);
                        }

                        warn!("Reconnected to server, continuing...");
                    }
                }
            }

            // 断开连接
            pusher.disconnect().await?;
            info!("Disconnected from streaming server");
        }
        
        Ok(())
    }
}

impl StreamPusherEnum {
    /// 连接到服务器
    pub async fn connect(&mut self) -> StreamResult<()> {
        match self {
            StreamPusherEnum::Rtmp(pusher) => pusher.connect().await,
            StreamPusherEnum::Srt(pusher) => pusher.connect().await,
        }
    }

    /// 推送媒体包
    pub async fn push_packet(&mut self, packet: MediaPacket) -> StreamResult<()> {
        match self {
            StreamPusherEnum::Rtmp(pusher) => pusher.push_packet(packet).await,
            StreamPusherEnum::Srt(pusher) => pusher.push_packet(packet).await,
        }
    }

    /// 重连到服务器
    pub async fn reconnect(&mut self) -> StreamResult<()> {
        match self {
            StreamPusherEnum::Rtmp(pusher) => pusher.reconnect().await,
            StreamPusherEnum::Srt(pusher) => pusher.reconnect().await,
        }
    }

    /// 断开连接
    pub async fn disconnect(&mut self) -> StreamResult<()> {
        match self {
            StreamPusherEnum::Rtmp(pusher) => pusher.disconnect().await,
            StreamPusherEnum::Srt(pusher) => pusher.disconnect().await,
        }
    }
}

/// 流推送器特征
pub trait StreamPusher: Send + Sync {
    /// 连接到服务器
    async fn connect(&mut self) -> StreamResult<()>;

    /// 推送媒体包
    async fn push_packet(&mut self, packet: MediaPacket) -> StreamResult<()>;

    /// 重连到服务器
    async fn reconnect(&mut self) -> StreamResult<()>;

    /// 断开连接
    async fn disconnect(&mut self) -> StreamResult<()>;
}

/// RTMP 推流器
#[derive(Clone)]
pub struct RtmpPusher {
    server_url: String,
    stream_key: String,
    app_name: String,
    network_config: NetworkConfig,
    connected: bool,
}

impl RtmpPusher {
    pub fn new(server_config: &ServerEndpoint, network_config: &NetworkConfig) -> Self {
        let server_url = format!("rtmp://{}:{}", server_config.host, server_config.port);
        let app_name = server_config.app_name.clone().unwrap_or_else(|| "live".to_string());
        
        Self {
            server_url,
            stream_key: server_config.stream_key.clone(),
            app_name,
            network_config: network_config.clone(),
            connected: false,
        }
    }
}

impl StreamPusher for RtmpPusher {
    async fn connect(&mut self) -> StreamResult<()> {
        info!("Connecting to RTMP server: {}/{}", self.server_url, self.app_name);
        
        // 实际的RTMP连接逻辑
        // 这里需要使用 rml_rtmp 库建立连接
        
        // 模拟连接过程
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        self.connected = true;
        info!("RTMP connection established");
        Ok(())
    }
    
    async fn push_packet(&mut self, packet: MediaPacket) -> StreamResult<()> {
        if !self.connected {
            return Err(StreamError::Network("Not connected to server".to_string()));
        }
        
        match packet {
            MediaPacket::Video { data, timestamp, is_keyframe } => {
                debug!("Pushing video packet: {} bytes, ts: {}, keyframe: {}", 
                       data.len(), timestamp, is_keyframe);
                
                // 实际的RTMP视频包发送逻辑
                // 这里需要将编码后的数据封装为FLV格式并通过RTMP发送
            }
            MediaPacket::Audio { data, timestamp } => {
                debug!("Pushing audio packet: {} bytes, ts: {}", data.len(), timestamp);
                
                // 实际的RTMP音频包发送逻辑
            }
            MediaPacket::Metadata { data } => {
                debug!("Pushing metadata packet: {} bytes", data.len());
                
                // 实际的RTMP元数据包发送逻辑
            }
        }
        
        Ok(())
    }
    
    async fn reconnect(&mut self) -> StreamResult<()> {
        info!("Reconnecting to RTMP server...");
        
        self.disconnect().await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.connect().await?;
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> StreamResult<()> {
        if self.connected {
            info!("Disconnecting from RTMP server");
            
            // 实际的RTMP断开连接逻辑
            
            self.connected = false;
            info!("RTMP connection closed");
        }
        
        Ok(())
    }
}

/// SRT 推流器 (未来扩展)
#[derive(Clone)]
pub struct SrtPusher {
    server_url: String,
    stream_key: String,
    network_config: NetworkConfig,
    connected: bool,
}

impl SrtPusher {
    pub fn new(server_config: &ServerEndpoint, network_config: &NetworkConfig) -> Self {
        let server_url = format!("srt://{}:{}", server_config.host, server_config.port);
        
        Self {
            server_url,
            stream_key: server_config.stream_key.clone(),
            network_config: network_config.clone(),
            connected: false,
        }
    }
}

impl StreamPusher for SrtPusher {
    async fn connect(&mut self) -> StreamResult<()> {
        info!("Connecting to SRT server: {}", self.server_url);
        
        // SRT连接逻辑 (待实现)
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        self.connected = true;
        info!("SRT connection established");
        Ok(())
    }
    
    async fn push_packet(&mut self, _packet: MediaPacket) -> StreamResult<()> {
        if !self.connected {
            return Err(StreamError::Network("Not connected to server".to_string()));
        }

        // SRT推流逻辑 (待实现)
        debug!("Pushing packet via SRT");
        Ok(())
    }
    
    async fn reconnect(&mut self) -> StreamResult<()> {
        self.disconnect().await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.connect().await?;
        Ok(())
    }
    
    async fn disconnect(&mut self) -> StreamResult<()> {
        if self.connected {
            info!("Disconnecting from SRT server");
            self.connected = false;
        }
        Ok(())
    }
}

/// 创建推流器
async fn create_pusher(
    server_config: &ServerEndpoint,
    network_config: &NetworkConfig,
) -> Result<StreamPusherEnum> {
    match server_config.protocol {
        StreamProtocol::Rtmp => {
            let pusher = RtmpPusher::new(server_config, network_config);
            Ok(StreamPusherEnum::Rtmp(pusher))
        }
        StreamProtocol::Srt => {
            let pusher = SrtPusher::new(server_config, network_config);
            Ok(StreamPusherEnum::Srt(pusher))
        }
        StreamProtocol::Custom => {
            Err(anyhow::anyhow!("Custom protocol not implemented yet"))
        }
    }
}
