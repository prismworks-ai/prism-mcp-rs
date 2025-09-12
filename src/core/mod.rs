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
//! use prism_mcp_rs::core::error::{McpError, McpResult};
//! use std::collections::HashMap;
//! use prism_mcp_rs::protocol::types::{ToolInfo, ToolResult, ContentBlock};
//! use serde_json::{json, Value};
//! use async_trait::async_trait;
//!
//! struct CalculatorTool;
//!
//! #[async_trait]
//! impl ToolHandler for CalculatorTool {
//!     async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
//!         // Tool implementation  
//!         Ok(ToolResult {
//!             content: vec![ContentBlock::Text {
//!                 text: "Result: 42".to_string(),
//!                 annotations: None,
//!                 meta: None,
//!             }],
//!             is_error: Some(false),
//!             meta: None,
//!             structured_content: None,
//!         })
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Build the tool with the handler
//! let tool = ToolBuilder::new("calculator")
//!     .description("Performs calculations")
//!     .build(CalculatorTool)?;
//! # Ok(())
//! # }
//! ```

//!
//! ## Resources
//! Resources provide access to external data sources:
//!
//! ```
//! use prism_mcp_rs::core::{Resource, ResourceHandler};
//! use prism_mcp_rs::core::error::McpResult;
//! use std::collections::HashMap;
//! use prism_mcp_rs::protocol::types::{ResourceContents, ContentBlock};
//! use prism_mcp_rs::core::{ResourceInfo};
//! use async_trait::async_trait;
//!
//! struct FileResource;
//!
//! #[async_trait]
//! impl ResourceHandler for FileResource {
//!     async fn read(
//!         &self,
//!         uri: &str,
//!         params: &HashMap<String, String>,
//!     ) -> McpResult<Vec<ResourceContents>> {
//!         Ok(vec![ResourceContents::Text {
//!             uri: uri.to_string(),
//!             mime_type: Some("text/plain".to_string()),
//!             text: "File contents".to_string(),
//!             meta: None,
//!         }])
//!     }
//!     
//!     async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
//!         Ok(vec![])
//!     }
//! }
//! ```
//!
//! ## Prompts
//! Prompts provide reusable interaction templates:
//!
//! ```
//! use prism_mcp_rs::core::{Prompt, PromptHandler};
//! use prism_mcp_rs::core::error::McpResult;
//! use std::collections::HashMap;
//! use prism_mcp_rs::protocol::types::{PromptResult, PromptMessage, Role, ContentBlock};
//! use serde_json::Value;
//! use async_trait::async_trait;
//!
//! struct GreetingPrompt;
//!
//! #[async_trait]
//! impl PromptHandler for GreetingPrompt {
//!     async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<PromptResult> {
//!         Ok(PromptResult {
//!             description: Some("A friendly greeting".to_string()),
//!             messages: vec![
//!                 PromptMessage {
//!                     role: Role::Assistant,
//!                     content: ContentBlock::Text {
//!                         text: "Hello! How can I help you today?".to_string(),
//!                         annotations: None,
//!                         meta: None,
//!                     },
//!                 }
//!             ],
//!             meta: None,
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
//!         .map_err(|e| McpError::Io(e.to_string()))?;
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
pub mod enhanced_errors;
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
pub use tool::{SimpleTool, Tool, ToolBuilder, ToolHandler};
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
