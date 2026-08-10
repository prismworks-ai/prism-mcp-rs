//! Protocol revision negotiation and MCP 2026-07-28 wire envelopes.
//!
//! MCP 2025-11-25 uses a connection-scoped `initialize` handshake. MCP
//! 2026-07-28 is stateless: every request declares its version, client
//! identity, and capabilities in `_meta`. Keeping this behavior in one module
//! prevents revision checks from leaking into application handlers.

use std::collections::HashMap;

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::core::error::{McpError, McpResult};

use super::{ClientCapabilities, Implementation, JsonRpcRequest, ServerCapabilities};

/// Final stateless MCP protocol revision supported by Prism v3.
pub const MODERN_PROTOCOL_VERSION: &str = "2026-07-28";
/// Stateful MCP protocol revision retained for production interoperability.
pub const LEGACY_PROTOCOL_VERSION: &str = "2025-11-25";
/// Protocol revisions supported by a dual-stack Prism server, in preference order.
pub const SUPPORTED_PROTOCOL_VERSIONS: [&str; 2] =
    [MODERN_PROTOCOL_VERSION, LEGACY_PROTOCOL_VERSION];

/// Reserved request `_meta` key carrying the protocol revision.
pub const PROTOCOL_VERSION_META_KEY: &str = "io.modelcontextprotocol/protocolVersion";
/// Reserved request `_meta` key carrying client identity.
pub const CLIENT_INFO_META_KEY: &str = "io.modelcontextprotocol/clientInfo";
/// Reserved request `_meta` key carrying per-request client capabilities.
pub const CLIENT_CAPABILITIES_META_KEY: &str = "io.modelcontextprotocol/clientCapabilities";
/// Reserved result `_meta` key carrying server identity.
pub const SERVER_INFO_META_KEY: &str = "io.modelcontextprotocol/serverInfo";
/// Standard HTTP protocol revision header.
pub const MCP_PROTOCOL_VERSION_HEADER: &str = "MCP-Protocol-Version";
/// Standard HTTP method routing header.
pub const MCP_METHOD_HEADER: &str = "Mcp-Method";
/// Standard HTTP resource/name routing header.
pub const MCP_NAME_HEADER: &str = "Mcp-Name";

/// MCP-defined header/body mismatch error code.
pub const HEADER_MISMATCH: i32 = -32020;
/// MCP-defined missing client capability error code.
pub const MISSING_REQUIRED_CLIENT_CAPABILITY: i32 = -32021;
/// MCP-defined unsupported revision error code.
pub const UNSUPPORTED_PROTOCOL_VERSION: i32 = -32022;

/// Runtime protocol selection policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolMode {
    /// Prefer MCP 2026-07-28 and fall back only when discovery is explicitly
    /// rejected as an unknown method.
    #[default]
    Auto,
    /// Require MCP 2026-07-28. Never downgrade.
    ModernOnly,
    /// Use the MCP 2025-11-25 initialize lifecycle byte-for-byte.
    LegacyOnly,
}

/// The lifecycle family selected for an active client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolEra {
    Modern,
    Legacy,
}

/// Stable record of the protocol selected for a connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegotiatedProtocol {
    pub version: String,
    pub era: ProtocolEra,
}

/// Revision-neutral result returned when a client establishes protocol behavior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectResult {
    pub protocol: NegotiatedProtocol,
    pub capabilities: ServerCapabilities,
    /// Server identity is optional in MCP 2026-07-28 and required by the
    /// legacy initialize result, so v3 exposes that distinction directly.
    pub server_info: Option<Implementation>,
    pub instructions: Option<String>,
}

impl NegotiatedProtocol {
    pub fn modern() -> Self {
        Self {
            version: MODERN_PROTOCOL_VERSION.to_string(),
            era: ProtocolEra::Modern,
        }
    }

    pub fn legacy() -> Self {
        Self {
            version: LEGACY_PROTOCOL_VERSION.to_string(),
            era: ProtocolEra::Legacy,
        }
    }
}

