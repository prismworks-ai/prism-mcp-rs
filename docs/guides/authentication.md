# Authentication Guide

This guide covers OAuth 2.1 authentication mechanisms available in the Prism MCP SDK.

## Overview

The Prism MCP SDK implements OAuth 2.1 authorization as specified in the MCP Authorization specification. The implementation follows OAuth 2.1, OAuth 2.0 Authorization Server Metadata, Dynamic Client Registration, and Protected Resource Metadata specifications.

## Enabling Authentication

Add the `auth` feature to your `Cargo.toml`:

```toml
[dependencies]
prism-mcp-rs = {
    version = "1.1.0",
    features = ["auth"]
}
```

## OAuth 2.1 Flow

### Basic Authorization Setup

The authentication system uses OAuth 2.1 with PKCE (Proof Key for Code Exchange):

```rust
use prism_mcp_rs::auth::{
    AuthConfig, AuthorizationContext, ClientRegistrationRequest, 
    TokenRequest, ProtectedResourceMetadata
};
use prism_mcp_rs::core::error::McpResult;

// Configure OAuth 2.1 client
let auth_config = AuthConfig {
    enabled: true,
    client_id: Some("your-client-id".to_string()),
    client_secret: None, // Public client with PKCE
    redirect_uri: "http://localhost:8080/callback".to_string(),
    scopes: vec!["mcp:read".to_string(), "mcp:write".to_string()],
    enable_dynamic_registration: true,
};

// Create authorization context
let mut auth_context = AuthorizationContext::new(
    "https://example.com/mcp-server".to_string()
);
```

### Dynamic Client Registration

For public clients without pre-registration:

```rust
use prism_mcp_rs::auth::{ClientRegistrationRequest, ClientRegistrationResponse};

// Register client dynamically
let registration_request = ClientRegistrationRequest {
    redirect_uris: vec!["http://localhost:8080/callback".to_string()],
    client_name: Some("My MCP Client".to_string()),
    client_uri: Some("https://example.com".to_string()),
    grant_types: Some(vec!["authorization_code".to_string()]),
    response_types: Some(vec!["code".to_string()]),
    token_endpoint_auth_method: Some("none".to_string()), // Public client
    scope: Some("mcp:read mcp:write".to_string()),
    software_id: Some("my-mcp-client".to_string()),
    software_version: Some("1.0.0".to_string()),
};

// This would be sent to the authorization server's registration endpoint
// let response: ClientRegistrationResponse = register_client(registration_request).await?;
```

### Authorization Code Flow with PKCE

```rust
use prism_mcp_rs::auth::{PkceChallenge, TokenRequest, TokenResponse};
use prism_mcp_rs::auth::pkce::PkceCodeChallenge;

// Generate PKCE challenge
let pkce = PkceCodeChallenge::new();
let code_challenge = pkce.challenge();
let code_verifier = pkce.verifier().to_string();

// Build authorization URL
let auth_url = format!(
    "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&state={}",
    "https://auth.example.com/authorize",
    "your-client-id",
    "http://localhost:8080/callback",
    "mcp:read mcp:write",
    code_challenge,
    "random-state-value"
);

// After user authorization, exchange code for token
let token_request = TokenRequest {
    grant_type: "authorization_code".to_string(),
    code: Some("received-auth-code".to_string()),
    redirect_uri: Some("http://localhost:8080/callback".to_string()),
    code_verifier: Some(code_verifier),
    client_id: Some("your-client-id".to_string()),
    resource: Some("https://example.com/mcp-server".to_string()),
    ..Default::default()
};

// Exchange for access token
// let token_response: TokenResponse = exchange_code_for_token(token_request).await?;
```

## Client Authentication

Clients authenticate using OAuth 2.1 access tokens:

```rust
use prism_mcp_rs::client::McpClient;
use prism_mcp_rs::transport::StdioTransport;
use prism_mcp_rs::auth::AuthorizationContext;

// Create client with OAuth context
let transport = StdioTransport::new();
let mut client = McpClient::new(transport);

// Set up authorization context with access token
let mut auth_context = AuthorizationContext::new(
    "https://example.com/mcp-server".to_string()
);
auth_context.access_token = Some("your-access-token".to_string());
auth_context.expires_at = Some(1234567890); // Unix timestamp

// The client will automatically include the Bearer token in requests
// when auth_context has a valid access token
```

