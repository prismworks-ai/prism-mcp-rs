// ! Token Management and Refresh
// !
// ! Module handles access token management, including automatic refresh
// ! when tokens expire

use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

use crate::auth::errors::AuthError;
use crate::auth::types::*;
use crate::core::error::{McpError, McpResult};

/// Token manager for handling access and refresh tokens
#[derive(Debug, Clone)]
pub struct TokenManager {
    context: Arc<RwLock<AuthorizationContext>>,
    http_client: Client,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new(resource: String) -> Self {
        Self {
            context: Arc::new(RwLock::new(AuthorizationContext::new(resource))),
            http_client: Client::new(),
        }
    }

    /// Create with existing context
    pub fn with_context(context: AuthorizationContext) -> Self {
        Self {
            context: Arc::new(RwLock::new(context)),
            http_client: Client::new(),
        }
    }

    /// Get current access token if valid
    pub async fn get_valid_token(&self) -> Option<String> {
        let ctx = self.context.read().await;
        if ctx.has_valid_token() {
            ctx.access_token.clone()
        } else {
            None
        }
    }

    /// Set tokens from a token response
    pub async fn set_tokens(&self, response: TokenResponse) -> McpResult<()> {
        let mut ctx = self.context.write().await;

        ctx.access_token = Some(response.access_token);
        ctx.refresh_token = response.refresh_token;

        // Calculate expiration time
        if let Some(expires_in) = response.expires_in {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            ctx.expires_at = Some(now + expires_in);
        }

        Ok(())
    }

