# Authentication Guide

This guide covers authentication mechanisms available in the Prism MCP SDK.

## Overview

The Prism MCP SDK supports various authentication methods to secure your MCP servers and clients. Authentication is available through the `auth` feature flag.

## Enabling Authentication

Add the `auth` feature to your `Cargo.toml`:

```toml
[dependencies]
prism-mcp-rs = {
    version = "0.1.0",
    features = ["auth"]
}
```

## Authentication Methods

### Token-Based Authentication

The most common authentication method uses bearer tokens:

```rust
use prism_mcp_rs::auth::{AuthHandler, TokenValidator};
use async_trait::async_trait;
use std::collections::HashSet;

struct BearerTokenAuth {
    valid_tokens: HashSet<String>,
}

#[async_trait]
impl TokenValidator for BearerTokenAuth {
    async fn validate_token(&self, token: &str) -> McpResult<bool> {
        Ok(self.valid_tokens.contains(token))
    }
}

// Configure server with authentication
let auth_handler = BearerTokenAuth {
    valid_tokens: HashSet::from([
        "secret-token-1".to_string(),
        "secret-token-2".to_string(),
    ]),
};

server.with_auth(auth_handler);
```

### API Key Authentication

```rust
struct ApiKeyAuth {
    api_keys: HashMap<String, UserInfo>,
}

#[async_trait]
impl AuthHandler for ApiKeyAuth {
    async fn authenticate(&self, credentials: &Credentials) -> McpResult<AuthResult> {
        if let Some(api_key) = credentials.api_key() {
            if let Some(user_info) = self.api_keys.get(api_key) {
                return Ok(AuthResult::Success {
                    user: user_info.clone(),
                    permissions: user_info.permissions.clone(),
                });
            }
        }
        Ok(AuthResult::Failure("Invalid API key".to_string()))
    }
}
```

### OAuth 2.0 Integration

```rust
struct OAuth2Handler {
    provider_url: String,
    client_id: String,
    client_secret: String,
}

#[async_trait]
impl AuthHandler for OAuth2Handler {
    async fn authenticate(&self, credentials: &Credentials) -> McpResult<AuthResult> {
        // Validate OAuth token with provider
        let token = credentials.bearer_token()
            .ok_or_else(|| McpError::Unauthorized)?;
        
        let validation_result = self.validate_with_provider(token).await?;
        
        if validation_result.is_valid {
            Ok(AuthResult::Success {
                user: validation_result.user,
                permissions: validation_result.scopes,
            })
        } else {
            Ok(AuthResult::Failure("Invalid OAuth token".to_string()))
        }
    }
}
```

## Client Authentication

Clients must provide authentication credentials:

```rust
use prism_mcp_rs::client::{ClientSession, AuthCredentials};

let mut session = ClientSession::new(transport);

// Provide authentication credentials
session.set_credentials(AuthCredentials::BearerToken(
    "your-secret-token".to_string()
));

// Initialize with authentication
session.initialize(
    "my-client".to_string(),
    "1.0.0".to_string(),
).await?;
```

## TLS/SSL Support

For secure transport, enable the `tls` feature:

```toml
[dependencies]
prism-mcp-rs = {
    version = "0.1.0",
    features = ["auth", "tls"]
}
```

### Configuring TLS

```rust
use prism_mcp_rs::transport::tls::TlsConfig;

let tls_config = TlsConfig::builder()
    .cert_file("path/to/cert.pem")
    .key_file("path/to/key.pem")
    .ca_file("path/to/ca.pem")
    .build()?;

server.with_tls(tls_config);
```

## Permission Management

### Role-Based Access Control (RBAC)

```rust
#[derive(Clone, Debug)]
struct UserPermissions {
    roles: Vec<String>,
    allowed_tools: HashSet<String>,
    allowed_resources: HashSet<String>,
}

impl PermissionChecker for UserPermissions {
    fn can_call_tool(&self, tool_name: &str) -> bool {
        self.allowed_tools.contains(tool_name) ||
        self.roles.contains(&"admin".to_string())
    }
    
    fn can_access_resource(&self, uri: &str) -> bool {
        self.allowed_resources.iter()
            .any(|pattern| uri.starts_with(pattern)) ||
        self.roles.contains(&"admin".to_string())
    }
}
```

### Applying Permissions

```rust
#[async_trait]
impl ToolHandler for SecureToolHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        // Check permissions from context
        let context = get_request_context()?;
        if !context.permissions.can_call_tool("sensitive_tool") {
            return Err(McpError::Forbidden {
                message: "Insufficient permissions".to_string(),
            });
        }
        
        // Execute tool logic
        Ok(ToolResult::text("Operation completed"))
    }
}
```

## Security Best Practices

### 1. Token Management

- Use strong, randomly generated tokens
- Implement token expiration
- Rotate tokens regularly
- Never log or expose tokens

```rust
use chrono::{DateTime, Utc, Duration};

struct ExpiringToken {
    token: String,
    expires_at: DateTime<Utc>,
}

impl ExpiringToken {
    fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
    
    fn new(duration: Duration) -> Self {
        Self {
            token: generate_secure_token(),
            expires_at: Utc::now() + duration,
        }
    }
}
```

### 2. Rate Limiting

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct RateLimiter {
    limits: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    async fn check_limit(&self, client_id: &str) -> McpResult<()> {
        let mut limits = self.limits.lock().await;
        let now = Instant::now();
        
        let requests = limits.entry(client_id.to_string())
            .or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        requests.retain(|&time| now.duration_since(time) < self.window);
        
        if requests.len() >= self.max_requests {
            return Err(McpError::RateLimitExceeded);
        }
        
        requests.push(now);
        Ok(())
    }
}
```

### 3. Audit Logging

```rust
use log::info;
use chrono::Utc;

struct AuditLogger;

impl AuditLogger {
    fn log_authentication(&self, user: &str, success: bool) {
        info!(
            "Authentication attempt - User: {}, Success: {}, Time: {}",
            user, success, Utc::now()
        );
    }
    
    fn log_tool_call(&self, user: &str, tool: &str, arguments: &HashMap<String, Value>) {
        info!(
            "Tool call - User: {}, Tool: {}, Arguments: {:?}, Time: {}",
            user, tool, arguments, Utc::now()
        );
    }
}
```

## Testing Authentication

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_valid_token() {
        let auth = BearerTokenAuth {
            valid_tokens: HashSet::from(["test-token".to_string()]),
        };
        
        assert!(auth.validate_token("test-token").await.unwrap());
        assert!(!auth.validate_token("invalid-token").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_permission_check() {
        let permissions = UserPermissions {
            roles: vec!["user".to_string()],
            allowed_tools: HashSet::from(["read_data".to_string()]),
            allowed_resources: HashSet::from(["public://".to_string()]),
        };
        
        assert!(permissions.can_call_tool("read_data"));
        assert!(!permissions.can_call_tool("write_data"));
        assert!(permissions.can_access_resource("public://file.txt"));
        assert!(!permissions.can_access_resource("private://secret.txt"));
    }
}
```

## Troubleshooting

### Common Issues

1. **"Unauthorized" errors**: Check that credentials are properly set and valid
2. **Token expiration**: Implement token refresh logic
3. **Permission denied**: Verify user roles and permissions
4. **TLS handshake failures**: Check certificate validity and configuration

### Debug Logging

Enable debug logging to troubleshoot authentication issues:

```bash
RUST_LOG=prism_mcp_rs::auth=debug cargo run
```

## Further Reading

- [Transport Security](../../examples/secure_transport.rs)
- [OAuth Integration Example](../../examples/oauth_server.rs)
- [API Documentation](https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/auth/)