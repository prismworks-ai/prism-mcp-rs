// ! improved Metadata Support for MCP Protocol (2025-06-18)
// !
// ! Module provides complete metadata handling for requests and responses,
// ! including progress token support and extensible metadata fields.

use crate::protocol::types::ProgressToken;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Request Metadata
// ============================================================================

/// improved request metadata with progress token support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RequestMetadata {
    /// Progress token for out-of-band progress notifications
    #[serde(rename = "progressToken", skip_serializing_if = "Option::is_none")]
    pub progress_token: Option<ProgressToken>,

    /// Additional custom metadata fields
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl RequestMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Create metadata with progress token
    pub fn with_progress_token<T: Into<ProgressToken>>(token: T) -> Self {
        Self {
            progress_token: Some(token.into()),
            custom: HashMap::new(),
        }
    }

    /// Add progress token to metadata
    pub fn set_progress_token<T: Into<ProgressToken>>(mut self, token: T) -> Self {
        self.progress_token = Some(token.into());
        self
    }

    /// Add custom metadata field
    pub fn add_custom<K: Into<String>, V: Serialize>(
        mut self,
        key: K,
        value: V,
    ) -> Result<Self, serde_json::Error> {
        self.custom.insert(key.into(), serde_json::to_value(value)?);
        Ok(self)
    }

    /// Check if metadata has any fields
    pub fn is_empty(&self) -> bool {
        self.progress_token.is_none() && self.custom.is_empty()
    }

    /// Convert to Option<HashMap> for compatibility
    pub fn to_hashmap(&self) -> Option<HashMap<String, serde_json::Value>> {
        if self.is_empty() {
            None
        } else {
            let mut map = self.custom.clone();
            if let Some(ref token) = self.progress_token {
                map.insert("progressToken".to_string(), token.clone());
            }
            Some(map)
        }
    }

    /// Create from Option<HashMap> for compatibility
    pub fn from_hashmap(map: Option<HashMap<String, serde_json::Value>>) -> Self {
        match map {
            None => Self::default(),
            Some(mut map) => {
                let progress_token = map.remove("progressToken");
                Self {
                    progress_token,
                    custom: map,
                }
            }
        }
    }
}

// ============================================================================
// Response Metadata
// ============================================================================

/// improved response metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResponseMetadata {
    /// Processing time in milliseconds
    #[serde(rename = "processingTime", skip_serializing_if = "Option::is_none")]
    pub processing_time: Option<u64>,

    /// Server timestamp (ISO 8601)
    #[serde(rename = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Request ID for correlation
    #[serde(rename = "requestId", skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// Additional custom metadata fields
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl ResponseMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Create metadata with processing time
    pub fn with_processing_time(time_ms: u64) -> Self {
        Self {
            processing_time: Some(time_ms),
            ..Default::default()
        }
    }

    /// Set timestamp
    pub fn set_timestamp<S: Into<String>>(mut self, timestamp: S) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Set request ID for correlation
    pub fn set_request_id<S: Into<String>>(mut self, id: S) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Add custom metadata field
    pub fn add_custom<K: Into<String>, V: Serialize>(
        mut self,
        key: K,
        value: V,
    ) -> Result<Self, serde_json::Error> {
        self.custom.insert(key.into(), serde_json::to_value(value)?);
        Ok(self)
    }

    /// Check if metadata has any fields
    pub fn is_empty(&self) -> bool {
        self.processing_time.is_none()
            && self.timestamp.is_none()
            && self.request_id.is_none()
            && self.custom.is_empty()
    }

    /// Convert to Option<HashMap> for compatibility
    pub fn to_hashmap(&self) -> Option<HashMap<String, serde_json::Value>> {
        if self.is_empty() {
            None
        } else {
            let mut map = self.custom.clone();
            if let Some(time) = self.processing_time {
                map.insert("processingTime".to_string(), serde_json::json!(time));
            }
            if let Some(ref ts) = self.timestamp {
                map.insert("timestamp".to_string(), serde_json::json!(ts));
            }
            if let Some(ref id) = self.request_id {
                map.insert("requestId".to_string(), serde_json::json!(id));
            }
            Some(map)
        }
    }

    /// Create from Option<HashMap> for compatibility
    pub fn from_hashmap(map: Option<HashMap<String, serde_json::Value>>) -> Self {
        match map {
            None => Self::default(),
            Some(mut map) => {
                let processing_time = map.remove("processingTime").and_then(|v| v.as_u64());
                let timestamp = map
                    .remove("timestamp")
                    .and_then(|v| v.as_str().map(String::from));
                let request_id = map
                    .remove("requestId")
                    .and_then(|v| v.as_str().map(String::from));

                Self {
                    processing_time,
                    timestamp,
                    request_id,
                    custom: map,
                }
            }
        }
    }
}

// ============================================================================
// Metadata Builder
// ============================================================================

/// Builder for creating complex metadata
pub struct MetadataBuilder {
    request: RequestMetadata,
    response: ResponseMetadata,
}