/// Required metadata envelope for a 2026-07-28 request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestMetaObject {
    #[serde(rename = "io.modelcontextprotocol/protocolVersion")]
    pub protocol_version: String,
    #[serde(
        rename = "io.modelcontextprotocol/clientInfo",
        skip_serializing_if = "Option::is_none"
    )]
    pub client_info: Option<Implementation>,
    #[serde(rename = "io.modelcontextprotocol/clientCapabilities")]
    pub client_capabilities: ClientCapabilities,
    #[serde(rename = "progressToken", skip_serializing_if = "Option::is_none")]
    pub progress_token: Option<Value>,
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

impl RequestMetaObject {
    pub fn modern(client_info: Implementation, capabilities: ClientCapabilities) -> Self {
        Self {
            protocol_version: MODERN_PROTOCOL_VERSION.to_string(),
            client_info: Some(client_info),
            client_capabilities: capabilities,
            progress_token: None,
            additional: HashMap::new(),
        }
    }
}

/// Parameters for `server/discover`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoverParams {
    #[serde(rename = "_meta")]
    pub meta: RequestMetaObject,
}

/// Cache sharing policy introduced by MCP 2026-07-28.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheScope {
    Public,
    Private,
}

/// The discriminant present on all modern successful results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultType {
    Complete,
    InputRequired,
    Task,
}

fn default_complete() -> ResultType {
    ResultType::Complete
}

/// Result of the stateless `server/discover` request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoverResult {
    #[serde(rename = "resultType", default = "default_complete")]
    pub result_type: ResultType,
    #[serde(rename = "supportedVersions")]
    pub supported_versions: Vec<String>,
    pub capabilities: ServerCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(rename = "ttlMs")]
    pub ttl_ms: u64,
    #[serde(rename = "cacheScope")]
    pub cache_scope: CacheScope,
    #[serde(rename = "_meta", default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, Value>,
}

impl DiscoverResult {
    pub fn server_info(&self) -> Option<Implementation> {
        self.meta
            .get(SERVER_INFO_META_KEY)
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok())
    }
}

/// A modern server response that asks the client to provide in-band input and retry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputRequiredResult {
    #[serde(rename = "resultType")]
    pub result_type: ResultType,
    #[serde(
        rename = "inputRequests",
        default,
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub input_requests: HashMap<String, Value>,
    #[serde(rename = "requestState", skip_serializing_if = "Option::is_none")]
    pub request_state: Option<String>,
    #[serde(rename = "_meta", default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, Value>,
}

impl InputRequiredResult {
    /// Build a validated input-required result. At least one input request or
    /// request-state token is required by the protocol.
    pub fn new(
        input_requests: HashMap<String, Value>,
        request_state: Option<String>,
    ) -> McpResult<Self> {
        if input_requests.is_empty() && request_state.is_none() {
            return Err(McpError::Validation(
                "input_required needs inputRequests or requestState".to_string(),
            ));
        }
        Ok(Self {
            result_type: ResultType::InputRequired,
            input_requests,
            request_state,
            meta: HashMap::new(),
        })
    }
}

/// A version-neutral operation result for MRTR-aware applications.
#[derive(Debug, Clone, PartialEq)]
pub enum OperationResult<T> {
    Complete(T),
    InputRequired(InputRequiredResult),
}

impl<T> OperationResult<T>
where
    T: serde::de::DeserializeOwned,
{
    pub fn from_value(value: Value) -> McpResult<Self> {
        match value.get("resultType").and_then(Value::as_str) {
            Some("input_required") => Ok(Self::InputRequired(serde_json::from_value(value)?)),
            Some("complete") | None => Ok(Self::Complete(serde_json::from_value(value)?)),
            Some(other) => Err(McpError::Protocol(format!(
                "unsupported MCP resultType: {other}"
            ))),
        }
    }
}

