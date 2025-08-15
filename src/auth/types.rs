// ! OAuth 2.1 Types and Data Structures
// !
// ! Module contains the core types used in the OAuth 2.1 authorization flow
// ! for MCP, including metadata structures, token types, and discovery responses.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// OAuth 2.0 Protected Resource Metadata (RFC 9728)
// ============================================================================

/// Protected Resource Metadata as defined in RFC 9728
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedResourceMetadata {
    /// The resource indicator (standard URI of the MCP server)
    pub resource: String,

    /// Array of authorization server issuer identifiers
    pub authorization_servers: Vec<String>,

    /// Array of OAuth 2.0 bearer token resource access authentication methods
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_methods_supported: Option<Vec<String>>,

    /// Array of resource-specific scopes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,

    /// Additional metadata
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

// ============================================================================
// OAuth 2.0 Authorization Server Metadata (RFC 8414)
// ============================================================================

/// Authorization Server Metadata as defined in RFC 8414
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationServerMetadata {
    /// The authorization server's issuer identifier
    pub issuer: String,

    /// URL of the authorization endpoint
    pub authorization_endpoint: String,

    /// URL of the token endpoint
    pub token_endpoint: String,

    /// URL of the registration endpoint for dynamic client registration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_endpoint: Option<String>,

    /// JSON array containing a list of scopes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,

    /// JSON array containing a list of response types
    pub response_types_supported: Vec<String>,

    /// JSON array containing a list of response modes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modes_supported: Option<Vec<String>>,

    /// JSON array containing a list of grant types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_types_supported: Option<Vec<String>>,

    /// JSON array containing a list of token endpoint authentication methods
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_methods_supported: Option<Vec<String>>,

    /// JSON array containing a list of PKCE code challenge methods
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challenge_methods_supported: Option<Vec<String>>,

    /// URL of the revocation endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint: Option<String>,

    /// URL of the introspection endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introspection_endpoint: Option<String>,

    /// Additional metadata fields
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

// ============================================================================
// OpenID Connect Discovery Metadata
// ============================================================================

/// OpenID Connect Provider Metadata (subset relevant for MCP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenIDProviderMetadata {
    /// The issuer identifier
    pub issuer: String,

    /// URL of the authorization endpoint
    pub authorization_endpoint: String,

    /// URL of the token endpoint
    pub token_endpoint: String,

    /// URL of the userinfo endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userinfo_endpoint: Option<String>,

    /// URL of the JWKS endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,

    /// URL of the registration endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_endpoint: Option<String>,

    /// Supported scopes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,

    /// Supported response types
    pub response_types_supported: Vec<String>,

    /// PKCE code challenge methods (extension commonly supported)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challenge_methods_supported: Option<Vec<String>>,

    /// Additional fields
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Dynamic Client Registration (RFC 7591)
// ============================================================================

/// Client Registration Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRegistrationRequest {
    /// Array of redirect URIs
    pub redirect_uris: Vec<String>,

    /// Human-readable name of the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,

    /// URL of the client's homepage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_uri: Option<String>,

    /// URL of the client's logo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,

    /// Array of OAuth 2.0 grant types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_types: Option<Vec<String>>,

    /// Array of OAuth 2.0 response types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_types: Option<Vec<String>>,

    /// Requested authentication method for the token endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_method: Option<String>,

    /// Space-separated list of scope values
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Software identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_id: Option<String>,

    /// Software version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_version: Option<String>,
}

/// Client Registration Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRegistrationResponse {
    /// Unique client identifier
    pub client_id: String,

    /// Client secret (for confidential clients)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    /// Time at which the client secret expires (0 = no expiration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret_expires_at: Option<u64>,

    /// Client registration access token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_access_token: Option<String>,

    /// Client configuration endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_client_uri: Option<String>,

    /// All registered redirect URIs
    pub redirect_uris: Vec<String>,

    /// All other registration parameters
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Token Types
// ============================================================================

/// OAuth 2.0 Token Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRequest {
    /// Grant type (e.g., "authorization_code", "refresh_token")
    pub grant_type: String,

    /// Authorization code (for authorization_code grant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Redirect URI (must match the one used in authorization request)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,

    /// PKCE code verifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_verifier: Option<String>,

    /// Refresh token (for refresh_token grant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// Resource indicator (RFC 8707)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,

    /// Client ID (for public clients)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// Client secret (for confidential clients)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    /// Scope (for refresh_token grant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// OAuth 2.0 Token Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// The access token
    pub access_token: String,

    /// The type of token (typically "Bearer")
    pub token_type: String,

    /// The lifetime in seconds of the access token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,

    /// The refresh token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// The scope of the access token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Additional parameters
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// OAuth 2.0 Error Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Error {
    /// Error code
    pub error: String,

    /// Human-readable error description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,

    /// URI for more information about the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_uri: Option<String>,
}

