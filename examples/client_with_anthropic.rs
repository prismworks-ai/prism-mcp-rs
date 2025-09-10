//! Example of MCP client with Anthropic Claude integration for sampling requests
//!
//! This example demonstrates how to implement a custom ClientRequestHandler
//! that integrates with Anthropic's Claude API for handling sampling/createMessage
//! requests from MCP servers.

use async_trait::async_trait;
use prism_mcp_rs::client::{ClientRequestHandler, McpClient};
use prism_mcp_rs::core::error::{McpError, McpResult};
use prism_mcp_rs::protocol::messages::*;
use prism_mcp_rs::protocol::types::*;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Anthropic API client wrapper
struct AnthropicClient {
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicClient {
    fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    async fn create_message(&self, request: AnthropicRequest) -> Result<AnthropicResponse, McpError> {
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| McpError::Protocol(format!("Failed to call Anthropic API: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(McpError::Protocol(format!("Anthropic API error: {}", error_text)));
        }

        response
            .json::<AnthropicResponse>()
            .await
            .map_err(|e| McpError::Protocol(format!("Failed to parse Anthropic response: {}", e)))
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<AnthropicContent>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// MCP Client handler with Anthropic Claude integration
pub struct AnthropicRequestHandler {
    anthropic_client: AnthropicClient,
    roots: Vec<Root>,
    default_model: String,
}

impl AnthropicRequestHandler {
    pub fn new(api_key: String) -> Self {
        Self {
            anthropic_client: AnthropicClient::new(api_key),
            roots: Vec::new(),
            default_model: "claude-3-5-sonnet-20241022".to_string(),
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
impl ClientRequestHandler for AnthropicRequestHandler {
    async fn handle_create_message(
        &self,
        params: CreateMessageParams,
    ) -> McpResult<CreateMessageResult> {
        // Convert MCP messages to Anthropic format
        let anthropic_messages: Vec<AnthropicMessage> = params
            .messages
            .iter()
            .map(|msg| {
                let content = match &msg.content {
                    SamplingContent::Text { text, .. } => text.clone(),
                    SamplingContent::Image { data, .. } => {
                        // For images, we'd need to handle base64 encoding
                        // This is a simplified example
                        format!("[Image data: {} bytes]", data.len())
                    }
                };
                AnthropicMessage {
                    role: match msg.role {
                        Role::User => "user".to_string(),
                        Role::Assistant => "assistant".to_string(),
                    },
                    content,
                }
            })
            .collect();

        // Determine which model to use
        let model = params
            .model_preferences
            .as_ref()
            .and_then(|prefs| prefs.hints.first())
            .map(|hint| hint.name.clone())
            .unwrap_or_else(|| self.default_model.clone());

        // Create Anthropic API request
        let request = AnthropicRequest {
            model,
            messages: anthropic_messages,
            max_tokens: params.max_tokens,
            temperature: params.temperature,
            system: params.system_prompt,
            stop_sequences: params.stop_sequences,
        };

        // Call Anthropic API
        let response = self.anthropic_client.create_message(request).await?;

        // Convert Anthropic response to MCP format
        let content_text = response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_else(|| "No response generated".to_string());

        let stop_reason = match response.stop_reason.as_deref() {
            Some("end_turn") => StopReason::EndTurn,
            Some("max_tokens") => StopReason::MaxTokens,
            Some("stop_sequence") => StopReason::StopSequence,
            _ => StopReason::EndTurn,
        };

        Ok(CreateMessageResult {
            model: response.model,
            stop_reason: Some(stop_reason),
            role: Role::Assistant,
            content: SamplingContent::Text {
                text: content_text,
                annotations: None,
                meta: None,
            },
            meta: Some(HashMap::from([(
                "usage".to_string(),
                serde_json::json!({
                    "input_tokens": response.usage.input_tokens,
                    "output_tokens": response.usage.output_tokens,
                }),
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
        // In production, you might want to implement proper user interaction
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
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    // Create MCP client with Anthropic handler
    let mut client = McpClient::new("anthropic-client".to_string(), "1.0.0".to_string());
    
    // Set up the handler with Anthropic integration
    let handler = AnthropicRequestHandler::new(api_key)
        .with_default_model("claude-3-5-sonnet-20241022".to_string())
        .add_root("file:///home/user/projects", Some("Projects"))
        .add_root("file:///home/user/documents", Some("Documents"));
    
    client.set_request_handler(handler);

    // Connect to MCP server via stdio
    println!("Connecting to MCP server...");
    client.connect_stdio().await?;
    
    // Initialize the connection
    println!("Initializing MCP connection...");
    let server_info = client.initialize().await?;
    println!("Connected to server: {:?}", server_info);

    // Now the client can handle sampling requests from the server
    // The server can call sampling/createMessage and get responses from Claude
    
    // Keep the client running
    println!("Client ready. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;
    
    println!("Shutting down...");
    Ok(())
}
