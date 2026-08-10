use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use prism_mcp_rs::client::{ClientRequestHandler, McpClient};
use prism_mcp_rs::core::error::{McpError, McpResult};
use prism_mcp_rs::core::{MultiRoundToolCall, MultiRoundToolHandler};
use prism_mcp_rs::protocol::messages::{
    CreateMessageParams, ElicitParams, ElicitResult, InitializeParams, ListRootsParams,
    ListRootsResult, PingParams, PingResult,
};
use prism_mcp_rs::protocol::types::{
    ClientCapabilities, ContentBlock, CreateMessageResult, ElicitationCapability, Implementation,
    JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, Tool, ToolInputSchema, ToolResult,
};
use prism_mcp_rs::protocol::version::*;
use prism_mcp_rs::protocol::{methods, ElicitationAction, TaskStatus, TASKS_EXTENSION_ID};
use prism_mcp_rs::server::{McpServer, TaskContext};
use prism_mcp_rs::transport::Transport;
use serde_json::{json, Value};

fn client_info() -> Implementation {
    Implementation::new("v3-test-client", "3.0.0")
}

fn modern_request(id: u64, method: &str, params: Value) -> JsonRpcRequest {
    let mut request = JsonRpcRequest::new(id.into(), method.to_string(), Some(params)).unwrap();
    decorate_modern_request(&mut request, &client_info(), &ClientCapabilities::default()).unwrap();
    request
}

fn modern_request_with_capabilities(
    id: u64,
    method: &str,
    params: Value,
    capabilities: &ClientCapabilities,
) -> JsonRpcRequest {
    let mut request = JsonRpcRequest::new(id.into(), method.to_string(), Some(params)).unwrap();
    decorate_modern_request(&mut request, &client_info(), capabilities).unwrap();
    request
}

fn task_tool(name: &str) -> Tool {
    Tool {
        name: name.to_string(),
        description: Some("Exercise the Tasks extension".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: HashMap::new(),
        },
        output_schema: None,
        annotations: None,
        title: None,
        icons: None,
        meta: None,
    }
}

struct SubscriptionResource {
    subscribed: Arc<AtomicUsize>,
    unsubscribed: Arc<AtomicUsize>,
}

