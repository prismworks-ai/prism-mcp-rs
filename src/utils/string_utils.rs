//! String manipulation utilities

use super::{UtilError, UtilResult};
use std::collections::HashMap;

/// Sanitize a string for safe use in identifiers
pub fn sanitize_identifier(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// Convert a string to snake_case
pub fn to_snake_case(input: &str) -> String {
    let mut result = String::new();
    let mut prev_was_uppercase = false;

    for (i, c) in input.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_was_uppercase {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_was_uppercase = true;
        } else {
            result.push(c);
            prev_was_uppercase = false;
        }
    }

    result
}

/// Convert a string to camelCase
pub fn to_camel_case(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in input.chars() {
        if c == '_' || c == '-' || c.is_whitespace() {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c.to_lowercase().next().unwrap());
        }
    }

    result
}

/// Convert a string to PascalCase
pub fn to_pascal_case(input: &str) -> String {
    let camel = to_camel_case(input);
    if let Some(first) = camel.chars().next() {
        first.to_uppercase().collect::<String>() + &camel[1..]
    } else {
        camel
    }
}

/// Truncate a string to a maximum length with ellipsis
pub fn truncate_with_ellipsis(input: &str, max_len: usize) -> String {
    if input.len() <= max_len {
        input.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &input[..max_len - 3])
    }
}

/// Extract variables from a template string (e.g., "Hello {name}")
pub fn extract_template_variables(template: &str) -> Vec<String> {
    let mut variables = Vec::new();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            let mut var_name = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c == '}' {
                    chars.next(); // consume '}'
                    if !var_name.is_empty() {
                        variables.push(var_name);
                    }
                    break;
                }
                var_name.push(chars.next().unwrap());
            }
        }
    }

    variables
}

/// Replace variables in a template string
pub fn replace_template_variables(
    template: &str,
    variables: &HashMap<String, String>,
) -> UtilResult<String> {
    let mut result = template.to_string();

    for (key, value) in variables {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }

    // Check for unreplaced variables
    let remaining_vars = extract_template_variables(&result);
    if !remaining_vars.is_empty() {
        return Err(UtilError::ValidationFailed(format!(
            "Unreplaced template variables: {:?}",
            remaining_vars
        )));
    }

    Ok(result)
}

/// Generate a random string with specified length and character set
pub fn random_string(length: usize, charset: &str) -> String {
    use rand::prelude::*;
    use rand::rng;

    let chars: Vec<char> = charset.chars().collect();
    let mut rng = rng();

    (0..length)
        .map(|_| *chars.choose(&mut rng).unwrap_or(&'a'))
        .collect()
}

/// Generate a random alphanumeric string
pub fn random_alphanumeric(length: usize) -> String {
    random_string(
        length,
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
    )
}

/// Escape a string for JSON
pub fn escape_json_string(input: &str) -> String {
    serde_json::to_string(input).unwrap_or_else(|_| format!("{:?}", input))
}

/// Check if a string is a valid semantic version
pub fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    parts
        .iter()
        .all(|part| part.parse::<u32>().is_ok() && !part.starts_with('0') || *part == "0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("hello world!"), "hello_world");
        assert_eq!(sanitize_identifier("test@email.com"), "test_email_com");
    }

    #[test]
    fn test_case_conversions() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
    }

    #[test]
    fn test_template_variables() {
        let template = "Hello {name}, you have {count} messages";
        let vars = extract_template_variables(template);
        assert_eq!(vars, vec!["name", "count"]);

        let mut replacements = HashMap::new();
        replacements.insert("name".to_string(), "Alice".to_string());
        replacements.insert("count".to_string(), "5".to_string());

        let result = replace_template_variables(template, &replacements).unwrap();
        assert_eq!(result, "Hello Alice, you have 5 messages");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate_with_ellipsis("hello", 10), "hello");
        assert_eq!(truncate_with_ellipsis("hello world", 8), "hello...");
    }

    #[test]
    fn test_semver_validation() {
        assert!(is_valid_semver("1.0.0"));
        assert!(is_valid_semver("0.1.0"));
        assert!(!is_valid_semver("1.0"));
        assert!(!is_valid_semver("01.0.0"));
    }
}
