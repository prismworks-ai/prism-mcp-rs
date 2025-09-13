//! Example: Advanced Tool Features

use prism_mcp_rs::prelude::*;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
struct BatchProcessor;

#[async_trait]
impl ToolHandler for BatchProcessor {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let default_items = vec![];
        let items = arguments
            .get("items")
            .and_then(|v| v.as_array())
            .unwrap_or(&default_items);

        let batch_size = arguments
            .get("batch_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(3) as usize;

        let mut results = Vec::new();

        for (i, chunk) in items.chunks(batch_size).enumerate() {
            // Simulate processing
            sleep(Duration::from_millis(100)).await;

            for item in chunk {
                if let Some(s) = item.as_str() {
                    results.push(format!("Processed: {}", s.to_uppercase()));
                }
            }

            println!(
                "Processed batch {} of {}",
                i + 1,
                items.len().div_ceil(batch_size)
            );
        }

        Ok(ToolResult {
            content: vec![ContentBlock::text(results.join(", "))],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}

#[derive(Clone)]
struct DataTransformer;

#[async_trait]
impl ToolHandler for DataTransformer {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let data = arguments.get("data").cloned().unwrap_or(json!({}));
        let transform = arguments
            .get("transform")
            .and_then(|v| v.as_str())
            .unwrap_or("identity");

        let result = match transform {
            "uppercase" => {
                if let Some(s) = data.as_str() {
                    json!(s.to_uppercase())
                } else {
                    data
                }
            }
            "count" => {
                if let Some(arr) = data.as_array() {
                    json!(arr.len())
                } else {
                    json!(1)
                }
            }
            _ => data,
        };

        Ok(ToolResult {
            content: vec![ContentBlock::text(result.to_string())],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    let server = McpServer::new("advanced-tools-example".to_string(), "1.0.0".to_string());

    server
        .add_tool(
            "batch_process".to_string(),
            Some("Process items in batches".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "items": { "type": "array" },
                    "batch_size": { "type": "integer" }
                }
            }),
            BatchProcessor,
        )
        .await?;

    server
        .add_tool(
            "transform_data".to_string(),
            Some("Transform data in various ways".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "data": {},
                    "transform": { "type": "string" }
                }
            }),
            DataTransformer,
        )
        .await?;

    println!("Advanced tools example server created");
    Ok(())
}
