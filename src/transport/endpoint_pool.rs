//! Multi-endpoint transport with conservative, idempotency-aware failover.

use async_trait::async_trait;
use std::time::{Duration, Instant};

use crate::core::error::{McpError, McpResult};
use crate::protocol::methods;
use crate::protocol::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use crate::transport::traits::Transport;

/// Failover and circuit configuration for an endpoint pool.
#[derive(Debug, Clone)]
pub struct EndpointPoolConfig {
    /// Consecutive recoverable failures before an endpoint is temporarily open.
    pub failure_threshold: u32,
    /// How long an open endpoint is excluded from selection.
    pub cooldown: Duration,
}

impl Default for EndpointPoolConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            cooldown: Duration::from_secs(30),
        }
    }
}

struct Endpoint {
    name: String,
    transport: Box<dyn Transport>,
    consecutive_failures: u32,
    open_until: Option<Instant>,
}

/// A round-robin endpoint pool with per-endpoint circuit state.
///
/// Read-only MCP methods may fail over to another endpoint. Mutating methods,
/// including `tools/call`, are attempted once unless the request contains
/// `params._meta.idempotencyKey`.
pub struct EndpointPoolTransport {
    endpoints: Vec<Endpoint>,
    config: EndpointPoolConfig,
    cursor: usize,
}

impl EndpointPoolTransport {
    pub fn new(config: EndpointPoolConfig) -> Self {
        Self {
            endpoints: Vec::new(),
            config,
            cursor: 0,
        }
    }

    pub fn add_endpoint(
        mut self,
        name: impl Into<String>,
        transport: impl Transport + 'static,
    ) -> Self {
        self.endpoints.push(Endpoint {
            name: name.into(),
            transport: Box::new(transport),
            consecutive_failures: 0,
            open_until: None,
        });
        self
    }

    pub fn endpoint_count(&self) -> usize {
        self.endpoints.len()
    }

    fn selectable_indices(&mut self) -> Vec<usize> {
        let now = Instant::now();
        for endpoint in &mut self.endpoints {
            if endpoint.open_until.is_some_and(|until| until <= now) {
                endpoint.open_until = None;
                endpoint.consecutive_failures = 0;
            }
        }

        let len = self.endpoints.len();
        if len == 0 {
            return Vec::new();
        }
        let start = self.cursor % len;
        self.cursor = (self.cursor + 1) % len;
        (0..len)
            .map(|offset| (start + offset) % len)
            .filter(|index| self.endpoints[*index].open_until.is_none())
            .collect()
    }

    fn record_success(&mut self, index: usize) {
        self.endpoints[index].consecutive_failures = 0;
        self.endpoints[index].open_until = None;
    }

    fn record_failure(&mut self, index: usize) {
        let endpoint = &mut self.endpoints[index];
        endpoint.consecutive_failures = endpoint.consecutive_failures.saturating_add(1);
        if endpoint.consecutive_failures >= self.config.failure_threshold.max(1) {
            endpoint.open_until = Some(Instant::now() + self.config.cooldown);
            tracing::warn!(
                endpoint = %endpoint.name,
                cooldown_ms = self.config.cooldown.as_millis() as u64,
                "endpoint circuit opened"
            );
        }
    }
}

