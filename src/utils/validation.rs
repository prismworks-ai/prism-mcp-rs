//! Validation utilities

use super::{UtilError, UtilResult};
use std::collections::HashSet;

/// Validate an email address
pub fn validate_email(email: &str) -> UtilResult<()> {
    if email.is_empty() {
        return Err(UtilError::ValidationFailed(
            "Email cannot be empty".to_string(),
        ));
    }

    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(UtilError::ValidationFailed(
            "Email must contain exactly one @".to_string(),
        ));
    }

    let (local, domain) = (parts[0], parts[1]);

    if local.is_empty() || local.len() > 64 {
        return Err(UtilError::ValidationFailed(
            "Invalid local part".to_string(),
        ));
    }

    if domain.is_empty() || domain.len() > 255 || !domain.contains('.') {
        return Err(UtilError::ValidationFailed(
            "Invalid domain part".to_string(),
        ));
    }

    Ok(())
}

/// Validate a URL
pub fn validate_url(url: &str) -> UtilResult<()> {
    if url.is_empty() {
        return Err(UtilError::ValidationFailed(
            "URL cannot be empty".to_string(),
        ));
    }

    let valid_schemes = ["http", "https", "ftp", "ftps"];
    let has_valid_scheme = valid_schemes
        .iter()
        .any(|scheme| url.starts_with(&format!("{}://", scheme)));

    if !has_valid_scheme {
        return Err(UtilError::ValidationFailed(
            "URL must have a valid scheme".to_string(),
        ));
    }

    Ok(())
}

/// Validate a JSON string
pub fn validate_json(json_str: &str) -> UtilResult<serde_json::Value> {
    serde_json::from_str(json_str)
        .map_err(|e| UtilError::ValidationFailed(format!("Invalid JSON: {}", e)))
}

/// Validate that a string contains only allowed characters
pub fn validate_charset(input: &str, allowed_chars: &str) -> UtilResult<()> {
    let allowed_set: HashSet<char> = allowed_chars.chars().collect();

    for c in input.chars() {
        if !allowed_set.contains(&c) {
            return Err(UtilError::ValidationFailed(format!(
                "Character '{}' is not allowed",
                c
            )));
        }
    }

    Ok(())
}

/// Validate string length constraints
pub fn validate_length(
    input: &str,
    min_len: Option<usize>,
    max_len: Option<usize>,
) -> UtilResult<()> {
    let len = input.len();

    if let Some(min) = min_len {
        if len < min {
            return Err(UtilError::ValidationFailed(format!(
                "String too short: {} < {}",
                len, min
            )));
        }
    }

    if let Some(max) = max_len {
        if len > max {
            return Err(UtilError::ValidationFailed(format!(
                "String too long: {} > {}",
                len, max
            )));
        }
    }

    Ok(())
}

/// Validate that a string matches a pattern
pub fn validate_pattern(input: &str, pattern: &str) -> UtilResult<()> {
    use regex::Regex;

    let regex = Regex::new(pattern)
        .map_err(|e| UtilError::ValidationFailed(format!("Invalid regex pattern: {}", e)))?;

    if !regex.is_match(input) {
        return Err(UtilError::ValidationFailed(format!(
            "String does not match pattern: {}",
            pattern
        )));
    }

    Ok(())
}

/// Validate a port number
pub fn validate_port(port: u16) -> UtilResult<()> {
    if port == 0 {
        return Err(UtilError::ValidationFailed("Port cannot be 0".to_string()));
    }
    Ok(())
}

/// Validate an IPv4 address
pub fn validate_ipv4(ip: &str) -> UtilResult<()> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return Err(UtilError::ValidationFailed(
            "IPv4 must have 4 octets".to_string(),
        ));
    }

    for part in parts {
        let _octet: u8 = part
            .parse()
            .map_err(|_| UtilError::ValidationFailed("Invalid octet".to_string()))?;

        if part.starts_with('0') && part.len() > 1 {
            return Err(UtilError::ValidationFailed(
                "Leading zeros not allowed".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validate a semantic version string
pub fn validate_semver(version: &str) -> UtilResult<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(UtilError::ValidationFailed(
            "Semantic version must have format MAJOR.MINOR.PATCH".to_string(),
        ));
    }

    let major = parts[0]
        .parse::<u32>()
        .map_err(|_| UtilError::ValidationFailed("Invalid major version".to_string()))?;
    let minor = parts[1]
        .parse::<u32>()
        .map_err(|_| UtilError::ValidationFailed("Invalid minor version".to_string()))?;
    let patch = parts[2]
        .parse::<u32>()
        .map_err(|_| UtilError::ValidationFailed("Invalid patch version".to_string()))?;

    // Check for leading zeros
    for (i, part) in parts.iter().enumerate() {
        if part.starts_with('0') && part.len() > 1 {
            let component = ["major", "minor", "patch"][i];
            return Err(UtilError::ValidationFailed(format!(
                "Leading zeros not allowed in {} version",
                component
            )));
        }
    }

    Ok((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid.email").is_err());
        assert!(validate_email("").is_err());
        assert!(validate_email("@example.com").is_err());
    }

    #[test]
    fn test_url_validation() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://localhost:8080").is_ok());
        assert!(validate_url("not-a-url").is_err());
        assert!(validate_url("").is_err());
    }

    #[test]
    fn test_json_validation() {
        assert!(validate_json(r#"{"key": "value"}"#).is_ok());
        assert!(validate_json("[1, 2, 3]").is_ok());
        assert!(validate_json("invalid json").is_err());
    }

    #[test]
    fn test_charset_validation() {
        assert!(validate_charset("abc123", "abcdefghijklmnopqrstuvwxyz0123456789").is_ok());
        assert!(validate_charset("abc@123", "abcdefghijklmnopqrstuvwxyz0123456789").is_err());
    }

    #[test]
    fn test_length_validation() {
        assert!(validate_length("hello", Some(3), Some(10)).is_ok());
        assert!(validate_length("hi", Some(3), Some(10)).is_err());
        assert!(validate_length("very long string", Some(3), Some(10)).is_err());
    }

    #[test]
    fn test_ipv4_validation() {
        assert!(validate_ipv4("192.168.1.1").is_ok());
        assert!(validate_ipv4("0.0.0.0").is_ok());
        assert!(validate_ipv4("256.1.1.1").is_err());
        assert!(validate_ipv4("192.168.1").is_err());
    }

    #[test]
    fn test_semver_validation() {
        assert_eq!(validate_semver("1.0.0").unwrap(), (1, 0, 0));
        assert_eq!(validate_semver("0.1.2").unwrap(), (0, 1, 2));
        assert!(validate_semver("1.0").is_err());
        assert!(validate_semver("01.0.0").is_err());
    }
}