/// Parsed modern request information. It is request-scoped by design.
#[derive(Debug, Clone, PartialEq)]
pub struct ModernRequestContext {
    pub version: String,
    pub client_info: Option<Implementation>,
    pub client_capabilities: ClientCapabilities,
}

/// Add or replace the required modern `_meta` fields without disturbing
/// application-defined metadata.
pub fn decorate_modern_request(
    request: &mut JsonRpcRequest,
    client_info: &Implementation,
    capabilities: &ClientCapabilities,
) -> McpResult<()> {
    let params = request
        .params
        .get_or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| McpError::Validation("MCP request params must be an object".to_string()))?;
    let meta = params
        .entry("_meta")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| McpError::Validation("MCP request _meta must be an object".to_string()))?;

    meta.insert(
        PROTOCOL_VERSION_META_KEY.to_string(),
        Value::String(MODERN_PROTOCOL_VERSION.to_string()),
    );
    meta.insert(
        CLIENT_INFO_META_KEY.to_string(),
        serde_json::to_value(client_info)?,
    );
    meta.insert(
        CLIENT_CAPABILITIES_META_KEY.to_string(),
        serde_json::to_value(capabilities)?,
    );
    Ok(())
}

/// Parse and validate the required modern request envelope.
pub fn modern_request_context(request: &JsonRpcRequest) -> McpResult<Option<ModernRequestContext>> {
    let Some(meta) = request
        .params
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|params| params.get("_meta"))
        .and_then(Value::as_object)
    else {
        return Ok(None);
    };
    let Some(version) = meta.get(PROTOCOL_VERSION_META_KEY).and_then(Value::as_str) else {
        return Ok(None);
    };

    if version != MODERN_PROTOCOL_VERSION {
        return Err(McpError::UnsupportedProtocolVersion {
            requested: version.to_string(),
            supported: SUPPORTED_PROTOCOL_VERSIONS
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        });
    }

    let capabilities = meta
        .get(CLIENT_CAPABILITIES_META_KEY)
        .cloned()
        .ok_or_else(|| {
            McpError::Validation(format!(
                "missing required _meta key {CLIENT_CAPABILITIES_META_KEY}"
            ))
        })
        .and_then(|value| {
            serde_json::from_value(value).map_err(|error| McpError::Validation(error.to_string()))
        })?;
    let client_info = meta
        .get(CLIENT_INFO_META_KEY)
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|error| McpError::Validation(error.to_string()))?;

    Ok(Some(ModernRequestContext {
        version: version.to_string(),
        client_info,
        client_capabilities: capabilities,
    }))
}

/// Methods removed from the stateless 2026 core.
pub fn is_legacy_only_method(method: &str) -> bool {
    matches!(
        method,
        "initialize"
            | "notifications/initialized"
            | "ping"
            | "logging/setLevel"
            | "resources/subscribe"
            | "resources/unsubscribe"
            | "notifications/roots/list_changed"
    )
}

/// Convert a handler-produced object into a standards-compliant modern result.
pub fn decorate_modern_result(
    method: &str,
    mut result: Value,
    server_info: &Implementation,
) -> McpResult<Value> {
    let object = result.as_object_mut().ok_or_else(|| {
        McpError::Protocol("MCP 2026 successful results must be JSON objects".to_string())
    })?;
    object
        .entry("resultType")
        .or_insert_with(|| Value::String("complete".to_string()));

    if is_cacheable_method(method) {
        object.entry("ttlMs").or_insert(Value::from(0_u64));
        object
            .entry("cacheScope")
            .or_insert_with(|| Value::String("private".to_string()));
    }

    let meta = object
        .entry("_meta")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| McpError::Protocol("result _meta must be an object".to_string()))?;
    meta.entry(SERVER_INFO_META_KEY)
        .or_insert(serde_json::to_value(server_info)?);
    Ok(result)
}

