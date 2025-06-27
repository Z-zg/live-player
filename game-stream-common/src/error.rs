use thiserror::Error;

pub type StreamResult<T> = Result<T, StreamError>;

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("RTMP error: {0}")]
    Rtmp(String),
    
    #[error("WebRTC error: {0}")]
    WebRtc(String),
    
    #[error("Codec error: {0}")]
    Codec(String),
    
    #[error("Capture error: {0}")]
    Capture(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Stream not found: {0}")]
    StreamNotFound(String),
    
    #[error("Invalid stream key: {0}")]
    InvalidStreamKey(String),
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Internal error: {0}")]
    Internal(String),
}
