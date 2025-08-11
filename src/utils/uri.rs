// ! URI handling utilities
// !
// ! Module provides utilities for parsing, validating, and manipulating URIs
// ! used in the MCP protocol for resources and other operations.

use crate::core::error::{McpError, McpResult};
use std::collections::HashMap;
use url::Url;

/// Parse a URI and extract query parameters
pub fn parse_uri_with_params(uri: &str) -> McpResult<(String, HashMap<String, String>)> {
    if uri.starts_with("file:///") || uri.contains("://") {
        // Full URI
        let parsed = Url::parse(uri)
            .map_err(|e| McpError::InvalidUri(format!("Invalid URI '{uri}': {e}")))?;

        let base_uri = format!(
            "{}://{}{}",
            parsed.scheme(),
            parsed.host_str().unwrap_or(""),
            parsed.path()
        );

        let mut params = HashMap::new();
        for (key, value) in parsed.query_pairs() {
            params.insert(key.to_string(), value.to_string());
        }

        Ok((base_uri, params))
    } else if uri.starts_with('/') {
        // Absolute path
        if let Some((path, query)) = uri.split_once('?') {
            let params = parse_query_string(query)?;
            Ok((path.to_string(), params))
        } else {
            Ok((uri.to_string(), HashMap::new()))
        }
    } else {
        // Relative path or simple identifier
        if let Some((path, query)) = uri.split_once('?') {
            let params = parse_query_string(query)?;
            Ok((path.to_string(), params))
        } else {
            Ok((uri.to_string(), HashMap::new()))
        }
    }
}

/// Parse a query string into parameters
pub fn parse_query_string(query: &str) -> McpResult<HashMap<String, String>> {
    let mut params = HashMap::new();

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }

        if let Some((key, value)) = pair.split_once('=') {
            let decoded_key = percent_decode(key)?;
            let decoded_value = percent_decode(value)?;
            params.insert(decoded_key, decoded_value);
        } else {
            let decoded_key = percent_decode(pair)?;
            params.insert(decoded_key, String::new());
        }
    }

    Ok(params)
}

/// Simple percent decoding for URI components
pub fn percent_decode(s: &str) -> McpResult<String> {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hex1 = chars
                .next()
                .ok_or_else(|| McpError::InvalidUri("Incomplete percent encoding".to_string()))?;
            let hex2 = chars
                .next()
                .ok_or_else(|| McpError::InvalidUri("Incomplete percent encoding".to_string()))?;

            let hex_str = format!("{hex1}{hex2}");
            let byte = u8::from_str_radix(&hex_str, 16).map_err(|_| {
                McpError::InvalidUri(format!("Invalid hex in percent encoding: {hex_str}"))
            })?;

            result.push(byte as char);
        } else if ch == '+' {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

/// Simple percent encoding for URI components
pub fn percent_encode(s: &str) -> String {
    let mut result = String::new();

    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => {
                result.push('+');
            }
            _ => {
                result.push_str(&format!("%{byte:02X}"));
            }
        }
    }

    result
}

/// Validate that a string is a valid URI
pub fn validate_uri(uri: &str) -> McpResult<()> {
    if uri.is_empty() {
        return Err(McpError::InvalidUri("URI cannot be empty".to_string()));
    }

    // Check for basic URI patterns
    if uri.contains("://") {
        // Full URI - try to parse with url crate
        Url::parse(uri).map_err(|e| McpError::InvalidUri(format!("Invalid URI '{uri}': {e}")))?;
    } else if uri.starts_with('/') {
        // Absolute path - basic validation
        if uri.contains('\0') || uri.contains('\n') || uri.contains('\r') {
            return Err(McpError::InvalidUri(
                "URI contains invalid characters".to_string(),
            ));
        }
    } else {
        // Relative path or identifier - allow most characters
        if uri.contains('\0') || uri.contains('\n') || uri.contains('\r') {
            return Err(McpError::InvalidUri(
                "URI contains invalid characters".to_string(),
            ));
        }
    }

    Ok(())
}

/// Normalize a URI to a standard form
pub fn normalize_uri(uri: &str) -> McpResult<String> {
    validate_uri(uri)?;

    if uri.contains("://") {
        // Full URI - normalize with url crate
        let parsed = Url::parse(uri)
            .map_err(|e| McpError::InvalidUri(format!("Invalid URI '{uri}': {e}")))?;
        let mut normalized = parsed.to_string();

        // Remove duplicate slashes in path
        if let Ok(mut url) = Url::parse(&normalized) {
            let path = url.path();
            let clean_path = path.replace("//", "/");
            url.set_path(&clean_path);
            normalized = url.to_string();
        }

        // Remove trailing slash unless it's the root
        if normalized.ends_with('/') && !normalized.ends_with("://") {
            let path_start = match normalized.find("://") {
                Some(pos) => pos + 3,
                None => {
                    return Err(McpError::InvalidUri(format!(
                        "URI '{normalized}' missing protocol separator"
                    )));
                }
            };
            if let Some(path_start_slash) = normalized[path_start..].find('/') {
                let full_path_start = path_start + path_start_slash;
                if full_path_start + 1 < normalized.len() {
                    normalized.pop();
                }
            }
        }

        Ok(normalized)
    } else {
        // Path - basic normalization
        let mut normalized = uri.to_string();

        // Remove duplicate slashes
        while normalized.contains("//") {
            normalized = normalized.replace("//", "/");
        }

        // Remove trailing slash unless it's the root
        if normalized.len() > 1 && normalized.ends_with('/') {
            normalized.pop();
        }

        Ok(normalized)
    }
}

