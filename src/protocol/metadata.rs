use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// Protocol capabilities metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolCapabilities {
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

impl Default for ProtocolCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolCapabilities {
    /// Create new empty capabilities
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Create with initial capabilities
    pub fn with_fields(fields: HashMap<String, Value>) -> Self {
        Self { fields }
    }

    /// Set a capability
    pub fn set<K: Into<String>, V: Into<Value>>(&mut self, key: K, value: V) {
        self.fields.insert(key.into(), value.into());
    }

    /// Get a capability
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }

    /// Check if a capability exists
    pub fn has(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    /// Remove a capability
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.fields.remove(key)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Get fields reference
    pub fn fields(&self) -> &HashMap<String, Value> {
        &self.fields
    }

    /// Convert to `Option<HashMap>` for compatibility
    pub fn to_hashmap(&self) -> Option<HashMap<String, serde_json::Value>> {
        if self.is_empty() {
            None
        } else {
            Some(self.fields.clone())
        }
    }

    /// Convert to Option for serialization
    pub fn to_option(self) -> Option<Self> {
        if self.is_empty() { None } else { Some(self) }
    }

    /// Create from `Option<HashMap>` for compatibility
    pub fn from_hashmap(map: Option<HashMap<String, serde_json::Value>>) -> Self {
        map.map(|fields| Self { fields }).unwrap_or_default()
    }
}

impl From<HashMap<String, Value>> for ProtocolCapabilities {
    fn from(fields: HashMap<String, Value>) -> Self {
        Self { fields }
    }
}

impl fmt::Display for ProtocolCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "No capabilities")
        } else {
            write!(
                f,
                "Capabilities: {:?}",
                self.fields.keys().collect::<Vec<_>>()
            )
        }
    }
}

/// Server information metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<ProtocolCapabilities>,
}

impl ServerInfo {
    /// Create new server info
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            protocol_version: None,
            capabilities: None,
        }
    }

    /// Set protocol version
    pub fn with_protocol_version(mut self, version: impl Into<String>) -> Self {
        self.protocol_version = Some(version.into());
        self
    }

    /// Set capabilities
    pub fn with_capabilities(mut self, capabilities: ProtocolCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Get or create capabilities
    pub fn capabilities_mut(&mut self) -> &mut ProtocolCapabilities {
        self.capabilities
            .get_or_insert_with(ProtocolCapabilities::new)
    }
}

impl fmt::Display for ServerInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} v{}", self.name, self.version)?;
        if let Some(proto) = &self.protocol_version {
            write!(f, " (protocol: {})", proto)?;
        }
        Ok(())
    }
}

/// Client information metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<ProtocolCapabilities>,
}

impl ClientInfo {
    /// Create new client info
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            protocol_version: None,
            capabilities: None,
        }
    }

    /// Convert to `Option<HashMap>` for compatibility
    pub fn capabilities_to_hashmap(&self) -> Option<HashMap<String, serde_json::Value>> {
        self.capabilities.as_ref().and_then(|c| c.to_hashmap())
    }

    /// Set protocol version
    pub fn with_protocol_version(mut self, version: impl Into<String>) -> Self {
        self.protocol_version = Some(version.into());
        self
    }

    /// Set capabilities
    pub fn with_capabilities(mut self, capabilities: ProtocolCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Create from `Option<HashMap>` for compatibility
    pub fn with_capabilities_hashmap(
        mut self,
        capabilities: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.capabilities = capabilities.map(|fields| ProtocolCapabilities { fields });
        self
    }

    /// Get or create capabilities
    pub fn capabilities_mut(&mut self) -> &mut ProtocolCapabilities {
        self.capabilities
            .get_or_insert_with(ProtocolCapabilities::new)
    }
}

impl fmt::Display for ClientInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} v{}", self.name, self.version)?;
        if let Some(proto) = &self.protocol_version {
            write!(f, " (protocol: {})", proto)?;
        }
        Ok(())
    }
}

/// Implementation metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

impl Implementation {
    /// Create new implementation info
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }

    /// Create default implementation info for this library
    pub fn default_library() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl Default for Implementation {
    fn default() -> Self {
        Self::default_library()
    }
}

impl fmt::Display for Implementation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} v{}", self.name, self.version)
    }
}

/// Default protocol version
pub static DEFAULT_PROTOCOL_VERSION: Lazy<String> = Lazy::new(|| "2024-11-05".to_string());

/// Get default protocol version
pub fn default_protocol_version() -> String {
    DEFAULT_PROTOCOL_VERSION.clone()
}

/// Protocol metadata builder
pub struct MetadataBuilder {
    name: String,
    version: String,
    protocol_version: Option<String>,
    capabilities: Option<ProtocolCapabilities>,
}

