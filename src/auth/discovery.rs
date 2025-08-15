// ! Authorization Server Discovery
// !
// ! Module implements discovery mechanisms for OAuth 2.0 authorization servers
// ! including Protected Resource Metadata (RFC 9728), Authorization Server Metadata
// ! (RFC 8414), and OpenID Connect Discovery.

use reqwest::Client;
use url::Url;

use crate::auth::types::*;
use crate::core::error::{McpError, McpResult};

/// Discovery client for authorization server metadata
pub struct DiscoveryClient {
    http_client: Client,
}

impl DiscoveryClient {
    /// Create a new discovery client
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    /// Create with custom HTTP client
    pub fn with_client(client: Client) -> Self {
        Self {
            http_client: client,
        }
    }

    /// Discover authorization server from Protected Resource Metadata
    ///
    /// This follows RFC 9728 to discover the authorization server(s)
    /// for a protected resource (MCP server).
    pub async fn discover_from_resource(
        &self,
        _resource_url: &str,
    ) -> McpResult<ProtectedResourceMetadata> {
        // Try to fetch protected resource metadata
        let metadata_url = self.build_resource_metadata_url(_resource_url)?;

        let response = self
            .http_client
            .get(&metadata_url)
            .send()
            .await
            .map_err(|e| McpError::Auth(format!("Failed to fetch resource metadata: {e}")))?;

        if !response.status().is_success() {
            return Err(McpError::Auth(format!(
                "Failed to fetch resource metadata: HTTP {}",
                response.status()
            )));
        }

        let metadata: ProtectedResourceMetadata = response
            .json()
            .await
            .map_err(|e| McpError::Auth(format!("Invalid resource metadata: {e}")))?;

        // Validate metadata
        if metadata.authorization_servers.is_empty() {
            return Err(McpError::Auth(
                "Resource metadata does not specify any authorization servers".to_string(),
            ));
        }

        Ok(metadata)
    }

    /// Parse WWW-Authenticate header and extract resource metadata URL
    pub fn parse_www_authenticate(&self, header_value: &str) -> McpResult<String> {
        let challenge = AuthChallenge::parse(header_value)
            .ok_or_else(|| McpError::Auth("Invalid WWW-Authenticate header".to_string()))?;

        challenge.resource_metadata.ok_or_else(|| {
            McpError::Auth("WWW-Authenticate header does not contain resource_metadata".to_string())
        })
    }

    /// Discover authorization server metadata
    ///
    /// This tries multiple discovery endpoints in priority order:
    /// 1. OAuth 2.0 Authorization Server Metadata (RFC 8414)
    /// 2. OpenID Connect Discovery
    pub async fn discover_auth_server(
        &self,
        issuer_url: &str,
    ) -> McpResult<AuthorizationServerMetadata> {
        let issuer = Url::parse(issuer_url)
            .map_err(|e| McpError::Auth(format!("Invalid issuer URL: {e}")))?;

        // Build discovery URLs based on issuer format
        let discovery_urls = self.build_discovery_urls(&issuer)?;

        let mut last_error = None;

        // Try each discovery URL in order
        for url in discovery_urls {
            match self.fetch_auth_server_metadata(&url).await {
                Ok(metadata) => {
                    // Validate issuer matches
                    if metadata.issuer != issuer_url {
                        continue; // Try next URL
                    }
                    return Ok(metadata);
                }
                Err(e) => {
                    last_error = Some(e);
                    continue; // Try next URL
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            McpError::Auth("Failed to discover authorization server metadata".to_string())
        }))
    }

    /// Build Protected Resource Metadata URL
    fn build_resource_metadata_url(&self, resource_url: &str) -> McpResult<String> {
        let base = Url::parse(resource_url)
            .map_err(|e| McpError::Auth(format!("Invalid resource URL: {e}")))?;

        // RFC 9728: /.well-known/oauth-protected-resource
        let metadata_url = base
            .join("/.well-known/oauth-protected-resource")
            .map_err(|e| McpError::Auth(format!("Failed to build metadata URL: {e}")))?;

        Ok(metadata_url.to_string())
    }

