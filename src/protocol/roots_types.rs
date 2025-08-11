// ! Roots Types for MCP Protocol (2025-06-18)
// !
// ! Module provides the Roots feature types for file system access,
// ! allowing servers to request access to specific directories or files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Roots Types (2025-06-18)
// ============================================================================

/// Represents a root directory or file that the server can operate on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Root {
    /// The URI identifying the root. This *must* start with file:///for now.
    /// This restriction may be relaxed in future versions of the protocol to allow
    /// other URI schemes.
    #[serde(rename = "uri")]
    pub uri: String,

    /// An optional name for the root. This can be used to provide a human-readable
    /// identifier for the root, which may be useful for display purposes or for
    /// referencing the root in other parts of the application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Metadata field for future extensions
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

impl Root {
    /// Create a new Root with just a URI
    pub fn new(uri: String) -> Self {
        Self {
            uri,
            name: None,
            meta: None,
        }
    }

    /// Create a new Root with a URI and name
    pub fn with_name(uri: String, name: String) -> Self {
        Self {
            uri,
            name: Some(name),
            meta: None,
        }
    }

    /// Validate that the URI is properly formatted
    pub fn validate(&self) -> Result<(), String> {
        if !self.uri.starts_with("file:///") {
            return Err(format!(
                "Root URI must start with 'file:///', got: {}",
                self.uri
            ));
        }
        Ok(())
    }
}

/// Request for listing roots (sent from server to client)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ListRootsRequest {
    /// Method name (always "roots/list")
    #[serde(default = "default_roots_list_method")]
    pub method: String,

    /// Request parameters (usually empty for roots/list)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

fn default_roots_list_method() -> String {
    "roots/list".to_string()
}

impl ListRootsRequest {
    /// Create a new ListRootsRequest
    pub fn new() -> Self {
        Self {
            method: "roots/list".to_string(),
            params: None,
        }
    }
}

/// Result of listing roots (sent from client to server)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListRootsResult {
    /// Array of Root objects representing available roots
    pub roots: Vec<Root>,

    /// Metadata field for future extensions
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

impl ListRootsResult {
    /// Create a new ListRootsResult with the given roots
    pub fn new(roots: Vec<Root>) -> Self {
        Self { roots, meta: None }
    }

    /// Create an empty result
    pub fn empty() -> Self {
        Self {
            roots: Vec::new(),
            meta: None,
        }
    }
}

/// Notification that the list of roots has changed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RootsListChangedNotification {
    /// Method name (always "notifications/roots/list_changed")
    #[serde(default = "default_roots_list_changed_method")]
    pub method: String,

    /// Notification parameters (usually empty)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

fn default_roots_list_changed_method() -> String {
    "notifications/roots/list_changed".to_string()
}

impl RootsListChangedNotification {
    /// Create a new RootsListChangedNotification
    pub fn new() -> Self {
        Self {
            method: "notifications/roots/list_changed".to_string(),
            params: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_creation() {
        let root = Root::new("file:///home/user/projects".to_string());
        assert_eq!(root.uri, "file:///home/user/projects");
        assert_eq!(root.name, None);
        assert_eq!(root.meta, None);

        let root_with_name = Root::with_name(
            "file:///home/user/documents".to_string(),
            "Documents".to_string(),
        );
        assert_eq!(root_with_name.uri, "file:///home/user/documents");
        assert_eq!(root_with_name.name, Some("Documents".to_string()));
    }

    #[test]
    fn test_root_validation() {
        let valid_root = Root::new("file:///home/user".to_string());
        assert!(valid_root.validate().is_ok());

        let invalid_root = Root::new("http://example.com".to_string());
        assert!(invalid_root.validate().is_err());
    }

    #[test]
    fn test_root_serialization() {
        let root = Root::with_name("file:///workspace".to_string(), "Workspace".to_string());

        let json = serde_json::to_value(&root).unwrap();
        assert_eq!(json["uri"], "file:///workspace");
        assert_eq!(json["name"], "Workspace");
        assert!(json.get("_meta").is_none());
    }

    #[test]
    fn test_list_roots_request() {
        let request = ListRootsRequest::new();
        assert_eq!(request.method, "roots/list");
        assert!(request.params.is_none());

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["method"], "roots/list");
    }

    #[test]
    fn test_list_roots_result() {
        let roots = vec![
            Root::new("file:///home/user".to_string()),
            Root::with_name("file:///workspace".to_string(), "Work".to_string()),
        ];

        let result = ListRootsResult::new(roots.clone());
        assert_eq!(result.roots.len(), 2);
        assert_eq!(result.roots[0].uri, "file:///home/user");
        assert_eq!(result.roots[1].name, Some("Work".to_string()));

        let json = serde_json::to_value(&result).unwrap();
        assert!(json["roots"].is_array());
        assert_eq!(json["roots"][0]["uri"], "file:///home/user");
    }

    #[test]
    fn test_roots_list_changed_notification() {
        let notification = RootsListChangedNotification::new();
        assert_eq!(notification.method, "notifications/roots/list_changed");
        assert!(notification.params.is_none());

        let json = serde_json::to_value(&notification).unwrap();
        assert_eq!(json["method"], "notifications/roots/list_changed");
    }
}
