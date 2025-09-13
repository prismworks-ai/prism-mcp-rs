//! Example 03: Prompts API (Working Version)
//! Demonstrates prompt handling with the actual API

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::protocol::types::ContentBlock;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Simple prompt handler
struct SimplePromptHandler;

#[async_trait]
impl PromptHandler for SimplePromptHandler {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<GetPromptResult> {
        let topic = arguments
            .get("topic")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        Ok(GetPromptResult {
            description: Some(format!("Prompt about {}", topic)),
            messages: vec![
                PromptMessage {
                    role: Role::Assistant,
                    content: Content::text("You are a helpful assistant."),
                },
                PromptMessage {
                    role: Role::User,
                    content: Content::text(format!("Tell me about {}", topic)),
                },
            ],
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handler = SimplePromptHandler;

    let mut args = HashMap::new();
    args.insert("topic".to_string(), json!("rust programming"));

    let result = handler.get(args).await?;
    println!("Prompt: {:?}", result.description);
    for msg in result.messages {
        let role_str = match msg.role {
            Role::User => "User",
            Role::Assistant => "Assistant",
        };
        let content_str = match msg.content {
            ContentBlock::Text { ref text, .. } => text.clone(),
            _ => "[non-text content]".to_string(),
        };
        println!("  {}: {}", role_str, content_str);
    }

    Ok(())
}