    /// Build discovery URLs for authorization server
    fn build_discovery_urls(&self, issuer: &Url) -> McpResult<Vec<String>> {
        let mut urls = Vec::new();

        // Get the path component (excluding leading slash)
        let path = issuer.path();
        let has_path = path != "/" && !path.is_empty();

        if has_path {
            // For issuer URLs with path components
            let path_component = path.trim_start_matches('/');

            // 1. OAuth 2.0 with path insertion
            let oauth_url = format!(
                "{}://{}/{}/.well-known/oauth-authorization-server/{}",
                issuer.scheme(),
                issuer.host_str().unwrap_or(""),
                issuer.port().map(|p| format!(":{p}")).unwrap_or_default(),
                path_component
            );
            urls.push(oauth_url);

            // 2. OpenID Connect with path insertion
            let oidc_insert_url = format!(
                "{}://{}/{}/.well-known/openid-configuration/{}",
                issuer.scheme(),
                issuer.host_str().unwrap_or(""),
                issuer.port().map(|p| format!(":{p}")).unwrap_or_default(),
                path_component
            );
            urls.push(oidc_insert_url);

            // 3. OpenID Connect with path appending
            let oidc_append_url = format!(
                "{}/.well-known/openid-configuration",
                issuer.as_str().trim_end_matches('/')
            );
            urls.push(oidc_append_url);
        } else {
            // For issuer URLs without path components

            // 1. OAuth 2.0 Authorization Server Metadata
            let oauth_url = format!(
                "{}/.well-known/oauth-authorization-server",
                issuer.as_str().trim_end_matches('/')
            );
            urls.push(oauth_url);

            // 2. OpenID Connect Discovery
            let oidc_url = format!(
                "{}/.well-known/openid-configuration",
                issuer.as_str().trim_end_matches('/')
            );
            urls.push(oidc_url);
        }

        Ok(urls)
    }

    /// Fetch authorization server metadata from a discovery URL
    async fn fetch_auth_server_metadata(
        &self,
        url: &str,
    ) -> McpResult<AuthorizationServerMetadata> {
        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| McpError::Auth(format!("Failed to fetch metadata: {e}")))?;

        if !response.status().is_success() {
            return Err(McpError::Auth(format!(
                "Failed to fetch metadata from {}: HTTP {}",
                url,
                response.status()
            )));
        }

        // Try to parse as OAuth 2.0 metadata first, then as OpenID Connect
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| McpError::Auth(format!("Invalid metadata JSON: {e}")))?;

        // Convert OpenID metadata to OAuth metadata if needed
        if let Ok(oidc_metadata) = serde_json::from_value::<OpenIDProviderMetadata>(json.clone()) {
            Ok(self.convert_oidc_to_oauth(oidc_metadata))
        } else {
            serde_json::from_value::<AuthorizationServerMetadata>(json)
                .map_err(|e| McpError::Auth(format!("Invalid authorization server metadata: {e}")))
        }
    }

    /// Convert OpenID Connect metadata to OAuth 2.0 metadata
    fn convert_oidc_to_oauth(&self, oidc: OpenIDProviderMetadata) -> AuthorizationServerMetadata {
        AuthorizationServerMetadata {
            issuer: oidc.issuer,
            authorization_endpoint: oidc.authorization_endpoint,
            token_endpoint: oidc.token_endpoint,
            registration_endpoint: oidc.registration_endpoint,
            scopes_supported: oidc.scopes_supported,
            response_types_supported: oidc.response_types_supported,
            response_modes_supported: None,
            grant_types_supported: None,
            token_endpoint_auth_methods_supported: None,
            code_challenge_methods_supported: oidc.code_challenge_methods_supported,
            revocation_endpoint: None,
            introspection_endpoint: None,
            additional: oidc.additional,
        }
    }
}

