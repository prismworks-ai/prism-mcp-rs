//! Example: HTTP Transport

use prism_mcp_rs::prelude::*;

#[cfg(feature = "http")]
use prism_mcp_rs::transport::HttpServerTransport;

#[tokio::main]
async fn main() -> McpResult<()> {
    let server = McpServer::new("http-transport-example".to_string(), "1.0.0".to_string());

    #[cfg(feature = "http")]
    {
        let transport = HttpServerTransport::new("127.0.0.1:8080");
        println!("HTTP Server configured on 127.0.0.1:8080");
        println!("Note: To actually run, server needs transport integration");
    }

    #[cfg(not(feature = "http"))]
    println!("HTTP feature not enabled. Add 'http' to features in Cargo.toml");

    Ok(())
}
