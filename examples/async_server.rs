//! Example demonstrating async server request handling
//!
//! This example shows how to use the MCP server with async request handlers
//! and demonstrates the async nature of tool, resource, and prompt handlers.

use prism_mcp_rs::prelude::*;
use prism_mcp_rs::server::{McpServer, ServerBuilder};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

// Example async tool that performs a delayed calculation
struct AsyncCalculatorTool;

#[async_trait]
impl ToolHandler for AsyncCalculatorTool {
    async fn call(
        &self,
        arguments: HashMap<String, Value>,
    ) -> McpResult<ToolResult> {
        // Simulate async work like API call or database query
        println!("Starting async calculation...");
        sleep(Duration::from_millis(100)).await;
        
        let expression = arguments
            .get("expression")
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        
        // Simulate more async work
        sleep(Duration::from_millis(100)).await;
        
        let result = match expression {
            "2+2" => 4,
            "10*10" => 100,
            _ => 0,
        };
        
        println!("Calculation complete: {} = {}", expression, result);
        
        Ok(ToolResult {
            content: vec![ContentBlock::Text(TextContent {
                text: format!("Result: {}", result),
                annotations: None,
            })],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}

// Example async resource handler that fetches data
struct AsyncDataResource;

#[async_trait]
impl ResourceHandler for AsyncDataResource {
    async fn read(&self, uri: &str) -> McpResult<ResourceContents> {
        println!("Fetching resource: {}", uri);
        
        // Simulate async data fetching
        sleep(Duration::from_millis(200)).await;
        
        let data = match uri {
            "data://users" => json!({
                "users": [
                    {"id": 1, "name": "Alice"},
                    {"id": 2, "name": "Bob"},
                ]
            }),
            "data://config" => json!({
                "version": "1.0.0",
                "features": ["async", "tools", "resources"]
            }),
            _ => json!({"error": "Unknown resource"}),
        };
        
        println!("Resource fetched successfully");
        
        Ok(ResourceContents {
            uri: uri.to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(serde_json::to_string_pretty(&data).unwrap()),
            blob: None,
        })
    }
}

// Example async prompt handler
struct AsyncPromptGenerator;

#[async_trait]
impl PromptHandler for AsyncPromptGenerator {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<PromptResult> {
        println!("Generating prompt...");
        
        // Simulate async prompt generation
        sleep(Duration::from_millis(150)).await;
        
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("User");
        
        let messages = vec![
            PromptMessage {
                role: Role::System,
                content: "You are a helpful assistant.".into(),
            },
            PromptMessage {
                role: Role::User,
                content: format!("Hello, my name is {}", name).into(),
            },
            PromptMessage {
                role: Role::Assistant,
                content: format!("Nice to meet you, {}! How can I help you today?", name).into(),
            },
        ];
        
        println!("Prompt generated for: {}", name);
        
        Ok(PromptResult {
            description: Some(format!("Personalized greeting for {}", name)),
            messages,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Async MCP Server Example\n");
    
    // Create server with async handlers
    let server = ServerBuilder::new()
        .name("async-example-server")
        .version("1.0.0")
        .with_tools()
        .with_resources()
        .with_prompts()
        .build();
    
    // Register handlers using the correct API
    println!("Registering async handlers...");
    
    // Add async tool
    server.add_tool(
        "async_calculator",
        Some("Performs async calculations"),
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Mathematical expression to evaluate"
                }
            },
            "required": ["expression"]
        }),
        AsyncCalculatorTool,
    ).await?;
    
    // Add async resources  
    server.add_resource(
        "User Data".to_string(),
        "data://users".to_string(),
        AsyncDataResource,
    ).await?;
    
    server.add_resource(
        "Configuration".to_string(),
        "data://config".to_string(),
        AsyncDataResource,
    ).await?;
    
    // Add async prompt
    let prompt_info = PromptInfo {
        name: "async_greeting".to_string(),
        description: Some("Generates personalized greetings asynchronously".to_string()),
        arguments: Some(vec![
            PromptArgument {
                name: "name".to_string(),
                description: Some("Name of the person to greet".to_string()),
                required: Some(true),
            },
        ]),
    };
    
    server.add_prompt(prompt_info, AsyncPromptGenerator).await?;
    
    println!("\nAsync handlers registered successfully!");
    println!("\nServer capabilities:");
    println!("- Tools: async_calculator");
    println!("- Resources: data://users, data://config");
    println!("- Prompts: async_greeting");
    
    // Demonstrate async request handling
    println!("\n=== Demonstrating Async Request Handling ===");
    
    // Simulate tool call
    println!("\n1. Testing async tool call:");
    let tool_request = JsonRpcRequest::new(
        json!("test-1"),
        "tools/call".to_string(),
        Some(json!({
            "name": "async_calculator",
            "arguments": {"expression": "2+2"}
        })),
    )?;
    
    // In a real server, this would be handled internally
    // Here we're just demonstrating the async nature
    println!("   Request sent, awaiting async response...");
    
    // Simulate resource read
    println!("\n2. Testing async resource read:");
    let resource_request = JsonRpcRequest::new(
        json!("test-2"),
        "resources/read".to_string(),
        Some(json!({
            "uri": "data://users"
        })),
    )?;
    
    println!("   Request sent, awaiting async response...");
    
    // Simulate prompt get
    println!("\n3. Testing async prompt generation:");
    let prompt_request = JsonRpcRequest::new(
        json!("test-3"),
        "prompts/get".to_string(),
        Some(json!({
            "name": "async_greeting",
            "arguments": {"name": "Alice"}
        })),
    )?;
    
    println!("   Request sent, awaiting async response...");
    
    // Demonstrate concurrent async operations
    println!("\n=== Demonstrating Concurrent Async Operations ===");
    println!("Launching multiple async operations simultaneously...");
    
    let start = std::time::Instant::now();
    
    // In a real scenario, these would be actual server operations
    // Here we simulate with direct handler calls
    let calc_future = AsyncCalculatorTool.call(
        HashMap::from([("expression".to_string(), json!("10*10"))])
    );
    
    let resource_future = AsyncDataResource.read(
        "data://config", 
        &HashMap::new()  // No parameters needed
    );
    
    let prompt_future = AsyncPromptGenerator.get(
        HashMap::from([("name".to_string(), json!("Bob"))])
    );
    
    // Wait for all operations to complete
    let (calc_result, resource_result, prompt_result) = 
        tokio::join!(calc_future, resource_future, prompt_future);
    
    let elapsed = start.elapsed();
    
    println!("\nAll operations completed in {:?}", elapsed);
    println!("Note: Operations ran concurrently, not sequentially!");
    
    // Display results
    if let Ok(calc) = calc_result {
        println!("\n✓ Calculator result: {:?}", calc.content[0]);
    }
    
    if let Ok(resource) = resource_result {
        println!("✓ Resource fetched: {} bytes", 
                resource.get(0)
                    .and_then(|r| r.text.as_ref())
                    .map(|t| t.len())
                    .unwrap_or(0));
    }
    
    if let Ok(prompt) = prompt_result {
        println!("✓ Prompt generated: {} messages", prompt.messages.len());
    }
    
    println!("\n=== Async Server Example Complete ===");
    println!("\nKey takeaways:");
    println!("• All handlers are async and can perform I/O operations");
    println!("• Multiple requests can be processed concurrently");
    println!("• The server efficiently handles async workloads");
    println!("• Perfect for integrating with databases, APIs, and services");
    
    Ok(())
}