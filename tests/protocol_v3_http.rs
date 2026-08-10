#![cfg(feature = "http")]

use prism_mcp_rs::client::McpClient;
use prism_mcp_rs::protocol::types::{ClientCapabilities, Implementation, JsonRpcRequest};
use prism_mcp_rs::protocol::version::{
    decorate_modern_request, MCP_METHOD_HEADER, MCP_NAME_HEADER, MCP_PROTOCOL_VERSION_HEADER,
    MODERN_PROTOCOL_VERSION,
};
use prism_mcp_rs::protocol::{methods, SubscriptionFilter};
use prism_mcp_rs::server::McpServer;
use prism_mcp_rs::transport::{HttpClientTransport, HttpServerTransport, Transport};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn modern_http_client_emits_standard_routing_headers() {
    let server = MockServer::start().await;
    let mut request = JsonRpcRequest::new(
        41.into(),
        "tools/call".into(),
        Some(json!({"name": "search", "arguments": {"q": "otters"}})),
    )
    .unwrap();
    decorate_modern_request(
        &mut request,
        &Implementation::new("http-test", "3.0.0"),
        &ClientCapabilities::default(),
    )
    .unwrap();

    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header(MCP_PROTOCOL_VERSION_HEADER, MODERN_PROTOCOL_VERSION))
        .and(header(MCP_METHOD_HEADER, "tools/call"))
        .and(header(MCP_NAME_HEADER, "search"))
        .and(body_json(serde_json::to_value(&request).unwrap()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 41,
            "result": {"resultType": "complete", "content": []}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mut transport = HttpClientTransport::new(server.uri(), None::<String>)
        .await
        .unwrap();
    transport.send_request(request).await.unwrap();
}

#[tokio::test]
async fn modern_http_client_emits_schema_declared_tool_headers() {
    let server = MockServer::start().await;
    let identity = Implementation::new("http-test", "3.0.0");
    let capabilities = ClientCapabilities::default();

    let mut list_request =
        JsonRpcRequest::new(50.into(), "tools/list".into(), Some(json!({}))).unwrap();
    decorate_modern_request(&mut list_request, &identity, &capabilities).unwrap();
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header(MCP_METHOD_HEADER, "tools/list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 50,
            "result": {
                "resultType": "complete",
                "tools": [{
                    "name": "search",
                    "description": "Search one region",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "region": {"type": "string", "x-mcp-header": "Region"},
                            "q": {"type": "string"}
                        }
                    }
                }]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header(MCP_METHOD_HEADER, "tools/call"))
        .and(header("Mcp-Param-Region", "eu-north-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 51,
            "result": {"resultType": "complete", "content": []}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mut transport = HttpClientTransport::new(server.uri(), None::<String>)
        .await
        .unwrap();
    transport.send_request(list_request).await.unwrap();

    let mut call_request = JsonRpcRequest::new(
        51.into(),
        "tools/call".into(),
        Some(json!({
            "name": "search",
            "arguments": {"q": "otters", "region": "eu-north-1"}
        })),
    )
    .unwrap();
    decorate_modern_request(&mut call_request, &identity, &capabilities).unwrap();
    transport.send_request(call_request).await.unwrap();
}

#[tokio::test]
async fn modern_http_client_encodes_unsafe_routing_names() {
    let server = MockServer::start().await;
    let mut request = JsonRpcRequest::new(
        61.into(),
        "resources/read".into(),
        Some(json!({"uri": "file:///résumé.txt"})),
    )
    .unwrap();
    decorate_modern_request(
        &mut request,
        &Implementation::new("http-test", "3.0.0"),
        &ClientCapabilities::default(),
    )
    .unwrap();

    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header(MCP_METHOD_HEADER, "resources/read"))
        .and(header(
            MCP_NAME_HEADER,
            "=?base64?ZmlsZTovLy9yw6lzdW3DqS50eHQ=?=",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 61,
            "result": {
                "resultType": "complete",
                "contents": [],
                "ttlMs": 0,
                "cacheScope": "private"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mut transport = HttpClientTransport::new(server.uri(), None::<String>)
        .await
        .unwrap();
    transport.send_request(request).await.unwrap();
}

#[tokio::test]
async fn modern_http_subscription_acknowledges_filters_and_routes_notifications() {
    let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = reserved.local_addr().unwrap();
    drop(reserved);

    let mut server = McpServer::create("subscription-test", "3.0.0");
    server
        .start(HttpServerTransport::new(address.to_string()))
        .await
        .unwrap();

    let mut client = McpClient::new("subscription-client".to_string(), "3.0.0".to_string());
    let server_url = format!("http://{address}");
    client.connect_with_http(&server_url, None).await.unwrap();
    let mut subscription = client
        .listen(SubscriptionFilter {
            tools_list_changed: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();

    let acknowledgement =
        tokio::time::timeout(std::time::Duration::from_secs(2), subscription.next())
            .await
            .unwrap()
            .unwrap();
    assert_eq!(acknowledgement.method, methods::SUBSCRIPTIONS_ACKNOWLEDGED);
    assert_eq!(
        acknowledgement.params.as_ref().unwrap()["notifications"]["toolsListChanged"],
        true
    );
    assert_eq!(
        acknowledgement.params.as_ref().unwrap()["_meta"]["io.modelcontextprotocol/subscriptionId"],
        subscription.id().clone()
    );

    server.notify_tools_list_changed().await.unwrap();
    let notification = tokio::time::timeout(std::time::Duration::from_secs(2), subscription.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(notification.method, methods::TOOLS_LIST_CHANGED);
    assert_eq!(
        notification.params.as_ref().unwrap()["_meta"]["io.modelcontextprotocol/subscriptionId"],
        subscription.id().clone()
    );

    client.cancel_subscription(&subscription).await.unwrap();
    server.stop().await.unwrap();
}
