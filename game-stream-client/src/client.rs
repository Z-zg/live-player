use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, warn, error};
use std::time::Duration;

use game_stream_common::{ClientConfig, StreamError, StreamResult};
use crate::capture::{CaptureManager, CapturedFrame};
use crate::encoder::EncoderManager;
use crate::pusher::PusherManager;

/// 主要的流媒体客户端
pub struct StreamingClient {
    config: ClientConfig,
    capture_manager: CaptureManager,
    encoder_manager: EncoderManager,
    pusher_manager: PusherManager,
}

impl StreamingClient {
    pub async fn new(config: ClientConfig) -> Result<Self> {
        info!("Initializing streaming client...");
        
        // 初始化捕获管理器
        let capture_manager = CaptureManager::new(&config.capture).await?;
        
        // 初始化编码管理器
        let encoder_manager = EncoderManager::new(&config.encoding).await?;
        
        // 初始化推流管理器
        let pusher_manager = PusherManager::new(&config.server, &config.network).await?;
        
        Ok(Self {
            config,
            capture_manager,
            encoder_manager,
            pusher_manager,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting streaming client...");
        
        let mut reconnect_attempts = 0;
        
        loop {
            match self.run_streaming_loop().await {
                Ok(_) => {
                    info!("Streaming completed successfully");
                    break;
                }
                Err(e) => {
                    error!("Streaming error: {}", e);
                    
                    if !self.config.stream.auto_reconnect {
                        return Err(e.into());
                    }
                    
                    reconnect_attempts += 1;
                    if reconnect_attempts > self.config.stream.max_reconnect_attempts {
                        error!("Max reconnection attempts reached, giving up");
                        return Err(e.into());
                    }
                    
                    warn!("Attempting to reconnect in {} seconds... (attempt {}/{})", 
                          self.config.stream.reconnect_interval,
                          reconnect_attempts,
                          self.config.stream.max_reconnect_attempts);
                    
                    tokio::time::sleep(Duration::from_secs(self.config.stream.reconnect_interval)).await;
                }
            }
        }
        
        Ok(())
    }
    
    async fn run_streaming_loop(&mut self) -> StreamResult<()> {
        info!("Starting streaming loop...");
        
        // 创建数据流通道
        let (frame_tx, frame_rx) = mpsc::unbounded_channel::<CapturedFrame>();
        let (encoded_tx, encoded_rx) = mpsc::unbounded_channel::<game_stream_common::MediaPacket>();
        
        // 启动捕获任务
        let capture_handle = {
            let mut capture_manager = self.capture_manager.clone();
            tokio::spawn(async move {
                if let Err(e) = capture_manager.start_capture(frame_tx).await {
                    error!("Capture error: {}", e);
                }
            })
        };

        // 启动编码任务
        let encoding_handle = {
            // 重新创建编码管理器
            let mut encoder_manager = EncoderManager::new(&self.config.encoding).await
                .map_err(|e| StreamError::Internal(format!("Failed to create encoder: {}", e)))?;
            tokio::spawn(async move {
                if let Err(e) = encoder_manager.start_encoding(frame_rx, encoded_tx).await {
                    error!("Encoding error: {}", e);
                }
            })
        };

        // 启动推流任务
        let pushing_handle = {
            // 重新创建推流管理器
            let mut pusher_manager = PusherManager::new(&self.config.server, &self.config.network).await
                .map_err(|e| StreamError::Internal(format!("Failed to create pusher: {}", e)))?;
            tokio::spawn(async move {
                if let Err(e) = pusher_manager.start_pushing(encoded_rx).await {
                    error!("Pushing error: {}", e);
                }
            })
        };
        
        // 等待任何一个任务完成或出错
        tokio::select! {
            result = capture_handle => {
                match result {
                    Ok(_) => info!("Capture task completed"),
                    Err(e) => error!("Capture task failed: {}", e),
                }
            }
            result = encoding_handle => {
                match result {
                    Ok(_) => info!("Encoding task completed"),
                    Err(e) => error!("Encoding task failed: {}", e),
                }
            }
            result = pushing_handle => {
                match result {
                    Ok(_) => info!("Pushing task completed"),
                    Err(e) => error!("Pushing task failed: {}", e),
                }
            }
        }
        
        Ok(())
    }
}

impl Drop for StreamingClient {
    fn drop(&mut self) {
        info!("Streaming client shutting down...");
    }
}