## Discovery and Metadata

### Resource Metadata Discovery

```rust
use prism_mcp_rs::auth::{ProtectedResourceMetadata, AuthorizationServerMetadata};

// Discover resource metadata (typically from /.well-known/protected_resource)
let resource_metadata = ProtectedResourceMetadata {
    resource: "https://example.com/mcp-server".to_string(),
    authorization_servers: vec![
        "https://auth.example.com".to_string()
    ],
    bearer_methods_supported: Some(vec![
        "header".to_string(),
        "query".to_string()
    ]),
    scopes_supported: Some(vec![
        "mcp:read".to_string(),
        "mcp:write".to_string(),
        "mcp:admin".to_string()
    ]),
    additional: std::collections::HashMap::new(),
};

// Discover authorization server metadata
let auth_server_metadata = AuthorizationServerMetadata {
    issuer: "https://auth.example.com".to_string(),
    authorization_endpoint: "https://auth.example.com/authorize".to_string(),
    token_endpoint: "https://auth.example.com/token".to_string(),
    registration_endpoint: Some("https://auth.example.com/register".to_string()),
    scopes_supported: Some(vec![
        "mcp:read".to_string(),
        "mcp:write".to_string()
    ]),
    response_types_supported: vec!["code".to_string()],
    grant_types_supported: Some(vec!["authorization_code".to_string()]),
    code_challenge_methods_supported: Some(vec!["S256".to_string()]),
    additional: std::collections::HashMap::new(),
};
```

## Token Management

### Access Token Usage

```rust
use prism_mcp_rs::auth::AuthorizationContext;

// Check token validity
let auth_context = AuthorizationContext::new(
    "https://example.com/mcp-server".to_string()
);

if auth_context.has_valid_token() {
    // Token is present and not expired
    println!("Token is valid");
} else if auth_context.is_token_expired() {
    // Token exists but is expired - refresh needed
    println!("Token expired, refresh required");
} else {
    // No token - need to authenticate
    println!("No token, authentication required");
}
```

### Token Refresh

```rust
use prism_mcp_rs::auth::TokenRequest;

// Refresh an expired access token
let refresh_request = TokenRequest {
    grant_type: "refresh_token".to_string(),
    refresh_token: Some(auth_context.refresh_token.clone().unwrap()),
    scope: Some("mcp:read mcp:write".to_string()),
    resource: Some("https://example.com/mcp-server".to_string()),
    client_id: Some("your-client-id".to_string()),
    ..Default::default()
};

// Send refresh request to token endpoint
// let new_tokens: TokenResponse = refresh_access_token(refresh_request).await?;
```

## Error Handling

### WWW-Authenticate Header Processing

```rust
use prism_mcp_rs::auth::AuthChallenge;
use prism_mcp_rs::core::error::McpError;

// Parse WWW-Authenticate header from server response
let auth_header = "Bearer realm=\"MCP Server\", error=\"invalid_token\", error_description=\"Token expired\"";

if let Some(challenge) = AuthChallenge::parse(auth_header) {
    match challenge.error.as_deref() {
        Some("invalid_token") => {
            // Token is invalid or expired
            println!("Need to refresh or re-authenticate");
        }
        Some("insufficient_scope") => {
            // Need additional permissions
            println!("Insufficient scope for this operation");
        }
        _ => {
            // Other auth error
            println!("Authentication error: {:?}", challenge.error_description);
        }
    }
}
```

### OAuth 2.0 Error Responses

```rust
use prism_mcp_rs::auth::OAuth2Error;

// Handle OAuth 2.0 error responses
let oauth_error = OAuth2Error {
    error: "access_denied".to_string(),
    error_description: Some("User denied access".to_string()),
    error_uri: Some("https://example.com/error".to_string()),
};

match oauth_error.error.as_str() {
    "invalid_request" => println!("Malformed request"),
    "invalid_client" => println!("Client authentication failed"),
    "invalid_grant" => println!("Authorization grant invalid"),
    "unauthorized_client" => println!("Client not authorized"),
    "unsupported_grant_type" => println!("Grant type not supported"),
    "invalid_scope" => println!("Requested scope invalid"),
    _ => println!("Other OAuth error: {}", oauth_error.error),
}
```

