//! Example 03: Prompts API (Fixed Version)
//! Demonstrates basic prompt handling

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Code generation prompt handler
struct CodeGenPrompt;

#[async_trait]
impl PromptHandler for CodeGenPrompt {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<GetPromptResult> {
        let language = arguments
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("python");

        let task = arguments
            .get("task")
            .and_then(|v| v.as_str())
            .unwrap_or("implement a function");

        let messages = vec![
            PromptMessage {
                role: Role::Assistant,
                content: ContentBlock::Text {
                    text: format!("You are a {} code generation assistant.", language),
                    annotations: None,
                    meta: None,
                },
            },
            PromptMessage {
                role: Role::User,
                content: ContentBlock::Text {
                    text: format!(
                        "Please {} in {}. Follow best practices and include comments.",
                        task, language
                    ),
                    annotations: None,
                    meta: None,
                },
            },
        ];

        Ok(GetPromptResult {
            description: Some(format!("Generate {} code for: {}", language, task)),
            messages,
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Prompts API example");
    println!("Demonstrating prompt handlers");

    let code_gen = CodeGenPrompt;

    // Example: Code generation prompt
    let mut args = HashMap::new();
    args.insert("language".to_string(), json!("rust"));
    args.insert(
        "task".to_string(),
        json!("implement a binary search algorithm"),
    );

    let result = code_gen.get(args).await?;
    println!("Code generation prompt:");
    println!("  Description: {:?}", result.description);
    for msg in &result.messages {
        println!("  {:?}: {:?}", msg.role, msg.content);
    }

    Ok(())
}
