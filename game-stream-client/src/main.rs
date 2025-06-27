use anyhow::Result;
use clap::Parser;
use tracing::{info, error};
use tracing_subscriber;

mod capture;
mod encoder;
mod pusher;
mod client;

use client::StreamingClient;
use game_stream_common::ClientConfig;

#[derive(Parser)]
#[command(name = "game-stream-client")]
#[command(about = "A high-performance game streaming client")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "client.toml")]
    config: String,
    
    /// Stream key
    #[arg(short, long)]
    stream_key: Option<String>,
    
    /// Server host
    #[arg(long)]
    host: Option<String>,
    
    /// Server port
    #[arg(long)]
    port: Option<u16>,
    
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
        .with_env_filter(format!("game_stream_client={},game_stream_common={}", log_level, log_level))
        .init();
    
    info!("Starting game streaming client...");
    
    // Load configuration
    let mut config = load_config(&args.config).unwrap_or_else(|_| {
        info!("Using default configuration");
        ClientConfig::default()
    });
    
    // Override config with command line arguments
    if let Some(stream_key) = args.stream_key {
        config.server.stream_key = stream_key;
    }
    if let Some(host) = args.host {
        config.server.host = host;
    }
    if let Some(port) = args.port {
        config.server.port = port;
    }
    
    info!("Configuration loaded: {:?}", config);
    
    // Create and start streaming client
    let mut client = StreamingClient::new(config).await?;
    
    // Handle Ctrl+C gracefully
    let client_handle = tokio::spawn(async move {
        if let Err(e) = client.start().await {
            error!("Streaming client error: {}", e);
        }
    });
    
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
        _ = client_handle => {
            info!("Client finished");
        }
    }
    
    info!("Game streaming client stopped");
    Ok(())
}

fn load_config(path: &str) -> Result<ClientConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: ClientConfig = toml::from_str(&content)?;
    Ok(config)
}