## Security Best Practices

### 1. PKCE Implementation

- Always use PKCE for public clients
- Use cryptographically secure random values
- Never transmit code verifier over insecure channels

```rust
use prism_mcp_rs::auth::pkce::PkceCodeChallenge;

// Generate secure PKCE challenge
let pkce = PkceCodeChallenge::new();
let challenge = pkce.challenge(); // URL-safe base64 encoded
let verifier = pkce.verifier(); // Store securely, never transmit

// Use S256 method (SHA256) for code_challenge_method
```

### 2. Token Storage

- Store tokens securely (encrypted at rest)
- Use secure storage mechanisms (keychain, secure enclave)
- Implement token rotation
- Clear tokens on logout

### 3. Transport Security

- Always use HTTPS for OAuth endpoints
- Validate TLS certificates
- Use certificate pinning when possible
- Implement proper timeout handling

## Testing OAuth 2.1 Flow

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use prism_mcp_rs::auth::{
        AuthorizationContext, PkceCodeChallenge, TokenResponse
    };
    
    #[tokio::test]
    async fn test_pkce_generation() {
        let pkce = PkceCodeChallenge::new();
        let challenge = pkce.challenge();
        let verifier = pkce.verifier();
        
        // Verify challenge is URL-safe base64
        assert!(!challenge.contains('+')); 
        assert!(!challenge.contains('/'));
        assert!(!challenge.contains('='));
        
        // Verify minimum length requirements
        assert!(verifier.len() >= 43);
        assert!(challenge.len() >= 43);
    }
    
    #[tokio::test]
    async fn test_token_expiration() {
        let mut auth_context = AuthorizationContext::new(
            "https://example.com/mcp-server".to_string()
        );
        
        // Test with expired token
        auth_context.access_token = Some("test-token".to_string());
        auth_context.expires_at = Some(1000000000); // Past timestamp
        
        assert!(!auth_context.has_valid_token());
        assert!(auth_context.is_token_expired());
        
        // Test with valid token
        auth_context.expires_at = Some(9999999999); // Future timestamp
        
        assert!(auth_context.has_valid_token());
        assert!(!auth_context.is_token_expired());
    }
    
    #[tokio::test]
    async fn test_metadata_parsing() {
        let metadata_json = r#"{
            "resource": "https://example.com/mcp-server",
            "authorization_servers": ["https://auth.example.com"],
            "scopes_supported": ["mcp:read", "mcp:write"]
        }"#;
        
        let metadata: ProtectedResourceMetadata = 
            serde_json::from_str(metadata_json).unwrap();
        
        assert_eq!(metadata.resource, "https://example.com/mcp-server");
        assert_eq!(metadata.authorization_servers.len(), 1);
        assert!(metadata.scopes_supported.is_some());
    }
}
```

## Troubleshooting

### Common Issues

1. **"invalid_token" errors**: Token may be expired, malformed, or revoked
2. **"insufficient_scope" errors**: Request additional scopes during authorization
3. **PKCE verification failures**: Ensure code_verifier matches code_challenge
4. **Discovery failures**: Check /.well-known endpoints are accessible
5. **Redirect URI mismatches**: Ensure exact match with registered URIs

### Debug Logging

Enable debug logging for OAuth troubleshooting:

```bash
RUST_LOG=prism_mcp_rs::auth=debug cargo run
```

### Authorization Server Requirements

For MCP server implementers, your authorization server should:

1. Support PKCE (RFC 7636) with S256 method
2. Implement resource indicators (RFC 8707)
3. Provide discovery endpoints:
   - `/.well-known/oauth-authorization-server`
   - `/.well-known/protected_resource`
4. Support dynamic client registration (RFC 7591) for public clients
5. Use proper CORS headers for browser-based flows

## Further Reading

- [MCP Authorization Specification](https://spec.modelcontextprotocol.io/)
- [OAuth 2.1 (RFC 9749)](https://datatracker.ietf.org/doc/html/rfc9749)
- [PKCE (RFC 7636)](https://datatracker.ietf.org/doc/html/rfc7636)
- [Resource Indicators (RFC 8707)](https://datatracker.ietf.org/doc/html/rfc8707)
- [API Documentation](https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/auth/)