impl MetadataBuilder {
    /// Create new builder
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            protocol_version: None,
            capabilities: None,
        }
    }

    /// Set protocol version
    pub fn protocol_version(mut self, version: impl Into<String>) -> Self {
        self.protocol_version = Some(version.into());
        self
    }

    /// Set capabilities
    pub fn capabilities(mut self, capabilities: ProtocolCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Add capability
    pub fn capability<K: Into<String>, V: Into<Value>>(mut self, key: K, value: V) -> Self {
        let caps = self
            .capabilities
            .get_or_insert_with(ProtocolCapabilities::new);
        caps.set(key, value);
        self
    }

    /// Build ServerInfo
    pub fn build_server(self) -> ServerInfo {
        ServerInfo {
            name: self.name,
            version: self.version,
            protocol_version: self.protocol_version,
            capabilities: self.capabilities,
        }
    }

    /// Build ClientInfo
    pub fn build_client(self) -> ClientInfo {
        ClientInfo {
            name: self.name,
            version: self.version,
            protocol_version: self.protocol_version,
            capabilities: self.capabilities,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_capabilities() {
        let mut caps = ProtocolCapabilities::new();
        assert!(caps.is_empty());

        caps.set("feature1", true);
        caps.set("feature2", "value");
        caps.set("feature3", 42);

        assert!(caps.has("feature1"));
        assert_eq!(
            caps.get("feature2"),
            Some(&Value::String("value".to_string()))
        );
        assert_eq!(caps.get("feature3"), Some(&Value::Number(42.into())));

        let removed = caps.remove("feature1");
        assert_eq!(removed, Some(Value::Bool(true)));
        assert!(!caps.has("feature1"));
    }

    #[test]
    fn test_server_info() {
        let server = ServerInfo::new("test-server", "1.0.0").with_protocol_version("2024-11-05");

        assert_eq!(server.name, "test-server");
        assert_eq!(server.version, "1.0.0");
        assert_eq!(server.protocol_version, Some("2024-11-05".to_string()));
        assert_eq!(
            server.to_string(),
            "test-server v1.0.0 (protocol: 2024-11-05)"
        );
    }

    #[test]
    fn test_client_info() {
        let mut client = ClientInfo::new("test-client", "2.0.0");
        let caps = client.capabilities_mut();
        caps.set("feature", true);

        assert_eq!(client.name, "test-client");
        assert_eq!(client.version, "2.0.0");
        assert!(client.capabilities.as_ref().unwrap().has("feature"));
    }

    #[test]
    fn test_implementation() {
        let impl_info = Implementation::new("custom-impl", "3.0.0");
        assert_eq!(impl_info.name, "custom-impl");
        assert_eq!(impl_info.version, "3.0.0");
        assert_eq!(impl_info.to_string(), "custom-impl v3.0.0");

        let default_impl = Implementation::default();
        assert_eq!(default_impl.name, env!("CARGO_PKG_NAME"));
        assert_eq!(default_impl.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_metadata_builder() {
        let server = MetadataBuilder::new("builder-test", "1.0.0")
            .protocol_version("2024-11-05")
            .capability("feature1", true)
            .capability("feature2", "enabled")
            .build_server();

        assert_eq!(server.name, "builder-test");
        assert_eq!(server.version, "1.0.0");
        assert_eq!(server.protocol_version, Some("2024-11-05".to_string()));
        assert!(server.capabilities.as_ref().unwrap().has("feature1"));
        assert!(server.capabilities.as_ref().unwrap().has("feature2"));
    }

    #[test]
    fn test_capabilities_serialization() {
        let mut caps = ProtocolCapabilities::new();
        caps.set("test", true);

        let json = serde_json::to_string(&caps).unwrap();
        let deserialized: ProtocolCapabilities = serde_json::from_str(&json).unwrap();

        assert_eq!(caps, deserialized);
    }

    #[test]
    fn test_empty_capabilities_to_option() {
        let caps = ProtocolCapabilities::new();
        assert_eq!(caps.to_option(), None);

        let mut caps = ProtocolCapabilities::new();
        caps.set("feature", true);
        assert!(caps.to_option().is_some());
    }

    #[test]
    fn test_capabilities_from_hashmap() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Value::Bool(true));
        map.insert("key2".to_string(), Value::String("value".to_string()));

        let caps = ProtocolCapabilities::from_hashmap(Some(map.clone()));
        assert_eq!(caps.fields, map);

        let empty_caps = ProtocolCapabilities::from_hashmap(None);
        assert!(empty_caps.is_empty());
    }
}
