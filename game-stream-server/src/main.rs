use anyhow::Result;
use clap::Parser;
use tracing::{info, error};
use tracing_subscriber;

mod server;
mod rtmp;
mod webrtc;
mod http;
mod auth;
mod hls;

use server::StreamingServer;
use game_stream_common::ServerConfig;

#[derive(Parser)]
#[command(name = "game-stream-server")]
#[command(about = "A high-performance game streaming server")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "server.toml")]
    config: String,
    
    /// RTMP bind port
    #[arg(long)]
    rtmp_port: Option<u16>,
    
    /// HTTP bind port
    #[arg(long)]
    http_port: Option<u16>,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("game_stream_server={},game_stream_common={}", log_level, log_level))
        .init();
    
    info!("Starting game streaming server...");
    
    // Load configuration
    let mut config = load_config(&args.config).unwrap_or_else(|_| {
        info!("Using default configuration");
        ServerConfig::default()
    });
    
    // Override config with command line arguments
    if let Some(rtmp_port) = args.rtmp_port {
        config.rtmp.port = rtmp_port;
    }
    if let Some(http_port) = args.http_port {
        config.http.port = http_port;
    }
    
    info!("Configuration loaded: {:?}", config);
    
    // Create and start streaming server
    let mut server = StreamingServer::new(config).await?;
    
    // Handle Ctrl+C gracefully
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            error!("Streaming server error: {}", e);
        }
    });
    
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
        _ = server_handle => {
            info!("Server finished");
        }
    }
    
    info!("Game streaming server stopped");
    Ok(())
}

fn load_config(path: &str) -> Result<ServerConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: ServerConfig = toml::from_str(&content)?;
    Ok(config)
}
