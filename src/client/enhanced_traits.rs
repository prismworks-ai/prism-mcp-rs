//! Enhanced trait implementations for better ergonomics

use crate::protocol::types::ClientInfo;
use std::fmt;

/// Enhanced Display implementation for ClientInfo
impl fmt::Display for ClientInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} v{}", self.name, self.version)
    }
}

/// From implementations for flexible client creation
impl From<&str> for ClientInfo {
    fn from(name: &str) -> Self {
        ClientInfo::new(name.to_string(), "1.0.0".to_string())
    }
}

impl From<(&str, &str)> for ClientInfo {
    fn from((name, version): (&str, &str)) -> Self {
        ClientInfo::new(name.to_string(), version.to_string())
    }
}

// Note: TryFrom<(&str, &str)> conflicts with blanket implementation
// Use ClientInfo::new() for validation instead
