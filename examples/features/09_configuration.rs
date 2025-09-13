//! Example: Configuration Management

use prism_mcp_rs::prelude::*;
use prism_mcp_rs::server::ServerConfig;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    server: ServerSettings,
    features: Features,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerSettings {
    name: String,
    version: String,
    port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct Features {
    enable_tools: bool,
    enable_resources: bool,
    enable_prompts: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerSettings {
                name: "config-example".to_string(),
                version: "1.0.0".to_string(),
                port: 8080,
            },
            features: Features {
                enable_tools: true,
                enable_resources: true,
                enable_prompts: true,
            },
        }
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Load or create config
    let config = if let Ok(content) = fs::read_to_string("config.json") {
        serde_json::from_str::<AppConfig>(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    };

    println!("Loaded config: {:?}", config);

    // Create server with configuration
    let server_config = ServerConfig {
        validate_requests: true,
        enable_logging: false,
        max_concurrent_requests: 100,
        request_timeout_ms: 30000,
    };

    let _server = McpServer::with_config(config.server.name, config.server.version, server_config);

    println!(
        "Configuration example server created on port {}",
        config.server.port
    );
    Ok(())
}
