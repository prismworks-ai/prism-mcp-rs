// ! HTTP Transport with OAuth 2.1 Authorization Support
// !
// ! Module extends the HTTP transport with automatic authorization
// ! handling, including token refresh and 401 response handling.

use async_trait::async_trait;
use std::sync::Arc;

use crate::auth::{AuthConfig, AuthorizationClient};
use crate::core::error::{McpError, McpResult};
use crate::protocol::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use crate::transport::{Transport, http::HttpClientTransport};

/// HTTP transport with automatic authorization support
pub struct AuthorizedHttpTransport {
    /// Base HTTP transport
    inner: HttpClientTransport,
    /// Authorization client
    auth_client: Arc<AuthorizationClient>,
    /// Whether authorization is enabled
    auth_enabled: bool,
}

impl AuthorizedHttpTransport {
    /// Create a new authorized HTTP transport
    pub async fn new(
        base_url: String,
        sse_url: Option<String>,
        auth_config: AuthConfig,
    ) -> McpResult<Self> {
        let auth_enabled = auth_config.enabled;
        let inner = HttpClientTransport::new(&base_url, sse_url.as_ref()).await?;
        let auth_client = Arc::new(AuthorizationClient::new(auth_config, base_url.clone()));

        Ok(Self {
            inner,
            auth_client,
            auth_enabled,
        })
    }

    /// Send a request with authorization
    async fn send_with_auth(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        // If auth is disabled, just pass through
        if !self.auth_enabled {
            return self.inner.send_request(request).await;
        }

        // Get or refresh token
        let token = self.auth_client.get_token().await?;

        // Add authorization header
        crate::auth::client::add_auth_header(&mut self.inner.headers, &token);

        // Try sending the request
        match self.inner.send_request(request.clone()).await {
            Ok(response) => Ok(response),
            Err(McpError::Http(msg)) if msg.contains("401") => {
                // Token might be expired, try refreshing
                let fresh_token = self.auth_client.token_manager().refresh_token().await?;

                // Update header with fresh token
                crate::auth::client::add_auth_header(&mut self.inner.headers, &fresh_token);

                // Retry the request
                self.inner.send_request(request).await
            }
            Err(e) => Err(e),
        }
    }

    /// Handle initial 401 response (no token yet)
    pub async fn handle_unauthorized(&self, www_authenticate: &str) -> McpResult<String> {
        self.auth_client.handle_unauthorized(www_authenticate).await
    }

    /// Handle OAuth callback
    pub async fn handle_callback(&self, callback_url: &str) -> McpResult<String> {
        self.auth_client.handle_callback(callback_url).await
    }

    /// Check if authenticated
    pub async fn is_authenticated(&self) -> bool {
        if !self.auth_enabled {
            return true; // Consider non-auth as authenticated
        }
        self.auth_client.is_authenticated().await
    }

    /// Get the authorization URL to start OAuth flow
    pub async fn get_authorization_url(&self) -> McpResult<String> {
        if !self.auth_enabled {
            return Err(McpError::Auth("Authorization is not enabled".to_string()));
        }

        // This requires auth server metadata to be already discovered
        // You might need to trigger discovery first
        let context = self.auth_client.token_manager().get_context().await;

        if let Some(auth_metadata) = context.auth_server_metadata {
            self.auth_client
                .start_authorization_flow(&auth_metadata)
                .await
        } else {
            Err(McpError::Auth(
                "Authorization server not discovered yet".to_string(),
            ))
        }
    }

    /// Logout and clear tokens
    pub async fn logout(&self) {
        self.auth_client.logout().await;
    }
}

#[async_trait]
impl Transport for AuthorizedHttpTransport {
    async fn send_request(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        self.send_with_auth(request).await
    }

    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        if self.auth_enabled {
            // Get token and add to headers
            let token = self.auth_client.get_token().await?;
            crate::auth::client::add_auth_header(&mut self.inner.headers, &token);
        }

        self.inner.send_notification(notification).await
    }

    async fn receive_notification(&mut self) -> McpResult<Option<JsonRpcNotification>> {
        self.inner.receive_notification().await
    }

    async fn close(&mut self) -> McpResult<()> {
        self.inner.close().await
    }
}

/// Builder for authorized HTTP transport
pub struct AuthorizedHttpTransportBuilder {
    base_url: String,
    sse_url: Option<String>,
    auth_config: AuthConfig,
}

impl AuthorizedHttpTransportBuilder {
    /// Create a new builder
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            sse_url: None,
            auth_config: AuthConfig::default(),
        }
    }

    /// Set SSE URL for notifications
    pub fn with_sse(mut self, url: String) -> Self {
        self.sse_url = Some(url);
        self
    }

    /// Enable authorization
    pub fn with_auth(mut self, enabled: bool) -> Self {
        self.auth_config.enabled = enabled;
        self
    }

    /// Set client credentials
    pub fn with_client_credentials(
        mut self,
        client_id: String,
        client_secret: Option<String>,
    ) -> Self {
        self.auth_config.client_id = Some(client_id);
        self.auth_config.client_secret = client_secret;
        self
    }

    /// Set redirect URI
    pub fn with_redirect_uri(mut self, uri: String) -> Self {
        self.auth_config.redirect_uri = uri;
        self
    }

    /// Set scopes
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.auth_config.scopes = scopes;
        self
    }

    /// Enable dynamic registration
    pub fn with_dynamic_registration(mut self, enabled: bool) -> Self {
        self.auth_config.enable_dynamic_registration = enabled;
        self
    }

    /// Build the transport
    pub async fn build(self) -> McpResult<AuthorizedHttpTransport> {
        AuthorizedHttpTransport::new(self.base_url, self.sse_url, self.auth_config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authorized_transport_creation() {
        let transport = AuthorizedHttpTransportBuilder::new("https://mcp.example.com".to_string())
            .with_auth(false) // Disable auth for testing
            .build()
            .await;

        assert!(transport.is_ok());
        let transport = transport.unwrap();
        assert!(transport.is_authenticated().await);
    }

    #[tokio::test]
    async fn test_auth_disabled_passthrough() {
        let transport = AuthorizedHttpTransportBuilder::new("https://mcp.example.com".to_string())
            .with_auth(false)
            .build()
            .await
            .unwrap();

        // Should always be authenticated when auth is disabled
        assert!(transport.is_authenticated().await);

        // Logout should work without error
        transport.logout().await;
    }
}
