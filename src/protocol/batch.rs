// ! JSON-RPC Batch Request/Response Support (2025-06-18)
// !
// ! Module provides support for JSON-RPC batch operations as defined in the
// ! JSON-RPC 2.0 specification, even though the MCP spec notes "simplified JSON-RPC
// ! without batching". Implementation is provided for completeness and future
// ! compatibility

use crate::core::error::{McpError, McpResult};
use crate::protocol::types::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// Batch Types
// ============================================================================

/// A JSON-RPC batch request containing multiple requests/notifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct BatchRequest {
    /// The individual requests in the batch
    pub requests: Vec<BatchRequestItem>,
}

/// Individual item in a batch request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BatchRequestItem {
    /// A regular request expecting a response
    Request(JsonRpcRequest),
    /// A notification (no response expected)
    Notification(JsonRpcNotification),
}

/// A JSON-RPC batch response containing multiple responses/errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct BatchResponse {
    /// The individual responses in the batch
    pub responses: Vec<BatchResponseItem>,
}

/// Individual item in a batch response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BatchResponseItem {
    /// A successful response
    Response(JsonRpcResponse),
    /// An error response
    Error(JsonRpcError),
}

// ============================================================================
// Batch Request Implementation
// ============================================================================

impl BatchRequest {
    /// Create a new empty batch request
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    /// Create a batch request with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            requests: Vec::with_capacity(capacity),
        }
    }

    /// Add a request to the batch
    pub fn add_request(mut self, request: JsonRpcRequest) -> Self {
        self.requests.push(BatchRequestItem::Request(request));
        self
    }

    /// Add a notification to the batch
    pub fn add_notification(mut self, notification: JsonRpcNotification) -> Self {
        self.requests
            .push(BatchRequestItem::Notification(notification));
        self
    }

    /// Add a method call as a request
    pub fn add_call<T: Serialize>(
        mut self,
        id: RequestId,
        method: String,
        params: Option<T>,
    ) -> McpResult<Self> {
        let request = JsonRpcRequest::new(id, method, params)?;
        self.requests.push(BatchRequestItem::Request(request));
        Ok(self)
    }

    /// Add a method call as a notification
    pub fn add_notify<T: Serialize>(
        mut self,
        method: String,
        params: Option<T>,
    ) -> McpResult<Self> {
        let notification = JsonRpcNotification::new(method, params)?;
        self.requests
            .push(BatchRequestItem::Notification(notification));
        Ok(self)
    }

    /// Get the number of requests in the batch
    pub fn len(&self) -> usize {
        self.requests.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    /// Validate the batch request
    pub fn validate(&self) -> McpResult<()> {
        if self.is_empty() {
            return Err(McpError::Protocol(
                "Batch request must contain at least one request".to_string(),
            ));
        }

        // Check for duplicate IDs among requests
        let mut seen_ids = std::collections::HashSet::new();
        for item in &self.requests {
            if let BatchRequestItem::Request(req) = item {
                if let serde_json::Value::Null = req.id {
                    // Null IDs are allowed
                    continue;
                }
                let id_str = serde_json::to_string(&req.id)
                    .map_err(|e| McpError::Protocol(format!("Invalid request ID: {e}")))?;
                if !seen_ids.insert(id_str) {
                    return Err(McpError::Protocol(format!(
                        "Duplicate request ID in batch: {:?}",
                        req.id
                    )));
                }
            }
        }

        Ok(())
    }

    /// Split batch into requests and notifications
    pub fn split(self) -> (Vec<JsonRpcRequest>, Vec<JsonRpcNotification>) {
        let mut requests = Vec::new();
        let mut notifications = Vec::new();

        for item in self.requests {
            match item {
                BatchRequestItem::Request(req) => requests.push(req),
                BatchRequestItem::Notification(notif) => notifications.push(notif),
            }
        }

        (requests, notifications)
    }
}

impl Default for BatchRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<JsonRpcRequest>> for BatchRequest {
    fn from(requests: Vec<JsonRpcRequest>) -> Self {
        Self {
            requests: requests
                .into_iter()
                .map(BatchRequestItem::Request)
                .collect(),
        }
    }
}

impl From<Vec<JsonRpcNotification>> for BatchRequest {
    fn from(notifications: Vec<JsonRpcNotification>) -> Self {
        Self {
            requests: notifications
                .into_iter()
                .map(BatchRequestItem::Notification)
                .collect(),
        }
    }
}

// ============================================================================
// Batch Response Implementation
// ============================================================================

impl BatchResponse {
    /// Create a new empty batch response
    pub fn new() -> Self {
        Self {
            responses: Vec::new(),
        }
    }

