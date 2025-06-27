use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, error, debug};

use game_stream_common::{
    EncodingConfig, MediaPacket, StreamResult, StreamError,
    VideoFrame, AudioFrame, VideoPixelFormat, AudioSampleFormat,
    EncoderFactory, VideoEncoderConfig, AudioEncoderConfig,
    VideoEncoder, AudioEncoder, VideoCodec, AudioCodec
};
use crate::capture::{CapturedFrame, FrameType};

/// 编码管理器
pub struct EncoderManager {
    config: EncodingConfig,
    video_encoder: Option<Box<dyn VideoEncoder>>,
    audio_encoder: Option<Box<dyn AudioEncoder>>,
}

impl EncoderManager {
    pub async fn new(config: &EncodingConfig) -> Result<Self> {
        info!("Initializing encoder manager...");
        
        // 创建视频编码器
        let video_encoder_config = VideoEncoderConfig {
            codec: config.video.codec.clone(),
            width: config.video.width,
            height: config.video.height,
            fps: config.video.fps,
            bitrate: config.video.bitrate,
            keyframe_interval: config.video.keyframe_interval,
            preset: config.video.preset.clone(),
        };
        
        let video_encoder = EncoderFactory::create_video_encoder(video_encoder_config)
            .map_err(|e| anyhow::anyhow!("Failed to create video encoder: {}", e))?;
        
        // 创建音频编码器
        let audio_encoder_config = AudioEncoderConfig {
            codec: config.audio.codec.clone(),
            sample_rate: config.audio.sample_rate,
            channels: config.audio.channels,
            bitrate: config.audio.bitrate,
        };
        
        let audio_encoder = EncoderFactory::create_audio_encoder(audio_encoder_config)
            .map_err(|e| anyhow::anyhow!("Failed to create audio encoder: {}", e))?;
        
        Ok(Self {
            config: config.clone(),
            video_encoder: Some(video_encoder),
            audio_encoder: Some(audio_encoder),
        })
    }
    
    pub async fn start_encoding(
        &mut self,
        mut frame_receiver: mpsc::UnboundedReceiver<CapturedFrame>,
        packet_sender: mpsc::UnboundedSender<MediaPacket>,
    ) -> StreamResult<()> {
        info!("Starting encoding...");
        
        while let Some(frame) = frame_receiver.recv().await {
            match self.encode_frame(frame).await {
                Ok(packets) => {
                    for packet in packets {
                        if let Err(_) = packet_sender.send(packet) {
                            error!("Failed to send encoded packet, receiver dropped");
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to encode frame: {}", e);
                    continue;
                }
            }
        }
        
        info!("Encoding finished");
        Ok(())
    }
    
    async fn encode_frame(&mut self, frame: CapturedFrame) -> StreamResult<Vec<MediaPacket>> {
        match frame.frame_type {
            FrameType::Video => self.encode_video_frame(frame).await,
            FrameType::Audio => self.encode_audio_frame(frame).await,
        }
    }
    
    async fn encode_video_frame(&mut self, frame: CapturedFrame) -> StreamResult<Vec<MediaPacket>> {
        debug!("Encoding video frame");
        
        let video_frame = VideoFrame {
            data: frame.data,
            width: frame.width.unwrap_or(1920),
            height: frame.height.unwrap_or(1080),
            format: VideoPixelFormat::Rgba32, // 假设捕获的是RGBA格式
            timestamp: frame.timestamp,
        };
        
        if let Some(encoder) = &mut self.video_encoder {
            let encoded_packets = encoder.encode_frame(&video_frame)?;
            
            let media_packets = encoded_packets.into_iter().map(|packet| {
                MediaPacket::Video {
                    data: packet.data,
                    timestamp: packet.timestamp,
                    is_keyframe: packet.is_keyframe,
                }
            }).collect();
            
            Ok(media_packets)
        } else {
            Err(StreamError::Codec("Video encoder not initialized".to_string()))
        }
    }
    
    async fn encode_audio_frame(&mut self, frame: CapturedFrame) -> StreamResult<Vec<MediaPacket>> {
        debug!("Encoding audio frame");
        
        let audio_frame = AudioFrame {
            data: frame.data,
            sample_rate: self.config.audio.sample_rate,
            channels: self.config.audio.channels,
            format: AudioSampleFormat::S16, // 假设捕获的是16位采样
            timestamp: frame.timestamp,
        };
        
        if let Some(encoder) = &mut self.audio_encoder {
            let encoded_packets = encoder.encode_frame(&audio_frame)?;
            
            let media_packets = encoded_packets.into_iter().map(|packet| {
                MediaPacket::Audio {
                    data: packet.data,
                    timestamp: packet.timestamp,
                }
            }).collect();
            
            Ok(media_packets)
        } else {
            Err(StreamError::Codec("Audio encoder not initialized".to_string()))
        }
    }
}

// 注意：EncoderManager 不实现 Clone，因为编码器状态不应该被复制
