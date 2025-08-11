// ! Authorization Error Types
// !
// ! Module defines error types specific to the OAuth 2.1 authorization flow.

use std::fmt;

/// Authorization-specific errors
#[derive(Debug, Clone)]
pub enum AuthError {
    /// No authorization server found
    NoAuthServer(String),

    /// PKCE not supported by authorization server
    PkceNotSupported,

    /// Invalid or expired token
    InvalidToken(String),

    /// Token expired
    TokenExpired,

    /// Insufficient permissions/scopes
    InsufficientScope(String),

    /// Dynamic registration failed
    RegistrationFailed(String),

    /// Discovery failed
    DiscoveryFailed(String),

    /// Authorization denied by user
    AuthorizationDenied,

    /// Invalid authorization code
    InvalidAuthorizationCode,

    /// Invalid refresh token
    InvalidRefreshToken,

    /// OAuth error response
    OAuthError {
        error: String,
        description: Option<String>,
        uri: Option<String>,
    },

    /// HTTP error during authorization
    HttpError(String),

    /// Configuration error
    ConfigError(String),

    /// State mismatch (CSRF protection)
    StateMismatch,

    /// Resource indicator error
    InvalidResource(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAuthServer(msg) => write!(f, "No authorization server: {}", msg),
            Self::PkceNotSupported => write!(
                f,
                "Authorization server does not support PKCE (required for MCP)"
            ),
            Self::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            Self::TokenExpired => write!(f, "Access token has expired"),
            Self::InsufficientScope(scope) => write!(f, "Insufficient scope: {}", scope),
            Self::RegistrationFailed(msg) => write!(f, "Client registration failed: {}", msg),
            Self::DiscoveryFailed(msg) => write!(f, "Discovery failed: {}", msg),
            Self::AuthorizationDenied => write!(f, "Authorization denied by user"),
            Self::InvalidAuthorizationCode => write!(f, "Invalid or expired authorization code"),
            Self::InvalidRefreshToken => write!(f, "Invalid or expired refresh token"),
            Self::OAuthError {
                error,
                description,
                uri,
            } => {
                write!(f, "OAuth error: {}", error)?;
                if let Some(desc) = description {
                    write!(f, " - {}", desc)?;
                }
                if let Some(uri) = uri {
                    write!(f, " (see: {})", uri)?;
                }
                Ok(())
            }
            Self::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::StateMismatch => write!(f, "State parameter mismatch (possible CSRF attack)"),
            Self::InvalidResource(msg) => write!(f, "Invalid resource indicator: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

/// Convert AuthError to McpError
impl From<AuthError> for crate::core::error::McpError {
    fn from(err: AuthError) -> Self {
        crate::core::error::McpError::Auth(err.to_string())
    }
}

/// Parse OAuth error from query parameters
pub fn parse_oauth_error(params: &[(String, String)]) -> Option<AuthError> {
    let error = params
        .iter()
        .find(|(k, _)| k == "error")
        .map(|(_, v)| v.clone())?;

    let description = params
        .iter()
        .find(|(k, _)| k == "error_description")
        .map(|(_, v)| v.clone());

    let uri = params
        .iter()
        .find(|(k, _)| k == "error_uri")
        .map(|(_, v)| v.clone());

    Some(AuthError::OAuthError {
        error,
        description,
        uri,
    })
}

/// Check if an error is recoverable (e.g., by refreshing token)
pub fn is_recoverable_error(error: &AuthError) -> bool {
    matches!(error, AuthError::TokenExpired | AuthError::InvalidToken(_))
}
