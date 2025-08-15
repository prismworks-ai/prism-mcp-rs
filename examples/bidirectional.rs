//! Example demonstrating bidirectional communication
//!
//! This example shows how a server can send requests to a client,
//! enabling bidirectional communication patterns like sampling requests,
//! elicitation, and root directory access.

use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use async_trait::async_trait;

/// Mock client handler that simulates client responses
struct MockClientHandler;

impl MockClientHandler {
    async fn handle_server_request(&self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        match request.method.as_str() {
            "sampling/createMessage" => {
                // Simulate LLM response
                let result = CreateMessageResult {
                    role: "assistant".to_string(),
                    content: vec![Content::text("This is a simulated LLM response.")],
                    model: "mock-model".to_string(),
                    meta: None,
                };
                JsonRpcResponse::success(request.id, result)
            }
            "roots/list" => {
                // Simulate root directories
                let result = ListRootsResult {
                    roots: vec![
                        RootInfo {
                            uri: "file:///home/user/project".to_string(),
                            name: Some("Project Directory".to_string()),
                            title: None,
                            meta: None,
                        },
                        RootInfo {
                            uri: "file:///home/user/documents".to_string(),
                            name: Some("Documents".to_string()),
                            title: None,
                            meta: None,
                        },
                    ],
                    meta: None,
                };
                JsonRpcResponse::success(request.id, result)
            }
            "elicitation/create" => {
                // Simulate user input
                let result = ElicitResult {
                    response: "User provided input".to_string(),
                    meta: None,
                };
                JsonRpcResponse::success(request.id, result)
            }
            _ => {
                Err(McpError::MethodNotFound(format!("Unknown method: {}", request.method)))
            }
        }
    }
}

/// Tool that demonstrates server-to-client requests
struct BidirectionalTool {
    client_handler: MockClientHandler,
}

#[async_trait]
impl ToolHandler for BidirectionalTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("sample");

        match action {
            "sample" => {
                // Create a sampling request to send to client
                let request = JsonRpcRequest::new(
                    json!(100),
                    "sampling/createMessage".to_string(),
                    Some(CreateMessageParams {
                        messages: vec![Message {
                            role: "user".to_string(),
                            content: vec![Content::text("What is 2+2?")],
                            priority: None,
                            meta: None,
                        }],
                        model_preferences: None,
                        system_prompt: None,
                        include_context: None,
                        temperature: None,
                        max_tokens: Some(100),
                        stop_sequences: None,
                        metadata: None,
                        meta: None,
                    }),
                )?;

                // Simulate sending to client and getting response
                let response = self.client_handler.handle_server_request(request).await?;
                
                Ok(ToolResult {
                    content: vec![Content::text(format!("Client response: {:?}", response))],
                    is_error: Some(false),
                    structured_content: None,
                    meta: None,
                })
            }
            "list_roots" => {
                // Request root directories from client
                let request = JsonRpcRequest::new(
                    json!(101),
                    "roots/list".to_string(),
                    None::<Value>,
                )?;

                let response = self.client_handler.handle_server_request(request).await?;
                
                Ok(ToolResult {
                    content: vec![Content::text(format!("Available roots: {:?}", response))],
                    is_error: Some(false),
                    structured_content: None,
                    meta: None,
                })
            }
            "elicit" => {
                // Request user input via elicitation
                let request = JsonRpcRequest::new(
                    json!(102),
                    "elicitation/create".to_string(),
                    Some(ElicitParams {
                        prompt: "Please enter your name:".to_string(),
                        elicit_type: ElicitationType::Text,
                        options: None,
                        default: None,
                        meta: None,
                    }),
                )?;

                let response = self.client_handler.handle_server_request(request).await?;
                
                Ok(ToolResult {
                    content: vec![Content::text(format!("User input: {:?}", response))],
                    is_error: Some(false),
                    structured_content: None,
                    meta: None,
                })
            }
            _ => {
                Ok(ToolResult {
                    content: vec![Content::text(format!("Unknown action: {}", action))],
                    is_error: Some(true),
                    structured_content: None,
                    meta: None,
                })
            }
        }
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🔄 Bidirectional Communication Example");
    println!("======================================\n");

    // Create server
    let mut server = McpServer::new(
        "bidirectional-server".to_string(),
        "1.0.0".to_string(),
    );

    // Configure capabilities to support bidirectional features
    server.set_capabilities(ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: None,
        prompts: None,
        sampling: Some(SamplingCapability {}),  // Enable sampling capability
        logging: None,
        completions: None,
        experimental: None,
    });

    println!("✅ Server configured with bidirectional capabilities\n");

    // Add bidirectional tool
    let client_handler = MockClientHandler;
    server
        .add_tool(
            "bidirectional",
            Some("Demonstrate server-to-client requests"),
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["sample", "list_roots", "elicit"],
                        "description": "Action to perform"
                    }
                },
                "required": ["action"]
            }),
            BidirectionalTool { client_handler },
        )
        .await?;

    println!("✅ Added bidirectional tool\n");

    // Demonstrate tool usage
    println!("📤 Testing bidirectional communication...\n");

    // Test sampling request
    println!("1️⃣ Testing LLM sampling request:");
    let sample_result = server
        .call_tool(
            "bidirectional",
            Some({
                let mut args = HashMap::new();
                args.insert("action".to_string(), json!("sample"));
                args
            }),
        )
        .await?;
    println!("   Result: {:?}\n", sample_result);

    // Test roots listing
    println!("2️⃣ Testing roots listing request:");
    let roots_result = server
        .call_tool(
            "bidirectional",
            Some({
                let mut args = HashMap::new();
                args.insert("action".to_string(), json!("list_roots"));
                args
            }),
        )
        .await?;
    println!("   Result: {:?}\n", roots_result);

    // Test elicitation
    println!("3️⃣ Testing elicitation request:");
    let elicit_result = server
        .call_tool(
            "bidirectional",
            Some({
                let mut args = HashMap::new();
                args.insert("action".to_string(), json!("elicit"));
                args
            }),
        )
        .await?;
    println!("   Result: {:?}\n", elicit_result);

    println!("✅ Bidirectional communication demonstration complete!");
    println!("\n📝 Summary:");
    println!("   - Server can request LLM sampling from client");
    println!("   - Server can request file system roots from client");
    println!("   - Server can request user input via elicitation");
    println!("   - All communication follows JSON-RPC protocol");

    Ok(())
}
