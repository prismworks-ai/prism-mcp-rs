//! Core abstractions and types for the MCP SDK
//!
//! This module contains the fundamental building blocks for MCP implementations,
//! including error handling, resource management, tool execution, and prompt handling.

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