// ============================================================================
// WWW-Authenticate Header Components
// ============================================================================

/// WWW-Authenticate challenge parameters
#[derive(Debug, Clone)]
pub struct AuthChallenge {
    /// Authentication scheme (e.g., "Bearer")
    pub scheme: String,

    /// Realm parameter
    pub realm: Option<String>,

    /// Error code
    pub error: Option<String>,

    /// Error description
    pub error_description: Option<String>,

    /// Resource metadata URL
    pub resource_metadata: Option<String>,

    /// Additional parameters
    pub additional: HashMap<String, String>,
}

impl AuthChallenge {
    /// Parse WWW-Authenticate header
    pub fn parse(header_value: &str) -> Option<Self> {
        let parts: Vec<&str> = header_value.splitn(2, ' ').collect();
        if parts.is_empty() {
            return None;
        }

        let scheme = parts[0].to_string();
        let mut challenge = AuthChallenge {
            scheme,
            realm: None,
            error: None,
            error_description: None,
            resource_metadata: None,
            additional: HashMap::new(),
        };

        if parts.len() > 1 {
            // Parse parameters
            let params = parts[1];
            for param in params.split(',') {
                let param = param.trim();
                if let Some(eq_pos) = param.find('=') {
                    let key = param[..eq_pos].trim();
                    let value = param[eq_pos + 1..].trim().trim_matches('"');

                    match key {
                        "realm" => challenge.realm = Some(value.to_string()),
                        "error" => challenge.error = Some(value.to_string()),
                        "error_description" => {
                            challenge.error_description = Some(value.to_string())
                        }
                        "resource_metadata" => {
                            challenge.resource_metadata = Some(value.to_string())
                        }
                        _ => {
                            challenge
                                .additional
                                .insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }

        Some(challenge)
    }

    /// Format as WWW-Authenticate header value
    pub fn format(&self) -> String {
        let mut result = self.scheme.clone();
        let mut params = Vec::new();

        if let Some(realm) = &self.realm {
            params.push(format!(r#"realm="{realm}""#));
        }
        if let Some(error) = &self.error {
            params.push(format!(r#"error="{error}""#));
        }
        if let Some(desc) = &self.error_description {
            params.push(format!(r#"error_description="{desc}""#));
        }
        if let Some(metadata) = &self.resource_metadata {
            params.push(format!(r#"resource_metadata="{metadata}""#));
        }

        for (key, value) in &self.additional {
            params.push(format!(r#"{key}="{value}""#));
        }

        if !params.is_empty() {
            result.push(' ');
            result.push_str(&params.join(", "));
        }

        result
    }
}

// ============================================================================
// Authorization Context
// ============================================================================

/// Authorization context for a client session
#[derive(Debug, Clone)]
pub struct AuthorizationContext {
    /// Current access token
    pub access_token: Option<String>,

    /// Current refresh token
    pub refresh_token: Option<String>,

    /// Token expiration time (Unix timestamp)
    pub expires_at: Option<u64>,

    /// Authorization server metadata
    pub auth_server_metadata: Option<AuthorizationServerMetadata>,

    /// Resource metadata
    pub resource_metadata: Option<ProtectedResourceMetadata>,

    /// Client registration details
    pub client_registration: Option<ClientRegistrationResponse>,

    /// PKCE verifier for current flow
    pub pkce_verifier: Option<String>,

    /// State parameter for current flow
    pub state: Option<String>,

    /// Resource indicator (standard URI of MCP server)
    pub resource: String,
}

impl AuthorizationContext {
    /// Create a new authorization context
    pub fn new(resource: String) -> Self {
        Self {
            access_token: None,
            refresh_token: None,
            expires_at: None,
            auth_server_metadata: None,
            resource_metadata: None,
            client_registration: None,
            pkce_verifier: None,
            state: None,
            resource,
        }
    }

    /// Check if the access token is expired
    pub fn is_token_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now >= expires_at
        } else {
            false
        }
    }

    /// Check if we have a valid access token
    pub fn has_valid_token(&self) -> bool {
        self.access_token.is_some() && !self.is_token_expired()
    }
}
