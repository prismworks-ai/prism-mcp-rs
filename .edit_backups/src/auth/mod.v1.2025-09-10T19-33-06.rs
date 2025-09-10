// ! OAuth 2.1 Authorization Support for MCP Protocol
// !
// ! Module implements the authorization flow for HTTP-based MCP transports
// ! as specified in the MCP Authorization specification (draft).
// !
// ! The implementation follows OAuth 2.1, OAuth 2.0 Authorization Server Metadata,
// ! Dynamic Client Registration, and Protected Resource Metadata specifications.

pub mod client;
pub mod discovery;
pub mod errors;
pub mod pkce;
pub mod token;
pub mod types;

pub use client::*;
pub use discovery::*;
pub use errors::*;
pub use pkce::*;
pub use token::*;
pub use types::*;

// Error types are re-exported from submodules

/// Authorization configuration for MCP
#[derive(Clone)]
pub struct AuthConfig {
    /// Enable authorization for this client/server
    pub enabled: bool,

    /// Client ID for OAuth (if pre-registered)
    pub client_id: Option<String>,

    /// Client secret (for confidential clients)
    pub client_secret: Option<String>,

    /// Redirect URI for authorization code flow
    pub redirect_uri: String,

    /// Scopes to request
    pub scopes: Vec<String>,

    /// Enable dynamic client registration
    pub enable_dynamic_registration: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            client_id: None,
            client_secret: None,
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes: vec![],
            enable_dynamic_registration: true,
        }
    }
}

impl AuthConfig {
    /// Create a new authorization configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable authorization
    pub fn with_auth(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set client credentials
    pub fn with_client_credentials(
        mut self,
        client_id: String,
        client_secret: Option<String>,
    ) -> Self {
        self.client_id = Some(client_id);
        self.client_secret = client_secret;
        self
    }

    /// Set redirect URI
    pub fn with_redirect_uri(mut self, uri: String) -> Self {
        self.redirect_uri = uri;
        self
    }

    /// Set scopes
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Generate a state parameter
    pub fn generate_state(&self) -> String {
        // Default: generate random state
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let state: String = (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
                chars[idx] as char
            })
            .collect();
        state
    }
}
