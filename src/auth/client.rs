// ! OAuth 2.1 Authorization Client
// !
// ! Module provides the main authorization client for MCP,
// ! handling the full OAuth 2.1 flow including discovery, registration,
// ! and token management

use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::auth::{
    AuthConfig,
    discovery::{DiscoveryClient, validate_auth_server_for_mcp},
    errors::AuthError,
    pkce::{PkceParams, select_challenge_method},
    token::{TokenManager, build_authorization_url, parse_callback_url},
    types::*,
};
use crate::core::error::{McpError, McpResult};

/// OAuth 2.1 Authorization Client for MCP
pub struct AuthorizationClient {
    config: AuthConfig,
    http_client: Client,
    token_manager: TokenManager,
    discovery_client: DiscoveryClient,
    state: Arc<RwLock<AuthState>>,
}

/// Internal state for authorization flow
#[derive(Debug, Clone)]
struct AuthState {
    /// Current PKCE parameters
    pkce: Option<PkceParams>,
    /// Current state parameter
    state: Option<String>,
    /// Resource metadata
    resource_metadata: Option<ProtectedResourceMetadata>,
    /// Authorization server metadata
    auth_server_metadata: Option<AuthorizationServerMetadata>,
    /// Client registration
    client_registration: Option<ClientRegistrationResponse>,
}

impl AuthorizationClient {
    /// Create a new authorization client
    pub fn new(config: AuthConfig, resource_url: String) -> Self {
        let http_client = Client::new();
        Self {
            config,
            http_client: http_client.clone(),
            token_manager: TokenManager::new(resource_url),
            discovery_client: DiscoveryClient::with_client(http_client),
            state: Arc::new(RwLock::new(AuthState {
                pkce: None,
                state: None,
                resource_metadata: None,
                auth_server_metadata: None,
                client_registration: None,
            })),
        }
    }

    /// Handle 401 Unauthorized response and initiate authorization
    pub async fn handle_unauthorized(&self, www_authenticate: &str) -> McpResult<String> {
        // Parse WWW-Authenticate header
        let metadata_url = self
            .discovery_client
            .parse_www_authenticate(www_authenticate)?;

        // Discover resource metadata
        let resource_metadata = self
            .discovery_client
            .discover_from_resource(&metadata_url)
            .await?;

        // Select authorization server (use first for now)
        let auth_server_url = resource_metadata
            .authorization_servers
            .first()
            .ok_or_else(|| McpError::Auth("No authorization servers available".to_string()))?
            .clone();

        // Discover authorization server metadata
        let auth_metadata = self
            .discovery_client
            .discover_auth_server(&auth_server_url)
            .await?;

        // Validate for MCP requirements
        validate_auth_server_for_mcp(&auth_metadata)?;

        // Store metadata
        {
            let mut state = self.state.write().await;
            state.resource_metadata = Some(resource_metadata.clone());
            state.auth_server_metadata = Some(auth_metadata.clone());
        }

        // Update token manager context
        self.token_manager
            .update_context(|ctx| {
                ctx.resource_metadata = Some(resource_metadata);
                ctx.auth_server_metadata = Some(auth_metadata.clone());
            })
            .await?;

        // Perform dynamic registration if needed
        if self.config.client_id.is_none() && self.config.enable_dynamic_registration {
            self.register_client(&auth_metadata).await?;
        }

        // Start authorization flow
        self.start_authorization_flow(&auth_metadata).await
    }

    /// Perform dynamic client registration
    async fn register_client(&self, auth_metadata: &AuthorizationServerMetadata) -> McpResult<()> {
        let registration_endpoint =
            auth_metadata
                .registration_endpoint
                .as_ref()
                .ok_or_else(|| {
                    McpError::Auth(
                        "Authorization server does not support dynamic registration".to_string(),
                    )
                })?;

        let request = ClientRegistrationRequest {
            redirect_uris: vec![self.config.redirect_uri.clone()],
            client_name: Some("MCP Client".to_string()),
            grant_types: Some(vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ]),
            response_types: Some(vec!["code".to_string()]),
            token_endpoint_auth_method: Some("client_secret_basic".to_string()),
            scope: if self.config.scopes.is_empty() {
                None
            } else {
                Some(self.config.scopes.join(" "))
            },
            software_id: Some("mcp-rust-sdk".to_string()),
            software_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            client_uri: None,
            logo_uri: None,
        };