/// Join a base URI with a relative path
pub fn join_uri(base: &str, relative: &str) -> McpResult<String> {
    if relative.contains("://") {
        // Relative is actually absolute
        return Ok(relative.to_string());
    }

    if relative.starts_with('/') {
        // Relative path is absolute, return it as-is
        return Ok(relative.to_string());
    }

    if base.contains("://") {
        // Full URI base
        let base_url = Url::parse(base)
            .map_err(|e| McpError::InvalidUri(format!("Invalid base URI '{base}': {e}")))?;
        let joined = base_url.join(relative).map_err(|e| {
            McpError::InvalidUri(format!("Cannot join '{relative}' to '{base}': {e}"))
        })?;
        Ok(joined.to_string())
    } else {
        // Path base
        let mut result = base.to_string();
        if !result.ends_with('/') && !relative.starts_with('/') {
            result.push('/');
        }
        result.push_str(relative);
        normalize_uri(&result)
    }
}

/// Extract the file extension from a URI
pub fn get_uri_extension(uri: &str) -> Option<String> {
    let path = if uri.contains("://") {
        Url::parse(uri).ok()?.path().to_string()
    } else {
        uri.to_string()
    };

    if let Some(dot_pos) = path.rfind('.') {
        if let Some(slash_pos) = path.rfind('/') {
            if dot_pos > slash_pos {
                return Some(path[dot_pos + 1..].to_lowercase());
            }
        } else {
            return Some(path[dot_pos + 1..].to_lowercase());
        }
    }

    None
}

/// Guess MIME type from URI extension
pub fn guess_mime_type(uri: &str) -> Option<String> {
    match get_uri_extension(uri)?.as_str() {
        "txt" => Some("text/plain".to_string()),
        "html" | "htm" => Some("text/html".to_string()),
        "css" => Some("text/css".to_string()),
        "js" => Some("application/javascript".to_string()),
        "json" => Some("application/json".to_string()),
        "xml" => Some("application/xml".to_string()),
        "pdf" => Some("application/pdf".to_string()),
        "zip" => Some("application/zip".to_string()),
        "png" => Some("image/png".to_string()),
        "jpg" | "jpeg" => Some("image/jpeg".to_string()),
        "gif" => Some("image/gif".to_string()),
        "webp" => Some("image/webp".to_string()),
        "svg" => Some("image/svg+xml".to_string()),
        "mp3" => Some("audio/mpeg".to_string()),
        "wav" => Some("audio/wav".to_string()),
        "mp4" => Some("video/mp4".to_string()),
        "webm" => Some("video/webm".to_string()),
        "csv" => Some("text/csv".to_string()),
        "md" => Some("text/markdown".to_string()),
        "yaml" | "yml" => Some("application/x-yaml".to_string()),
        "toml" => Some("application/toml".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uri_with_params() {
        let (uri, params) =
            parse_uri_with_params("https://example.com/path?key=value&foo=bar").unwrap();
        assert_eq!(uri, "https://example.com/path");
        assert_eq!(params.get("key"), Some(&"value".to_string()));
        assert_eq!(params.get("foo"), Some(&"bar".to_string()));
    }

    #[test]
    fn test_parse_query_string() {
        let params = parse_query_string("key=value&foo=bar&empty=").unwrap();
        assert_eq!(params.get("key"), Some(&"value".to_string()));
        assert_eq!(params.get("foo"), Some(&"bar".to_string()));
        assert_eq!(params.get("empty"), Some(&"".to_string()));
    }

    #[test]
    fn test_percent_encode_decode() {
        let original = "hello world!@#$%";
        let encoded = percent_encode(original);
        let decoded = percent_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_validate_uri() {
        assert!(validate_uri("https://example.com").is_ok());
        assert!(validate_uri("/absolute/path").is_ok());
        assert!(validate_uri("relative/path").is_ok());
        assert!(validate_uri("").is_err());
        assert!(validate_uri("invalid\0uri").is_err());
    }

    #[test]
    fn test_normalize_uri() {
        assert_eq!(
            normalize_uri("https://example.com//path//").unwrap(),
            "https://example.com/path"
        );
        assert_eq!(normalize_uri("/path//to//file/").unwrap(), "/path/to/file");
        assert_eq!(normalize_uri("/").unwrap(), "/");
    }

    #[test]
    fn test_join_uri() {
        assert_eq!(
            join_uri("https://example.com", "path/to/file").unwrap(),
            "https://example.com/path/to/file"
        );
        assert_eq!(
            join_uri("/base", "relative/path").unwrap(),
            "/base/relative/path"
        );
        assert_eq!(join_uri("/base/", "/absolute").unwrap(), "/absolute");
    }

    #[test]
    fn test_get_uri_extension() {
        assert_eq!(get_uri_extension("file.txt"), Some("txt".to_string()));
        assert_eq!(
            get_uri_extension("https://example.com/file.JSON"),
            Some("json".to_string())
        );
        assert_eq!(
            get_uri_extension("/path/to/file.tar.gz"),
            Some("gz".to_string())
        );
        assert_eq!(get_uri_extension("no-extension"), None);
    }

    #[test]
    fn test_guess_mime_type() {
        assert_eq!(
            guess_mime_type("file.json"),
            Some("application/json".to_string())
        );
        assert_eq!(guess_mime_type("image.PNG"), Some("image/png".to_string()));
        assert_eq!(
            guess_mime_type("document.pdf"),
            Some("application/pdf".to_string())
        );
        assert_eq!(guess_mime_type("unknown.xyz"), None);
    }
}
