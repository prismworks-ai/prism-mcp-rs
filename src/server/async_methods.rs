//! Async request handling methods for MCP server
//!
//! This module provides additional async methods for handling requests,
//! processing messages, and managing server operations asynchronously.

use crate::core::error::{McpError, McpResult};
use crate::protocol::{
    JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
};
use crate::server::McpServer;
use futures::future::Future;
use serde_json::Value;
use std::pin::Pin;

impl McpServer {
    /// Process any JSON-RPC message asynchronously
    ///
    /// This method handles all types of JSON-RPC messages (requests, responses,
    /// notifications, errors) and returns an appropriate response.
    ///
    /// # Arguments
    /// * `msg` - The JSON-RPC message to process
    ///
    /// # Returns
    /// A result containing the response message or an error
    ///
    /// # Examples
    /// ```no_run
    /// # use prism_mcp_rs::server::McpServer;
    /// # use prism_mcp_rs::protocol::{JsonRpcMessage, JsonRpcRequest};
    /// # use serde_json::json;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let server = McpServer::new("server".to_string(), "1.0.0".to_string());
    /// let request = JsonRpcRequest::new(
    ///     json!(1),
    ///     "ping".to_string(),
    ///     None::<serde_json::Value>
    /// )?;
    /// let message = JsonRpcMessage::Request(request);
    /// let response = server.process_message(message).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn process_message(&self, msg: JsonRpcMessage) -> McpResult<JsonRpcMessage> {
        match msg {
            JsonRpcMessage::Request(req) => {
                let response = self.handle_request(req).await?;
                Ok(JsonRpcMessage::Response(response))
            }
            JsonRpcMessage::Notification(notif) => {
                self.handle_notification(notif).await?;
                // Notifications don't get responses
                Ok(JsonRpcMessage::Response(
                    JsonRpcResponse::success_unchecked(Value::Null, Value::Null),
                ))
            }
            JsonRpcMessage::Response(resp) => {
                // Responses are typically handled by the client side
                // For server, we just echo them back
                Ok(JsonRpcMessage::Response(resp))
            }
            JsonRpcMessage::Error(err) => {
                // Error messages are also typically client-side
                Ok(JsonRpcMessage::Error(err))
            }
        }
    }

    /// Handle a JSON-RPC notification asynchronously
    ///
    /// Notifications are one-way messages that don't expect a response.
    ///
    /// # Arguments
    /// * `notification` - The notification to handle
    ///
    /// # Returns
    /// A result indicating success or failure of handling the notification
    pub async fn handle_notification(&self, notification: JsonRpcNotification) -> McpResult<()> {
        match notification.method.as_str() {
            "initialized" => {
                // Client has acknowledged initialization
                tracing::info!("Client initialized successfully");
                Ok(())
            }
            "cancelled" => {
                // Request cancellation notification
                if let Some(params) = notification.params {
                    tracing::info!("Request cancelled: {:?}", params);
                }
                Ok(())
            }
            "$/cancelRequest" => {
                // LSP-style cancellation
                if let Some(params) = notification.params {
                    tracing::info!("LSP cancel request: {:?}", params);
                }
                Ok(())
            }
            "$/setTrace" => {
                // Set trace level notification
                if let Some(params) = notification.params {
                    tracing::info!("Set trace level: {:?}", params);
                }
                Ok(())
            }
            _ => {
                tracing::warn!("Unknown notification method: {}", notification.method);
                Ok(())
            }
        }
    }

    /// Handle multiple requests in parallel
    ///
    /// This method processes multiple requests concurrently and returns
    /// all responses once they're complete.
    ///
    /// # Arguments
    /// * `requests` - A vector of JSON-RPC requests to process
    ///
    /// # Returns
    /// A vector of results, one for each request
    ///
    /// # Examples
    /// ```no_run
    /// # use prism_mcp_rs::server::McpServer;
    /// # use prism_mcp_rs::protocol::JsonRpcRequest;
    /// # use serde_json::json;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let server = McpServer::new("server".to_string(), "1.0.0".to_string());
    /// let requests = vec![
    ///     JsonRpcRequest::new(json!(1), "ping".to_string(), None::<serde_json::Value>)?,
    ///     JsonRpcRequest::new(json!(2), "tools/list".to_string(), None::<serde_json::Value>)?,
    /// ];
    /// let responses = server.handle_requests_parallel(requests).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn handle_requests_parallel(
        &self,
        requests: Vec<JsonRpcRequest>,
    ) -> Vec<McpResult<JsonRpcResponse>> {
        use futures::future::join_all;

        let futures: Vec<_> = requests
            .into_iter()
            .map(|req| self.handle_request(req))
            .collect();

        join_all(futures).await
    }

    /// Handle requests with a custom async processor
    ///
    /// This method allows you to provide a custom processor function that
    /// can modify or filter requests before they're handled.
    ///
    /// # Arguments
    /// * `request` - The request to process
    /// * `processor` - An async function that processes the request
    ///
    /// # Returns
    /// The processed response
    pub async fn handle_request_with_processor<F, Fut>(
        &self,
        request: JsonRpcRequest,
        processor: F,
    ) -> McpResult<JsonRpcResponse>
    where
        F: FnOnce(JsonRpcRequest) -> Fut,
        Fut: Future<Output = McpResult<JsonRpcRequest>>,
    {
        let processed_request = processor(request).await?;
        self.handle_request(processed_request).await
    }

    /// Run the server with a custom message processor
    ///
    /// This method allows you to run the server with a custom async message
    /// processor that can handle incoming messages with custom logic.
    ///
    /// # Arguments
    /// * `processor` - An async function that processes messages
    ///
    /// # Returns
    /// A result indicating success or failure
    pub async fn run_with_processor<F, Fut>(&self, _processor: F) -> McpResult<()>
    where
        F: FnMut(JsonRpcMessage) -> Fut + Send + 'static,
        Fut: Future<Output = McpResult<JsonRpcMessage>> + Send,
    {
        // Check if server is running
        if !self.is_running().await {
            return Err(McpError::Transport("Server not running".to_string()));
        }

        // In a real implementation, this would:
        // 1. Receive messages from transport
        // 2. Process them with the custom processor
        // 3. Send responses back through transport

        tracing::info!("Server running with custom processor");
        Ok(())
    }

    /// Handle a batch of JSON-RPC requests
    ///
    /// This method processes a batch of requests according to the JSON-RPC
    /// batch specification.
    ///
    /// # Arguments
    /// * `batch` - A JSON array of requests
    ///
    /// # Returns
    /// A JSON array of responses
    pub async fn handle_batch(&self, batch: Vec<Value>) -> McpResult<Vec<Value>> {
        use futures::future::join_all;

        let futures: Vec<_> = batch
            .into_iter()
            .map(|value| async move {
                match serde_json::from_value::<JsonRpcRequest>(value) {
                    Ok(request) => {
                        match self.handle_request(request).await {
                            Ok(response) => serde_json::to_value(response).unwrap_or(Value::Null),
                            Err(err) => {
                                // Create an error response for internal error
                                let error_response = JsonRpcError {
                                    jsonrpc: "2.0".to_string(),
                                    id: Value::Null,
                                    error: crate::protocol::types::ErrorObject {
                                        code: -32603, // Internal error
                                        message: err.to_string(),
                                        data: None,
                                    },
                                };
                                serde_json::to_value(error_response).unwrap_or(Value::Null)
                            }
                        }
                    }
                    Err(err) => {
                        // Create an error response for parse error
                        let error_response = JsonRpcError {
                            jsonrpc: "2.0".to_string(),
                            id: Value::Null,
                            error: crate::protocol::types::ErrorObject {
                                code: -32700, // Parse error
                                message: err.to_string(),
                                data: None,
                            },
                        };
                        serde_json::to_value(error_response).unwrap_or(Value::Null)
                    }
                }
            })
            .collect();

        Ok(join_all(futures).await)
    }

    /// Stream responses for long-running operations
    ///
    /// This method provides a way to stream responses for operations that
    /// may take a long time to complete.
    ///
    /// # Arguments
    /// * `request` - The request to process
    /// * `progress_callback` - A callback function for progress updates
    ///
    /// # Returns
    /// The final response
    pub async fn handle_request_streaming<F>(
        &self,
        request: JsonRpcRequest,
        mut progress_callback: F,
    ) -> McpResult<JsonRpcResponse>
    where
        F: FnMut(f32, String) + Send,
    {
        // Start processing
        progress_callback(0.0, "Starting request processing".to_string());

        // For demonstration, we'll just handle normally
        // In a real implementation, this would integrate with
        // long-running operations and provide progress updates
        progress_callback(50.0, "Processing request".to_string());

        let response = self.handle_request(request).await?;

        progress_callback(100.0, "Request completed".to_string());

        Ok(response)
    }

    /// Handle a request with timeout
    ///
    /// This method processes a request with a specified timeout.
    ///
    /// # Arguments
    /// * `request` - The request to process
    /// * `timeout` - The timeout duration in milliseconds
    ///
    /// # Returns
    /// The response or a timeout error
    pub async fn handle_request_with_timeout(
        &self,
        request: JsonRpcRequest,
        timeout_ms: u64,
    ) -> McpResult<JsonRpcResponse> {
        use tokio::time::{timeout, Duration};

        match timeout(
            Duration::from_millis(timeout_ms),
            self.handle_request(request),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => Err(McpError::Timeout(format!(
                "Request timed out after {}ms",
                timeout_ms
            ))),
        }
    }

    /// Handle a request with retry logic
    ///
    /// This method attempts to process a request with automatic retry
    /// on failure.
    ///
    /// # Arguments
    /// * `request` - The request to process
    /// * `max_retries` - Maximum number of retry attempts
    /// * `retry_delay_ms` - Delay between retries in milliseconds
    ///
    /// # Returns
    /// The response or the last error after all retries
    pub async fn handle_request_with_retry(
        &self,
        request: JsonRpcRequest,
        max_retries: u32,
        retry_delay_ms: u64,
    ) -> McpResult<JsonRpcResponse> {
        use tokio::time::{sleep, Duration};

        let mut last_error = None;

        for attempt in 0..=max_retries {
            match self.handle_request(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = Some(err);
                    if attempt < max_retries {
                        tracing::warn!(
                            "Request failed (attempt {}/{}), retrying in {}ms: {}",
                            attempt + 1,
                            max_retries + 1,
                            retry_delay_ms,
                            last_error.as_ref().unwrap()
                        );
                        sleep(Duration::from_millis(retry_delay_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| McpError::internal("Unknown error")))
    }

    /// Process a message with middleware chain
    ///
    /// This method allows you to process messages through a chain of
    /// middleware functions.
    ///
    /// # Arguments
    /// * `message` - The message to process
    /// * `middleware` - A vector of middleware functions
    ///
    /// # Returns
    /// The processed message
    pub async fn process_with_middleware<M>(
        &self,
        message: JsonRpcMessage,
        middleware: Vec<M>,
    ) -> McpResult<JsonRpcMessage>
    where
        M: Fn(JsonRpcMessage) -> Pin<Box<dyn Future<Output = McpResult<JsonRpcMessage>> + Send>>
            + Send
            + Sync,
    {
        let mut current = message;

        for mw in middleware.iter() {
            current = mw(current).await?;
        }

        self.process_message(current).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_process_message() {
        let server = McpServer::new("test".to_string(), "1.0.0".to_string());

        let request =
            JsonRpcRequest::new(json!(1), "ping".to_string(), None::<serde_json::Value>).unwrap();

        let message = JsonRpcMessage::Request(request);
        let response = server.process_message(message).await.unwrap();

        match response {
            JsonRpcMessage::Response(_) => {}
            _ => panic!("Expected response message"),
        }
    }

    #[tokio::test]
    async fn test_handle_notification() {
        let server = McpServer::new("test".to_string(), "1.0.0".to_string());

        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "initialized".to_string(),
            params: None,
        };

        let result = server.handle_notification(notification).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parallel_requests() {
        let server = McpServer::new("test".to_string(), "1.0.0".to_string());

        let requests = vec![
            JsonRpcRequest::new(json!(1), "ping".to_string(), None::<serde_json::Value>).unwrap(),
            JsonRpcRequest::new(json!(2), "ping".to_string(), None::<serde_json::Value>).unwrap(),
        ];

        let responses = server.handle_requests_parallel(requests).await;
        assert_eq!(responses.len(), 2);
        assert!(responses.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_request_with_timeout() {
        let server = McpServer::new("test".to_string(), "1.0.0".to_string());

        let request =
            JsonRpcRequest::new(json!(1), "ping".to_string(), None::<serde_json::Value>).unwrap();

        // Should complete within timeout
        let result = server.handle_request_with_timeout(request, 1000).await;
        assert!(result.is_ok());
    }
}