impl MetadataBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            request: RequestMetadata::new(),
            response: ResponseMetadata::new(),
        }
    }

    /// Add progress token for request
    pub fn with_progress_token<T: Into<ProgressToken>>(mut self, token: T) -> Self {
        self.request.progress_token = Some(token.into());
        self
    }

    /// Add processing time for response
    pub fn with_processing_time(mut self, time_ms: u64) -> Self {
        self.response.processing_time = Some(time_ms);
        self
    }

    /// Add timestamp for response
    pub fn with_timestamp<S: Into<String>>(mut self, timestamp: S) -> Self {
        self.response.timestamp = Some(timestamp.into());
        self
    }

    /// Add request ID for correlation
    pub fn with_request_id<S: Into<String>>(mut self, id: S) -> Self {
        self.response.request_id = Some(id.into());
        self
    }

    /// Add custom field to request metadata
    pub fn add_request_field<K: Into<String>, V: Serialize>(
        mut self,
        key: K,
        value: V,
    ) -> Result<Self, serde_json::Error> {
        self.request
            .custom
            .insert(key.into(), serde_json::to_value(value)?);
        Ok(self)
    }

    /// Add custom field to response metadata
    pub fn add_response_field<K: Into<String>, V: Serialize>(
        mut self,
        key: K,
        value: V,
    ) -> Result<Self, serde_json::Error> {
        self.response
            .custom
            .insert(key.into(), serde_json::to_value(value)?);
        Ok(self)
    }

    /// Build request metadata
    pub fn build_request(self) -> RequestMetadata {
        self.request
    }

    /// Build response metadata
    pub fn build_response(self) -> ResponseMetadata {
        self.response
    }

    /// Build both request and response metadata
    pub fn build(self) -> (RequestMetadata, ResponseMetadata) {
        (self.request, self.response)
    }
}

impl Default for MetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Traits for Metadata Support
// ============================================================================

/// Trait for types that support request metadata
pub trait HasRequestMetadata {
    /// Get request metadata
    fn metadata(&self) -> Option<&RequestMetadata>;

    /// Get mutable request metadata
    fn metadata_mut(&mut self) -> Option<&mut RequestMetadata>;

    /// Set request metadata
    fn set_metadata(&mut self, metadata: RequestMetadata);

    /// Add progress token
    fn add_progress_token<T: Into<ProgressToken>>(&mut self, token: T) {
        if let Some(meta) = self.metadata_mut() {
            meta.progress_token = Some(token.into());
        } else {
            self.set_metadata(RequestMetadata::with_progress_token(token));
        }
    }
}

/// Trait for types that support response metadata
pub trait HasResponseMetadata {
    /// Get response metadata
    fn metadata(&self) -> Option<&ResponseMetadata>;

    /// Get mutable response metadata
    fn metadata_mut(&mut self) -> Option<&mut ResponseMetadata>;

    /// Set response metadata
    fn set_metadata(&mut self, metadata: ResponseMetadata);

    /// Add processing time
    fn add_processing_time(&mut self, time_ms: u64) {
        if let Some(meta) = self.metadata_mut() {
            meta.processing_time = Some(time_ms);
        } else {
            self.set_metadata(ResponseMetadata::with_processing_time(time_ms));
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_metadata() {
        let meta = RequestMetadata::with_progress_token("token-123")
            .add_custom("source", "test")
            .unwrap();

        assert_eq!(meta.progress_token, Some(json!("token-123")));
        assert_eq!(meta.custom.get("source"), Some(&json!("test")));

        // Test serialization
        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["progressToken"], "token-123");
        assert_eq!(json["source"], "test");

        // Test deserialization
        let meta2: RequestMetadata = serde_json::from_value(json).unwrap();
        assert_eq!(meta, meta2);
    }

    #[test]
    fn test_response_metadata() {
        let meta = ResponseMetadata::with_processing_time(150)
            .set_timestamp("2025-01-12T15:00:00Z")
            .set_request_id("req-123");

        assert_eq!(meta.processing_time, Some(150));
        assert_eq!(meta.timestamp, Some("2025-01-12T15:00:00Z".to_string()));
        assert_eq!(meta.request_id, Some("req-123".to_string()));

        // Test conversion to HashMap
        let map = meta.to_hashmap().unwrap();
        assert_eq!(map.get("processingTime"), Some(&json!(150)));
        assert_eq!(map.get("timestamp"), Some(&json!("2025-01-12T15:00:00Z")));
        assert_eq!(map.get("requestId"), Some(&json!("req-123")));
    }

    #[test]
    fn test_metadata_builder() {
        let builder = MetadataBuilder::new()
            .with_progress_token("progress-123")
            .with_processing_time(200)
            .with_timestamp("2025-01-12T15:00:00Z")
            .with_request_id("req-456");

        let (req_meta, resp_meta) = builder.build();

        assert_eq!(req_meta.progress_token, Some(json!("progress-123")));
        assert_eq!(resp_meta.processing_time, Some(200));
        assert_eq!(
            resp_meta.timestamp,
            Some("2025-01-12T15:00:00Z".to_string())
        );
        assert_eq!(resp_meta.request_id, Some("req-456".to_string()));
    }

    #[test]
    fn test_empty_metadata() {
        let req_meta = RequestMetadata::new();
        assert!(req_meta.is_empty());
        assert!(req_meta.to_hashmap().is_none());

        let resp_meta = ResponseMetadata::new();
        assert!(resp_meta.is_empty());
        assert!(resp_meta.to_hashmap().is_none());
    }

    #[test]
    fn test_hashmap_conversion() {
        let mut map = HashMap::new();
        map.insert("progressToken".to_string(), json!("token-789"));
        map.insert("custom_field".to_string(), json!("value"));

        let req_meta = RequestMetadata::from_hashmap(Some(map));
        assert_eq!(req_meta.progress_token, Some(json!("token-789")));
        assert_eq!(req_meta.custom.get("custom_field"), Some(&json!("value")));

        // Test round-trip
        let map2 = req_meta.to_hashmap().unwrap();
        let req_meta2 = RequestMetadata::from_hashmap(Some(map2));
        assert_eq!(req_meta, req_meta2);
    }
}
