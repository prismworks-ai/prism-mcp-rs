//! Example 02: Resources API (Working Version)
//! Demonstrates resource handling with the actual API

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

/// Simple text resource handler
struct TextResourceHandler;

#[async_trait]
impl ResourceHandler for TextResourceHandler {
    async fn read(
        &self,
        uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        // Return text content
        Ok(vec![ResourceContents::Text {
            uri: uri.to_string(),
            mime_type: Some("text/plain".to_string()),
            text: format!("Content for resource: {}", uri),
            meta: None,
        }])
    }

    async fn list(&self) -> McpResult<Vec<prism_mcp_rs::protocol::types::Resource>> {
        Ok(vec![prism_mcp_rs::protocol::types::Resource {
            uri: "text://example.txt".to_string(),
            name: "example.txt".to_string(),
            description: Some("Example text resource".to_string()),
            mime_type: Some("text/plain".to_string()),
            annotations: None,
            size: None,
            icons: None,
            title: None,
            meta: None,
        }])
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handler = TextResourceHandler;

    // Test reading
    let content = handler.read("text://example.txt", &HashMap::new()).await?;
    println!("Read content: {:?}", content);

    // Test listing
    let resources = handler.list().await?;
    println!("Available resources: {:?}", resources);

    Ok(())
}
