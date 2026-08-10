//! HTTP transport implementation for MCP
//!
//! Module provides HTTP-based transport for MCP communication,
//! including Server-Sent Events (SSE) for real-time communication.
//!
//! ## Feature Requirements
//!
//! - Basic HTTP transport: Requires "http" feature
//! - Server-Sent Events: Requires both "http" and "sse" features
//!
//! ```toml
//! # Cargo.toml
//! [dependencies]
//! prism-mcp-rs = { version = "3", features = ["http", "sse"] }

use async_trait::async_trait;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use axum::response::{sse::Event, Sse};
use reqwest::Client;
use serde_json::Value;
use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};

#[cfg(feature = "sse")]
use futures::Stream;
use futures::StreamExt;

#[cfg(feature = "sse")]
use tokio_stream::wrappers::BroadcastStream;

use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::Instrument;

#[cfg(feature = "tls")]
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder as HyperServerBuilder,
    service::TowerToHyperService,
};

use crate::core::error::{McpError, McpResult};
use crate::core::logging::ErrorContext;
use crate::protocol::{
    encode_http_header_value, has_tasks_extension, json_rpc_error_details, methods,
    modern_request_context, request_protocol_version, request_routing_name, tool_call_headers,
    tool_header_mappings,
    types::{
        error_codes, JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest,
        JsonRpcResponse,
    },
    validate_http_headers, validate_tool_call_headers, ServerCapabilities, SubscriptionFilter,
    SubscriptionsAcknowledgedParams, SubscriptionsListenParams, HEADER_MISMATCH, MCP_METHOD_HEADER,
    MCP_NAME_HEADER, MCP_PROTOCOL_VERSION_HEADER, SUBSCRIPTION_ID_META_KEY, TASKS_EXTENSION_ID,
    UNSUPPORTED_PROTOCOL_VERSION,
};
use crate::transport::traits::{
    ClientSubscription, ConnectionState, ServerTransport, Transport, TransportConfig,
};

const FORBIDDEN_ERROR: i32 = -32010;
const RATE_LIMITED_ERROR: i32 = -32011;

fn parse_sse_response(bytes: &[u8], request_id: &Value) -> McpResult<Value> {
    let body = String::from_utf8_lossy(bytes).replace("\r\n", "\n");
    for event in body.split("\n\n") {
        let data = event
            .lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim_start)
            .collect::<Vec<_>>()
            .join("\n");
        if data.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&data)
            .map_err(|error| McpError::Serialization(format!("invalid SSE JSON data: {error}")))?;
        if value.get("id") == Some(request_id)
            && (value.get("result").is_some() || value.get("error").is_some())
        {
            return Ok(value);
        }
    }
    Err(McpError::Serialization(
        "SSE response ended without a JSON-RPC result for the request".to_string(),
    ))
}

#[cfg(feature = "otel")]
struct HeaderExtractor<'a>(&'a HeaderMap);

#[cfg(feature = "otel")]
impl opentelemetry::propagation::Extractor for HeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(axum::http::HeaderName::as_str).collect()
    }
}

#[cfg(feature = "otel")]
struct MapInjector<'a>(&'a mut HashMap<String, String>);

#[cfg(feature = "otel")]
impl opentelemetry::propagation::Injector for MapInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key.to_string(), value);
    }
}

#[cfg(feature = "otel")]
fn inject_trace_context(mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let context = tracing::Span::current().context();
    let mut headers = HashMap::new();
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&context, &mut MapInjector(&mut headers));
    });
    for (key, value) in headers {
        request = request.header(key, value);
    }
    request
}

/// PEM-encoded client identity and trust root for mutual TLS.
#[cfg(feature = "tls")]
#[derive(Debug, Clone)]
pub struct MtlsClientConfig {
    pub identity_pem: Vec<u8>,
    pub ca_certificate_pem: Vec<u8>,
}

#[cfg(feature = "tls")]
impl MtlsClientConfig {
    pub fn new(identity_pem: impl Into<Vec<u8>>, ca_certificate_pem: impl Into<Vec<u8>>) -> Self {
        Self {
            identity_pem: identity_pem.into(),
            ca_certificate_pem: ca_certificate_pem.into(),
        }
    }
}

/// PEM-encoded server identity and client CA used to require client certificates.
#[cfg(feature = "tls")]
#[derive(Debug, Clone)]
pub struct MtlsServerConfig {
    pub certificate_chain_pem: Vec<u8>,
    pub private_key_pem: Vec<u8>,
    pub client_ca_pem: Vec<u8>,
}

#[cfg(feature = "tls")]
impl MtlsServerConfig {
    pub fn new(
        certificate_chain_pem: impl Into<Vec<u8>>,
        private_key_pem: impl Into<Vec<u8>>,
        client_ca_pem: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            certificate_chain_pem: certificate_chain_pem.into(),
            private_key_pem: private_key_pem.into(),
            client_ca_pem: client_ca_pem.into(),
        }
    }

    fn build_rustls(&self) -> McpResult<rustls::ServerConfig> {
        use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
        use rustls::server::WebPkiClientVerifier;
        use rustls::RootCertStore;

        let certificates = CertificateDer::pem_slice_iter(&self.certificate_chain_pem)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                McpError::Authentication(format!("invalid server certificate: {error}"))
            })?;
        if certificates.is_empty() {
            return Err(McpError::Authentication(
                "mTLS server certificate chain is empty".to_string(),
            ));
        }

        let private_key =
            PrivateKeyDer::from_pem_slice(&self.private_key_pem).map_err(|error| {
                McpError::Authentication(format!("invalid server private key: {error}"))
            })?;

        let client_ca = CertificateDer::pem_slice_iter(&self.client_ca_pem)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| McpError::Authentication(format!("invalid client CA: {error}")))?;
        let mut roots = RootCertStore::empty();
        let (accepted, rejected) = roots.add_parsable_certificates(client_ca);
        if accepted == 0 || rejected > 0 {
            return Err(McpError::Authentication(format!(
                "client CA contained {accepted} accepted and {rejected} rejected certificates"
            )));
        }

        let verifier = WebPkiClientVerifier::builder(Arc::new(roots))
            .build()
            .map_err(|error| {
                McpError::Authentication(format!("invalid client verifier: {error}"))
            })?;
        rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
            .with_client_cert_verifier(verifier)
            .with_single_cert(certificates, private_key)
            .map_err(|error| McpError::Authentication(format!("invalid server identity: {error}")))
    }
}

// ============================================================================
// HTTP Client Transport
// ============================================================================

/// HTTP transport for MCP clients
///
/// This transport communicates with an MCP server via HTTP requests and
/// optionally uses Server-Sent Events for real-time notifications.
#[derive(Debug)]
pub struct HttpClientTransport {
    pub(crate) client: Client,
    pub(crate) base_url: String,
    pub(crate) sse_url: Option<String>,
    pub(crate) headers: HeaderMap,
    /// For tracking active requests (currently used for metrics/debugging)
    pending_requests: Arc<Mutex<HashMap<Value, tokio::sync::oneshot::Sender<JsonRpcResponse>>>>,
    notification_receiver: Option<mpsc::UnboundedReceiver<JsonRpcNotification>>,
    pub(crate) config: TransportConfig,
    state: ConnectionState,
    request_id_counter: Arc<Mutex<u64>>,
    /// Most recently accepted input schema for each discovered tool.
    tool_schemas: HashMap<String, Value>,
    subscription_tasks: Arc<Mutex<HashMap<String, tokio::task::AbortHandle>>>,
}

impl HttpClientTransport {
    /// Create a new HTTP client transport
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the MCP server
    /// * `sse_url` - Optional URL for Server-Sent Events (for notifications)
    ///
    /// # Returns
    /// Result containing the transport or an error
    pub async fn new<S: AsRef<str>>(base_url: S, sse_url: Option<S>) -> McpResult<Self> {
        Self::with_config(base_url, sse_url, TransportConfig::default()).await
    }

