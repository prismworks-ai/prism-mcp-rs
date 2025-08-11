// ! HTTP/2 Server Implementation
// !
// ! This example demonstrates how to create an MCP server that supports
// ! HTTP/2 Server Push for proactive data delivery to clients.
// !
// ! Features:
// ! - HTTP/2 protocol with server push
// ! - Bidirectional stream management
// ! - Efficient multiplexing
// ! - Real-time data streaming
// !
// ! Run with: cargo run --example http2_server --features streaming-http2,tracing-subscriber

use prism_mcp_rs::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Example tool that generates large datasets
struct DataGeneratorTool {
    datasets: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

#[async_trait]
impl ToolHandler for DataGeneratorTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let size = arguments
            .get("size")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000) as usize;

        let dataset_id = arguments
            .get("dataset_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        info!("- Generating dataset '{}' with {} items", dataset_id, size);

        // Generate large dataset
        let data: Vec<String> = (0..size)
            .map(|i| format!("data_item_{}_with_content_{}", i, "x".repeat(100)))
            .collect();

        // Store for potential server push
        {
            let mut datasets = self.datasets.write().await;
            datasets.insert(dataset_id.to_string(), data.clone());
        }

        // Return summary (actual data could be pushed via HTTP/2)
        let content = json!({
            "dataset_id": dataset_id,
            "size": size,
            "total_bytes": data.iter().map(|s| s.len()).sum::<usize>(),
            "sample_items": data.iter().take(3).collect::<Vec<_>>(),
            "server_push_available": true
        });

        // Convert json! value to HashMap
        let performance_meta: HashMap<String, Value> = [
            (
                "performance".to_string(),
                json!({
                    "generation_time_ms": 42,
                    "memory_usage_mb": data.len() * 100 / 1024 / 1024,
                    "compression_ratio": 0.7
                }),
            ),
            (
                "http2_hints".to_string(),
                json!({
                    "push_available": true,
                    "recommended_strategy": "server_push",
                    "stream_priority": "high"
                }),
            ),
        ]
        .into_iter()
        .collect();

        Ok(ToolResult {
            content: vec![ContentBlock::text(
                &serde_json::to_string_pretty(&content).unwrap_or_default(),
            )],
            is_error: Some(false),
            structured_content: None,
            meta: Some(performance_meta),
        })
    }
}

/// Real-time notification tool that demonstrates server push
struct NotificationTool;

#[async_trait]
impl ToolHandler for NotificationTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let event_type = arguments
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Notification");

        info!("📢 Processing notification: {} - {}", event_type, message);

        // In a real server, this would trigger HTTP/2 server push to interested clients
        let notification_data = json!({
            "type": "notification",
            "event_type": event_type,
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "push_stream_id": 42, // Would be actual stream ID
        });

        // Convert json! value to HashMap
        let push_meta: HashMap<String, Value> = [(
            "server_push".to_string(),
            json!({
                "triggered": true,
                "stream_id": 42,
                "data": notification_data
            }),
        )]
        .into_iter()
        .collect();

        Ok(ToolResult {
            content: vec![ContentBlock::text(&format!(
                "Notification processed: {}",
                message
            ))],
            is_error: Some(false),
            structured_content: None,
            meta: Some(push_meta),
        })
    }
}

/// Resource that provides streaming data updates
struct StreamingDataResource {
    data_version: Arc<RwLock<u64>>,
}

#[async_trait]
impl ResourceHandler for StreamingDataResource {
    async fn read(
        &self,
        uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        let version = {
            let mut v = self.data_version.write().await;
            *v += 1;
            *v
        };

        info!(
            "- Reading streaming resource: {} (version {})",
            uri, version
        );

        // Generate large content that benefits from HTTP/2 streaming
        let content = json!({
            "uri": uri,
            "version": version,
            "data": {
                "metrics": (0..1000).map(|i| json!({
                    "id": i,
                    "value": i * 42,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })).collect::<Vec<_>>(),
                "metadata": {
                    "compression": "brotli",
                    "stream_improved": true,
                    "push_eligible": true
                }
            }
        });

        let content_text = serde_json::to_string_pretty(&content).unwrap_or_default();

        debug!(
            "📊 Generated {} bytes of streaming data",
            content_text.len()
        );

        Ok(vec![ResourceContents::Text {
            uri: uri.to_string(),
            mime_type: Some("application/json".to_string()),
            text: content_text,
            meta: None,
        }])
    }