#[async_trait]
impl prism_mcp_rs::core::ResourceHandler for SubscriptionResource {
    async fn read(
        &self,
        _uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<prism_mcp_rs::protocol::ResourceContents>> {
        Ok(Vec::new())
    }

    async fn list(&self) -> McpResult<Vec<prism_mcp_rs::protocol::Resource>> {
        Ok(Vec::new())
    }

    async fn subscribe(&self, _uri: &str) -> McpResult<()> {
        self.subscribed.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn unsubscribe(&self, _uri: &str) -> McpResult<()> {
        self.unsubscribed.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn legacy_resource_subscriptions_reach_the_registered_handler() {
    let server = McpServer::create("legacy-resource-server", "3.0.0");
    let subscribed = Arc::new(AtomicUsize::new(0));
    let unsubscribed = Arc::new(AtomicUsize::new(0));
    server
        .add_resource(
            "watched".to_string(),
            "file:///watched".to_string(),
            SubscriptionResource {
                subscribed: subscribed.clone(),
                unsubscribed: unsubscribed.clone(),
            },
        )
        .await
        .unwrap();

    for (id, method) in [
        (1, methods::RESOURCES_SUBSCRIBE),
        (2, methods::RESOURCES_UNSUBSCRIBE),
    ] {
        let request = JsonRpcRequest::new(
            id.into(),
            method.to_string(),
            Some(json!({"uri": "file:///watched"})),
        )
        .unwrap();
        server.handle_request(request).await.unwrap();
    }

    assert_eq!(subscribed.load(Ordering::SeqCst), 1);
    assert_eq!(unsubscribed.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn dual_server_discovers_both_eras_and_stamps_identity() {
    let server = McpServer::create("dual-server", "3.0.0");
    let request = modern_request(1, methods::SERVER_DISCOVER, json!({}));
    let response = server.handle_request(request).await.unwrap();
    let result = response.result.unwrap();

    assert_eq!(result["resultType"], "complete");
    assert_eq!(result["supportedVersions"][0], MODERN_PROTOCOL_VERSION);
    assert_eq!(result["supportedVersions"][1], LEGACY_PROTOCOL_VERSION);
    assert_eq!(result["ttlMs"], 0);
    assert_eq!(result["cacheScope"], "private");
    assert_eq!(result["_meta"][SERVER_INFO_META_KEY]["name"], "dual-server");
}

#[tokio::test]
async fn dual_server_keeps_legacy_initialize_wire_compatible() {
    let server = McpServer::create("legacy-compatible", "3.0.0");
    let params = InitializeParams::new(
        LEGACY_PROTOCOL_VERSION.to_string(),
        ClientCapabilities::default(),
        client_info(),
    );
    let request = JsonRpcRequest::new(7.into(), methods::INITIALIZE.into(), Some(params)).unwrap();
    let result = server
        .handle_request(request)
        .await
        .unwrap()
        .result
        .unwrap();

    assert_eq!(result["protocolVersion"], LEGACY_PROTOCOL_VERSION);
    assert!(result.get("resultType").is_none());
}

#[tokio::test]
async fn modern_server_rejects_removed_stateful_methods() {
    let server = McpServer::create("modern-server", "3.0.0");
    let request = modern_request(1, methods::PING, json!({}));
    assert!(matches!(
        server.handle_request(request).await,
        Err(McpError::MethodNotFound(_))
    ));
}

#[tokio::test]
async fn tasks_extension_requires_capability_and_completes_input_round_trip() {
    let server = McpServer::create("tasks-server", "3.0.0");
    server
        .add_task_tool(
            task_tool("durable_echo"),
            |_arguments: HashMap<String, Value>, context: TaskContext| async move {
                let mut requests = HashMap::new();
                requests.insert(
                    "approval".to_string(),
                    json!({
                        "method": "elicitation/create",
                        "params": {
                            "message": "Approve?",
                            "requestedSchema": {"type": "object"}
                        }
                    }),
                );
                requests.insert(
                    "reason".to_string(),
                    json!({
                        "method": "elicitation/create",
                        "params": {
                            "message": "Why?",
                            "requestedSchema": {"type": "object"}
                        }
                    }),
                );
                let responses = context
                    .require_input(requests, Some("Waiting for approval".to_string()))
                    .await?;
                Ok(ToolResult::text(format!(
                    "approved={}",
                    responses["approval"]["content"]["approved"]
                )))
            },
        )
        .await
        .unwrap();

    let missing_capability = modern_request(
        20,
        methods::TOOLS_CALL,
        json!({"name": "durable_echo", "arguments": {}}),
    );
    assert!(matches!(
        server.handle_request(missing_capability).await,
        Err(McpError::MissingRequiredClientCapability(_))
    ));

    let capabilities = ClientCapabilities {
        extensions: Some(HashMap::from([(TASKS_EXTENSION_ID.to_string(), json!({}))])),
        ..Default::default()
    };
    let created = server
        .handle_request(modern_request_with_capabilities(
            21,
            methods::TOOLS_CALL,
            json!({"name": "durable_echo", "arguments": {}}),
            &capabilities,
        ))
        .await
        .unwrap()
        .result
        .unwrap();
    assert_eq!(created["resultType"], "task");
    let task_id = created["taskId"].as_str().unwrap().to_string();

    let mut current = Value::Null;
    for request_id in 22..42 {
        current = server
            .handle_request(modern_request_with_capabilities(
                request_id,
                methods::TASKS_GET,
                json!({"taskId": task_id}),
                &capabilities,
            ))
            .await
            .unwrap()
            .result
            .unwrap();
        if current["status"] == serde_json::to_value(TaskStatus::InputRequired).unwrap() {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert_eq!(current["status"], "input_required");
    assert!(current["inputRequests"].get("approval").is_some());

    server
        .handle_request(modern_request_with_capabilities(
            40,
            methods::TASKS_UPDATE,
            json!({"taskId": task_id, "inputResponses": {}}),
            &capabilities,
        ))
        .await
        .unwrap();

    server
        .handle_request(modern_request_with_capabilities(
            41,
            methods::TASKS_UPDATE,
            json!({
                "taskId": task_id,
                "inputResponses": {"not-requested": {"action": "accept"}}
            }),
            &capabilities,
        ))
        .await
        .unwrap();
    current = server
        .handle_request(modern_request_with_capabilities(
            39,
            methods::TASKS_GET,
            json!({"taskId": task_id}),
            &capabilities,
        ))
        .await
        .unwrap()
        .result
        .unwrap();
    assert!(current["inputRequests"].get("approval").is_some());
    assert!(current["inputRequests"].get("reason").is_some());

    server
        .handle_request(modern_request_with_capabilities(
            42,
            methods::TASKS_UPDATE,
            json!({
                "taskId": task_id,
                "inputResponses": {
                    "approval": {"action": "accept", "content": {"approved": true}}
                }
            }),
            &capabilities,
        ))
        .await
        .unwrap();

    current = server
        .handle_request(modern_request_with_capabilities(
            43,
            methods::TASKS_GET,
            json!({"taskId": task_id}),
            &capabilities,
        ))
        .await
        .unwrap()
        .result
        .unwrap();
    assert_eq!(current["status"], "input_required");
    assert!(current["inputRequests"].get("approval").is_none());
    assert!(current["inputRequests"].get("reason").is_some());

    server
        .handle_request(modern_request_with_capabilities(
            44,
            methods::TASKS_UPDATE,
            json!({
                "taskId": task_id,
                "inputResponses": {
                    "reason": {"action": "accept", "content": {"reason": "test"}}
                }
            }),
            &capabilities,
        ))
        .await
        .unwrap();

    for request_id in 45..65 {
        current = server
            .handle_request(modern_request_with_capabilities(
                request_id,
                methods::TASKS_GET,
                json!({"taskId": task_id}),
                &capabilities,
            ))
            .await
            .unwrap()
            .result
            .unwrap();
        if current["status"] == "completed" {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert_eq!(current["status"], "completed");
    assert_eq!(current["result"]["content"][0]["text"], "approved=true");
}

struct CompositionPreflight;

#[async_trait]
impl MultiRoundToolHandler for CompositionPreflight {
    async fn call(
        &self,
        call: MultiRoundToolCall,
    ) -> McpResult<prism_mcp_rs::protocol::OperationResult<ToolResult>> {
        if call.input_responses.is_empty() {
            return Ok(prism_mcp_rs::protocol::OperationResult::InputRequired(
                prism_mcp_rs::protocol::InputRequiredResult::new(
                    HashMap::from([(
                        "user_name".to_string(),
                        json!({
                            "method": "elicitation/create",
                            "params": {
                                "message": "What is your name?",
                                "requestedSchema": {"type": "object"}
                            }
                        }),
                    )]),
                    Some("composition-round-1".to_string()),
                )?,
            ));
        }
        Ok(prism_mcp_rs::protocol::OperationResult::Complete(
            ToolResult::text("ready"),
        ))
    }
}

#[tokio::test]
async fn multi_round_tool_can_escalate_to_a_durable_task() {
    let server = McpServer::create("composed-tasks-server", "3.0.0");
    server
        .add_composed_task_tool(
            task_tool("test_tool_with_task"),
            CompositionPreflight,
            |call: MultiRoundToolCall, _context: TaskContext| async move {
                let name = call.input_responses["user_name"]["content"]["name"]
                    .as_str()
                    .unwrap_or("unknown");
                Ok(ToolResult::text(format!("Hello, {name}")))
            },
        )
        .await
        .unwrap();

    let capabilities = ClientCapabilities {
        elicitation: Some(ElicitationCapability::default()),
        extensions: Some(HashMap::from([(TASKS_EXTENSION_ID.to_string(), json!({}))])),
        ..Default::default()
    };
    let first = server
        .handle_request(modern_request_with_capabilities(
            80,
            methods::TOOLS_CALL,
            json!({"name": "test_tool_with_task", "arguments": {}}),
            &capabilities,
        ))
        .await
        .unwrap()
        .result
        .unwrap();
    assert_eq!(first["resultType"], "input_required");
    assert!(first.get("taskId").is_none());

    let second = server
        .handle_request(modern_request_with_capabilities(
            81,
            methods::TOOLS_CALL,
            json!({
                "name": "test_tool_with_task",
                "arguments": {},
                "requestState": first["requestState"],
                "inputResponses": {
                    "user_name": {
                        "action": "accept",
                        "content": {"name": "Alice"}
                    }
                }
            }),
            &capabilities,
        ))
        .await
        .unwrap()
        .result
        .unwrap();
    assert_eq!(second["resultType"], "task");
    assert!(second.get("requestState").is_none());
    let task_id = second["taskId"].as_str().unwrap();

    let mut task = Value::Null;
    for request_id in 82..102 {
        task = server
            .handle_request(modern_request_with_capabilities(
                request_id,
                methods::TASKS_GET,
                json!({"taskId": task_id}),
                &capabilities,
            ))
            .await
            .unwrap()
            .result
            .unwrap();
        if task["status"] == "completed" {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert_eq!(task["status"], "completed");
    assert_eq!(task["result"]["content"][0]["text"], "Hello, Alice");
}

#[test]
fn modern_http_headers_must_match_the_body() {
    let request = modern_request(1, methods::TOOLS_CALL, json!({"name": "search"}));
    validate_http_headers(
        &request,
        Some(MODERN_PROTOCOL_VERSION),
        Some(methods::TOOLS_CALL),
        Some("search"),
    )
    .unwrap();

    assert!(matches!(
        validate_http_headers(
            &request,
            Some(MODERN_PROTOCOL_VERSION),
            Some(methods::TOOLS_LIST),
            Some("search")
        ),
        Err(McpError::HeaderMismatch(_))
    ));
}

#[derive(Clone)]
enum MockAction {
    ModernDiscover,
    LegacyInitialize,
    MethodNotFound,
    Unsupported,
    InputRequired,
    CompleteTool,
}

struct ScriptedTransport {
    actions: VecDeque<MockAction>,
    requests: Arc<Mutex<Vec<JsonRpcRequest>>>,
}

impl ScriptedTransport {
    fn new(
        actions: impl IntoIterator<Item = MockAction>,
    ) -> (Self, Arc<Mutex<Vec<JsonRpcRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                actions: actions.into_iter().collect(),
                requests: requests.clone(),
            },
            requests,
        )
    }
}

#[async_trait]
impl Transport for ScriptedTransport {
    async fn send_request(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        self.requests.lock().unwrap().push(request.clone());
        match self.actions.pop_front().expect("unexpected request") {
            MockAction::ModernDiscover => JsonRpcResponse::success(
                request.id,
                json!({
                    "resultType": "complete",
                    "supportedVersions": [MODERN_PROTOCOL_VERSION, LEGACY_PROTOCOL_VERSION],
                    "capabilities": {},
                    "ttlMs": 0,
                    "cacheScope": "private",
                    "_meta": {
                        SERVER_INFO_META_KEY: {"name": "mock-modern", "version": "3.0.0"}
                    }
                }),
            )
            .map_err(Into::into),
            MockAction::LegacyInitialize => JsonRpcResponse::success(
                request.id,
                json!({
                    "protocolVersion": LEGACY_PROTOCOL_VERSION,
                    "capabilities": {},
                    "serverInfo": {"name": "mock-legacy", "version": "2.0.0"}
                }),
            )
            .map_err(Into::into),
            MockAction::MethodNotFound => Err(McpError::MethodNotFound("server/discover".into())),
            MockAction::Unsupported => Err(McpError::UnsupportedProtocolVersion {
                requested: MODERN_PROTOCOL_VERSION.into(),
                supported: vec![LEGACY_PROTOCOL_VERSION.into()],
            }),
            MockAction::InputRequired => JsonRpcResponse::success(
                request.id,
                json!({
                    "resultType": "input_required",
                    "inputRequests": {
                        "confirm": {
                            "method": "elicitation/create",
                            "params": {
                                "message": "Proceed?",
                                "requestedSchema": {"type": "object", "properties": {}}
                            }
                        }
                    },
                    "requestState": "opaque-byte-exact-state"
                }),
            )
            .map_err(Into::into),
            MockAction::CompleteTool => JsonRpcResponse::success(
                request.id,
                json!({
                    "resultType": "complete",
                    "content": [{"type": "text", "text": "done"}],
                    "isError": false
                }),
            )
            .map_err(Into::into),
        }
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

#[tokio::test]
async fn auto_negotiation_prefers_modern() {
    let (transport, requests) = ScriptedTransport::new([MockAction::ModernDiscover]);
    let mut client = McpClient::new("client".into(), "3.0.0".into());
    let result = client.connect(transport).await.unwrap();

    assert_eq!(result.protocol, NegotiatedProtocol::modern());
    let sent = requests.lock().unwrap();
    assert_eq!(sent[0].method, methods::SERVER_DISCOVER);
    assert_eq!(
        sent[0].params.as_ref().unwrap()["_meta"][PROTOCOL_VERSION_META_KEY],
        MODERN_PROTOCOL_VERSION
    );
}

#[tokio::test]
async fn auto_negotiation_falls_back_only_on_method_not_found() {
    let (transport, requests) =
        ScriptedTransport::new([MockAction::MethodNotFound, MockAction::LegacyInitialize]);
    let mut client = McpClient::new("client".into(), "3.0.0".into());
    let result = client.connect(transport).await.unwrap();
    assert_eq!(result.protocol, NegotiatedProtocol::legacy());
    {
        let sent = requests.lock().unwrap();
        assert_eq!(sent[0].method, methods::SERVER_DISCOVER);
        assert_eq!(sent[1].method, methods::INITIALIZE);
        assert_eq!(
            sent[1].params.as_ref().unwrap()["protocolVersion"],
            LEGACY_PROTOCOL_VERSION
        );
    }

    let (transport, requests) = ScriptedTransport::new([MockAction::Unsupported]);
    let mut strict_client = McpClient::new("client".into(), "3.0.0".into());
    assert!(matches!(
        strict_client.connect(transport).await,
        Err(McpError::UnsupportedProtocolVersion { .. })
    ));
    assert_eq!(requests.lock().unwrap().len(), 1);
}

struct AcceptingHandler;

#[async_trait]
impl ClientRequestHandler for AcceptingHandler {
    async fn handle_create_message(
        &self,
        _params: CreateMessageParams,
    ) -> McpResult<CreateMessageResult> {
        unreachable!()
    }

    async fn handle_list_roots(&self, _params: ListRootsParams) -> McpResult<ListRootsResult> {
        Ok(ListRootsResult {
            roots: vec![],
            meta: None,
        })
    }

    async fn handle_elicit(&self, _params: ElicitParams) -> McpResult<ElicitResult> {
        Ok(ElicitResult {
            action: ElicitationAction::Accept,
            content: Some(HashMap::from([("confirmed".into(), json!(true))])),
            meta: None,
        })
    }

    async fn handle_ping(&self, _params: PingParams) -> McpResult<PingResult> {
        Ok(PingResult { meta: None })
    }
}

#[tokio::test]
async fn modern_client_completes_bounded_multi_round_trip_request() {
    let (transport, requests) = ScriptedTransport::new([
        MockAction::ModernDiscover,
        MockAction::InputRequired,
        MockAction::CompleteTool,
    ]);
    let mut client = McpClient::new("client".into(), "3.0.0".into());
    client.set_request_handler(AcceptingHandler);
    client.connect(transport).await.unwrap();
    let result = client
        .call_tool("dangerous-operation".into(), None)
        .await
        .unwrap();
    assert_eq!(result.content.len(), 1);

    let sent = requests.lock().unwrap();
    assert_eq!(sent.len(), 3);
    let retry = sent[2].params.as_ref().unwrap();
    assert_eq!(retry["requestState"], "opaque-byte-exact-state");
    assert_eq!(retry["inputResponses"]["confirm"]["action"], "accept");
    assert_ne!(sent[1].id, sent[2].id);
}

struct ConfirmingTool;

#[async_trait]
impl MultiRoundToolHandler for ConfirmingTool {
    async fn call(&self, call: MultiRoundToolCall) -> McpResult<OperationResult<ToolResult>> {
        if call.input_responses.contains_key("confirm") {
            if call.request_state.as_deref() != Some("opaque-server-state") {
                return Err(McpError::Validation("tampered request state".into()));
            }
            return Ok(OperationResult::Complete(ToolResult {
                content: vec![ContentBlock::text("confirmed")],
                is_error: Some(false),
                structured_content: None,
                meta: None,
            }));
        }

        let mut input_requests = HashMap::new();
        input_requests.insert(
            "confirm".to_string(),
            json!({
                "jsonrpc": "2.0",
                "id": "confirm-1",
                "method": methods::ELICITATION_CREATE,
                "params": {
                    "message": "Proceed?",
                    "requestedSchema": {"type": "object", "properties": {}}
                }
            }),
        );
        Ok(OperationResult::InputRequired(InputRequiredResult::new(
            input_requests,
            Some("opaque-server-state".into()),
        )?))
    }
}

#[tokio::test]
async fn modern_server_supports_continuation_aware_tool_handlers() {
    let server = McpServer::create("mrtr-server", "3.0.0");
    server
        .add_multi_round_tool(
            "confirming-tool",
            Some("Requires confirmation"),
            json!({"type": "object"}),
            ConfirmingTool,
        )
        .await
        .unwrap();

    let missing_capability = modern_request(
        79,
        methods::TOOLS_CALL,
        json!({"name": "confirming-tool", "arguments": {}}),
    );
    assert!(matches!(
        server.handle_request(missing_capability).await,
        Err(McpError::MissingRequiredClientCapability(_))
    ));

    let capabilities = ClientCapabilities {
        elicitation: Some(ElicitationCapability::default()),
        ..Default::default()
    };
    let mut first = JsonRpcRequest::new(
        80.into(),
        methods::TOOLS_CALL.into(),
        Some(json!({"name": "confirming-tool", "arguments": {}})),
    )
    .unwrap();
    decorate_modern_request(&mut first, &client_info(), &capabilities).unwrap();
    let first_result = server.handle_request(first).await.unwrap().result.unwrap();
    assert_eq!(first_result["resultType"], "input_required");
    assert_eq!(first_result["requestState"], "opaque-server-state");

    let mut second = JsonRpcRequest::new(
        81.into(),
        methods::TOOLS_CALL.into(),
        Some(json!({
            "name": "confirming-tool",
            "arguments": {},
            "inputResponses": {"confirm": {"action": "accept", "content": {}}},
            "requestState": "opaque-server-state"
        })),
    )
    .unwrap();
    decorate_modern_request(&mut second, &client_info(), &capabilities).unwrap();
    let second_result = server.handle_request(second).await.unwrap().result.unwrap();
    assert_eq!(second_result["resultType"], "complete");
    assert_eq!(second_result["content"][0]["text"], "confirmed");
}
