//! Test script for OpenAI handler functionality
use prism_mcp_rs::client::ClientRequestHandler;
use prism_mcp_rs::protocol::messages::*;
use prism_mcp_rs::protocol::types::*;
use std::collections::HashMap;

// Import the handler from the example
#[path = "../examples/client_with_openai.rs"]
mod client_with_openai;

#[tokio::main]
async fn main() {
    println!("Testing OpenAI handler...");

    // Create handler
    let handler = client_with_openai::OpenAIRequestHandler::new("test-key".to_string());

    // Test sampling request
    let params = CreateMessageParams {
        messages: vec![SamplingMessage {
            role: Role::User,
            content: SamplingContent::Text {
                text: "Hello, what's 2+2?".to_string(),
                annotations: None,
                meta: None,
            },
        }],
        max_tokens: 100,
        system_prompt: Some("You are a helpful assistant.".to_string()),
        include_context: None,
        temperature: Some(0.7),
        stop_sequences: None,
        model_preferences: None,
        metadata: None,
        meta: None,
    };

    match handler.handle_create_message(params).await {
        Ok(result) => {
            println!("✅ Sampling works!");
            println!("Model: {}", result.model);
            if let SamplingContent::Text { text, .. } = result.content {
                println!("Response: {}", text);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test roots
    match handler
        .handle_list_roots(ListRootsParams { meta: None })
        .await
    {
        Ok(result) => {
            println!("✅ Roots: {} configured", result.roots.len());
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nAll tests complete!");
}