    /// Create a new HTTP client transport with custom configuration
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the MCP server
    /// * `sse_url` - Optional URL for Server-Sent Events
    /// * `config` - Transport configuration
    ///
    /// # Returns
    /// Result containing the transport or an error
    pub async fn with_config<S: AsRef<str>>(
        base_url: S,
        sse_url: Option<S>,
        config: TransportConfig,
    ) -> McpResult<Self> {
        let client_builder = Client::builder()
            .timeout(Duration::from_millis(
                config.read_timeout_ms.unwrap_or(60_000),
            ))
            .connect_timeout(Duration::from_millis(
                config.connect_timeout_ms.unwrap_or(30_000),
            ));

        // Note: reqwest doesn't have a gzip() method, it's enabled by default with features

        let client = client_builder
            .build()
            .map_err(|e| McpError::Http(format!("Failed to create HTTP client: {e}")))?;

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert(
            "Accept",
            "application/json, text/event-stream".parse().unwrap(),
        );

        // Add custom headers from config
        for (key, value) in &config.headers {
            if let (Ok(header_name), Ok(header_value)) = (
                key.parse::<axum::http::HeaderName>(),
                value.parse::<axum::http::HeaderValue>(),
            ) {
                headers.insert(header_name, header_value);
            }
        }

        let (notification_sender, notification_receiver) = mpsc::unbounded_channel();

        // Set up SSE connection for notifications if URL provided
        if let Some(sse_url) = &sse_url {
            let sse_url = sse_url.as_ref().to_string();
            let client_clone = client.clone();
            let headers_clone = headers.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_sse_stream(
                    client_clone,
                    sse_url,
                    headers_clone,
                    notification_sender,
                )
                .await
                {
                    tracing::error!("SSE stream error: {}", e);
                }
            });
        }

        Ok(Self {
            client,
            base_url: base_url.as_ref().to_string(),
            sse_url: sse_url.map(|s| s.as_ref().to_string()),
            headers,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            notification_receiver: Some(notification_receiver),
            config,
            state: ConnectionState::Connected,
            request_id_counter: Arc::new(Mutex::new(0)),
            tool_schemas: HashMap::new(),
            subscription_tasks: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Create an HTTP client that presents a certificate and validates the
    /// server against the supplied private CA.
    #[cfg(feature = "tls")]
    pub async fn with_mtls<S: AsRef<str>>(
        base_url: S,
        sse_url: Option<S>,
        config: TransportConfig,
        mtls: MtlsClientConfig,
    ) -> McpResult<Self> {
        let identity = reqwest::Identity::from_pem(&mtls.identity_pem).map_err(|error| {
            McpError::Authentication(format!("invalid client identity: {error}"))
        })?;
        let root = reqwest::Certificate::from_pem(&mtls.ca_certificate_pem)
            .map_err(|error| McpError::Authentication(format!("invalid server CA: {error}")))?;
        let client = Client::builder()
            .timeout(Duration::from_millis(
                config.read_timeout_ms.unwrap_or(60_000),
            ))
            .connect_timeout(Duration::from_millis(
                config.connect_timeout_ms.unwrap_or(30_000),
            ))
            .identity(identity)
            .tls_certs_only([root])
            .min_tls_version(reqwest::tls::Version::TLS_1_3)
            .build()
            .map_err(|error| McpError::Http(format!("failed to create mTLS client: {error}")))?;

        let base = base_url.as_ref().to_string();
        let sse = sse_url.as_ref().map(|url| url.as_ref().to_string());
        let mut transport = Self::with_config(base.as_str(), None::<&str>, config).await?;
        transport.client = client.clone();
        transport.sse_url = sse.clone();

        if let Some(url) = sse {
            let headers = transport.headers.clone();
            let sender = {
                let (sender, receiver) = mpsc::unbounded_channel();
                transport.notification_receiver = Some(receiver);
                sender
            };
            tokio::spawn(async move {
                if let Err(error) = Self::handle_sse_stream(client, url, headers, sender).await {
                    tracing::error!(%error, "mTLS SSE stream failed");
                }
            });
        }
        Ok(transport)
    }

    async fn handle_sse_stream(
        client: Client,
        sse_url: String,
        headers: HeaderMap,
        notification_sender: mpsc::UnboundedSender<JsonRpcNotification>,
    ) -> McpResult<()> {
        let mut request = client.get(&sse_url);
        #[cfg(feature = "otel")]
        {
            request = inject_trace_context(request);
        }
        for (name, value) in headers.iter() {
            // Convert axum headers to reqwest headers
            let name_str = name.as_str();
            let value_bytes = value.as_bytes();
            request = request.header(name_str, value_bytes);
        }

        let _response = request
            .send()
            .await
            .map_err(|e| McpError::Http(format!("SSE connection failed: {e}")))?;

        #[cfg(feature = "sse")]
        {
            let mut stream = _response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.lines() {
                            if let Some(data) = line.strip_prefix("data: ") {
                                // Remove "data: " prefix
                                if let Ok(notification) =
                                    serde_json::from_str::<JsonRpcNotification>(data)
                                {
                                    if notification_sender.send(notification).is_err() {
                                        tracing::debug!("Notification receiver dropped");
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("SSE stream error: {}", e);
                        break;
                    }
                }
            }
        }

        #[cfg(not(feature = "sse"))]
        {
            let _ = notification_sender; // Silence unused warning
            tracing::warn!("SSE streaming requires SSE feature");
        }

        Ok(())
    }

    pub async fn next_request_id(&self) -> u64 {
        let mut counter = self.request_id_counter.lock().await;
        *counter += 1;
        *counter
    }

    fn mcp_url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        if base.ends_with("/mcp") {
            base.to_string()
        } else {
            format!("{base}/mcp")
        }
    }

    /// Track request for metrics/debugging purposes
    async fn track_request(&self, request_id: &Value) {
        // For HTTP transport, we mainly use this for debugging and metrics
        // Since HTTP is synchronous request/response, we don't need the async
        // tracking that WebSocket uses, but we keep the interface for consistency
        let mut pending = self.pending_requests.lock().await;
        let (sender, _receiver) = tokio::sync::oneshot::channel();
        pending.insert(request_id.clone(), sender);
    }

    /// Remove tracked request
    async fn untrack_request(&self, request_id: &Value) {
        let mut pending = self.pending_requests.lock().await;
        pending.remove(request_id);
    }

    /// Get count of active requests (for debugging/metrics)
    pub async fn active_request_count(&self) -> usize {
        let pending = self.pending_requests.lock().await;
        pending.len()
    }

    fn capture_tool_schemas(&mut self, response: &mut JsonRpcResponse) {
        let Some(tools) = response
            .result
            .as_mut()
            .and_then(Value::as_object_mut)
            .and_then(|result| result.get_mut("tools"))
            .and_then(Value::as_array_mut)
        else {
            return;
        };
        self.tool_schemas.clear();
        tools.retain(|tool| {
            let Some(name) = tool.get("name").and_then(Value::as_str) else {
                return false;
            };
            let Some(schema) = tool.get("inputSchema") else {
                return false;
            };
            match tool_header_mappings(schema) {
                Ok(_) => {
                    self.tool_schemas.insert(name.to_string(), schema.clone());
                    true
                }
                Err(error) => {
                    tracing::warn!(tool.name = name, %error, "excluding tool with invalid x-mcp-header schema");
                    false
                }
            }
        });
    }

    #[cfg(test)]
    pub fn has_notification_receiver(&self) -> bool {
        self.notification_receiver.is_some()
    }
}

#[async_trait]
impl Transport for HttpClientTransport {
    async fn send_request(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        // Generate request ID if not present or ensure we have a valid ID
        let request_with_id = if request.id == Value::Null {
            let request_id = self.next_request_id().await;
            JsonRpcRequest {
                id: Value::from(request_id),
                ..request
            }
        } else {
            request
        };

        // Create error context for logging
        let context = ErrorContext::new("http_send_request")
            .with_transport("http")
            .with_method(&request_with_id.method)
            .with_extra("request_id", request_with_id.id.clone())
            .with_extra("base_url", serde_json::Value::String(self.base_url.clone()));

        // Track the request for debugging/metrics
        self.track_request(&request_with_id.id).await;

        let url = self.mcp_url();

        let mut http_request = self.client.post(&url);

        #[cfg(feature = "otel")]
        {
            http_request = inject_trace_context(http_request);
        }

        // Apply headers from config and defaults
        for (name, value) in self.headers.iter() {
            let name_str = name.as_str();
            let value_bytes = value.as_bytes();
            http_request = http_request.header(name_str, value_bytes);
        }

        if let Some(version) = request_protocol_version(&request_with_id) {
            http_request = http_request
                .header(MCP_PROTOCOL_VERSION_HEADER, version)
                .header(MCP_METHOD_HEADER, request_with_id.method.as_str());
            if let Some(name) = request_routing_name(&request_with_id) {
                http_request = http_request.header(MCP_NAME_HEADER, encode_http_header_value(name));
            }
            if request_with_id.method == methods::TOOLS_CALL {
                if let Some(params) = request_with_id.params.as_ref().and_then(Value::as_object) {
                    if let Some(tool_name) = params.get("name").and_then(Value::as_str) {
                        if let Some(schema) = self.tool_schemas.get(tool_name) {
                            let arguments = params.get("arguments").unwrap_or(&Value::Null);
                            for (name, value) in tool_call_headers(schema, arguments)? {
                                http_request = http_request.header(name, value);
                            }
                        }
                    }
                }
            }
        }

        // Apply timeout from config if specified
        if let Some(timeout_ms) = self.config.read_timeout_ms {
            http_request = http_request.timeout(Duration::from_millis(timeout_ms));
        }

        let response = http_request
            .json(&request_with_id)
            .send()
            .await
            .map_err(|e| {
                // Untrack request on error
                let request_id = request_with_id.id.clone();
                let pending_requests = self.pending_requests.clone();
                tokio::spawn(async move {
                    let mut pending = pending_requests.lock().await;
                    pending.remove(&request_id);
                });

                // Create appropriate error based on the reqwest error
                let error = if e.is_timeout() {
                    McpError::timeout("HTTP request timeout")
                } else if e.is_connect() {
                    McpError::connection(format!("HTTP connection failed: {e}"))
                } else {
                    McpError::Http(format!("HTTP request failed: {e}"))
                };

                // Log error with context
                let error_clone = error.clone();
                let context_clone = context.clone();
                tokio::spawn(async move {
                    error_clone.log_with_context(context_clone).await;
                });

                error
            })?;

        let response_status = response.status();
        let response_content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|error| McpError::Http(format!("failed to read HTTP response: {error}")))?;
        let parsed_response = if response_content_type.starts_with("text/event-stream") {
            parse_sse_response(&response_bytes, &request_with_id.id)
        } else {
            serde_json::from_slice(&response_bytes)
                .map_err(|error| McpError::Serialization(format!("invalid JSON response: {error}")))
        };
        let json_value: Value = match parsed_response {
            Ok(value) => value,
            Err(_error) if !response_status.is_success() => {
                self.untrack_request(&request_with_id.id).await;
                return Err(McpError::Http(format!(
                    "HTTP error: {} {}",
                    response_status.as_u16(),
                    response_status.canonical_reason().unwrap_or("Unknown")
                )));
            }
            Err(error) => {
                self.untrack_request(&request_with_id.id).await;
                error.clone().log_with_context(context).await;
                return Err(error);
            }
        };

        let mut result = if json_value.get("error").is_some() {
            serde_json::from_value::<JsonRpcError>(json_value)
                .map_err(|error| McpError::Serialization(error.to_string()))
                .and_then(|json_error| {
                    if json_error.id != request_with_id.id {
                        Err(McpError::Http(format!(
                            "Error response ID {:?} does not match request ID {:?}",
                            json_error.id, request_with_id.id
                        )))
                    } else {
                        Err(match json_error.error.code {
                            FORBIDDEN_ERROR => McpError::Forbidden(json_error.error.message),
                            RATE_LIMITED_ERROR => McpError::RateLimited {
                                retry_after_ms: json_error
                                    .error
                                    .data
                                    .and_then(|data| data.get("retryAfterMs").cloned())
                                    .and_then(|value| value.as_u64())
                                    .unwrap_or_default(),
                            },
                            error_codes::METHOD_NOT_FOUND => {
                                McpError::MethodNotFound(json_error.error.message)
                            }
                            HEADER_MISMATCH => McpError::HeaderMismatch(json_error.error.message),
                            crate::protocol::MISSING_REQUIRED_CLIENT_CAPABILITY => {
                                let required = json_error
                                    .error
                                    .data
                                    .and_then(|data| data.get("requiredCapabilities").cloned())
                                    .unwrap_or_else(|| serde_json::json!({}));
                                McpError::MissingRequiredClientCapability(required)
                            }
                            UNSUPPORTED_PROTOCOL_VERSION => {
                                let data = json_error.error.data.unwrap_or_default();
                                McpError::UnsupportedProtocolVersion {
                                    requested: data
                                        .get("requested")
                                        .and_then(Value::as_str)
                                        .unwrap_or("unknown")
                                        .to_string(),
                                    supported: data
                                        .get("supported")
                                        .and_then(Value::as_array)
                                        .into_iter()
                                        .flatten()
                                        .filter_map(Value::as_str)
                                        .map(str::to_string)
                                        .collect(),
                                }
                            }
                            code => McpError::Protocol(format!(
                                "JSON-RPC error {code}: {}",
                                json_error.error.message
                            )),
                        })
                    }
                })
        } else if !response_status.is_success() {
            Err(McpError::Http(format!(
                "HTTP error: {} {}",
                response_status.as_u16(),
                response_status.canonical_reason().unwrap_or("Unknown")
            )))
        } else {
            serde_json::from_value::<JsonRpcResponse>(json_value)
                .map_err(|error| McpError::Serialization(error.to_string()))
                .and_then(|json_response| {
                    if json_response.id != request_with_id.id {
                        Err(McpError::Http(format!(
                            "Response ID {:?} does not match request ID {:?}",
                            json_response.id, request_with_id.id
                        )))
                    } else {
                        Ok(json_response)
                    }
                })
        };

        if request_with_id.method == methods::TOOLS_LIST {
            if let Ok(response) = &mut result {
                self.capture_tool_schemas(response);
            }
        }
        self.untrack_request(&request_with_id.id).await;
        result
    }

    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        let url = self.mcp_url();

        let mut http_request = self.client.post(&url);

        #[cfg(feature = "otel")]
        {
            http_request = inject_trace_context(http_request);
        }

        // Apply headers from config and defaults
        for (name, value) in self.headers.iter() {
            let name_str = name.as_str();
            let value_bytes = value.as_bytes();
            http_request = http_request.header(name_str, value_bytes);
        }

        // Apply write timeout from config if specified
        if let Some(timeout_ms) = self.config.write_timeout_ms {
            http_request = http_request.timeout(Duration::from_millis(timeout_ms));
        }

        let response = http_request
            .json(&notification)
            .send()
            .await
            .map_err(|e| McpError::Http(format!("HTTP notification failed: {e}")))?;

        if !response.status().is_success() {
            return Err(McpError::Http(format!(
                "HTTP notification error: {} {}",
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        Ok(())
    }

    async fn receive_notification(&mut self) -> McpResult<Option<JsonRpcNotification>> {
        if let Some(ref mut receiver) = self.notification_receiver {
            match receiver.try_recv() {
                Ok(notification) => Ok(Some(notification)),
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => Err(McpError::Http(
                    "Notification channel disconnected".to_string(),
                )),
            }
        } else {
            Ok(None)
        }
    }

    async fn open_subscription(
        &mut self,
        request: JsonRpcRequest,
    ) -> McpResult<ClientSubscription> {
        if request.method != methods::SUBSCRIPTIONS_LISTEN {
            return Err(McpError::InvalidParams(
                "open_subscription requires subscriptions/listen".to_string(),
            ));
        }
        let version = request_protocol_version(&request).ok_or_else(|| {
            McpError::InvalidParams("subscription request is missing modern metadata".to_string())
        })?;
        let url = self.mcp_url();
        let mut http_request = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header(MCP_PROTOCOL_VERSION_HEADER, version)
            .header(MCP_METHOD_HEADER, methods::SUBSCRIPTIONS_LISTEN);
        for (name, value) in self.headers.iter() {
            if !name.as_str().eq_ignore_ascii_case("accept") {
                http_request = http_request.header(name.as_str(), value.as_bytes());
            }
        }
        #[cfg(feature = "otel")]
        {
            http_request = inject_trace_context(http_request);
        }
        let response = http_request
            .json(&request)
            .send()
            .await
            .map_err(|error| McpError::Http(format!("subscription request failed: {error}")))?;
        let status = response.status();
        if !status.is_success() {
            let bytes = response.bytes().await.unwrap_or_default();
            if let Ok(error) = serde_json::from_slice::<JsonRpcError>(&bytes) {
                return Err(match error.error.code {
                    crate::protocol::MISSING_REQUIRED_CLIENT_CAPABILITY => {
                        McpError::MissingRequiredClientCapability(
                            error
                                .error
                                .data
                                .and_then(|value| value.get("requiredCapabilities").cloned())
                                .unwrap_or_else(|| serde_json::json!({})),
                        )
                    }
                    HEADER_MISMATCH => McpError::HeaderMismatch(error.error.message),
                    code => McpError::Protocol(format!(
                        "JSON-RPC error {code}: {}",
                        error.error.message
                    )),
                });
            }
            return Err(McpError::Http(format!(
                "subscription HTTP error: {}",
                status.as_u16()
            )));
        }
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        if !content_type.starts_with("text/event-stream") {
            return Err(McpError::Http(format!(
                "subscriptions/listen requires text/event-stream, received {content_type}"
            )));
        }

        let (notification_tx, notification_rx) = mpsc::unbounded_channel();
        let (completion_tx, completion_rx) = tokio::sync::oneshot::channel();
        let request_id = request.id.clone();
        let key = request_id.to_string();
        let tasks = self.subscription_tasks.clone();
        let task_key = key.clone();
        let task = tokio::spawn(async move {
            let mut completion_tx = Some(completion_tx);
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(error) => {
                        if let Some(sender) = completion_tx.take() {
                            let _ = sender.send(Err(McpError::Http(format!(
                                "subscription stream failed: {error}"
                            ))));
                        }
                        tasks.lock().await.remove(&task_key);
                        return;
                    }
                };
                buffer.push_str(&String::from_utf8_lossy(&chunk));
                buffer = buffer.replace("\r\n", "\n");
                while let Some(boundary) = buffer.find("\n\n") {
                    let event = buffer[..boundary].to_string();
                    buffer.drain(..boundary + 2);
                    let data = event
                        .lines()
                        .filter_map(|line| line.strip_prefix("data:"))
                        .map(str::trim_start)
                        .collect::<Vec<_>>()
                        .join("\n");
                    if data.is_empty() {
                        continue;
                    }
                    let Ok(value) = serde_json::from_str::<Value>(&data) else {
                        continue;
                    };
                    if value.get("method").is_some() && value.get("id").is_none() {
                        if let Ok(notification) = serde_json::from_value(value) {
                            if notification_tx.send(notification).is_err() {
                                tasks.lock().await.remove(&task_key);
                                return;
                            }
                        }
                    } else if value.get("result").is_some() {
                        if let Some(sender) = completion_tx.take() {
                            let result = serde_json::from_value(value)
                                .map_err(|error| McpError::Serialization(error.to_string()));
                            let _ = sender.send(result);
                        }
                        tasks.lock().await.remove(&task_key);
                        return;
                    } else if value.get("error").is_some() {
                        if let Some(sender) = completion_tx.take() {
                            let message = value
                                .get("error")
                                .and_then(|error| error.get("message"))
                                .and_then(Value::as_str)
                                .unwrap_or("subscription failed");
                            let _ = sender.send(Err(McpError::Protocol(message.to_string())));
                        }
                        tasks.lock().await.remove(&task_key);
                        return;
                    }
                }
            }
            if let Some(sender) = completion_tx.take() {
                let _ = sender.send(Err(McpError::Transport(
                    "subscription stream closed without a final response".to_string(),
                )));
            }
            tasks.lock().await.remove(&task_key);
        });
        let abort_handle = task.abort_handle();
        self.subscription_tasks
            .lock()
            .await
            .insert(key, abort_handle.clone());
        Ok(
            ClientSubscription::new(request_id, notification_rx, completion_rx)
                .with_abort_handle(abort_handle),
        )
    }

    async fn cancel_subscription(&mut self, request_id: &Value) -> McpResult<()> {
        if let Some(handle) = self
            .subscription_tasks
            .lock()
            .await
            .remove(&request_id.to_string())
        {
            handle.abort();
        }
        Ok(())
    }

    async fn close(&mut self) -> McpResult<()> {
        for (_, task) in self.subscription_tasks.lock().await.drain() {
            task.abort();
        }
        self.state = ConnectionState::Disconnected;
        self.notification_receiver = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }

    fn connection_info(&self) -> String {
        format!(
            "HTTP transport (base: {}, sse: {:?}, state: {:?})",
            self.base_url, self.sse_url, self.state
        )
    }
}

// ============================================================================
// HTTP Server Transport
// ============================================================================

type HttpRequestHandler = Arc<
    dyn Fn(JsonRpcRequest) -> tokio::sync::oneshot::Receiver<McpResult<JsonRpcResponse>>
        + Send
        + Sync,
>;

/// Shared state for HTTP server transport
#[derive(Clone)]
struct HttpServerState {
    notification_sender: broadcast::Sender<JsonRpcNotification>,
    request_handler: Option<HttpRequestHandler>,
    tool_schemas: HashMap<String, Value>,
    capabilities: ServerCapabilities,
}

/// HTTP transport for MCP servers
///
/// This transport serves MCP requests over HTTP and provides Server-Sent Events
/// for real-time notifications to clients.
pub struct HttpServerTransport {
    bind_addr: String,
    config: TransportConfig,
    state: Arc<RwLock<HttpServerState>>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
    running: Arc<RwLock<bool>>,
    pending_request_handler: Option<crate::transport::traits::ServerRequestHandler>,
    pending_tool_schemas: HashMap<String, Value>,
    pending_capabilities: ServerCapabilities,
    pending_task_notifications: Option<broadcast::Receiver<JsonRpcNotification>>,
    #[cfg(feature = "tls")]
    mtls_config: Option<MtlsServerConfig>,
}

impl HttpServerTransport {
    /// Create a new HTTP server transport
    ///
    /// # Arguments
    /// * `bind_addr` - Address to bind the HTTP server to (e.g., "0.0.0.0:3000")
    ///
    /// # Returns
    /// New HTTP server transport instance
    pub fn new<S: Into<String>>(bind_addr: S) -> Self {
        Self::with_config(bind_addr, TransportConfig::default())
    }

    /// Create a new HTTP server transport with custom configuration
    ///
    /// # Arguments
    /// * `bind_addr` - Address to bind the HTTP server to
    /// * `config` - Transport configuration
    ///
    /// # Returns
    /// New HTTP server transport instance
    pub fn with_config<S: Into<String>>(bind_addr: S, config: TransportConfig) -> Self {
        let (notification_sender, _) = broadcast::channel(1000);

        Self {
            bind_addr: bind_addr.into(),
            config,
            state: Arc::new(RwLock::new(HttpServerState {
                notification_sender,
                request_handler: None,
                tool_schemas: HashMap::new(),
                capabilities: ServerCapabilities::default(),
            })),
            server_handle: None,
            running: Arc::new(RwLock::new(false)),
            pending_request_handler: None,
            pending_tool_schemas: HashMap::new(),
            pending_capabilities: ServerCapabilities::default(),
            pending_task_notifications: None,
            #[cfg(feature = "tls")]
            mtls_config: None,
        }
    }

    /// Set the request handler function
    ///
    /// # Arguments
    /// * `handler` - Function that processes incoming requests
    pub async fn set_request_handler<F>(&mut self, handler: F)
    where
        F: Fn(JsonRpcRequest) -> tokio::sync::oneshot::Receiver<JsonRpcResponse>
            + Send
            + Sync
            + 'static,
    {
        let mut state = self.state.write().await;
        state.request_handler = Some(Arc::new(move |request| {
            let response = handler(request);
            let (tx, rx) = tokio::sync::oneshot::channel();
            tokio::spawn(async move {
                let result = response.await.map_err(|error| {
                    McpError::Internal(format!("HTTP request handler channel closed: {error}"))
                });
                let _ = tx.send(result);
            });
            rx
        }));
    }

    /// Require TLS 1.3 client certificates signed by the configured client CA.
    #[cfg(feature = "tls")]
    pub fn with_mtls(mut self, config: MtlsServerConfig) -> Self {
        self.mtls_config = Some(config);
        self
    }

    #[cfg(test)]
    pub fn get_bind_addr(&self) -> &str {
        &self.bind_addr
    }

    #[cfg(test)]
    pub fn get_config(&self) -> &TransportConfig {
        &self.config
    }
}

#[async_trait]
impl ServerTransport for HttpServerTransport {
    fn set_tool_schemas(&mut self, schemas: HashMap<String, Value>) -> McpResult<()> {
        for (name, schema) in &schemas {
            tool_header_mappings(schema).map_err(|error| {
                McpError::Validation(format!(
                    "tool {name} has an invalid x-mcp-header schema: {error}"
                ))
            })?;
        }
        // Server construction occurs outside request processing; blocking here
        // would be unsafe, so retain schemas on the transport and copy them
        // into shared state when start() runs.
        self.pending_tool_schemas = schemas;
        Ok(())
    }

    fn set_server_capabilities(&mut self, capabilities: ServerCapabilities) -> McpResult<()> {
        self.pending_capabilities = capabilities;
        Ok(())
    }

    fn set_task_notifications(
        &mut self,
        receiver: broadcast::Receiver<JsonRpcNotification>,
    ) -> McpResult<()> {
        self.pending_task_notifications = Some(receiver);
        Ok(())
    }

    async fn start(&mut self) -> McpResult<()> {
        tracing::info!("Starting HTTP server on {}", self.bind_addr);

        if let Some(handler) = self.pending_request_handler.take() {
            let http_handler = Arc::new(move |request: JsonRpcRequest| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let handler_future = handler(request);
                let parent_span = tracing::Span::current();
                tokio::spawn(
                    async move {
                        let _ = tx.send(handler_future.await);
                    }
                    .instrument(parent_span),
                );
                rx
            });
            self.state.write().await.request_handler = Some(http_handler);
        }
        self.state.write().await.tool_schemas = std::mem::take(&mut self.pending_tool_schemas);
        self.state.write().await.capabilities = std::mem::take(&mut self.pending_capabilities);
        if let Some(mut task_notifications) = self.pending_task_notifications.take() {
            let sender = self.state.read().await.notification_sender.clone();
            tokio::spawn(async move {
                loop {
                    match task_notifications.recv().await {
                        Ok(notification) => {
                            let _ = sender.send(notification);
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            });
        }

        let state = self.state.clone();
        let bind_addr = self.bind_addr.clone();
        let running = self.running.clone();
        let _config = self.config.clone();

        // Create the Axum app with configuration-based settings
        let mut app = Router::new()
            .route("/mcp", post(handle_mcp_request))
            .route("/mcp/notify", post(handle_mcp_notification))
            .route("/mcp/events", get(handle_sse_events))
            .route("/health", get(handle_health_check))
            .with_state(state);

        // Apply CORS configuration
        let cors_layer = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        app = app.layer(ServiceBuilder::new().layer(cors_layer).into_inner());

        // Note: Timeout configuration is handled at the HTTP client level
        // Server-side timeouts are managed by the underlying Axum/Hyper stack

        #[cfg(feature = "tls")]
        let server_tls_config = self
            .mtls_config
            .as_ref()
            .map(MtlsServerConfig::build_rustls)
            .transpose()?
            .map(Arc::new);

        // Start the server after all fallible configuration has been validated.
        let listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .map_err(|e| McpError::Http(format!("Failed to bind to {bind_addr}: {e}")))?;

        *running.write().await = true;

        let server_handle = tokio::spawn(async move {
            #[cfg(feature = "tls")]
            if let Some(server_tls_config) = server_tls_config {
                let acceptor = tokio_rustls::TlsAcceptor::from(server_tls_config);
                loop {
                    let (tcp_stream, peer) = match listener.accept().await {
                        Ok(connection) => connection,
                        Err(error) => {
                            tracing::error!(%error, "mTLS TCP accept failed");
                            break;
                        }
                    };
                    let acceptor = acceptor.clone();
                    let service = app.clone();
                    tokio::spawn(async move {
                        let tls_stream = match acceptor.accept(tcp_stream).await {
                            Ok(stream) => stream,
                            Err(error) => {
                                tracing::warn!(%peer, %error, "mTLS handshake rejected");
                                return;
                            }
                        };
                        let service = TowerToHyperService::new(service);
                        if let Err(error) = HyperServerBuilder::new(TokioExecutor::new())
                            .serve_connection_with_upgrades(TokioIo::new(tls_stream), service)
                            .await
                        {
                            tracing::debug!(%peer, %error, "mTLS HTTP connection ended");
                        }
                    });
                }
                return;
            }

            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("HTTP server error: {}", e);
            }
        });

        self.server_handle = Some(server_handle);

        tracing::info!("HTTP server started successfully on {}", self.bind_addr);
        Ok(())
    }

    fn set_request_handler(&mut self, handler: crate::transport::traits::ServerRequestHandler) {
        self.pending_request_handler = Some(handler);
    }

    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        let state = self.state.read().await;

        if state.notification_sender.send(notification).is_err() {
            tracing::warn!("No SSE clients connected to receive notification");
        }

        Ok(())
    }

    async fn stop(&mut self) -> McpResult<()> {
        tracing::info!("Stopping HTTP server");

        *self.running.write().await = false;

        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }

        Ok(())
    }

    fn is_running(&self) -> bool {
        // Check if we have an active server handle
        self.server_handle.is_some()
    }

    fn server_info(&self) -> String {
        format!("HTTP server transport (bind: {})", self.bind_addr)
    }
}

