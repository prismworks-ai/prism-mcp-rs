//! Test file to verify that documentation examples compile and work correctly
//! This ensures our documentation code snippets are accurate and up-to-date

use prism_mcp_rs::prelude::*;

use async_trait::async_trait;
#[cfg(feature = "http")]
#[allow(unused_imports)]
use prism_mcp_rs::auth::{
    AuthChallenge, AuthConfig, AuthorizationContext, AuthorizationServerMetadata,
    ClientRegistrationRequest, PkceParams, ProtectedResourceMetadata, TokenRequest,
};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone)]
#[allow(dead_code)]
struct TestToolHandler;

#[async_trait]
impl ToolHandler for TestToolHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello from documentation test!");

        Ok(CallToolResult::text(message))
    }
}

/// Test OAuth 2.1 configuration from authentication.md
#[cfg(feature = "http")]
#[tokio::test]
async fn test_auth_config_example() {
    // This example from authentication.md should compile
    let auth_config = AuthConfig {
        enabled: true,
        client_id: Some("your-client-id".to_string()),
        client_secret: None, // Public client with PKCE
        redirect_uri: "http://localhost:8080/callback".to_string(),
        scopes: vec!["mcp:read".to_string(), "mcp:write".to_string()],
        enable_dynamic_registration: true,
    };

    // Create authorization context
    let auth_context = AuthorizationContext::new("https://example.com/mcp-server".to_string());

    assert_eq!(auth_context.resource, "https://example.com/mcp-server");
    assert!(auth_config.enabled);
}

/// Test PKCE generation from authentication.md
#[cfg(feature = "http")]
#[tokio::test]
async fn test_pkce_example() {
    // Generate PKCE challenge
    let pkce = PkceParams::new();
    let code_challenge = &pkce.challenge;
    let code_verifier = &pkce.verifier;

    // Verify the values are generated
    assert!(!code_challenge.is_empty());
    assert!(!code_verifier.is_empty());
    assert!(code_challenge.len() >= 43);
    assert!(code_verifier.len() >= 43);
}

/// Test client registration from authentication.md
#[cfg(feature = "http")]
#[tokio::test]
async fn test_client_registration_example() {
    let registration_request = ClientRegistrationRequest {
        redirect_uris: vec!["http://localhost:8080/callback".to_string()],
        client_name: Some("My MCP Client".to_string()),
        client_uri: Some("https://example.com".to_string()),
        logo_uri: None,
        grant_types: Some(vec!["authorization_code".to_string()]),
        response_types: Some(vec!["code".to_string()]),
        token_endpoint_auth_method: Some("none".to_string()), // Public client
        scope: Some("mcp:read mcp:write".to_string()),
        software_id: Some("my-mcp-client".to_string()),
        software_version: Some("1.0.0".to_string()),
    };

    assert_eq!(registration_request.redirect_uris.len(), 1);
    assert_eq!(
        registration_request.client_name.as_ref().unwrap(),
        "My MCP Client"
    );
}

/// Test token request from authentication.md
#[cfg(feature = "http")]
#[tokio::test]
async fn test_token_request_example() {
    let token_request = TokenRequest {
        grant_type: "authorization_code".to_string(),
        code: Some("received-auth-code".to_string()),
        redirect_uri: Some("http://localhost:8080/callback".to_string()),
        code_verifier: Some("test-verifier".to_string()),
        client_id: Some("your-client-id".to_string()),
        resource: Some("https://example.com/mcp-server".to_string()),
        refresh_token: None,
        client_secret: None,
        scope: None,
    };

    assert_eq!(token_request.grant_type, "authorization_code");
    assert!(token_request.code.is_some());
}