impl Default for DiscoveryClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Perform full discovery flow from 401 response
pub async fn discover_from_401(
    client: &Client,
    www_authenticate: &str,
    _resource_url: &str,
) -> McpResult<(ProtectedResourceMetadata, AuthorizationServerMetadata)> {
    let discovery = DiscoveryClient::with_client(client.clone());

    // Parse WWW-Authenticate header
    let metadata_url = discovery.parse_www_authenticate(www_authenticate)?;

    // Fetch resource metadata
    let resource_metadata = discovery.discover_from_resource(&metadata_url).await?;

    // Select first authorization server (client should implement selection logic)
    let auth_server_url = resource_metadata
        .authorization_servers
        .first()
        .ok_or_else(|| {
            McpError::Auth("No authorization servers specified in resource metadata".to_string())
        })?
        .clone();

    // Discover authorization server metadata
    let auth_metadata = discovery.discover_auth_server(&auth_server_url).await?;

    Ok((resource_metadata, auth_metadata))
}

/// Check if an authorization server supports required MCP features
pub fn validate_auth_server_for_mcp(metadata: &AuthorizationServerMetadata) -> McpResult<()> {
    // Check for PKCE support (required)
    if metadata.code_challenge_methods_supported.is_none()
        || metadata
            .code_challenge_methods_supported
            .as_ref()
            .unwrap()
            .is_empty()
    {
        return Err(McpError::Auth(
            "Authorization server does not support PKCE (required for MCP)".to_string(),
        ));
    }

    // Check for authorization code flow support
    if !metadata
        .response_types_supported
        .contains(&"code".to_string())
        && !metadata
            .response_types_supported
            .contains(&"code id_token".to_string())
    {
        return Err(McpError::Auth(
            "Authorization server does not support authorization code flow".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_www_authenticate() {
        let header = r#"Bearer realm="example", resource_metadata="https://example.com/.well-known/oauth-protected-resource", error="invalid_token""#;

        let challenge = AuthChallenge::parse(header).unwrap();
        assert_eq!(challenge.scheme, "Bearer");
        assert_eq!(challenge.realm, Some("example".to_string()));
        assert_eq!(
            challenge.resource_metadata,
            Some("https://example.com/.well-known/oauth-protected-resource".to_string())
        );
        assert_eq!(challenge.error, Some("invalid_token".to_string()));
    }

    #[test]
    fn test_build_discovery_urls_no_path() {
        let client = DiscoveryClient::new();
        let issuer = Url::parse("https://auth.example.com").unwrap();

        let urls = client.build_discovery_urls(&issuer).unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(
            urls[0],
            "https://auth.example.com/.well-known/oauth-authorization-server"
        );
        assert_eq!(
            urls[1],
            "https://auth.example.com/.well-known/openid-configuration"
        );
    }

    #[test]
    fn test_build_discovery_urls_with_path() {
        let client = DiscoveryClient::new();
        let issuer = Url::parse("https://auth.example.com/tenant1").unwrap();

        let urls = client.build_discovery_urls(&issuer).unwrap();
        assert_eq!(urls.len(), 3);
        assert!(urls[0].contains("/.well-known/oauth-authorization-server/tenant1"));
        assert!(urls[1].contains("/.well-known/openid-configuration/tenant1"));
        assert!(urls[2].contains("/tenant1/.well-known/openid-configuration"));
    }

    #[test]
    fn test_validate_auth_server() {
        let mut metadata = AuthorizationServerMetadata {
            issuer: "https://auth.example.com".to_string(),
            authorization_endpoint: "https://auth.example.com/authorize".to_string(),
            token_endpoint: "https://auth.example.com/token".to_string(),
            registration_endpoint: None,
            scopes_supported: None,
            response_types_supported: vec!["code".to_string()],
            response_modes_supported: None,
            grant_types_supported: None,
            token_endpoint_auth_methods_supported: None,
            code_challenge_methods_supported: Some(vec!["S256".to_string()]),
            revocation_endpoint: None,
            introspection_endpoint: None,
            additional: Default::default(),
        };

        // Should pass validation
        assert!(validate_auth_server_for_mcp(&metadata).is_ok());

        // Should fail without PKCE
        metadata.code_challenge_methods_supported = None;
        assert!(validate_auth_server_for_mcp(&metadata).is_err());

        // Should fail without code flow
        metadata.code_challenge_methods_supported = Some(vec!["S256".to_string()]);
        metadata.response_types_supported = vec!["token".to_string()];
        assert!(validate_auth_server_for_mcp(&metadata).is_err());
    }
}