    /// Refresh the access token using the refresh token
    pub async fn refresh_token(&self) -> McpResult<String> {
        let (refresh_token, token_endpoint, client_id, client_secret, resource) = {
            let ctx = self.context.read().await;

            let refresh_token = ctx
                .refresh_token
                .as_ref()
                .ok_or_else(|| McpError::Auth("No refresh token available".to_string()))?
                .clone();

            let token_endpoint = ctx
                .auth_server_metadata
                .as_ref()
                .ok_or_else(|| McpError::Auth("No authorization server metadata".to_string()))?
                .token_endpoint
                .clone();

            let client_id = ctx
                .client_registration
                .as_ref()
                .map(|r| r.client_id.clone());

            let client_secret = ctx
                .client_registration
                .as_ref()
                .and_then(|r| r.client_secret.clone());

            (
                refresh_token,
                token_endpoint,
                client_id,
                client_secret,
                ctx.resource.clone(),
            )
        };

        // Build refresh token request
        let mut params = vec![
            ("grant_type".to_string(), "refresh_token".to_string()),
            ("refresh_token".to_string(), refresh_token),
            ("resource".to_string(), resource),
        ];

        if let Some(client_id) = client_id {
            params.push(("client_id".to_string(), client_id));
        }

        if let Some(client_secret) = client_secret {
            params.push(("client_secret".to_string(), client_secret));
        }

        // Send token request
        let response = self
            .http_client
            .post(&token_endpoint)
            .form(&params)
            .send()
            .await
            .map_err(|e| McpError::Auth(format!("Failed to refresh token: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            if let Ok(oauth_error) = serde_json::from_str::<OAuth2Error>(&error_text) {
                return Err(AuthError::OAuthError {
                    error: oauth_error.error,
                    description: oauth_error.error_description,
                    uri: oauth_error.error_uri,
                }
                .into());
            }
            return Err(McpError::Auth(format!(
                "Token refresh failed: {}",
                error_text
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| McpError::Auth(format!("Invalid token response: {}", e)))?;

        // Update stored tokens
        self.set_tokens(token_response.clone()).await?;

        Ok(token_response.access_token)
    }

    /// Get or refresh access token
    pub async fn get_or_refresh_token(&self) -> McpResult<String> {
        // First check if we have a valid token
        if let Some(token) = self.get_valid_token().await {
            return Ok(token);
        }

        // Try to refresh
        self.refresh_token().await
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(
        &self,
        code: String,
        redirect_uri: String,
        code_verifier: Option<String>,
    ) -> McpResult<TokenResponse> {
        let (token_endpoint, client_id, client_secret, resource) = {
            let ctx = self.context.read().await;

            let token_endpoint = ctx
                .auth_server_metadata
                .as_ref()
                .ok_or_else(|| McpError::Auth("No authorization server metadata".to_string()))?
                .token_endpoint
                .clone();

            let client_id = ctx
                .client_registration
                .as_ref()
                .map(|r| r.client_id.clone());

            let client_secret = ctx
                .client_registration
                .as_ref()
                .and_then(|r| r.client_secret.clone());

            (
                token_endpoint,
                client_id,
                client_secret,
                ctx.resource.clone(),
            )
        };

        // Build token request
        let mut params = vec![
            ("grant_type".to_string(), "authorization_code".to_string()),
            ("code".to_string(), code),
            ("redirect_uri".to_string(), redirect_uri),
            ("resource".to_string(), resource),
        ];

        if let Some(verifier) = code_verifier {
            params.push(("code_verifier".to_string(), verifier));
        }

        if let Some(client_id) = client_id {
            params.push(("client_id".to_string(), client_id));
        }

        if let Some(client_secret) = client_secret {
            params.push(("client_secret".to_string(), client_secret));
        }

        // Send token request
        let response = self
            .http_client
            .post(&token_endpoint)
            .form(&params)
            .send()
            .await
            .map_err(|e| McpError::Auth(format!("Failed to exchange code: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            if let Ok(oauth_error) = serde_json::from_str::<OAuth2Error>(&error_text) {
                return Err(AuthError::OAuthError {
                    error: oauth_error.error,
                    description: oauth_error.error_description,
                    uri: oauth_error.error_uri,
                }
                .into());
            }
            return Err(McpError::Auth(format!(
                "Code exchange failed: {}",
                error_text
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| McpError::Auth(format!("Invalid token response: {}", e)))?;

        // Store tokens
        self.set_tokens(token_response.clone()).await?;

        Ok(token_response)
    }

    /// Clear all tokens
    pub async fn clear_tokens(&self) {
        let mut ctx = self.context.write().await;
        ctx.access_token = None;
        ctx.refresh_token = None;
        ctx.expires_at = None;
    }

    /// Get the authorization context
    pub async fn get_context(&self) -> AuthorizationContext {
        self.context.read().await.clone()
    }

    /// Update the authorization context
    pub async fn update_context<F>(&self, f: F) -> McpResult<()>
    where
        F: FnOnce(&mut AuthorizationContext),
    {
        let mut ctx = self.context.write().await;
        f(&mut ctx);
        Ok(())
    }
}

/// Build authorization URL for OAuth flow
pub fn build_authorization_url(
    auth_endpoint: &str,
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    code_challenge: &str,
    code_challenge_method: &str,
    resource: &str,
    scopes: &[String],
) -> McpResult<String> {
    let mut url = Url::parse(auth_endpoint)
        .map_err(|e| McpError::Auth(format!("Invalid authorization endpoint: {}", e)))?;

    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", code_challenge_method)
        .append_pair("resource", resource);

    if !scopes.is_empty() {
        url.query_pairs_mut()
            .append_pair("scope", &scopes.join(" "));
    }

    Ok(url.to_string())
}

/// Parse authorization callback URL
pub fn parse_callback_url(callback_url: &str) -> McpResult<CallbackParams> {
    let url = Url::parse(callback_url)
        .map_err(|e| McpError::Auth(format!("Invalid callback URL: {}", e)))?;

    let params: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    // Check for error response
    if let Some(error) = crate::auth::errors::parse_oauth_error(&params) {
        return Err(error.into());
    }

    // Extract code and state
    let code = params
        .iter()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.clone())
        .ok_or_else(|| McpError::Auth("No authorization code in callback".to_string()))?;

    let state = params
        .iter()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.clone());

    Ok(CallbackParams { code, state })
}

/// Parameters extracted from OAuth callback
#[derive(Debug, Clone)]
pub struct CallbackParams {
    /// Authorization code
    pub code: String,
    /// State parameter (for CSRF protection)
    pub state: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_authorization_url() {
        let url = build_authorization_url(
            "https://auth.example.com/authorize",
            "client123",
            "http://localhost:8080/callback",
            "random_state",
            "challenge123",
            "S256",
            "https://mcp.example.com",
            &["read".to_string(), "write".to_string()],
        )
        .unwrap();

        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=client123"));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("state=random_state"));
        assert!(url.contains("code_challenge=challenge123"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("resource="));
        assert!(url.contains("scope=read+write"));
    }

    #[test]
    fn test_parse_callback_success() {
        let callback = "http://localhost:8080/callback?code=auth123&state=random_state";
        let params = parse_callback_url(callback).unwrap();

        assert_eq!(params.code, "auth123");
        assert_eq!(params.state, Some("random_state".to_string()));
    }

    #[test]
    fn test_parse_callback_error() {
        let callback = "http://localhost:8080/callback?error=access_denied&error_description=User+denied+access";
        let result = parse_callback_url(callback);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("access_denied"));
    }

    #[tokio::test]
    async fn test_token_manager() {
        let manager = TokenManager::new("https://mcp.example.com".to_string());

        // Initially no token
        assert!(manager.get_valid_token().await.is_none());

        // Set a token
        let response = TokenResponse {
            access_token: "token123".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: Some("refresh123".to_string()),
            scope: None,
            additional: Default::default(),
        };

        manager.set_tokens(response).await.unwrap();

        // Now should have a valid token
        assert_eq!(
            manager.get_valid_token().await,
            Some("token123".to_string())
        );
    }
}