/// List/discovery operations that require explicit cache policy in 2026-07-28.
pub fn is_cacheable_method(method: &str) -> bool {
    matches!(
        method,
        "server/discover"
            | "tools/list"
            | "prompts/list"
            | "resources/list"
            | "resources/templates/list"
            | "resources/read"
    )
}

/// Extract the modern protocol version from a request body for HTTP routing.
pub fn request_protocol_version(request: &JsonRpcRequest) -> Option<&str> {
    request
        .params
        .as_ref()?
        .as_object()?
        .get("_meta")?
        .as_object()?
        .get(PROTOCOL_VERSION_META_KEY)?
        .as_str()
}

/// Derive the `Mcp-Name` header value from the corresponding request field.
pub fn request_routing_name(request: &JsonRpcRequest) -> Option<&str> {
    let params = request.params.as_ref()?.as_object()?;
    let field = match request.method.as_str() {
        "tools/call" | "prompts/get" => "name",
        "resources/read" => "uri",
        "tasks/get" | "tasks/update" | "tasks/cancel" => "taskId",
        _ => return None,
    };
    params.get(field).and_then(Value::as_str)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolHeaderMapping {
    pub header_name: String,
    pub parameter_path: Vec<String>,
}

fn valid_header_token(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

/// Extract and validate `x-mcp-header` annotations from a tool input schema.
pub fn tool_header_mappings(schema: &Value) -> McpResult<Vec<ToolHeaderMapping>> {
    fn walk(
        schema: &Value,
        path: &mut Vec<String>,
        properties_reachable: bool,
        is_property: bool,
        mappings: &mut Vec<ToolHeaderMapping>,
        seen: &mut std::collections::HashSet<String>,
    ) -> McpResult<()> {
        let Some(object) = schema.as_object() else {
            return Ok(());
        };
        if let Some(header) = object.get("x-mcp-header") {
            if !properties_reachable || !is_property {
                return Err(McpError::Validation(
                    "x-mcp-header is not reachable through properties only".to_string(),
                ));
            }
            let suffix = header
                .as_str()
                .ok_or_else(|| McpError::Validation("x-mcp-header must be a string".to_string()))?;
            if !valid_header_token(suffix) {
                return Err(McpError::Validation(format!(
                    "invalid x-mcp-header token: {suffix}"
                )));
            }
            let parameter_type = object.get("type").and_then(Value::as_str);
            if !matches!(parameter_type, Some("string" | "integer" | "boolean")) {
                return Err(McpError::Validation(format!(
                    "x-mcp-header {suffix} must annotate string, integer, or boolean"
                )));
            }
            let normalized = suffix.to_ascii_lowercase();
            if !seen.insert(normalized) {
                return Err(McpError::Validation(format!(
                    "duplicate x-mcp-header name: {suffix}"
                )));
            }
            mappings.push(ToolHeaderMapping {
                header_name: format!("Mcp-Param-{suffix}"),
                parameter_path: path.clone(),
            });
        }
        if let Some(properties) = object.get("properties").and_then(Value::as_object) {
            for (name, property) in properties {
                path.push(name.clone());
                walk(property, path, properties_reachable, true, mappings, seen)?;
                path.pop();
            }
        }

        // Crossing any other subschema keyword makes an annotation
        // statically unreachable for x-mcp-header extraction.
        for keyword in [
            "items",
            "contains",
            "not",
            "if",
            "then",
            "else",
            "additionalProperties",
            "unevaluatedProperties",
            "propertyNames",
        ] {
            if let Some(subschema) = object.get(keyword).filter(|value| value.is_object()) {
                walk(subschema, path, false, false, mappings, seen)?;
            }
        }
        for keyword in ["allOf", "anyOf", "oneOf", "prefixItems"] {
            if let Some(subschemas) = object.get(keyword).and_then(Value::as_array) {
                for subschema in subschemas {
                    walk(subschema, path, false, false, mappings, seen)?;
                }
            }
        }
        for keyword in [
            "$defs",
            "definitions",
            "patternProperties",
            "dependentSchemas",
        ] {
            if let Some(subschemas) = object.get(keyword).and_then(Value::as_object) {
                for subschema in subschemas.values() {
                    walk(subschema, path, false, false, mappings, seen)?;
                }
            }
        }
        Ok(())
    }

    let mut mappings = Vec::new();
    let mut seen = std::collections::HashSet::new();
    walk(
        schema,
        &mut Vec::new(),
        true,
        false,
        &mut mappings,
        &mut seen,
    )?;
    Ok(mappings)
}

fn parameter_at_path<'a>(arguments: &'a Value, path: &[String]) -> Option<&'a Value> {
    path.iter()
        .try_fold(arguments, |value, segment| value.get(segment))
}

