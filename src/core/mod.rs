//! Core abstractions and types for the MCP SDK
//!
//! This module contains the fundamental building blocks for MCP implementations,
//! including error handling, resource management, tool execution, and prompt handling.
//!
//! # Core Concepts
//!
//! ## Tools
//! Tools represent executable functions that can be called by the MCP protocol:
//!
//! ```
//! use prism_mcp_rs::core::{Tool, ToolBuilder, ToolHandler};
//! use prism_mcp_rs::protocol::ToolInfo;
//! use serde_json::{json, Value};
//! use async_trait::async_trait;
//!
//! struct CalculatorTool;
//!
//! #[async_trait]
//! impl ToolHandler for CalculatorTool {
//!     async fn handle(&self, params: Option<Value>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
//!         // Tool implementation
//!         Ok(json!({"result": 42}))
//!     }
//! }
//!
//! let tool = ToolBuilder::new("calculator")
//!     .description("Performs calculations")
//!     .input_schema(json!({
//!         "type": "object",
//!         "properties": {
//!             "expression": {"type": "string"}
//!         }
//!     }))
//!     .handler(Box::new(CalculatorTool))
//!     .build();
//! ```
//!
//! ## Resources
//! Resources provide access to external data sources:
//!
//! ```
//! use prism_mcp_rs::core::{Resource, ResourceHandler};
//! use prism_mcp_rs::protocol::{ResourceContents, ContentBlock, TextContent};
//! use async_trait::async_trait;
//!
//! struct FileResource;
//!
//! #[async_trait]
//! impl ResourceHandler for FileResource {
//!     async fn handle(&self, uri: &str) -> Result<ResourceContents, Box<dyn std::error::Error + Send + Sync>> {
//!         Ok(ResourceContents {
//!             uri: uri.to_string(),
//!             mime_type: Some("text/plain".to_string()),
//!             text: Some("File contents".to_string()),
//!             blob: None,
//!         })
//!     }
//! }
//! ```
//!
//! ## Prompts
//! Prompts provide reusable interaction templates:
//!
//! ```
//! use prism_mcp_rs::core::{Prompt, PromptHandler};
//! use prism_mcp_rs::protocol::{PromptResult, PromptMessage, Role};
//! use serde_json::Value;
//! use async_trait::async_trait;
//!
//! struct GreetingPrompt;
//!
//! #[async_trait]
//! impl PromptHandler for GreetingPrompt {
//!     async fn handle(&self, params: Option<Value>) -> Result<PromptResult, Box<dyn std::error::Error + Send + Sync>> {
//!         Ok(PromptResult {
//!             description: Some("A friendly greeting".to_string()),
//!             messages: vec![
//!                 PromptMessage {
//!                     role: Role::Assistant,
//!                     content: "Hello! How can I help you today?".into(),
//!                 }
//!             ],
//!         })
//!     }
//! }
//! ```
//!
//! ## Error Handling
//! The SDK provides comprehensive error handling:
//!
//! ```
//! use prism_mcp_rs::core::{McpError, McpResult};
//!
//! fn process_data() -> McpResult<String> {
//!     // Return errors using the ? operator
//!     let data = std::fs::read_to_string("data.txt")
//!         .map_err(|e| McpError::Io(e))?;
//!     
//!     Ok(data)
//! }
//! ```
//!
//! # Features
//!
//! - **Type-Safe Handlers**: Strongly typed handler traits
//! - **Async Support**: All handlers are async by default
//! - **Builder Patterns**: Ergonomic construction APIs
//! - **Validation**: Built-in parameter validation
//! - **Discovery**: Tool discovery and metadata support

pub mod completion;
pub mod completion_handlers;
pub mod error;
pub mod health;
pub mod logging;
pub mod metrics;
pub mod prompt;
pub mod resource;
pub mod retry;
pub mod tool;
pub mod tool_discovery;
pub mod tool_metadata;
pub mod validation;

// Re-export commonly used items
pub use completion::{
    CompletionContext, CompletionHandler, CompositeCompletionHandler, PromptCompletionHandler,
    ResourceUriCompletionHandler, ToolCompletionHandler,
};
pub use completion_handlers::{
    CompositeCompletionHandler as completeCompositeCompletionHandler, FileSystemCompletionHandler,
    FuzzyCompletionHandler, SchemaCompletionHandler,
};
pub use error::{McpError, McpResult};
pub use prompt::{Prompt, PromptHandler};
pub use resource::{Resource, ResourceHandler, ResourceTemplate};
pub use tool::{Tool, ToolBuilder, ToolHandler};
pub use tool_discovery::{
    DeprecationCleanupPolicy, DiscoveryCriteria, DiscoveryResult, GlobalToolStats, ToolRegistry,
};
pub use tool_metadata::{
    CategoryFilter, DeprecationSeverity, ImprovedToolMetadata, ToolBehaviorHints, ToolCategory,
    ToolDeprecation,
};
pub use validation::{ParameterType, ParameterValidator, ValidationConfig};

// Re-export protocol types through core for convenience
pub use crate::protocol::types::{
    PromptArgument, PromptInfo, PromptMessage, PromptResult, ResourceInfo, ToolInfo,
};