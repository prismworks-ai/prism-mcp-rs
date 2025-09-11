//! Fluent interfaces for resources and prompts

use crate::core::enhanced_errors::{McpError, McpResult};
use crate::protocol::{
    messages::{
        ListPromptsResult, ListResourcesResult, ReadResourceResult, SubscribeResourceResult,
    },
    types::GetPromptResult,
};
use serde_json::Value;
use std::collections::HashMap;

/// Fluent interface for resource operations
pub struct ResourcesBuilder<'a> {
    client: &'a crate::client::McpClient,
}

impl<'a> ResourcesBuilder<'a> {
    pub(crate) fn new(client: &'a crate::client::McpClient) -> Self {
        Self { client }
    }

    /// Read a resource by URI
    pub async fn read(self, uri: impl Into<String>) -> McpResult<ReadResourceResult> {
        self.client
            .read_resource(uri.into())
            .await
            .map_err(|e| McpError::internal(e.to_string()))
    }

    /// List available resources
    pub async fn list(self) -> McpResult<ListResourcesResult> {
        self.client
            .list_resources(None)
            .await
            .map_err(|e| McpError::internal(e.to_string()))
    }

    /// Subscribe to resource updates
    pub async fn subscribe(self, uri: impl Into<String>) -> McpResult<SubscribeResourceResult> {
        self.client
            .subscribe_resource(uri.into())
            .await
            .map_err(|e| McpError::internal(e.to_string()))
    }
}

/// Fluent interface for prompt operations
pub struct PromptsBuilder<'a> {
    client: &'a crate::client::McpClient,
    prompt_name: Option<String>,
    arguments: HashMap<String, String>,
}

impl<'a> PromptsBuilder<'a> {
    pub(crate) fn new(client: &'a crate::client::McpClient) -> Self {
        Self {
            client,
            prompt_name: None,
            arguments: HashMap::new(),
        }
    }

    /// Set the prompt to get
    pub fn get(mut self, name: impl Into<String>) -> Self {
        self.prompt_name = Some(name.into());
        self
    }

    /// Add arguments (converts values to strings)
    pub fn args(mut self, args: impl Into<Value>) -> Self {
        if let Value::Object(map) = args.into() {
            for (k, v) in map {
                self.arguments.insert(k, v.to_string());
            }
        }
        self
    }

    /// Add a single argument
    pub fn arg(mut self, key: impl Into<String>, value: impl ToString) -> Self {
        self.arguments.insert(key.into(), value.to_string());
        self
    }

    /// Execute the prompt
    pub async fn execute(self) -> McpResult<GetPromptResult> {
        let prompt_name = self
            .prompt_name
            .ok_or_else(|| McpError::validation("Prompt name is required"))?;

        self.client
            .get_prompt(
                prompt_name,
                if self.arguments.is_empty() {
                    None
                } else {
                    Some(self.arguments)
                },
            )
            .await
            .map_err(|e| McpError::internal(e.to_string()))
    }

    /// List available prompts
    pub async fn list(self) -> McpResult<ListPromptsResult> {
        self.client
            .list_prompts(None)
            .await
            .map_err(|e| McpError::internal(e.to_string()))
    }
}
