use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error};

use game_stream_common::{ServerConfig, StreamManager, StreamResult};
use crate::rtmp::RtmpServer;
use crate::webrtc::WebRtcServer;
use crate::http::HttpServer;
use crate::auth::AuthManager;
use crate::hls::HlsManager;

/// 主要的流媒体服务器
pub struct StreamingServer {
    config: ServerConfig,
    stream_manager: Arc<StreamManager>,
    auth_manager: Arc<AuthManager>,
    hls_manager: Arc<HlsManager>,
    rtmp_server: RtmpServer,
    webrtc_server: WebRtcServer,
    http_server: HttpServer,
}

impl StreamingServer {
    pub async fn new(config: ServerConfig) -> Result<Self> {
        info!("Initializing streaming server...");
        
        // 创建共享组件
        let stream_manager = Arc::new(StreamManager::new());
        let auth_manager = Arc::new(AuthManager::new(&config.auth));
        let hls_manager = Arc::new(HlsManager::new(&config.storage).await?);
        
        // 创建各个服务器组件
        let rtmp_server = RtmpServer::new(
            &config.rtmp,
            stream_manager.clone(),
            auth_manager.clone(),
        ).await?;
        
        let webrtc_server = WebRtcServer::new(
            &config.webrtc,
            stream_manager.clone(),
        ).await?;
        
        let http_server = HttpServer::new(
            &config.http,
            stream_manager.clone(),
            webrtc_server.get_signaling_handler(),
            hls_manager.clone(),
        ).await?;
        
        Ok(Self {
            config,
            stream_manager,
            auth_manager,
            hls_manager,
            rtmp_server,
            webrtc_server,
            http_server,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting streaming server...");
        
        // 启动各个服务器组件
        let rtmp_handle = {
            let mut rtmp_server = self.rtmp_server.clone();
            tokio::spawn(async move {
                if let Err(e) = rtmp_server.start().await {
                    error!("RTMP server error: {}", e);
                }
            })
        };
        
        let webrtc_handle = {
            let mut webrtc_server = self.webrtc_server.clone();
            tokio::spawn(async move {
                if let Err(e) = webrtc_server.start().await {
                    error!("WebRTC server error: {}", e);
                }
            })
        };
        
        let http_handle = {
            let mut http_server = self.http_server.clone();
            tokio::spawn(async move {
                if let Err(e) = http_server.start().await {
                    error!("HTTP server error: {}", e);
                }
            })
        };
        
        let hls_handle = {
            let hls_manager = self.hls_manager.clone();
            let stream_manager = self.stream_manager.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::start_hls_processing(hls_manager, stream_manager).await {
                    error!("HLS processing error: {}", e);
                }
            })
        };
        
        info!("All server components started");
        info!("RTMP server listening on: {}:{}", self.config.rtmp.bind_addr, self.config.rtmp.port);
        info!("HTTP server listening on: {}:{}", self.config.http.bind_addr, self.config.http.port);
        
        // 等待任何一个服务器组件完成或出错
        tokio::select! {
            result = rtmp_handle => {
                match result {
                    Ok(_) => info!("RTMP server completed"),
                    Err(e) => error!("RTMP server task failed: {}", e),
                }
            }
            result = webrtc_handle => {
                match result {
                    Ok(_) => info!("WebRTC server completed"),
                    Err(e) => error!("WebRTC server task failed: {}", e),
                }
            }
            result = http_handle => {
                match result {
                    Ok(_) => info!("HTTP server completed"),
                    Err(e) => error!("HTTP server task failed: {}", e),
                }
            }
            result = hls_handle => {
                match result {
                    Ok(_) => info!("HLS processing completed"),
                    Err(e) => error!("HLS processing task failed: {}", e),
                }
            }
        }
        
        Ok(())
    }
    
    async fn start_hls_processing(
        hls_manager: Arc<HlsManager>,
        stream_manager: Arc<StreamManager>,
    ) -> StreamResult<()> {
        info!("Starting HLS processing...");
        
        loop {
            // 获取所有活跃的流
            let streams = stream_manager.list_streams().await;
            
            for (stream_key, stream) in streams {
                // 为每个流生成HLS片段
                if let Err(e) = hls_manager.process_stream(&stream_key, &stream).await {
                    error!("Failed to process HLS for stream {}: {}", stream_key, e);
                }
            }
            
            // 等待一段时间再处理下一轮
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

impl Drop for StreamingServer {
    fn drop(&mut self) {
        info!("Streaming server shutting down...");
    }
}