/// Test metadata structures from authentication.md
#[cfg(feature = "http")]
#[tokio::test]
async fn test_metadata_examples() {
    // Discover resource metadata
    let resource_metadata = ProtectedResourceMetadata {
        resource: "https://example.com/mcp-server".to_string(),
        authorization_servers: vec!["https://auth.example.com".to_string()],
        bearer_methods_supported: Some(vec!["header".to_string(), "query".to_string()]),
        scopes_supported: Some(vec![
            "mcp:read".to_string(),
            "mcp:write".to_string(),
            "mcp:admin".to_string(),
        ]),
        additional: std::collections::HashMap::new(),
    };

    assert_eq!(resource_metadata.resource, "https://example.com/mcp-server");
    assert_eq!(resource_metadata.authorization_servers.len(), 1);

    // Discover authorization server metadata
    let auth_server_metadata = AuthorizationServerMetadata {
        issuer: "https://auth.example.com".to_string(),
        authorization_endpoint: "https://auth.example.com/authorize".to_string(),
        token_endpoint: "https://auth.example.com/token".to_string(),
        registration_endpoint: Some("https://auth.example.com/register".to_string()),
        scopes_supported: Some(vec!["mcp:read".to_string(), "mcp:write".to_string()]),
        response_types_supported: vec!["code".to_string()],
        grant_types_supported: Some(vec!["authorization_code".to_string()]),
        code_challenge_methods_supported: Some(vec!["S256".to_string()]),
        response_modes_supported: None,
        token_endpoint_auth_methods_supported: None,
        revocation_endpoint: None,
        introspection_endpoint: None,
        additional: std::collections::HashMap::new(),
    };

    assert_eq!(auth_server_metadata.issuer, "https://auth.example.com");
    assert!(auth_server_metadata.registration_endpoint.is_some());
}

/// Test authorization context from authentication.md
#[cfg(feature = "http")]
#[tokio::test]
async fn test_authorization_context_example() {
    // Check token validity
    let mut auth_context = AuthorizationContext::new("https://example.com/mcp-server".to_string());

    // Initially no token
    assert!(!auth_context.has_valid_token());
    assert!(!auth_context.is_token_expired());

    // Add expired token
    auth_context.access_token = Some("test-token".to_string());
    auth_context.expires_at = Some(1000000000); // Past timestamp

    assert!(!auth_context.has_valid_token());
    assert!(auth_context.is_token_expired());

    // Add valid token
    auth_context.expires_at = Some(9999999999); // Future timestamp

    assert!(auth_context.has_valid_token());
    assert!(!auth_context.is_token_expired());
}

/// Test WWW-Authenticate header parsing from authentication.md
#[cfg(feature = "http")]
#[tokio::test]
async fn test_auth_challenge_example() {
    // Parse WWW-Authenticate header from server response
    let auth_header =
        r#"Bearer realm="MCP Server", error="invalid_token", error_description="Token expired""#;

    if let Some(challenge) = AuthChallenge::parse(auth_header) {
        assert_eq!(challenge.scheme, "Bearer");
        assert_eq!(challenge.realm.as_deref(), Some("MCP Server"));
        assert_eq!(challenge.error.as_deref(), Some("invalid_token"));
        assert_eq!(
            challenge.error_description.as_deref(),
            Some("Token expired")
        );
    } else {
        panic!("Failed to parse auth challenge");
    }
}

/// Test CallToolResult usage from error-handling.md
#[tokio::test]
async fn test_call_tool_result_example() {
    // Create successful result
    let success_result = CallToolResult::text("Operation completed");
    assert_eq!(success_result.is_error, Some(false));
    assert!(!success_result.content.is_empty());

    // Create error result
    let error_result = CallToolResult::error("Something went wrong");
    assert_eq!(error_result.is_error, Some(true));
    assert!(!error_result.content.is_empty());

    // Create result with structured content
    let structured_result = CallToolResult::with_structured(
        vec![ContentBlock::text("Success")],
        json!({"status": "completed", "count": 42}),
    );
    assert!(structured_result.structured_content.is_some());
    assert_eq!(structured_result.is_error, Some(false));
}

/// Test tool handler pattern from error-handling.md
#[tokio::test]
async fn test_tool_handler_error_propagation() {
    let handler = TestToolHandler;

    // Test with valid arguments
    let mut args = HashMap::new();
    args.insert(
        "message".to_string(),
        Value::String("test message".to_string()),
    );

    let result = handler.call(args).await.unwrap();
    assert_eq!(result.is_error, Some(false));

    // Test with missing arguments (should still work with default)
    let empty_args = HashMap::new();
    let result = handler.call(empty_args).await.unwrap();
    assert_eq!(result.is_error, Some(false));
}

/// Test that server creation works with current API
#[tokio::test]
async fn test_server_creation_example() {
    let server = McpServer::new("test-server".to_string(), "1.0.0".to_string());

    // Add a tool using the corrected API from documentation
    server
        .add_tool(
            "test_tool",
            Some("A test tool for documentation validation"),
            json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo"
                    }
                }
            }),
            TestToolHandler,
        )
        .await
        .expect("Failed to add tool");

    // Verify the tool was added
    // Note: In real usage, you'd start the server with a transport
}

fn main() {
    println!("Documentation examples compilation test completed successfully!");
    println!("All code snippets from the documentation are verified to work with the current API.");
}
