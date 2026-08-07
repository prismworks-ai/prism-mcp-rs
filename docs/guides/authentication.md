# Authentication

The optional auth module provides OAuth-oriented client primitives: protected-resource and authorization-server discovery, PKCE, authorization URL/callback handling, token management, dynamic registration types, and bearer-header helpers. It does not automatically protect an `McpServer`.

## Enable the module

The auth module is currently wired through HTTP support:

```toml
[dependencies]
prism-mcp-rs = { version = "2", features = ["http", "auth"] }
```

## Client configuration

```rust,no_run
use prism_mcp_rs::auth::{AuthConfig, AuthorizationClient, PkceParams};

let config = AuthConfig::new()
    .with_auth(true)
    .with_client_credentials("client-id".to_string(), None)
    .with_redirect_uri("http://127.0.0.1:8765/callback".to_string())
    .with_scopes(vec!["mcp:read".to_string()]);

let pkce = PkceParams::new();
let client = AuthorizationClient::new(config, "https://mcp.example".to_string());

// Keep `pkce.verifier` secret until the authorization-code exchange.
let _ = (client, pkce);
```

Use `DiscoveryClient` to resolve protected-resource and authorization-server metadata. Validate issuer equality, HTTPS requirements, supported PKCE methods, redirect URI, state, and the resource indicator before following discovered endpoints. Do not allow arbitrary discovery URLs to become an SSRF path.

## Server integration

Authentication happens before common request dispatch:

1. Extract credentials from the transport (for example, a bearer token or verified client certificate).
2. Validate signature/issuer/audience/expiry or certificate chain and revocation policy.
3. Map the result to a stable `Principal` and trusted roles/attributes.
4. Build `RequestContext` with correlation and transport details.
5. Call `handle_request_with_context` so RBAC and rate limits use the authenticated identity.

Never derive roles from unsigned request parameters or accept a caller-supplied principal ID. The SDK's `handle_request` compatibility method uses an anonymous principal.

## Token handling

- Prefer authorization code with S256 PKCE for public clients.
- Generate and verify state for every authorization attempt.
- Keep client secrets and refresh tokens in a managed secret store or OS keychain.
- Never log tokens, authorization codes, code verifiers, or full callback URLs.
- Apply clock skew deliberately and validate issuer, audience/resource, expiry, and scopes.
- Refresh before expiry and handle refresh-token rotation atomically.
- Use HTTPS outside loopback development.

`AuthorizationContext` tracks client token state; it is separate from `security::RequestContext`, which represents the authenticated server request. Do not confuse the two types.

## Authorization is separate

A valid token proves an identity and claims; it does not grant every MCP operation. Install `RbacAuthorizer` and rate limiting as described in [Production Controls](../PRODUCTION_CONTROLS.md).

## Testing

Test invalid issuer/audience, expired and not-yet-valid tokens, state mismatch, PKCE mismatch, scope denial, key rotation, discovery failure, and revoked credentials. Use a local mock authorization server in automated tests rather than public network endpoints.
