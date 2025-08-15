// ! PKCE (Proof Key for Code Exchange) Implementation
// !
// ! Module implements PKCE as defined in RFC 7636 for OAuth 2.1.
// ! PKCE is mandatory for MCP authorization to prevent authorization code
// ! interception attacks

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::core::error::{McpError, McpResult};

/// PKCE code challenge methods
#[derive(Debug, Clone, PartialEq)]
pub enum CodeChallengeMethod {
    /// Plain text (not recommended, only for compatibility)
    Plain,
    /// SHA-256 hash (recommended)
    S256,
}

impl CodeChallengeMethod {
    /// Get the string representation for OAuth parameters
    pub fn as_str(&self) -> &str {
        match self {
            Self::Plain => "plain",
            Self::S256 => "S256",
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "plain" => Some(Self::Plain),
            "S256" => Some(Self::S256),
            _ => None,
        }
    }
}

/// PKCE parameters for authorization flow
#[derive(Debug, Clone)]
pub struct PkceParams {
    /// The code verifier (random string)
    pub verifier: String,
    /// The code challenge (derived from verifier)
    pub challenge: String,
    /// The challenge method used
    pub method: CodeChallengeMethod,
}

impl PkceParams {
    /// Generate new PKCE parameters with S256 method (recommended)
    pub fn new() -> Self {
        Self::with_method(CodeChallengeMethod::S256)
    }

    /// Generate new PKCE parameters with specified method
    pub fn with_method(method: CodeChallengeMethod) -> Self {
        let verifier = Self::generate_verifier();
        let challenge = Self::compute_challenge(&verifier, &method);

        Self {
            verifier,
            challenge,
            method,
        }
    }

    /// Generate a code verifier
    ///
    /// According to RFC 7636, the verifier should be a cryptographically random
    /// string using unreserved characters [A-Z] / [a-z] / [0-9] / "-" / "." / "_" / "~"
    /// with a minimum length of 43 characters and maximum of 128 characters.
    fn generate_verifier() -> String {
        let mut rng = rand::thread_rng();
        let _length = rng.gen_range(43..=128);

        // Use URL-safe base64 alphabet which matches the unreserved characters
        let mut bytes = [0u8; 32];
        for byte in &mut bytes {
            *byte = rng.r#gen::<u8>();
        }

        // Convert to URL-safe base64 without padding
        URL_SAFE_NO_PAD.encode(&bytes[..32]) // Use 32 bytes = 43 chars in base64
    }

    /// Compute the code challenge from the verifier
    fn compute_challenge(verifier: &str, method: &CodeChallengeMethod) -> String {
        match method {
            CodeChallengeMethod::Plain => verifier.to_string(),
            CodeChallengeMethod::S256 => {
                let mut hasher = Sha256::new();
                hasher.update(verifier.as_bytes());
                let hash = hasher.finalize();
                URL_SAFE_NO_PAD.encode(hash)
            }
        }
    }

    /// Verify that a verifier matches a challenge
    pub fn verify(verifier: &str, challenge: &str, method: &CodeChallengeMethod) -> bool {
        let computed = Self::compute_challenge(verifier, method);
        // Use constant-time comparison to prevent timing attacks
        constant_time_eq(&computed, challenge)
    }
}

impl Default for PkceParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut result = 0u8;

    for i in 0..a.len() {
        result |= a_bytes[i] ^ b_bytes[i];
    }

    result == 0
}

/// Check if the authorization server supports PKCE
pub fn check_pkce_support(metadata: &crate::auth::types::AuthorizationServerMetadata) -> bool {
    metadata
        .code_challenge_methods_supported
        .as_ref()
        .map(|methods| !methods.is_empty())
        .unwrap_or(false)
}

/// Check if the authorization server supports S256 method
pub fn supports_s256(metadata: &crate::auth::types::AuthorizationServerMetadata) -> bool {
    metadata
        .code_challenge_methods_supported
        .as_ref()
        .map(|methods| methods.contains(&"S256".to_string()))
        .unwrap_or(false)
}

/// Select the best available PKCE method from server metadata
pub fn select_challenge_method(
    metadata: &crate::auth::types::AuthorizationServerMetadata,
) -> McpResult<CodeChallengeMethod> {
    let methods = metadata
        .code_challenge_methods_supported
        .as_ref()
        .ok_or_else(|| {
            McpError::Auth(
                "Authorization server does not support PKCE (required for MCP)".to_string(),
            )
        })?;

    if methods.is_empty() {
        return Err(McpError::Auth(
            "Authorization server does not support any PKCE methods".to_string(),
        ));
    }

    // Prefer S256 over plain
    if methods.contains(&"S256".to_string()) {
        Ok(CodeChallengeMethod::S256)
    } else if methods.contains(&"plain".to_string()) {
        Ok(CodeChallengeMethod::Plain)
    } else {
        Err(McpError::Auth(format!(
            "No supported PKCE methods. Server supports: {methods:?}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let pkce = PkceParams::new();

        // Verifier should be at least 43 characters
        assert!(pkce.verifier.len() >= 43);
        assert!(pkce.verifier.len() <= 128);

        // Challenge should be different from verifier for S256
        assert_ne!(pkce.verifier, pkce.challenge);
        assert_eq!(pkce.method, CodeChallengeMethod::S256);

        // Should be URL-safe base64
        assert!(!pkce.verifier.contains('+'));
        assert!(!pkce.verifier.contains('/'));
        assert!(!pkce.verifier.contains('='));
        assert!(!pkce.challenge.contains('+'));
        assert!(!pkce.challenge.contains('/'));
        assert!(!pkce.challenge.contains('='));
    }

    #[test]
    fn test_pkce_verification() {
        let pkce = PkceParams::new();

        // Should verify correctly
        assert!(PkceParams::verify(
            &pkce.verifier,
            &pkce.challenge,
            &pkce.method
        ));

        // Should fail with wrong verifier
        assert!(!PkceParams::verify(
            "wrong_verifier",
            &pkce.challenge,
            &pkce.method
        ));

        // Should fail with wrong challenge
        assert!(!PkceParams::verify(
            &pkce.verifier,
            "wrong_challenge",
            &pkce.method
        ));
    }

    #[test]
    fn test_plain_method() {
        let pkce = PkceParams::with_method(CodeChallengeMethod::Plain);

        // For plain method, challenge should equal verifier
        assert_eq!(pkce.verifier, pkce.challenge);
        assert_eq!(pkce.method, CodeChallengeMethod::Plain);
    }

    #[test]
    fn test_constant_time_comparison() {
        assert!(constant_time_eq("hello", "hello"));
        assert!(!constant_time_eq("hello", "world"));
        assert!(!constant_time_eq("hello", "hello!"));
        assert!(!constant_time_eq("", "a"));
    }

    #[test]
    fn test_s256_challenge() {
        // Test with known values from RFC 7636 Appendix B
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let expected_challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

        let challenge = PkceParams::compute_challenge(verifier, &CodeChallengeMethod::S256);

        assert_eq!(challenge, expected_challenge);
    }
}