/// Whether a request is safe to replay on another endpoint.
pub fn is_request_idempotent(request: &JsonRpcRequest) -> bool {
    let naturally_idempotent = matches!(
        request.method.as_str(),
        methods::PING
            | methods::TOOLS_LIST
            | methods::RESOURCES_LIST
            | methods::RESOURCES_READ
            | methods::RESOURCES_TEMPLATES_LIST
            | methods::PROMPTS_LIST
            | methods::PROMPTS_GET
            | methods::RPC_DISCOVER
    );
    naturally_idempotent
        || request
            .params
            .as_ref()
            .and_then(|params| params.get("_meta"))
            .and_then(|meta| meta.get("idempotencyKey"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|key| !key.is_empty())
}

#[async_trait]
impl Transport for EndpointPoolTransport {
    async fn send_request(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let indices = self.selectable_indices();
        if indices.is_empty() {
            return Err(McpError::Connection(
                "endpoint pool has no available endpoints".to_string(),
            ));
        }

        let max_attempts = if is_request_idempotent(&request) {
            indices.len()
        } else {
            1
        };
        let mut last_error = None;

        for index in indices.into_iter().take(max_attempts) {
            match self.endpoints[index]
                .transport
                .send_request(request.clone())
                .await
            {
                Ok(response) => {
                    self.record_success(index);
                    return Ok(response);
                }
                Err(error) => {
                    let recoverable = error.is_recoverable();
                    if recoverable {
                        self.record_failure(index);
                    } else {
                        // The endpoint responded; a protocol, validation, or
                        // authorization rejection is not endpoint unhealthiness.
                        self.record_success(index);
                    }
                    last_error = Some(error);
                    if !recoverable {
                        break;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| McpError::Connection("all selected endpoints failed".to_string())))
    }

    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        let index = self
            .selectable_indices()
            .into_iter()
            .next()
            .ok_or_else(|| McpError::Connection("no available endpoint".to_string()))?;
        let result = self.endpoints[index]
            .transport
            .send_notification(notification)
            .await;
        match &result {
            Ok(()) => self.record_success(index),
            Err(error) if error.is_recoverable() => self.record_failure(index),
            Err(_) => self.record_success(index),
        }
        result
    }

    async fn receive_notification(&mut self) -> McpResult<Option<JsonRpcNotification>> {
        for index in self.selectable_indices() {
            match self.endpoints[index].transport.receive_notification().await {
                Ok(Some(notification)) => return Ok(Some(notification)),
                Ok(None) => {}
                Err(error) if error.is_recoverable() => self.record_failure(index),
                Err(error) => return Err(error),
            }
        }
        Ok(None)
    }

    async fn close(&mut self) -> McpResult<()> {
        let mut first_error = None;
        for endpoint in &mut self.endpoints {
            if let Err(error) = endpoint.transport.close().await {
                first_error.get_or_insert(error);
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    fn is_connected(&self) -> bool {
        self.endpoints
            .iter()
            .any(|endpoint| endpoint.open_until.is_none() && endpoint.transport.is_connected())
    }

    fn connection_info(&self) -> String {
        format!("endpoint pool ({} endpoints)", self.endpoints.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    struct MockTransport {
        calls: Arc<Mutex<usize>>,
        results: VecDeque<McpResult<JsonRpcResponse>>,
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn send_request(&mut self, _request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
            *self.calls.lock().unwrap() += 1;
            self.results.pop_front().unwrap()
        }

        async fn send_notification(&mut self, _notification: JsonRpcNotification) -> McpResult<()> {
            Ok(())
        }

        async fn receive_notification(&mut self) -> McpResult<Option<JsonRpcNotification>> {
            Ok(None)
        }

        async fn close(&mut self) -> McpResult<()> {
            Ok(())
        }
    }

    fn request(method: &str, params: Option<Value>) -> JsonRpcRequest {
        JsonRpcRequest::new(json!(1), method.to_string(), params).unwrap()
    }

    #[tokio::test]
    async fn reads_fail_over_to_the_next_endpoint() {
        let first_calls = Arc::new(Mutex::new(0));
        let second_calls = Arc::new(Mutex::new(0));
        let first = MockTransport {
            calls: first_calls.clone(),
            results: VecDeque::from([Err(McpError::connection("down"))]),
        };
        let second = MockTransport {
            calls: second_calls.clone(),
            results: VecDeque::from([Ok(JsonRpcResponse::success(json!(1), json!([])).unwrap())]),
        };
        let mut pool = EndpointPoolTransport::new(EndpointPoolConfig::default())
            .add_endpoint("first", first)
            .add_endpoint("second", second);

        assert!(pool
            .send_request(request(methods::TOOLS_LIST, None))
            .await
            .is_ok());
        assert_eq!(*first_calls.lock().unwrap(), 1);
        assert_eq!(*second_calls.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn unkeyed_tool_calls_are_never_replayed() {
        let first_calls = Arc::new(Mutex::new(0));
        let second_calls = Arc::new(Mutex::new(0));
        let first = MockTransport {
            calls: first_calls.clone(),
            results: VecDeque::from([Err(McpError::connection("response lost"))]),
        };
        let second = MockTransport {
            calls: second_calls.clone(),
            results: VecDeque::from([Ok(JsonRpcResponse::success(json!(1), json!({})).unwrap())]),
        };
        let mut pool = EndpointPoolTransport::new(EndpointPoolConfig::default())
            .add_endpoint("first", first)
            .add_endpoint("second", second);

        assert!(pool
            .send_request(request(
                methods::TOOLS_CALL,
                Some(json!({"name": "charge", "arguments": {}}))
            ))
            .await
            .is_err());
        assert_eq!(*first_calls.lock().unwrap(), 1);
        assert_eq!(*second_calls.lock().unwrap(), 0);
    }

    #[test]
    fn tool_call_requires_an_idempotency_key_for_replay() {
        assert!(!is_request_idempotent(&request(
            methods::TOOLS_CALL,
            Some(json!({"name": "charge"}))
        )));
        assert!(is_request_idempotent(&request(
            methods::TOOLS_CALL,
            Some(json!({
                "name": "charge",
                "_meta": {"idempotencyKey": "operation-42"}
            }))
        )));
    }
}