/// Encode a body string as an MCP HTTP routing-header value.
pub fn encode_http_header_value(raw: &str) -> String {
    let sentinel = raw.starts_with("=?base64?") && raw.ends_with("?=");
    let unsafe_value = raw.starts_with([' ', '\t'])
        || raw.ends_with([' ', '\t'])
        || !raw.bytes().all(|byte| (0x20..=0x7e).contains(&byte))
        || sentinel;
    if unsafe_value {
        format!(
            "=?base64?{}?=",
            base64::engine::general_purpose::STANDARD.encode(raw.as_bytes())
        )
    } else {
        raw.to_string()
    }
}

/// Decode and validate an MCP HTTP routing-header value.
pub fn decode_http_header_value(value: &str) -> McpResult<String> {
    if let Some(encoded) = value
        .strip_prefix("=?base64?")
        .and_then(|value| value.strip_suffix("?="))
    {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|_| McpError::HeaderMismatch("invalid base64 header value".to_string()))?;
        return String::from_utf8(bytes)
            .map_err(|_| McpError::HeaderMismatch("header value is not UTF-8".to_string()));
    }
    if value.starts_with([' ', '\t'])
        || value.ends_with([' ', '\t'])
        || !value
            .bytes()
            .all(|byte| byte == b'\t' || (0x20..=0x7e).contains(&byte))
    {
        return Err(McpError::HeaderMismatch(
            "unsafe plain MCP header value".to_string(),
        ));
    }
    Ok(value.to_string())
}

fn header_parameter_string(value: &Value) -> McpResult<Option<String>> {
    let raw = match value {
        Value::Null => return Ok(None),
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) if value.is_i64() => {
            let integer = value.as_i64().expect("checked integer");
            if !(-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&integer) {
                return Err(McpError::Validation(
                    "x-mcp-header integer exceeds JavaScript safe range".to_string(),
                ));
            }
            integer.to_string()
        }
        Value::Number(value) if value.is_u64() => {
            let integer = value.as_u64().expect("checked integer");
            if integer > 9_007_199_254_740_991 {
                return Err(McpError::Validation(
                    "x-mcp-header integer exceeds JavaScript safe range".to_string(),
                ));
            }
            integer.to_string()
        }
        _ => {
            return Err(McpError::Validation(
                "x-mcp-header value must be string, integer, boolean, or null".to_string(),
            ))
        }
    };
    Ok(Some(encode_http_header_value(&raw)))
}

/// Build the custom HTTP headers required for one tool call.
pub fn tool_call_headers(schema: &Value, arguments: &Value) -> McpResult<HashMap<String, String>> {
    let mut headers = HashMap::new();
    for mapping in tool_header_mappings(schema)? {
        if let Some(value) = parameter_at_path(arguments, &mapping.parameter_path) {
            if let Some(value) = header_parameter_string(value)? {
                headers.insert(mapping.header_name, value);
            }
        }
    }
    Ok(headers)
}

