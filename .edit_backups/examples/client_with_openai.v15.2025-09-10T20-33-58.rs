//! Example of MCP client with OpenAI integration for sampling requests
//!
//! This example demonstrates how to implement a custom ClientRequestHandler
//! that integrates with OpenAI's API for handling sampling/createMessage
//! requests from MCP servers.

use async_trait::async_trait;
use prism_mcp_rs::client::{ClientRequestHandler, McpClient};
use prism_mcp_rs::core::error::{McpError, McpResult};

use prism_mcp_rs::protocol::messages::*;
use prism_mcp_rs::protocol::types::*;

// reqwest would be added as dependency for production use
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// OpenAI API client wrapper (simplified for example)
struct OpenAIClient {
    #[allow(dead_code)]
    api_key: String,
}

impl OpenAIClient {
    fn new(api_key: String) -> Self {
        Self { api_key }
    }

    async fn create_completion(&self, _request: OpenAIRequest) -> Result<OpenAIResponse, McpError> {
        // In production, you would use reqwest or another HTTP client here
        // Example response for demonstration
        Ok(OpenAIResponse {
            id: "chatcmpl-example".to_string(),
            model: "gpt-4-turbo-preview".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: "This is a simulated response. In production, integrate with OpenAI API.".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: OpenAIUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
        })
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    id: String,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    index: i32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// MCP Client handler with OpenAI integration
pub struct OpenAIRequestHandler {
    openai_client: OpenAIClient,
    roots: Vec<Root>,
    default_model: String,
}

impl OpenAIRequestHandler {
    pub fn new(api_key: String) -> Self {
        Self {
            openai_client: OpenAIClient::new(api_key),
            roots: Vec::new(),
            default_model: "gpt-4-turbo-preview".to_string(),
        }
    }

    pub fn with_default_model(mut self, model: String) -> Self {
        self.default_model = model;
        self
    }

    pub fn add_root(mut self, uri: String, name: Option<String>) -> Self {
        let mut root = Root::new(uri);
        if let Some(n) = name {
            root = root.with_name(n);
        }
        self.roots.push(root);
        self
    }
}

#[async_trait]
impl ClientRequestHandler for OpenAIRequestHandler {
    async fn handle_create_message(
        &self,
        params: CreateMessageParams,
    ) -> McpResult<CreateMessageResult> {
        // Convert MCP messages to OpenAI format
        let mut openai_messages: Vec<OpenAIMessage> = vec![];
        
        // Add system prompt if provided
        if let Some(system_prompt) = params.system_prompt {
            openai_messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: system_prompt,
            });
        }
        
        // Convert conversation messages
        for msg in params.messages.iter() {
            let content = match &msg.content {
                SamplingContent::Text { text, .. } => text.clone(),
                SamplingContent::Image { data, .. } => {
                    // OpenAI supports base64 images in a different format
                    // This is simplified - real implementation would handle properly
                    format!("[Image data: {} bytes]", data.len())
                }
                SamplingContent::Audio { .. } => {
                    // Audio not supported in this example
                    "[Audio content]".to_string()
                }
            };
            openai_messages.push(OpenAIMessage {
                role: match msg.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content,
            });
        }

        // Determine which model to use
        let model = params
            .model_preferences
            .as_ref()
            .and_then(|prefs| prefs.hints.as_ref())
            .and_then(|hints| hints.first())
            .and_then(|hint| hint.name.clone())
            .unwrap_or_else(|| self.default_model.clone());

        // Create OpenAI API request
        let request = OpenAIRequest {
            model: model.clone(),
            messages: openai_messages,
            max_tokens: Some(params.max_tokens as i32),
            temperature: params.temperature.map(|t| t as f64),
            stop: params.stop_sequences,
            stream: Some(false),
        };

        // Call OpenAI API
        let response = self.openai_client.create_completion(request).await?;

        // Get the first choice's message
        let choice = response
            .choices
            .first()
            .ok_or_else(|| McpError::Protocol("No response from OpenAI".to_string()))?;

        // Convert finish reason to MCP stop reason
        let stop_reason = match choice.finish_reason.as_deref() {
            Some("stop") => StopReason::EndTurn,
            Some("length") => StopReason::MaxTokens,
            Some("content_filter") => StopReason::StopSequence,
            _ => StopReason::EndTurn,
        };

        Ok(CreateMessageResult {
            model,
            stop_reason: Some(stop_reason),
            role: Role::Assistant,
            content: SamplingContent::Text {
                text: choice.message.content.clone(),
                annotations: None,
                meta: None,
            },
            meta: Some(HashMap::from([
                (
                    "usage".to_string(),
                    serde_json::json!({
                        "prompt_tokens": response.usage.prompt_tokens,
                        "completion_tokens": response.usage.completion_tokens,
                        "total_tokens": response.usage.total_tokens,
                    }),
                ),
                (
                    "response_id".to_string(),
                    serde_json::Value::String(response.id),
                ),
            ])),
        })
    }

    async fn handle_list_roots(&self, _params: ListRootsParams) -> McpResult<ListRootsResult> {
        Ok(ListRootsResult {
            roots: self.roots.clone(),
            meta: None,
        })
    }

    async fn handle_elicit(&self, params: ElicitParams) -> McpResult<ElicitResult> {
        // For automated handling, accept with empty data
        println!("[Elicitation Request] {}", params.message);
        
        Ok(ElicitResult {
            action: ElicitationAction::Accept,
            content: Some(HashMap::new()),
            meta: None,
        })
    }

    async fn handle_ping(&self, _params: PingParams) -> McpResult<PingResult> {
        Ok(PingResult { meta: None })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get API key from environment
    let api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable must be set");

    // Create MCP client with OpenAI handler
    let mut client = McpClient::new("openai-client".to_string(), "1.0.0".to_string());
    
    // Set up the handler with OpenAI integration
    let handler = OpenAIRequestHandler::new(api_key)
        .with_default_model("gpt-4-turbo-preview".to_string())
        .add_root("file:///home/user/projects".to_string(), Some("Projects".to_string()))
        .add_root("file:///home/user/documents".to_string(), Some("Documents".to_string()));
    
    client.set_request_handler(handler);

    // Example showing connection would work (commented out as it needs a server)
    println!("Client configured with OpenAI handler.");
    // In production:
    // client.connect_stdio().await?;
    // let server_info = client.initialize().await?;
    // println!("Connected to server: {:?}", server_info);

    // Now the client can handle sampling requests from the server
    // The server can call sampling/createMessage and get responses from GPT-4
    
    // Keep the client running
    println!("Client ready. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;
    
    println!("Shutting down...");
    Ok(())
}
