//! Example: WebSocket Transport

use prism_mcp_rs::prelude::*;

#[cfg(feature = "websocket")]
use prism_mcp_rs::transport::WebSocketServerTransport;

#[tokio::main]
async fn main() -> McpResult<()> {
    let _server = McpServer::new(
        "websocket-transport-example".to_string(),
        "1.0.0".to_string(),
    );

    #[cfg(feature = "websocket")]
    {
        let _transport = WebSocketServerTransport::new("127.0.0.1:9000");
        println!("WebSocket Server configured on 127.0.0.1:9000");
        println!("Note: To actually run, server needs transport integration");
    }

    #[cfg(not(feature = "websocket"))]
    println!("WebSocket feature not enabled. Add 'websocket' to features in Cargo.toml");

    Ok(())
}