/// Validate received `Mcp-Param-*` values against a tool call body.
pub fn validate_tool_call_headers(
    schema: &Value,
    arguments: &Value,
    received: &HashMap<String, String>,
) -> McpResult<()> {
    let expected = tool_call_headers(schema, arguments)?;
    let received = received
        .iter()
        .map(|(name, value)| (name.to_ascii_lowercase(), value.as_str()))
        .collect::<HashMap<_, _>>();
    for mapping in tool_header_mappings(schema)? {
        let name = mapping.header_name.to_ascii_lowercase();
        let expected_value = expected.get(&mapping.header_name);
        let received_value = received.get(&name).copied();
        let matches = match (expected_value, received_value) {
            (None, None) => true,
            (Some(expected), Some(received)) => {
                decode_http_header_value(expected)? == decode_http_header_value(received)?
            }
            _ => false,
        };
        if !matches {
            return Err(McpError::HeaderMismatch(format!(
                "{} does not match the tool arguments",
                mapping.header_name
            )));
        }
    }
    Ok(())
}

/// Validate standard HTTP routing headers against a modern request body.
pub fn validate_http_headers(
    request: &JsonRpcRequest,
    protocol_version: Option<&str>,
    method: Option<&str>,
    name: Option<&str>,
) -> McpResult<()> {
    let Some(body_version) = request_protocol_version(request) else {
        // Legacy requests retain their existing transport behavior.
        return Ok(());
    };
    if protocol_version != Some(body_version) {
        return Err(McpError::HeaderMismatch(format!(
            "{MCP_PROTOCOL_VERSION_HEADER} does not match request _meta"
        )));
    }
    if method != Some(request.method.as_str()) {
        return Err(McpError::HeaderMismatch(format!(
            "{MCP_METHOD_HEADER} does not match request method"
        )));
    }
    let body_name = request_routing_name(request);
    let name_matches = match (name, body_name) {
        (None, None) => true,
        (Some(header), Some(body)) => decode_http_header_value(header)? == body,
        _ => false,
    };
    if !name_matches {
        return Err(McpError::HeaderMismatch(format!(
            "{MCP_NAME_HEADER} does not match the request target"
        )));
    }
    Ok(())
}

/// Map SDK errors to their JSON-RPC code and structured MCP error data.
pub fn json_rpc_error_details(error: &McpError) -> (i32, Option<Value>) {
    match error {
        McpError::MethodNotFound(_) => (-32601, None),
        McpError::InvalidParams(_) | McpError::Validation(_) => (-32602, None),
        McpError::HeaderMismatch(_) => (HEADER_MISMATCH, None),
        McpError::MissingRequiredClientCapability(capability) => (
            MISSING_REQUIRED_CLIENT_CAPABILITY,
            Some(serde_json::json!({"requiredCapabilities": capability})),
        ),
        McpError::UnsupportedProtocolVersion {
            requested,
            supported,
        } => (
            UNSUPPORTED_PROTOCOL_VERSION,
            Some(serde_json::json!({
                "requested": requested,
                "supported": supported,
            })),
        ),
        McpError::ToolNotFound(_) | McpError::ResourceNotFound(_) | McpError::PromptNotFound(_) => {
            (-32602, None)
        }
        _ => (-32603, None),
    }
}