// ============================================================================
// HTTP Route Handlers
// ============================================================================

/// Handle MCP JSON-RPC requests
async fn handle_mcp_request(
    State(state): State<Arc<RwLock<HttpServerState>>>,
    headers: HeaderMap,
    Json(message): Json<JsonRpcMessage>,
) -> Result<Response, StatusCode> {
    let protocol_header = headers
        .get(MCP_PROTOCOL_VERSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let method_header = headers
        .get(MCP_METHOD_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let name_header = headers
        .get(MCP_NAME_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let accept_header = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let custom_headers: HashMap<String, String> = headers
        .iter()
        .filter_map(|(name, value)| {
            name.as_str()
                .to_ascii_lowercase()
                .starts_with("mcp-param-")
                .then(|| {
                    value
                        .to_str()
                        .ok()
                        .map(|value| (name.as_str().to_string(), value.to_string()))
                })
                .flatten()
        })
        .collect();
    let dispatch = async move {
        match message {
            JsonRpcMessage::Request(request) => {
                let is_modern = request_protocol_version(&request).is_some();
                if let Err(error) = validate_http_headers(
                    &request,
                    protocol_header.as_deref(),
                    method_header.as_deref(),
                    name_header.as_deref(),
                ) {
                    let (code, data) = json_rpc_error_details(&error);
                    let body = JsonRpcMessage::Error(JsonRpcError::error(
                        request.id,
                        code,
                        error.to_string(),
                        data,
                    ));
                    return Ok((StatusCode::BAD_REQUEST, Json(body)).into_response());
                }
                if is_modern && request.method == methods::SUBSCRIPTIONS_LISTEN {
                    if !accept_header
                        .split(',')
                        .any(|value| value.trim().starts_with("text/event-stream"))
                    {
                        let error = McpError::HeaderMismatch(
                            "subscriptions/listen requires Accept: text/event-stream".to_string(),
                        );
                        let (code, data) = json_rpc_error_details(&error);
                        let body = JsonRpcMessage::Error(JsonRpcError::error(
                            request.id,
                            code,
                            error.to_string(),
                            data,
                        ));
                        return Ok((StatusCode::BAD_REQUEST, Json(body)).into_response());
                    }
                    return handle_subscription_stream(state, request).await;
                }
                if is_modern && request.method == methods::TOOLS_CALL {
                    let params = request.params.as_ref().and_then(Value::as_object);
                    let tool_name = params
                        .and_then(|params| params.get("name"))
                        .and_then(Value::as_str);
                    let arguments = params
                        .and_then(|params| params.get("arguments"))
                        .unwrap_or(&Value::Null);
                    let schema = if let Some(tool_name) = tool_name {
                        state.read().await.tool_schemas.get(tool_name).cloned()
                    } else {
                        None
                    };
                    let validation = match schema {
                        Some(schema) => {
                            validate_tool_call_headers(&schema, arguments, &custom_headers)
                        }
                        None => Ok(()),
                    };
                    if let Err(error) = validation {
                        let (code, data) = json_rpc_error_details(&error);
                        let body = JsonRpcMessage::Error(JsonRpcError::error(
                            request.id,
                            code,
                            error.to_string(),
                            data,
                        ));
                        return Ok((StatusCode::BAD_REQUEST, Json(body)).into_response());
                    }
                }
                handle_mcp_jsonrpc_request(state, request)
                    .await
                    .map(|message| {
                        let status = match &message {
                            JsonRpcMessage::Error(error)
                                if matches!(
                                    error.error.code,
                                    HEADER_MISMATCH
                                        | UNSUPPORTED_PROTOCOL_VERSION
                                        | crate::protocol::MISSING_REQUIRED_CLIENT_CAPABILITY
                                        | error_codes::INVALID_REQUEST
                                        | error_codes::INVALID_PARAMS
                                ) =>
                            {
                                StatusCode::BAD_REQUEST
                            }
                            JsonRpcMessage::Error(error)
                                if is_modern
                                    && error.error.code == error_codes::METHOD_NOT_FOUND =>
                            {
                                StatusCode::NOT_FOUND
                            }
                            JsonRpcMessage::Error(error) if error.error.code == FORBIDDEN_ERROR => {
                                StatusCode::FORBIDDEN
                            }
                            JsonRpcMessage::Error(error)
                                if error.error.code == RATE_LIMITED_ERROR =>
                            {
                                StatusCode::TOO_MANY_REQUESTS
                            }
                            _ => StatusCode::OK,
                        };
                        (status, Json(message)).into_response()
                    })
            }
            JsonRpcMessage::Notification(notification) => {
                handle_mcp_jsonrpc_notification(state, notification).await?;
                Ok(StatusCode::ACCEPTED.into_response())
            }
            JsonRpcMessage::Response(_) | JsonRpcMessage::Error(_) => Err(StatusCode::BAD_REQUEST),
        }
    };

    #[cfg(feature = "otel")]
    {
        use tracing_opentelemetry::OpenTelemetrySpanExt;

        let parent = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(&headers))
        });
        let span = tracing::info_span!("mcp.http", otel.kind = "server");
        let _ = span.set_parent(parent);
        dispatch.instrument(span).await
    }

    #[cfg(not(feature = "otel"))]
    {
        dispatch.await
    }
}

