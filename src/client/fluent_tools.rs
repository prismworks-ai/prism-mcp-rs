//! Fluent interface for tool operations

use crate::core::enhanced_errors::{McpError, McpResult};
use crate::protocol::{messages::ListToolsResult, types::CallToolResult};
use serde_json::Value;
use std::collections::HashMap;

/// Fluent interface for tool operations
pub struct ToolsBuilder<'a> {
    client: &'a crate::client::McpClient,
    tool_name: Option<String>,
    arguments: HashMap<String, Value>,
}

impl<'a> ToolsBuilder<'a> {
    pub(crate) fn new(client: &'a crate::client::McpClient) -> Self {
        Self {
            client,
            tool_name: None,
            arguments: HashMap::new(),
        }
    }

    /// Set the tool to call
    pub fn call(mut self, name: impl Into<String>) -> Self {
        self.tool_name = Some(name.into());
        self
    }

    /// Add arguments using JSON value
    pub fn args(mut self, args: impl Into<Value>) -> Self {
        if let Value::Object(map) = args.into() {
            for (k, v) in map {
                self.arguments.insert(k, v);
            }
        }
        self
    }

    /// Add a single argument
    pub fn arg(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Execute the tool call
    pub async fn execute(self) -> McpResult<CallToolResult> {
        let tool_name = self
            .tool_name
            .ok_or_else(|| McpError::validation("Tool name is required"))?;

        self.client
            .call_tool(
                tool_name,
                if self.arguments.is_empty() {
                    None
                } else {
                    Some(self.arguments)
                },
            )
            .await
            .map_err(|e| McpError::internal(e.to_string()))
    }

    /// List available tools
    pub async fn list(self) -> McpResult<ListToolsResult> {
        self.client
            .list_tools(None)
            .await
            .map_err(|e| McpError::internal(e.to_string()))
    }
}
