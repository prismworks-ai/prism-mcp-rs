//! MCP protocol implementation (2025-06-18)
//!
//! This module contains the core protocol types and message handling for the
//! Model Context Protocol version 2025-06-18, including JSON-RPC message
//! serialization, validation, and new features like improved content system,
//! annotations, improved capabilities, full metadata support, batch operations,
//! and complete schema introspection.

pub mod batch;
pub mod discovery;
pub mod messages;
pub mod metadata;
pub mod methods;
pub mod missing_types;
pub mod roots_types;
pub mod schema_introspection;
pub mod types;
// NOTE: types_2025 is temporarily disabled to resolve ContentBlock duplication conflicts
// during schema upgrade to 2025-06-18. Will be removed after consolidation.
// pub mod types_2025;
pub mod validation;

// Re-export commonly used types and constants
pub use batch::*;
pub use discovery::*;
pub use messages::*;
pub use metadata::*;
pub use missing_types::*;
// Re-export roots_types items except those that conflict with messages
pub use roots_types::{
    ListRootsRequest,
    RootsListChangedNotification,
    // Explicitly exclude Root and ListRootsResult which are already in messages
};
pub use schema_introspection::*;
pub use types::*;
pub use validation::*;

// Re-export method constants for convenience
pub use methods::{
    CANCELLED, COMPLETION_COMPLETE, INITIALIZE, INITIALIZED, LOGGING_MESSAGE, LOGGING_SET_LEVEL,
    PING, PROGRESS, PROMPTS_GET, PROMPTS_LIST, PROMPTS_LIST_CHANGED, RESOURCES_LIST,
    RESOURCES_LIST_CHANGED, RESOURCES_READ, RESOURCES_SUBSCRIBE, RESOURCES_TEMPLATES_LIST,
    RESOURCES_UNSUBSCRIBE, RESOURCES_UPDATED, ROOTS_LIST, ROOTS_LIST_CHANGED, RPC_DISCOVER,
    SAMPLING_CREATE_MESSAGE, TOOLS_CALL, TOOLS_LIST, TOOLS_LIST_CHANGED,
};

// Re-export 2025-06-18 specific constants and error codes
pub use types::error_codes;
pub use types::{JSONRPC_VERSION, LATEST_PROTOCOL_VERSION};

// Legacy constant for compatibility
pub const MCP_PROTOCOL_VERSION: &str = LATEST_PROTOCOL_VERSION;

// NOTE: types_2025 re-export disabled during consolidation
// Export types_2025 for complete tests
// pub use types_2025 as types_2025_complete;