async fn handle_subscription_stream(
    state: Arc<RwLock<HttpServerState>>,
    request: JsonRpcRequest,
) -> Result<Response, StatusCode> {
    let context = modern_request_context(&request).map_err(|_| StatusCode::BAD_REQUEST)?;
    let context = context.ok_or(StatusCode::BAD_REQUEST)?;
    let params: SubscriptionsListenParams =
        request
            .params
            .clone()
            .ok_or(StatusCode::BAD_REQUEST)
            .and_then(|value| serde_json::from_value(value).map_err(|_| StatusCode::BAD_REQUEST))?;
    if params.notifications.requests_tasks() && !has_tasks_extension(&context.client_capabilities) {
        let error = McpError::MissingRequiredClientCapability(serde_json::json!({
            "extensions": {(TASKS_EXTENSION_ID): {}}
        }));
        let (code, data) = json_rpc_error_details(&error);
        let body = JsonRpcMessage::Error(JsonRpcError::error(
            request.id,
            code,
            error.to_string(),
            data,
        ));
        return Ok((StatusCode::BAD_REQUEST, Json(body)).into_response());
    }

    let state_guard = state.read().await;
    let capabilities = &state_guard.capabilities;
    let tasks_enabled = capabilities
        .extensions
        .as_ref()
        .is_some_and(|extensions| extensions.contains_key(TASKS_EXTENSION_ID));
    let accepted = SubscriptionFilter {
        tools_list_changed: (params.notifications.tools_list_changed == Some(true)
            && capabilities
                .tools
                .as_ref()
                .is_some_and(|value| value.list_changed == Some(true)))
        .then_some(true),
        prompts_list_changed: (params.notifications.prompts_list_changed == Some(true)
            && capabilities
                .prompts
                .as_ref()
                .is_some_and(|value| value.list_changed == Some(true)))
        .then_some(true),
        resources_list_changed: (params.notifications.resources_list_changed == Some(true)
            && capabilities
                .resources
                .as_ref()
                .is_some_and(|value| value.list_changed == Some(true)))
        .then_some(true),
        resource_subscriptions: if capabilities
            .resources
            .as_ref()
            .is_some_and(|value| value.subscribe == Some(true))
        {
            params.notifications.resource_subscriptions.clone()
        } else {
            Vec::new()
        },
        task_ids: if tasks_enabled {
            params.notifications.task_ids.clone()
        } else {
            Vec::new()
        },
    };
    let receiver = state_guard.notification_sender.subscribe();
    drop(state_guard);

    let subscription_id = request.id.clone();
    let mut ack_meta = HashMap::new();
    ack_meta.insert(
        SUBSCRIPTION_ID_META_KEY.to_string(),
        subscription_id.clone(),
    );
    let acknowledgement = JsonRpcNotification::new(
        methods::SUBSCRIPTIONS_ACKNOWLEDGED.to_string(),
        Some(SubscriptionsAcknowledgedParams {
            notifications: accepted.clone(),
            meta: ack_meta,
        }),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stream = futures::stream::unfold(
        (Some(acknowledgement), receiver, accepted, subscription_id),
        |(first, mut receiver, filter, subscription_id)| async move {
            if let Some(notification) = first {
                let data =
                    serde_json::to_string(&notification).unwrap_or_else(|_| "{}".to_string());
                return Some((
                    Ok::<Event, Infallible>(Event::default().data(data)),
                    (None, receiver, filter, subscription_id),
                ));
            }
            loop {
                let mut notification = match receiver.recv().await {
                    Ok(notification) => notification,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => return None,
                };
                if !filter.matches(&notification.method, notification.params.as_ref()) {
                    continue;
                }
                let params = notification
                    .params
                    .get_or_insert_with(|| Value::Object(serde_json::Map::new()));
                let Some(object) = params.as_object_mut() else {
                    continue;
                };
                let meta = object
                    .entry("_meta")
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
                let Some(meta) = meta.as_object_mut() else {
                    continue;
                };
                meta.insert(
                    SUBSCRIPTION_ID_META_KEY.to_string(),
                    subscription_id.clone(),
                );
                let data =
                    serde_json::to_string(&notification).unwrap_or_else(|_| "{}".to_string());
                return Some((
                    Ok::<Event, Infallible>(Event::default().data(data)),
                    (None, receiver, filter, subscription_id),
                ));
            }
        },
    );

    Ok(Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("keep-alive"),
        )
        .into_response())
}

