//! Server builder implementation for fluent API construction
//!
//! This module provides a builder pattern implementation for creating and configuring
//! MCP servers with a fluent, type-safe API.

use std::collections::HashMap;

use crate::core::{Prompt, Resource, Tool};
use crate::protocol::types::{
    CompletionsCapability, LoggingCapability, PromptsCapability, ResourceTemplate,
    ResourcesCapability, SamplingCapability, ServerCapabilities, ToolsCapability,
};
use crate::server::{McpServer, ServerConfig};

/// Builder for creating MCP servers with fluent API
///
/// # Examples
///
/// ```rust,no_run
/// use prism_mcp_rs::server::ServerBuilder;
///
/// let server = ServerBuilder::new()
///     .name("my-server")
///     .version("1.0.0")
///     .with_prompts()
///     .with_resources()
///     .with_tools()
///     .build();
/// ```
pub struct ServerBuilder {
    name: Option<String>,
    version: Option<String>,
    capabilities: ServerCapabilities,
    config: ServerConfig,
    resources: HashMap<String, Resource>,
    tools: HashMap<String, Tool>,
    prompts: HashMap<String, Prompt>,
    resource_templates: HashMap<String, ResourceTemplate>,
}

impl ServerBuilder {
    /// Create a new server builder
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            capabilities: ServerCapabilities::default(),
            config: ServerConfig::default(),
            resources: HashMap::new(),
            tools: HashMap::new(),
            prompts: HashMap::new(),
            resource_templates: HashMap::new(),
        }
    }

    /// Set the server name
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the server version
    pub fn version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set full server capabilities
    pub fn capabilities(mut self, capabilities: ServerCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Enable prompt capabilities
    pub fn with_prompts(mut self) -> Self {
        self.capabilities.prompts = Some(PromptsCapability {
            list_changed: Some(true),
        });
        self
    }

    /// Enable resource capabilities with optional subscription support
    pub fn with_resources(mut self) -> Self {
        self.capabilities.resources = Some(ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        });
        self
    }

    /// Enable tool capabilities
    pub fn with_tools(mut self) -> Self {
        self.capabilities.tools = Some(ToolsCapability {
            list_changed: Some(true),
        });
        self
    }

    /// Enable sampling capabilities
    pub fn with_sampling(mut self) -> Self {
        self.capabilities.sampling = Some(SamplingCapability::default());
        self
    }

    /// Enable logging capabilities
    pub fn with_logging(mut self) -> Self {
        self.capabilities.logging = Some(LoggingCapability::default());
        self
    }

    /// Enable completion capabilities
    pub fn with_completions(mut self) -> Self {
        self.capabilities.completions = Some(CompletionsCapability::default());
        self
    }

    /// Enable roots capabilities (for clients)
    pub fn with_roots(self) -> Self {
        // Note: Roots is typically a client capability, but we'll include it
        // in case the server needs to declare support
        self
    }

    /// Add experimental capabilities
    pub fn with_experimental<K: Into<String>>(mut self, key: K, value: serde_json::Value) -> Self {
        if self.capabilities.experimental.is_none() {
            self.capabilities.experimental = Some(HashMap::new());
        }
        if let Some(ref mut experimental) = self.capabilities.experimental {
            experimental.insert(key.into(), value);
        }
        self
    }

    /// Set server configuration
    pub fn config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Set maximum concurrent requests
    pub fn max_concurrent_requests(mut self, max: usize) -> Self {
        self.config.max_concurrent_requests = max;
        self
    }

    /// Set request timeout in milliseconds
    pub fn request_timeout_ms(mut self, timeout: u64) -> Self {
        self.config.request_timeout_ms = timeout;
        self
    }

    /// Enable or disable request validation
    pub fn validate_requests(mut self, validate: bool) -> Self {
        self.config.validate_requests = validate;
        self
    }

    /// Enable or disable logging
    pub fn enable_logging(mut self, enable: bool) -> Self {
        self.config.enable_logging = enable;
        self
    }

    /// Add a resource to the server
    pub fn add_resource(mut self, resource: Resource) -> Self {
        self.resources.insert(resource.info.uri.clone(), resource);
        self
    }

    /// Add a tool to the server
    pub fn add_tool(mut self, tool: Tool) -> Self {
        self.tools.insert(tool.info.name.clone(), tool);
        self
    }

    /// Add a prompt to the server
    pub fn add_prompt(mut self, prompt: Prompt) -> Self {
        self.prompts.insert(prompt.info.name.clone(), prompt);
        self
    }

    /// Add a resource template to the server
    pub fn add_resource_template(mut self, template: ResourceTemplate) -> Self {
        self.resource_templates
            .insert(template.uri_template.clone(), template);
        self
    }

    /// Build the MCP server
    ///
    /// # Panics
    ///
    /// Panics if name or version are not set
    pub fn build(self) -> McpServer {
        let name = self.name.expect("Server name is required");
        let version = self.version.expect("Server version is required");

        let mut server = McpServer::new(name, version);
        server.set_capabilities(self.capabilities);
        server.set_config(self.config);

        // Transfer resources, tools, prompts, and templates to the server
        // Note: This requires the server to expose methods to bulk-add items,
        // which we'll add in the server implementation
        server.set_initial_resources(self.resources);
        server.set_initial_tools(self.tools);
        server.set_initial_prompts(self.prompts);
        server.set_initial_resource_templates(self.resource_templates);

        server
    }

    /// Try to build the MCP server, returning an error if required fields are missing
    pub fn try_build(self) -> Result<McpServer, ServerBuilderError> {
        let name = self.name.ok_or(ServerBuilderError::MissingName)?;
        let version = self.version.ok_or(ServerBuilderError::MissingVersion)?;

        let mut server = McpServer::new(name, version);
        server.set_capabilities(self.capabilities);
        server.set_config(self.config);

        server.set_initial_resources(self.resources);
        server.set_initial_tools(self.tools);
        server.set_initial_prompts(self.prompts);
        server.set_initial_resource_templates(self.resource_templates);

        Ok(server)
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur when building a server
#[derive(Debug, Clone, PartialEq)]
pub enum ServerBuilderError {
    /// Server name was not provided
    MissingName,
    /// Server version was not provided
    MissingVersion,
}

impl std::fmt::Display for ServerBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingName => write!(f, "Server name is required"),
            Self::MissingVersion => write!(f, "Server version is required"),
        }
    }
}

