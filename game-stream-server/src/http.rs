use anyhow::Result;
use std::sync::Arc;
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum::extract::ws::{WebSocket, Message};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{info, error, debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use game_stream_common::{
    HttpServerConfig, StreamManager, WebRtcSignal, StreamInfo,
    StreamResult, StreamError
};
use crate::webrtc::WebRtcSignalingHandler;
use crate::hls::HlsManager;

/// HTTP 服务器
#[derive(Clone)]
pub struct HttpServer {
    config: HttpServerConfig,
    app_state: AppState,
}

#[derive(Clone)]
struct AppState {
    stream_manager: Arc<StreamManager>,
    webrtc_handler: Arc<WebRtcSignalingHandler>,
    hls_manager: Arc<HlsManager>,
}

impl HttpServer {
    pub async fn new(
        config: &HttpServerConfig,
        stream_manager: Arc<StreamManager>,
        webrtc_handler: Arc<WebRtcSignalingHandler>,
        hls_manager: Arc<HlsManager>,
    ) -> Result<Self> {
        info!("Initializing HTTP server...");
        
        let app_state = AppState {
            stream_manager,
            webrtc_handler,
            hls_manager,
        };
        
        Ok(Self {
            config: config.clone(),
            app_state,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        let bind_addr = format!("{}:{}", self.config.bind_addr, self.config.port);
        
        // 构建路由
        let app = self.build_router().await;
        
        info!("HTTP server listening on {}", bind_addr);
        
        // 启动服务器
        let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
    
    async fn build_router(&self) -> Router {
        let cors = if self.config.cors_enabled {
            CorsLayer::permissive()
        } else {
            CorsLayer::new()
        };
        
        Router::new()
            // API 路由
            .route("/api/streams", get(list_streams))
            .route("/api/streams/:stream_key", get(get_stream_info))
            .route("/api/streams/:stream_key/stats", get(get_stream_stats))
            
            // WebRTC 信令
            .route("/api/webrtc/signal", post(webrtc_signal))
            .route("/api/webrtc/ws", get(webrtc_websocket))
            
            // HLS 播放列表
            .route("/hls/:stream_key/playlist.m3u8", get(hls_playlist))
            .route("/hls/:stream_key/:segment", get(hls_segment))
            
            // 静态文件服务
            .nest_service("/", ServeDir::new(&self.config.static_dir))
            
            // 状态和中间件
            .with_state(self.app_state.clone())
            .layer(ServiceBuilder::new().layer(cors))
    }
}

// API 处理函数

/// 获取所有流列表
async fn list_streams(State(state): State<AppState>) -> Result<Json<Vec<StreamInfo>>, AppError> {
    let streams = state.stream_manager.list_streams().await;
    let stream_infos = futures::future::join_all(
        streams.into_iter().map(|(_, stream)| async move {
            stream.get_info().await
        })
    ).await;
    
    Ok(Json(stream_infos))
}

/// 获取特定流信息
async fn get_stream_info(
    Path(stream_key): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<StreamInfo>, AppError> {
    let stream = state.stream_manager.get_stream(&stream_key).await
        .ok_or(AppError::StreamNotFound(stream_key))?;
    
    let info = stream.get_info().await;
    Ok(Json(info))
}

/// 获取流统计信息
async fn get_stream_stats(
    Path(stream_key): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<StreamStats>, AppError> {
    let stream = state.stream_manager.get_stream(&stream_key).await
        .ok_or(AppError::StreamNotFound(stream_key))?;
    
    let stats = StreamStats {
        viewer_count: stream.get_viewer_count().await,
        status: stream.get_status().await,
        uptime: chrono::Utc::now().signed_duration_since(
            stream.get_info().await.created_at
        ).num_seconds(),
    };
    
    Ok(Json(stats))
}

/// WebRTC 信令处理 (HTTP POST)
async fn webrtc_signal(
    State(state): State<AppState>,
    Json(signal): Json<WebRtcSignal>,
) -> Result<Json<Option<WebRtcSignal>>, AppError> {
    debug!("Received WebRTC signal: {:?}", signal);
    
    match state.webrtc_handler.handle_signal(signal).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("WebRTC signal error: {}", e);
            Err(AppError::WebRtcError(e.to_string()))
        }
    }
}

/// WebRTC 信令处理 (WebSocket)
async fn webrtc_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_webrtc_websocket(socket, state))
}

async fn handle_webrtc_websocket(mut socket: WebSocket, state: AppState) {
    info!("New WebRTC WebSocket connection");
    
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<WebRtcSignal>(&text) {
                    Ok(signal) => {
                        debug!("Received WebRTC signal via WebSocket: {:?}", signal);
                        
                        match state.webrtc_handler.handle_signal(signal).await {
                            Ok(Some(response)) => {
                                if let Ok(response_text) = serde_json::to_string(&response) {
                                    if let Err(e) = socket.send(Message::Text(response_text)).await {
                                        error!("Failed to send WebSocket response: {}", e);
                                        break;
                                    }
                                }
                            }
                            Ok(None) => {
                                // 无需响应
                            }
                            Err(e) => {
                                error!("WebRTC signal error: {}", e);
                                let error_response = WebRtcSignal::Error {
                                    message: e.to_string(),
                                };
                                if let Ok(error_text) = serde_json::to_string(&error_response) {
                                    let _ = socket.send(Message::Text(error_text)).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse WebRTC signal: {}", e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebRTC WebSocket connection closed");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {
                // 忽略其他消息类型
            }
        }
    }
}

/// HLS 播放列表
async fn hls_playlist(
    Path(stream_key): Path<String>,
    State(state): State<AppState>,
) -> Result<String, AppError> {
    let playlist = state.hls_manager.get_playlist(&stream_key).await
        .map_err(|e| AppError::HlsError(e.to_string()))?;
    
    Ok(playlist)
}

/// HLS 片段
async fn hls_segment(
    Path((stream_key, segment)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Vec<u8>, AppError> {
    let segment_data = state.hls_manager.get_segment(&stream_key, &segment).await
        .map_err(|e| AppError::HlsError(e.to_string()))?;
    
    Ok(segment_data)
}

// 数据结构

#[derive(Serialize)]
struct StreamStats {
    viewer_count: u32,
    status: game_stream_common::StreamStatus,
    uptime: i64, // seconds
}

// 错误处理

#[derive(Debug)]
enum AppError {
    StreamNotFound(String),
    WebRtcError(String),
    HlsError(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::StreamNotFound(stream_key) => {
                (StatusCode::NOT_FOUND, format!("Stream not found: {}", stream_key))
            }
            AppError::WebRtcError(msg) => {
                (StatusCode::BAD_REQUEST, format!("WebRTC error: {}", msg))
            }
            AppError::HlsError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("HLS error: {}", msg))
            }
            AppError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", msg))
            }
        };
        
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
