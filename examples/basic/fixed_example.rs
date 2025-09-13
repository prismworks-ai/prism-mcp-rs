//! Fixed example that demonstrates correct API usage
//!
//! This is a minimal, working example that shows how to create
//! an MCP server with the correct API calls.

use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> McpResult<()> {
    println!("Creating MCP server with correct API usage");

    // Create a basic server - this is the correct, minimal approach
    let server = McpServer::new("fixed-example".to_string(), "1.0.0".to_string());

    println!(
        "Server created successfully: {} v{}",
        server.info().name,
        server.info().version
    );

    println!("\nThis example demonstrates the minimal, correct way to:");
    println!("✅ Import the prelude (use prism_mcp_rs::prelude::*)");
    println!("✅ Create an MCP server with proper types");
    println!("✅ Use the correct result type (McpResult<()>)");

    println!("\n📝 To extend this example, you can:");
    println!("   • Add tools using server.add_tool()");
    println!("   • Add resources using server.add_resource()");
    println!("   • Add prompts using server.add_prompt()");
    println!("   • Set up transports (HTTP/WebSocket/Stdio)");
    println!("   • See working examples in examples/*_working.rs");

    Ok(())
}