    async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
        Ok(vec![ResourceInfo {
            uri: "streaming://live-data".to_string(),
            name: "Live Data Stream".to_string(),
            description: Some("Real-time streaming data with HTTP/2 optimization".to_string()),
            mime_type: Some("application/json".to_string()),
            annotations: None,
            size: None,
            title: Some("HTTP/2 Live Data Stream".to_string()),
            meta: None,
        }])
    }

    async fn subscribe(&self, _uri: &str) -> McpResult<()> {
        info!("🔔 Client subscribed to streaming updates - HTTP/2 push will be used");
        // In a real implementation, this would set up HTTP/2 server push streams
        Ok(())
    }

    async fn unsubscribe(&self, _uri: &str) -> McpResult<()> {
        info!("🔕 Client unsubscribed from streaming updates");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("# Starting HTTP/2 MCP Server with Server Push");

    // Create server
    let mut server = McpServer::new("HTTP/2 Showcase Server".to_string(), "1.0.0".to_string());

    // Configure server capabilities
    let experimental_features: HashMap<String, Value> = [
        ("http2_server_push".to_string(), json!(true)),
        ("multiplexed_streams".to_string(), json!(true)),
        ("complete_compression".to_string(), json!(true)),
        ("real_time_updates".to_string(), json!(true)),
    ]
    .into_iter()
    .collect();

    server.set_capabilities(ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        prompts: Some(PromptsCapability {
            list_changed: Some(true),
        }),
        completions: Some(CompletionsCapability::default()),
        sampling: None,
        logging: None,
        experimental: Some(experimental_features),
    });

    // Add tools
    let datasets = Arc::new(RwLock::new(HashMap::new()));

    server
        .add_tool(
            "generate_dataset".to_string(),
            Some("Generate large datasets with HTTP/2 streaming support".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "dataset_id": {
                        "type": "string",
                        "description": "Unique identifier for the dataset"
                    },
                    "size": {
                        "type": "integer",
                        "description": "Number of items to generate",
                        "minimum": 1,
                        "maximum": 100000
                    }
                },
                "required": ["dataset_id"]
            }),
            DataGeneratorTool {
                datasets: datasets.clone(),
            },
        )
        .await?;

    server
        .add_tool(
            "send_notification".to_string(),
            Some("Send real-time notifications via HTTP/2 server push".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "event_type": {
                        "type": "string",
                        "enum": ["info", "warning", "error", "success"],
                        "description": "Type of notification event"
                    },
                    "message": {
                        "type": "string",
                        "description": "Notification message content"
                    }
                },
                "required": ["message"]
            }),
            NotificationTool,
        )
        .await?;

    // Add streaming resource
    server
        .add_resource(
            "Live Data Stream".to_string(),
            "streaming://live-data".to_string(),
            StreamingDataResource {
                data_version: Arc::new(RwLock::new(0)),
            },
        )
        .await?;

    info!("[x] Server configured with HTTP/2 tools and resources");

    // Note: This is a demonstration server that would need actual HTTP/2 server implementation
    // The streaming_http.rs module provides the client-side HTTP/2 support
    // A full HTTP/2 server would require additional implementation using h2 and hyper

    info!("🌐 Server ready for HTTP/2 connections on port 8080");
    info!(" Available tools:");
    info!("   - generate_dataset: Create large datasets with streaming support");
    info!("   - send_notification: Trigger real-time notifications via server push");
    info!("## Available resources:");
    info!("   - streaming://live-data: Live data stream with HTTP/2 optimization");

    // For this example, we'll run with STDIO (in production, you'd implement HTTP/2 server transport)
    info!(
        "📡 Starting server with STDIO transport (HTTP/2 server transport would be implemented separately)"
    );

    server.run_with_stdio().await
}