async fn handle_mcp_jsonrpc_request(
    state: Arc<RwLock<HttpServerState>>,
    request: JsonRpcRequest,
) -> Result<JsonRpcMessage, StatusCode> {
    let state_guard = state.read().await;

    if let Some(ref handler) = state_guard.request_handler {
        let request_id = request.id.clone();
        let response_rx = handler(request);
        drop(state_guard); // Release the lock

        match response_rx.await {
            Ok(Ok(response)) => Ok(JsonRpcMessage::Response(response)),
            Ok(Err(error)) => {
                let (code, data) = match &error {
                    McpError::Forbidden(_) => (FORBIDDEN_ERROR, None),
                    McpError::RateLimited { retry_after_ms } => (
                        RATE_LIMITED_ERROR,
                        Some(serde_json::json!({"retryAfterMs": retry_after_ms})),
                    ),
                    _ => json_rpc_error_details(&error),
                };
                Ok(JsonRpcMessage::Error(JsonRpcError::error(
                    request_id,
                    code,
                    error.to_string(),
                    data,
                )))
            }
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        let error_response = JsonRpcError::error(
            request.id,
            error_codes::METHOD_NOT_FOUND,
            "No request handler configured".to_string(),
            None,
        );
        Ok(JsonRpcMessage::Error(error_response))
    }
}

async fn handle_mcp_jsonrpc_notification(
    state: Arc<RwLock<HttpServerState>>,
    notification: JsonRpcNotification,
) -> Result<(), StatusCode> {
    if !is_supported_http_notification(&notification) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let state_guard = state.read().await;
    if state_guard.notification_sender.send(notification).is_err() {
        tracing::debug!("No SSE clients connected to receive notification");
    }

    Ok(())
}

fn is_supported_http_notification(notification: &JsonRpcNotification) -> bool {
    if notification.jsonrpc != "2.0" {
        return false;
    }

    matches!(
        notification.method.as_str(),
        methods::INITIALIZED
            | methods::TOOLS_LIST_CHANGED
            | methods::RESOURCES_UPDATED
            | methods::RESOURCES_LIST_CHANGED
            | methods::PROMPTS_LIST_CHANGED
            | methods::ROOTS_LIST_CHANGED
            | methods::ELICITATION_COMPLETE
            | methods::TASKS_STATUS
            | methods::TASKS_STATUS_UPDATE
            | methods::LOGGING_MESSAGE
            | methods::PROGRESS
            | methods::CANCELLED
    )
}

/// Handle MCP notification requests
async fn handle_mcp_notification(Json(_notification): Json<JsonRpcNotification>) -> StatusCode {
    // Notifications don't require a response
    StatusCode::OK
}

/// Handle Server-Sent Events for real-time notifications
#[cfg(feature = "sse")]
async fn handle_sse_events(
    State(state): State<Arc<RwLock<HttpServerState>>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let state_guard = state.read().await;
    let receiver = state_guard.notification_sender.subscribe();
    drop(state_guard);

    let stream = BroadcastStream::new(receiver).map(|result| {
        match result {
            Ok(notification) => match serde_json::to_string(&notification) {
                Ok(json) => Ok(Event::default().data(json)),
                Err(e) => {
                    tracing::error!("Failed to serialize notification: {}", e);
                    Ok(Event::default().data("{}"))
                }
            },
            Err(_) => Ok(Event::default().data("{}")), // Lagged or closed
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    )
}

/// Handle Server-Sent Events (fallback when SSE feature not available)
#[cfg(not(feature = "sse"))]
async fn handle_sse_events(_state: State<Arc<RwLock<HttpServerState>>>) -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

/// Handle health check requests
async fn handle_health_check() -> Json<Value> {
    let timestamp = chrono::Utc::now().to_rfc3339();

    Json(serde_json::json!({
        "status": "healthy",
        "transport": "http",
        "timestamp": timestamp
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::methods;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn parses_json_rpc_result_from_standard_post_sse() {
        let body = b"event: message\r\ndata: {\"jsonrpc\":\"2.0\",\"id\":7,\"result\":{\"ok\":true}}\r\n\r\n";
        let parsed = parse_sse_response(body, &serde_json::json!(7)).unwrap();
        assert_eq!(parsed["result"]["ok"], true);
    }

    #[tokio::test]
    async fn test_http_client_creation() {
        let transport = HttpClientTransport::new("http://localhost:3000", None).await;
        assert!(transport.is_ok());

        let transport = transport.unwrap();
        assert!(transport.is_connected());
        assert_eq!(transport.base_url, "http://localhost:3000");
    }

    #[tokio::test]
    async fn test_http_server_creation() {
        let transport = HttpServerTransport::new("127.0.0.1:0");
        assert_eq!(transport.bind_addr, "127.0.0.1:0");
        assert!(!transport.is_running());
    }

    #[test]
    fn test_http_server_with_config() {
        let config = TransportConfig {
            compression: true,
            ..Default::default()
        };

        let transport = HttpServerTransport::with_config("0.0.0.0:8080", config);
        assert_eq!(transport.bind_addr, "0.0.0.0:8080");
        assert!(transport.config.compression);
    }

    #[tokio::test]
    async fn test_http_client_with_sse() {
        let transport = HttpClientTransport::new(
            "http://localhost:3000",
            Some("http://localhost:3000/events"),
        )
        .await;

        assert!(transport.is_ok());
        let transport = transport.unwrap();
        assert!(transport.sse_url.is_some());
        assert_eq!(transport.sse_url.unwrap(), "http://localhost:3000/events");
    }

    // Add complete tests for maximum coverage
    #[tokio::test]
    async fn test_request_id_generation_sequence() {
        let transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        let id1 = transport.next_request_id().await;
        let id2 = transport.next_request_id().await;
        let id3 = transport.next_request_id().await;

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[tokio::test]
    async fn test_request_tracking_complete() {
        let transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        // Initially no active requests
        assert_eq!(transport.active_request_count().await, 0);

        // Track multiple requests with different ID types
        let request_ids = vec![
            Value::from(123),
            Value::String("string-id".to_string()),
            Value::Null,
            Value::Array(vec![Value::from(1), Value::from(2)]),
        ];

        for id in &request_ids {
            transport.track_request(id).await;
        }
        assert_eq!(transport.active_request_count().await, request_ids.len());

        // Untrack all requests
        for id in &request_ids {
            transport.untrack_request(id).await;
        }
        assert_eq!(transport.active_request_count().await, 0);

        // Untrack non-existent request (should not panic)
        transport.untrack_request(&Value::from(999)).await;
        assert_eq!(transport.active_request_count().await, 0);
    }

    #[tokio::test]
    async fn test_connection_state_management() {
        let mut transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        // Initially connected
        assert!(transport.is_connected());
        assert!(transport.has_notification_receiver());

        let info_before = transport.connection_info();
        assert!(info_before.contains("Connected"));

        // Close transport
        let result = transport.close().await;
        assert!(result.is_ok());

        // Should be disconnected
        assert!(!transport.is_connected());
        assert!(!transport.has_notification_receiver());

        let info_after = transport.connection_info();
        assert!(info_after.contains("Disconnected"));
    }

    #[tokio::test]
    async fn test_receive_notification_states() {
        let mut transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        // Without SSE URL, the notification channel gets disconnected
        // This should return an error indicating disconnection
        let result = transport.receive_notification().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disconnected"));

        // After closing, should still return None (receiver is set to None)
        transport.close().await.unwrap();
        let result = transport.receive_notification().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        // Test again after close to ensure consistent behavior
        let result2 = transport.receive_notification().await;
        assert!(result2.is_ok());
        assert!(result2.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_http_server_lifecycle_complete() {
        let mut transport = HttpServerTransport::new("127.0.0.1:0");

        // Check initial state
        assert_eq!(transport.get_bind_addr(), "127.0.0.1:0");
        assert!(!transport.is_running());

        let info = transport.server_info();
        assert!(info.contains("HTTP server transport"));
        assert!(info.contains("127.0.0.1:0"));

        // Start server
        let result = transport.start().await;
        assert!(result.is_ok());
        assert!(transport.is_running());

        // Send notification while running
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test_notification".to_string(),
            params: Some(serde_json::json!({"test": true})),
        };
        let result = transport.send_notification(notification).await;
        assert!(result.is_ok());

        // Stop server
        let result = transport.stop().await;
        assert!(result.is_ok());
        assert!(!transport.is_running());

        // Should be able to stop again without error
        let result = transport.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_http_server_request_handler() {
        let mut transport = HttpServerTransport::new("127.0.0.1:0");

        let handler = |request: JsonRpcRequest| {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::json!({
                    "method_received": request.method,
                    "handled": true
                })),
            };
            let _ = tx.send(response);
            rx
        };

        transport.set_request_handler(handler).await;
        // Handler should be set successfully (we can't easily test actual handling)
    }

    #[tokio::test]
    async fn test_http_server_with_custom_config() {
        let mut config = TransportConfig {
            compression: true,
            ..Default::default()
        };
        config
            .headers
            .insert("Server".to_string(), "MCP-Test/1.0".to_string());

        let transport = HttpServerTransport::with_config("0.0.0.0:8080", config);

        assert_eq!(transport.get_bind_addr(), "0.0.0.0:8080");
        assert!(transport.get_config().compression);
        assert_eq!(
            transport.get_config().headers.get("Server"),
            Some(&"MCP-Test/1.0".to_string())
        );
    }

    #[tokio::test]
    async fn test_http_client_with_custom_config() {
        let mut config = TransportConfig {
            read_timeout_ms: Some(5000),
            connect_timeout_ms: Some(2000),
            write_timeout_ms: Some(3000),
            ..Default::default()
        };
        config
            .headers
            .insert("X-Custom-Header".to_string(), "test-value".to_string());
        config
            .headers
            .insert("Authorization".to_string(), "Bearer token123".to_string());

        let transport = HttpClientTransport::with_config(
            "http://localhost:3000",
            Some("http://localhost:3000/events"),
            config,
        )
        .await;

        assert!(transport.is_ok());
        let transport = transport.unwrap();
        assert_eq!(transport.config.read_timeout_ms, Some(5000));
        assert_eq!(transport.config.connect_timeout_ms, Some(2000));
        assert_eq!(transport.config.write_timeout_ms, Some(3000));
        assert!(transport.sse_url.is_some());
    }

    // Route handler tests
    #[tokio::test]
    async fn test_handle_health_check() {
        let result = handle_health_check().await;

        let Json(health_data) = result;
        assert_eq!(health_data["status"], "healthy");
        assert_eq!(health_data["transport"], "http");
        assert!(health_data["timestamp"].is_string());
    }

    #[tokio::test]
    async fn test_handle_mcp_notification() {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test_notification".to_string(),
            params: Some(serde_json::json!({"test": "notification"})),
        };
        let json_notification = Json(notification);

        let result = handle_mcp_notification(json_notification).await;

        // Notifications should always return OK
        assert_eq!(result, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_mcp_request_accepts_initialized_notification() {
        let (notification_sender, mut notification_receiver) = broadcast::channel(100);
        let state = Arc::new(RwLock::new(HttpServerState {
            notification_sender,
            request_handler: None,
            tool_schemas: HashMap::new(),
            capabilities: ServerCapabilities::default(),
        }));
        let message = serde_json::from_value::<JsonRpcMessage>(serde_json::json!({
            "jsonrpc": "2.0",
            "method": methods::INITIALIZED,
            "params": {}
        }))
        .unwrap();

        let response = handle_mcp_request(State(state), HeaderMap::new(), Json(message))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let received = notification_receiver.recv().await.unwrap();
        assert_eq!(received.method, methods::INITIALIZED);
    }

    #[tokio::test]
    async fn test_handle_mcp_request_rejects_unknown_notification() {
        let (notification_sender, _) = broadcast::channel(100);
        let state = Arc::new(RwLock::new(HttpServerState {
            notification_sender,
            request_handler: None,
            tool_schemas: HashMap::new(),
            capabilities: ServerCapabilities::default(),
        }));
        let message = serde_json::from_value::<JsonRpcMessage>(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/unknown",
            "params": {}
        }))
        .unwrap();

        let result = handle_mcp_request(State(state), HeaderMap::new(), Json(message)).await;

        assert!(matches!(result, Err(StatusCode::BAD_REQUEST)));
    }

    #[cfg(not(feature = "sse"))]
    #[tokio::test]
    async fn test_handle_sse_events_not_implemented() {
        let (notification_sender, _) = broadcast::channel(100);

        let state = Arc::new(RwLock::new(HttpServerState {
            notification_sender,
            request_handler: None,
            tool_schemas: HashMap::new(),
            capabilities: ServerCapabilities::default(),
        }));

        let state_extract = State(state);

        let result = handle_sse_events(state_extract).await;

        // Should return NOT_IMPLEMENTED when features are not available
        assert_eq!(result, StatusCode::NOT_IMPLEMENTED);
    }

    // Edge cases and complete coverage tests
    #[tokio::test]
    async fn test_transport_config_variations() {
        // Test default config
        let default_config = TransportConfig::default();
        assert_eq!(default_config.read_timeout_ms, Some(60_000));
        assert_eq!(default_config.write_timeout_ms, Some(30_000));
        assert_eq!(default_config.connect_timeout_ms, Some(30_000));
        assert!(default_config.headers.is_empty());

        // Test config with all options
        let mut full_config = TransportConfig {
            read_timeout_ms: Some(10000),
            write_timeout_ms: Some(5000),
            connect_timeout_ms: Some(3000),
            compression: true,
            ..Default::default()
        };
        full_config
            .headers
            .insert("Test-Header".to_string(), "test-value".to_string());

        let transport =
            HttpClientTransport::with_config("http://localhost:3000", None, full_config)
                .await
                .unwrap();

        assert_eq!(transport.config.read_timeout_ms, Some(10000));
        assert_eq!(transport.config.write_timeout_ms, Some(5000));
        assert_eq!(transport.config.connect_timeout_ms, Some(3000));
        assert!(transport.config.compression);
    }

    #[tokio::test]
    async fn test_sse_url_variations() {
        // Test with SSE URL as &str
        let transport1 = HttpClientTransport::new(
            "http://localhost:3000",
            Some("http://localhost:3000/events"),
        )
        .await
        .unwrap();
        assert!(transport1.sse_url.is_some());
        assert_eq!(
            transport1.sse_url.as_ref().unwrap(),
            "http://localhost:3000/events"
        );

        // Test with SSE URL as String
        let transport2 = HttpClientTransport::new(
            "http://localhost:3000",
            Some("http://localhost:3000/events"),
        )
        .await
        .unwrap();
        assert!(transport2.sse_url.is_some());

        // Test without SSE URL
        let transport3 = HttpClientTransport::new("http://localhost:3000", None::<&str>)
            .await
            .unwrap();
        assert!(transport3.sse_url.is_none());

        // Test connection info formatting
        let info1 = transport1.connection_info();
        assert!(info1.contains("http://localhost:3000/events"));

        let info3 = transport3.connection_info();
        assert!(info3.contains("sse: None"));
    }

    #[tokio::test]
    async fn test_concurrent_request_id_generation() {
        let transport = std::sync::Arc::new(
            HttpClientTransport::new("http://localhost:3000", None)
                .await
                .unwrap(),
        );

        let mut handles = vec![];

        // Spawn multiple tasks generating request IDs concurrently
        for _ in 0..3 {
            let transport_clone = transport.clone();
            let handle = tokio::spawn(async move {
                let mut ids = vec![];
                for _ in 0..3 {
                    ids.push(transport_clone.next_request_id().await);
                }
                ids
            });
            handles.push(handle);
        }

        let mut all_ids = vec![];
        for handle in handles {
            let ids = handle.await.unwrap();
            all_ids.extend(ids);
        }

        // All IDs should be unique
        all_ids.sort();
        let mut unique_ids = all_ids.clone();
        unique_ids.dedup();

        assert_eq!(all_ids.len(), unique_ids.len());
        assert_eq!(all_ids.len(), 9); // 3 tasks * 3 IDs each
    }

    #[tokio::test]
    async fn test_server_bind_addresses() {
        let test_cases = vec!["127.0.0.1:0", "0.0.0.0:8080", "localhost:9000"];

        for addr in test_cases {
            let server = HttpServerTransport::new(addr);
            assert_eq!(server.get_bind_addr(), addr);
            assert!(!server.is_running());

            let info = server.server_info();
            assert!(info.contains("HTTP server transport"));
            assert!(info.contains(addr));
        }
    }

    // Mock server tests for actual Transport trait implementation coverage
    #[tokio::test]
    async fn test_transport_send_request_with_mock() {
        let mock_server = MockServer::start().await;

        // Set up mock response
        let expected_response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Value::from(42),
            result: Some(serde_json::json!({
                "capabilities": {
                    "tools": true,
                    "resources": true
                }
            })),
        };

        Mock::given(method("POST"))
            .and(path("/mcp"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&expected_response))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(42),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            })),
        };

        let result = transport.send_request(request).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.id, Value::from(42));
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_transport_send_notification_with_mock() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/mcp"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "initialized".to_string(),
            params: Some(serde_json::json!({})),
        };

        let result = transport.send_notification(notification).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transport_request_auto_id() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"status": "ok"}
            })))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        // Request with null ID should get auto-generated ID
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::Null,
            method: "ping".to_string(),
            params: None,
        };

        let result = transport.send_request(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.id, Value::from(1));
    }

    #[tokio::test]
    async fn test_transport_error_scenarios() {
        let mock_server = MockServer::start().await;

        // Test HTTP 500 error
        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(1),
            method: "test".to_string(),
            params: None,
        };

        let result = transport.send_request(request).await;
        assert!(result.is_err());

        if let Err(McpError::Http(msg)) = result {
            assert!(msg.contains("HTTP error: 500"));
        } else {
            panic!("Expected HTTP error");
        }
    }

    #[tokio::test]
    async fn test_transport_notification_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test_notification".to_string(),
            params: None,
        };

        let result = transport.send_notification(notification).await;
        assert!(result.is_err());

        if let Err(McpError::Http(msg)) = result {
            assert!(msg.contains("HTTP notification error: 400"));
        } else {
            panic!("Expected HTTP notification error");
        }
    }

    #[tokio::test]
    async fn test_transport_connection_failure() {
        // Use invalid port to trigger connection error
        let mut transport = HttpClientTransport::new("http://127.0.0.1:1", None)
            .await
            .unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(1),
            method: "test".to_string(),
            params: None,
        };

        let result = transport.send_request(request).await;
        assert!(result.is_err());
        // Connection errors can manifest as different error types
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transport_invalid_json_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(1),
            method: "test".to_string(),
            params: None,
        };

        let result = transport.send_request(request).await;
        assert!(result.is_err());

        if let Err(McpError::Connection(msg)) = result {
            assert!(msg.contains("Request serialization failed"));
        } else {
            // Accept other error types for JSON parsing failures
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_transport_response_id_mismatch() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": 999, // Different from request ID
                "result": {"success": true}
            })))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(1),
            method: "test".to_string(),
            params: None,
        };

        let result = transport.send_request(request).await;
        assert!(result.is_err());

        if let Err(McpError::Http(msg)) = result {
            assert!(msg.contains("Response ID") && msg.contains("does not match request ID"));
        } else {
            panic!("Expected HTTP error for ID mismatch");
        }
    }

    #[tokio::test]
    async fn http_server_installs_and_runs_the_mcp_request_handler() {
        use crate::server::McpServer;

        let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = reserved.local_addr().unwrap();
        drop(reserved);

        let mut server = McpServer::create("http-e2e", "1.0.0");
        server
            .start(HttpServerTransport::new(address.to_string()))
            .await
            .unwrap();

        let mut client = HttpClientTransport::new(format!("http://{address}"), None)
            .await
            .unwrap();
        let response = client
            .send_request(
                JsonRpcRequest::new(Value::from(42), methods::PING.to_string(), None::<Value>)
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.id, Value::from(42));
        assert!(response.result.is_some());
        server.stop().await.unwrap();
    }

    #[tokio::test]
    async fn http_preserves_policy_errors_and_request_ids() {
        use crate::security::{Permission, RbacAuthorizer, RequestPolicy};
        use crate::server::McpServer;

        let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = reserved.local_addr().unwrap();
        drop(reserved);

        let policy = RequestPolicy::new(RbacAuthorizer::new([Permission::new(
            "operator",
            methods::PING,
        )]));
        let mut server = McpServer::create("http-policy", "1.0.0").with_request_policy(policy);
        server
            .start(HttpServerTransport::new(address.to_string()))
            .await
            .unwrap();

        let mut client = HttpClientTransport::new(format!("http://{address}"), None)
            .await
            .unwrap();
        let error = client
            .send_request(
                JsonRpcRequest::new(Value::from(43), methods::PING.to_string(), None::<Value>)
                    .unwrap(),
            )
            .await
            .unwrap_err();
        assert!(matches!(error, McpError::Forbidden(_)));
        server.stop().await.unwrap();
    }

    #[cfg(feature = "tls")]
    #[tokio::test]
    async fn mtls_server_rejects_empty_identity_before_starting() {
        let mut transport = HttpServerTransport::new("127.0.0.1:0")
            .with_mtls(MtlsServerConfig::new(Vec::new(), Vec::new(), Vec::new()));
        assert!(transport.start().await.is_err());
        assert!(!transport.is_running());
    }
}