        let response = self
            .http_client
            .post(registration_endpoint)
            .json(&request)
            .send()
            .await
            .map_err(|e| McpError::Auth(format!("Registration request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(McpError::Auth(format!(
                "Client registration failed: {}",
                error_text
            )));
        }

        let registration: ClientRegistrationResponse = response
            .json()
            .await
            .map_err(|e| McpError::Auth(format!("Invalid registration response: {}", e)))?;

        // Store registration
        {
            let mut state = self.state.write().await;
            state.client_registration = Some(registration.clone());
        }

        // Update token manager
        self.token_manager
            .update_context(|ctx| {
                ctx.client_registration = Some(registration);
            })
            .await?;

        Ok(())
    }

    /// Start the authorization flow
    pub async fn start_authorization_flow(
        &self,
        auth_metadata: &AuthorizationServerMetadata,
    ) -> McpResult<String> {
        // Select PKCE method
        let pkce_method = select_challenge_method(auth_metadata)?;
        let pkce = PkceParams::with_method(pkce_method);

        // Generate state
        let state = self.config.generate_state();

        // Store PKCE and state
        {
            let mut auth_state = self.state.write().await;
            auth_state.pkce = Some(pkce.clone());
            auth_state.state = Some(state.clone());
        }

        // Get client ID
        let client_id = if let Some(ref id) = self.config.client_id {
            id.clone()
        } else {
            let auth_state = self.state.read().await;
            auth_state
                .client_registration
                .as_ref()
                .map(|r| r.client_id.clone())
                .ok_or_else(|| McpError::Auth("No client ID available".to_string()))?
        };

        // Get resource URL
        let resource = self.token_manager.get_context().await.resource;

        // Build authorization URL
        let auth_url = build_authorization_url(
            &auth_metadata.authorization_endpoint,
            &client_id,
            &self.config.redirect_uri,
            &state,
            &pkce.challenge,
            pkce.method.as_str(),
            &resource,
            &self.config.scopes,
        )?;

        Ok(auth_url)
    }

    /// Handle authorization callback
    pub async fn handle_callback(&self, callback_url: &str) -> McpResult<String> {
        let params = parse_callback_url(callback_url)?;

        // Verify state
        let stored_state = {
            let auth_state = self.state.read().await;
            auth_state.state.clone()
        };

        if let Some(expected_state) = stored_state {
            if params.state.as_ref() != Some(&expected_state) {
                return Err(AuthError::StateMismatch.into());
            }
        }

        // Get PKCE verifier
        let pkce_verifier = {
            let auth_state = self.state.read().await;
            auth_state.pkce.as_ref().map(|p| p.verifier.clone())
        };

        // Exchange code for tokens
        let token_response = self
            .token_manager
            .exchange_code(params.code, self.config.redirect_uri.clone(), pkce_verifier)
            .await?;

        // Clear temporary state
        {
            let mut auth_state = self.state.write().await;
            auth_state.pkce = None;
            auth_state.state = None;
        }

        Ok(token_response.access_token)
    }

    /// Get current access token (refreshing if needed)
    pub async fn get_token(&self) -> McpResult<String> {
        self.token_manager.get_or_refresh_token().await
    }

    /// Clear all tokens and state
    pub async fn logout(&self) {
        self.token_manager.clear_tokens().await;

        let mut state = self.state.write().await;
        *state = AuthState {
            pkce: None,
            state: None,
            resource_metadata: None,
            auth_server_metadata: None,
            client_registration: None,
        };
    }

    /// Get the token manager
    pub fn token_manager(&self) -> &TokenManager {
        &self.token_manager
    }

    /// Check if we have a valid token
    pub async fn is_authenticated(&self) -> bool {
        self.token_manager.get_valid_token().await.is_some()
    }
}

/// Helper to add authorization header to HTTP requests
pub fn add_auth_header(headers: &mut reqwest::header::HeaderMap, token: &str) {
    use reqwest::header::{AUTHORIZATION, HeaderValue};

    let value = format!("Bearer {}", token);
    if let Ok(header_value) = HeaderValue::from_str(&value) {
        headers.insert(AUTHORIZATION, header_value);
    }
}

/// Extract bearer token from Authorization header
pub fn extract_bearer_token(auth_header: &str) -> Option<String> {
    if auth_header.starts_with("Bearer ") {
        Some(auth_header[7..].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_auth_header() {
        let mut headers = reqwest::header::HeaderMap::new();
        add_auth_header(&mut headers, "test_token");

        let auth = headers.get(reqwest::header::AUTHORIZATION).unwrap();
        assert_eq!(auth.to_str().unwrap(), "Bearer test_token");
    }

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(
            extract_bearer_token("Bearer abc123"),
            Some("abc123".to_string())
        );

        assert_eq!(extract_bearer_token("Basic abc123"), None);

        assert_eq!(extract_bearer_token("Invalid"), None);
    }

    #[tokio::test]
    async fn test_authorization_client_creation() {
        let config = AuthConfig::new()
            .with_auth(true)
            .with_redirect_uri("http://localhost:8080/callback".to_string())
            .with_scopes(vec!["read".to_string()]);

        let client = AuthorizationClient::new(config, "https://mcp.example.com".to_string());

        // Initially not authenticated
        assert!(!client.is_authenticated().await);
    }
}