impl std::error::Error for ServerBuilderError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_builder_basic() {
        let server = ServerBuilder::new()
            .name("test-server")
            .version("1.0.0")
            .build();

        // The actual test would verify the server properties
        // but we need to expose getters on McpServer first
    }

    #[test]
    fn test_builder_with_capabilities() {
        let server = ServerBuilder::new()
            .name("test-server")
            .version("1.0.0")
            .with_prompts()
            .with_resources()
            .with_tools()
            .with_sampling()
            .with_logging()
            .with_completions()
            .build();
    }

    #[test]
    fn test_builder_with_config() {
        let server = ServerBuilder::new()
            .name("test-server")
            .version("1.0.0")
            .max_concurrent_requests(50)
            .request_timeout_ms(60000)
            .validate_requests(false)
            .enable_logging(false)
            .build();
    }

    #[test]
    fn test_builder_with_experimental() {
        let server = ServerBuilder::new()
            .name("test-server")
            .version("1.0.0")
            .with_experimental("custom_feature", json!(true))
            .with_experimental("beta_mode", json!({"enabled": true}))
            .build();
    }

    #[test]
    #[should_panic(expected = "Server name is required")]
    fn test_builder_missing_name() {
        ServerBuilder::new().version("1.0.0").build();
    }

    #[test]
    #[should_panic(expected = "Server version is required")]
    fn test_builder_missing_version() {
        ServerBuilder::new().name("test-server").build();
    }

    #[test]
    fn test_try_build_success() {
        let result = ServerBuilder::new()
            .name("test-server")
            .version("1.0.0")
            .try_build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_try_build_missing_name() {
        let result = ServerBuilder::new().version("1.0.0").try_build();

        assert!(matches!(result, Err(ServerBuilderError::MissingName)));
    }

    #[test]
    fn test_try_build_missing_version() {
        let result = ServerBuilder::new().name("test-server").try_build();

        assert!(matches!(result, Err(ServerBuilderError::MissingVersion)));
    }
}