    /// Create a batch response with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            responses: Vec::with_capacity(capacity),
        }
    }

    /// Add a successful response to the batch
    pub fn add_response(mut self, response: JsonRpcResponse) -> Self {
        self.responses.push(BatchResponseItem::Response(response));
        self
    }

    /// Add an error response to the batch
    pub fn add_error(mut self, error: JsonRpcError) -> Self {
        self.responses.push(BatchResponseItem::Error(error));
        self
    }

    /// Add a success result
    pub fn add_success<T: Serialize>(mut self, id: RequestId, result: T) -> McpResult<Self> {
        let response = JsonRpcResponse::success(id, result)?;
        self.responses.push(BatchResponseItem::Response(response));
        Ok(self)
    }

    /// Add an error result
    pub fn add_failure(
        mut self,
        id: RequestId,
        code: i32,
        message: String,
        data: Option<serde_json::Value>,
    ) -> Self {
        let error = JsonRpcError::error(id, code, message, data);
        self.responses.push(BatchResponseItem::Error(error));
        self
    }

    /// Get the number of responses in the batch
    pub fn len(&self) -> usize {
        self.responses.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.responses.is_empty()
    }

    /// Validate the batch response
    pub fn validate(&self) -> McpResult<()> {
        // Batch responses can be empty if all requests were notifications
        // No duplicate ID check needed for responses
        Ok(())
    }

    /// Split batch into successes and errors
    pub fn split(self) -> (Vec<JsonRpcResponse>, Vec<JsonRpcError>) {
        let mut successes = Vec::new();
        let mut errors = Vec::new();

        for item in self.responses {
            match item {
                BatchResponseItem::Response(resp) => successes.push(resp),
                BatchResponseItem::Error(err) => errors.push(err),
            }
        }

        (successes, errors)
    }

    /// Check if all responses are successful
    pub fn all_successful(&self) -> bool {
        self.responses
            .iter()
            .all(|item| matches!(item, BatchResponseItem::Response(_)))
    }

    /// Check if any response is an error
    pub fn has_errors(&self) -> bool {
        self.responses
            .iter()
            .any(|item| matches!(item, BatchResponseItem::Error(_)))
    }

    /// Get all error responses
    pub fn errors(&self) -> Vec<&JsonRpcError> {
        self.responses
            .iter()
            .filter_map(|item| {
                if let BatchResponseItem::Error(err) = item {
                    Some(err)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for BatchResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<JsonRpcResponse>> for BatchResponse {
    fn from(responses: Vec<JsonRpcResponse>) -> Self {
        Self {
            responses: responses
                .into_iter()
                .map(BatchResponseItem::Response)
                .collect(),
        }
    }
}

impl From<Vec<JsonRpcError>> for BatchResponse {
    fn from(errors: Vec<JsonRpcError>) -> Self {
        Self {
            responses: errors.into_iter().map(BatchResponseItem::Error).collect(),
        }
    }
}

// ============================================================================
// Batch Processor
// ============================================================================

/// Helper for processing batch requests
pub struct BatchProcessor;

impl BatchProcessor {
    /// Process a batch request and return a batch response
    pub async fn process<F, Fut>(batch: BatchRequest, handler: F) -> McpResult<BatchResponse>
    where
        F: Fn(JsonRpcRequest) -> Fut,
        Fut: std::future::Future<Output = McpResult<serde_json::Value>>,
    {
        batch.validate()?;

        let mut response = BatchResponse::new();
        let (requests, _notifications) = batch.split();

        // Process each request
        // Note: Notifications don't get responses
        for request in requests {
            let id = request.id.clone();
            match handler(request).await {
                Ok(result) => {
                    response = response.add_success(id, result)?;
                }
                Err(err) => {
                    let (code, message) = match err {
                        McpError::Protocol(msg) => (error_codes::INVALID_REQUEST, msg),
                        McpError::MethodNotFound(msg) => (error_codes::METHOD_NOT_FOUND, msg),
                        McpError::InvalidParams(msg) => (error_codes::INVALID_PARAMS, msg),
                        _ => (error_codes::INTERNAL_ERROR, err.to_string()),
                    };
                    response = response.add_failure(id, code, message, None);
                }
            }
        }

        Ok(response)
    }

    /// Create a parse error response for invalid batch JSON
    pub fn parse_error() -> BatchResponse {
        BatchResponse {
            responses: vec![BatchResponseItem::Error(JsonRpcError::error(
                serde_json::Value::Null,
                error_codes::PARSE_ERROR,
                "Invalid JSON-RPC batch request".to_string(),
                None,
            ))],
        }
    }

    /// Create an empty batch error response
    pub fn empty_batch_error() -> BatchResponse {
        BatchResponse {
            responses: vec![BatchResponseItem::Error(JsonRpcError::error(
                serde_json::Value::Null,
                error_codes::INVALID_REQUEST,
                "Batch request must not be empty".to_string(),
                None,
            ))],
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
    fn test_batch_request_creation() {
        let batch = BatchRequest::new()
            .add_call(
                json!(1),
                "method1".to_string(),
                Some(json!({"key": "value"})),
            )
            .unwrap()
            .add_notify("notification1".to_string(), Some(json!({"data": "test"})))
            .unwrap();

        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());

        let (requests, notifications) = batch.split();
        assert_eq!(requests.len(), 1);
        assert_eq!(notifications.len(), 1);
    }

    #[test]
    fn test_batch_response_creation() {
        let batch = BatchResponse::new()
            .add_success(json!(1), json!({"result": "success"}))
            .unwrap()
            .add_failure(
                json!(2),
                error_codes::METHOD_NOT_FOUND,
                "Method not found".to_string(),
                None,
            );

        assert_eq!(batch.len(), 2);
        assert!(!batch.all_successful());
        assert!(batch.has_errors());
        assert_eq!(batch.errors().len(), 1);
    }

    #[test]
    fn test_batch_serialization() {
        // Test batch request serialization
        let req1 = JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: json!(1),
            method: "test.method1".to_string(),
            params: Some(json!({"param": "value"})),
        };

        let notif1 = JsonRpcNotification {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: "test.notify".to_string(),
            params: Some(json!({"event": "occurred"})),
        };

        let batch = BatchRequest {
            requests: vec![
                BatchRequestItem::Request(req1),
                BatchRequestItem::Notification(notif1),
            ],
        };

        let json = serde_json::to_value(&batch).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0]["id"], 1);
        assert_eq!(json[0]["method"], "test.method1");
        assert!(json[1]["id"].is_null());
        assert_eq!(json[1]["method"], "test.notify");

        // Test deserialization
        let batch2: BatchRequest = serde_json::from_value(json).unwrap();
        assert_eq!(batch.len(), batch2.len());
    }

    #[test]
    fn test_batch_validation() {
        // Empty batch should fail validation
        let empty_batch = BatchRequest::new();
        assert!(empty_batch.validate().is_err());

        // Batch with duplicate IDs should fail
        let duplicate_batch = BatchRequest::new()
            .add_call(json!(1), "method1".to_string(), None::<()>)
            .unwrap()
            .add_call(json!(1), "method2".to_string(), None::<()>)
            .unwrap();
        assert!(duplicate_batch.validate().is_err());

        // Valid batch should pass
        let valid_batch = BatchRequest::new()
            .add_call(json!(1), "method1".to_string(), None::<()>)
            .unwrap()
            .add_call(json!(2), "method2".to_string(), None::<()>)
            .unwrap()
            .add_notify("notify".to_string(), None::<()>)
            .unwrap();
        assert!(valid_batch.validate().is_ok());
    }

    #[test]
    fn test_batch_response_helpers() {
        let batch = BatchResponse::new()
            .add_success(json!(1), json!({"data": "result1"}))
            .unwrap()
            .add_success(json!(2), json!({"data": "result2"}))
            .unwrap();

        assert!(batch.all_successful());
        assert!(!batch.has_errors());
        assert_eq!(batch.errors().len(), 0);

        let batch_with_error = batch.add_failure(
            json!(3),
            error_codes::INTERNAL_ERROR,
            "Internal error".to_string(),
            None,
        );

        assert!(!batch_with_error.all_successful());
        assert!(batch_with_error.has_errors());
        assert_eq!(batch_with_error.errors().len(), 1);
    }

    #[tokio::test]
    async fn test_batch_processor() {
        let batch = BatchRequest::new()
            .add_call(json!(1), "echo".to_string(), Some(json!({"msg": "hello"})))
            .unwrap()
            .add_call(json!(2), "echo".to_string(), Some(json!({"msg": "world"})))
            .unwrap();

        let response = BatchProcessor::process(batch, |req| async move {
            if req.method == "echo" {
                Ok(req.params.unwrap_or(json!({})))
            } else {
                Err(McpError::MethodNotFound(format!(
                    "Unknown method: {}",
                    req.method
                )))
            }
        })
        .await
        .unwrap();

        assert_eq!(response.len(), 2);
        assert!(response.all_successful());
    }

    #[test]
    fn test_special_batch_errors() {
        let parse_error = BatchProcessor::parse_error();
        assert_eq!(parse_error.len(), 1);
        assert!(parse_error.has_errors());

        let empty_error = BatchProcessor::empty_batch_error();
        assert_eq!(empty_error.len(), 1);
        assert!(empty_error.has_errors());
    }
}