/// Test whether an error is the one safe automatic-downgrade signal.
pub fn is_method_not_found(error: &McpError) -> bool {
    match error {
        McpError::MethodNotFound(_) => true,
        McpError::Protocol(message) => message.contains("-32601"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn info() -> Implementation {
        Implementation::new("prism-test", "3.0.0")
    }

    #[test]
    fn modern_request_is_self_describing() {
        let mut request =
            JsonRpcRequest::new(1.into(), "tools/list".into(), Some(json!({}))).expect("request");
        decorate_modern_request(&mut request, &info(), &ClientCapabilities::default())
            .expect("decorate");
        let context = modern_request_context(&request)
            .expect("valid")
            .expect("modern");
        assert_eq!(context.version, MODERN_PROTOCOL_VERSION);
        assert_eq!(context.client_info.expect("identity").name, "prism-test");
    }

    #[test]
    fn modern_result_gets_identity_and_cache_policy() {
        let result = decorate_modern_result("tools/list", json!({"tools": []}), &info())
            .expect("decorate result");
        assert_eq!(result["resultType"], "complete");
        assert_eq!(result["ttlMs"], 0);
        assert_eq!(result["cacheScope"], "private");
        assert_eq!(result["_meta"][SERVER_INFO_META_KEY]["name"], "prism-test");
    }

    #[test]
    fn input_required_requires_resume_material() {
        assert!(InputRequiredResult::new(HashMap::new(), None).is_err());
        assert!(InputRequiredResult::new(HashMap::new(), Some("opaque".into())).is_ok());
    }

    #[test]
    fn custom_tool_headers_support_nested_primitive_parameters() {
        let schema = json!({
            "type": "object",
            "properties": {
                "region": {"type": "string", "x-mcp-header": "Region"},
                "options": {
                    "type": "object",
                    "properties": {
                        "priority": {"type": "integer", "x-mcp-header": "Priority"},
                        "dryRun": {"type": "boolean", "x-mcp-header": "Dry-Run"}
                    }
                }
            }
        });
        let arguments = json!({
            "region": "eu-north-1",
            "options": {"priority": 7, "dryRun": true}
        });

        let headers = tool_call_headers(&schema, &arguments).expect("headers");
        assert_eq!(headers.get("Mcp-Param-Region"), Some(&"eu-north-1".into()));
        assert_eq!(headers.get("Mcp-Param-Priority"), Some(&"7".into()));
        assert_eq!(headers.get("Mcp-Param-Dry-Run"), Some(&"true".into()));
        validate_tool_call_headers(&schema, &arguments, &headers).expect("matching headers");
    }

    #[test]
    fn unsafe_tool_header_values_are_base64_wrapped() {
        let schema = json!({
            "type": "object",
            "properties": {
                "token": {"type": "string", "x-mcp-header": "Token"}
            }
        });
        let headers =
            tool_call_headers(&schema, &json!({"token": " secret\n"})).expect("encoded header");
        assert_eq!(
            headers.get("Mcp-Param-Token"),
            Some(&"=?base64?IHNlY3JldAo=?=".to_string())
        );
    }

    #[test]
    fn duplicate_custom_header_names_are_rejected_case_insensitively() {
        let schema = json!({
            "type": "object",
            "properties": {
                "first": {"type": "string", "x-mcp-header": "Region"},
                "second": {"type": "string", "x-mcp-header": "region"}
            }
        });
        assert!(tool_header_mappings(&schema).is_err());
    }

    #[test]
    fn custom_headers_outside_properties_only_paths_are_rejected() {
        let schema = json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "region": {"type": "string", "x-mcp-header": "Region"}
                        }
                    }
                }
            }
        });
        assert!(tool_header_mappings(&schema).is_err());
    }

    #[test]
    fn routing_names_use_and_validate_base64_sentinel_encoding() {
        let mut request = JsonRpcRequest::new(
            1.into(),
            "resources/read".into(),
            Some(json!({"uri": "file:///résumé.txt"})),
        )
        .expect("request");
        decorate_modern_request(&mut request, &info(), &ClientCapabilities::default())
            .expect("decorate");
        let encoded = encode_http_header_value("file:///résumé.txt");
        validate_http_headers(
            &request,
            Some(MODERN_PROTOCOL_VERSION),
            Some("resources/read"),
            Some(&encoded),
        )
        .expect("encoded name");
    }

    #[test]
    fn unknown_custom_headers_are_ignored() {
        let schema = json!({
            "type": "object",
            "properties": {
                "region": {"type": "string", "x-mcp-header": "Region"}
            }
        });
        let received = HashMap::from([
            ("mcp-param-region".to_string(), "eu-north-1".to_string()),
            ("mcp-param-proxy-only".to_string(), "route-a".to_string()),
        ]);
        validate_tool_call_headers(&schema, &json!({"region": "eu-north-1"}), &received)
            .expect("unknown header ignored");
    }
}
