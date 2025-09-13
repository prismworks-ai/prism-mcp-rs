//! Example 03: Prompts API
//! Shows how to implement dynamic prompt handlers

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

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
                content: ContentBlock::text(format!(
                    "You are a {} code generation assistant.",
                    language
                )),
            },
            PromptMessage {
                role: Role::User,
                content: ContentBlock::text(format!(
                    "Please {} in {}. Follow best practices and include comments.",
                    task, language
                )),
            },
        ];
        
        Ok(GetPromptResult {
            description: Some(format!("Generate {} code for: {}", language, task)),
            messages,
        })
    }
}

/// Data analysis prompt handler
struct DataAnalysisPrompt;

#[async_trait]
impl PromptHandler for DataAnalysisPrompt {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<GetPromptResult> {
        let dataset = arguments
            .get("dataset")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown dataset");
        
        let analysis_type = arguments
            .get("analysis_type")
            .and_then(|v| v.as_str())
            .unwrap_or("exploratory");
        
        let messages = vec![
            PromptMessage {
                role: Role::Assistant,
                content: ContentBlock::text(
                    "You are a data analysis expert. Provide insights and recommendations based on the data."
                ),
            },
            PromptMessage {
                role: Role::User,
                content: ContentBlock::text(format!(
                    "Perform {} analysis on the {} dataset. Focus on key patterns and anomalies.",
                    analysis_type, dataset
                )),
            },
        ];
        
        Ok(GetPromptResult {
            description: Some(format!("Analyze {}: {} analysis", dataset, analysis_type)),
            messages,
        })
    }
}

/// Custom template prompt handler
struct TemplatePrompt {
    template: String,
}

#[async_trait]
impl PromptHandler for TemplatePrompt {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<GetPromptResult> {
        let mut result = self.template.clone();
        
        // Replace placeholders with arguments
        for (key, value) in arguments {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                Value::String(s) => s,
                v => v.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
        
        let messages = vec![
            PromptMessage {
                role: Role::User,
                content: ContentBlock::text(result),
            },
        ];
        
        Ok(GetPromptResult {
            description: Some("Custom template prompt".to_string()),
            messages,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("Prompts API example");
    info!("Demonstrating various prompt handlers");
    
    // Create prompt handlers
    let code_gen = CodeGenPrompt;
    let data_analysis = DataAnalysisPrompt;
    let template = TemplatePrompt {
        template: "Write a {{type}} about {{topic}} in {{style}} style.".to_string(),
    };
    
    // Example 1: Code generation prompt
    let mut args = HashMap::new();
    args.insert("language".to_string(), json!("rust"));
    args.insert("task".to_string(), json!("implement a binary search algorithm"));
    
    let result = code_gen.get(args).await?;
    info!("Code generation prompt:");
    info!("  Description: {:?}", result.description);
    for msg in &result.messages {
        info!("  {}: {}", msg.role, msg.content);
    }
    
    // Example 2: Data analysis prompt
    let mut args = HashMap::new();
    args.insert("dataset".to_string(), json!("sales_data_2024"));
    args.insert("analysis_type".to_string(), json!("trend"));
    
    let result = data_analysis.get(args).await?;
    info!("\nData analysis prompt:");
    info!("  Description: {:?}", result.description);
    for msg in &result.messages {
        info!("  {}: {}", msg.role, msg.content);
    }
    
    // Example 3: Template prompt
    let mut args = HashMap::new();
    args.insert("type".to_string(), json!("blog post"));
    args.insert("topic".to_string(), json!("machine learning"));
    args.insert("style".to_string(), json!("technical"));
    
    let result = template.get(args).await?;
    info!("\nTemplate prompt:");
    info!("  Description: {:?}", result.description);
    for msg in &result.messages {
        info!("  {}: {}", msg.role, msg.content);
    }
    
    // Create and use prompts with a server
    let server = McpServer::new("prompts-example".to_string(), "1.0.0".to_string());
    
    // Register prompts
    let code_prompt_info = Prompt {
        name: "code_generation".to_string(),
        description: Some("Generate code in various languages".to_string()),
        arguments: Some(vec![
            PromptArgument {
                name: "language".to_string(),
                description: Some("Programming language".to_string()),
                required: Some(true),
            },
            PromptArgument {
                name: "task".to_string(),
                description: Some("What to implement".to_string()),
                required: Some(true),
            },
        ]),
    };
    
    info!("\nRegistered prompts: code_generation");
    info!("Server ready to handle prompt requests");
    
    Ok(())
}