# Error Handling Guide

This guide covers comprehensive error handling patterns and best practices for the Prism MCP SDK.

## Overview

The Prism MCP SDK provides a robust error handling system built on Rust's `Result` type and custom error types. All fallible operations return `McpResult<T>`, which is a type alias for `Result<T, McpError>`.

## Error Types

### McpError

The main error type for the SDK:

```rust
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Protocol error: {message}")]
    Protocol { message: String },
    
    #[error("Transport error: {0}")]
    Transport(String),
    
    #[error("Invalid parameters: {message}")]
    InvalidParams { message: String },
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Tool execution error - {tool}: {error}")]
    ToolExecution { tool: String, error: String },
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden: {message}")]
    Forbidden { message: String },
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Timeout after {seconds} seconds")]
    Timeout { seconds: u64 },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}
```

### CallToolResult Error Handling

Tools can return errors as part of their result:

```rust
use prism_mcp_rs::protocol::types::{CallToolResult, ContentBlock};
use serde_json::Value;
use std::collections::HashMap;

// CallToolResult is the main type, with ToolResult as an alias
pub struct CallToolResult {
    pub content: Vec<ContentBlock>,
    pub is_error: Option<bool>,
    pub structured_content: Option<Value>,
    pub meta: Option<HashMap<String, Value>>,
}

impl CallToolResult {
    /// Create an error result
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::text(message.into())],
            is_error: Some(true),
            structured_content: None,
            meta: None,
        }
    }
    
    /// Create a successful text result
    pub fn text(message: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::text(message.into())],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        }
    }
}

// Type alias for compatibility
pub type ToolResult = CallToolResult;
```

## Error Handling Patterns

### Basic Error Propagation

Use the `?` operator for automatic error propagation:

```rust
#[async_trait]
impl ToolHandler for DataProcessor {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        // Automatically propagate errors with ?
        let input = arguments.get("input")
            .ok_or_else(|| McpError::InvalidParams {
                message: "Missing 'input' parameter".to_string(),
            })?;
        
        let processed = self.process_data(input)?;  // Propagates any processing errors
        
        Ok(ToolResult::text(processed))
    }
}
```

### Graceful Error Recovery

Handle errors without failing the entire operation:

```rust
#[async_trait]
impl ToolHandler for ResilientHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        // Try primary operation
        match self.primary_operation(&arguments).await {
            Ok(result) => Ok(ToolResult::text(result)),
            Err(primary_err) => {
                // Log the error
                log::warn!("Primary operation failed: {}", primary_err);
                
                // Try fallback
                match self.fallback_operation(&arguments).await {
                    Ok(fallback_result) => Ok(ToolResult::text(
                        format!("Fallback result: {}", fallback_result)
                    )),
                    Err(fallback_err) => {
                        // Both failed, return error result
                        Ok(ToolResult::error(format!(
                            "All operations failed. Primary: {}. Fallback: {}",
                            primary_err, fallback_err
                        )))
                    }
                }
            }
        }
    }
}
```

### Context-Rich Errors

Add context to errors for better debugging:

```rust
use std::fmt;

#[derive(Debug)]
struct ErrorContext {
    operation: String,
    details: HashMap<String, String>,
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Operation: {}, Details: {:?}", self.operation, self.details)
    }
}

fn process_with_context(data: &str) -> McpResult<String> {
    let context = ErrorContext {
        operation: "data_processing".to_string(),
        details: HashMap::from([
            ("input_length".to_string(), data.len().to_string()),
            ("timestamp".to_string(), Utc::now().to_string()),
        ]),
    };
    
    parse_data(data)
        .map_err(|e| McpError::Other(
            format!("Failed to process data. Context: {}. Error: {}", context, e)
        ))
}
```

### Validation Errors

Comprehensive input validation with detailed error messages:

```rust
struct InputValidator;

impl InputValidator {
    fn validate_email(&self, email: &str) -> McpResult<()> {
        if !email.contains('@') {
            return Err(McpError::InvalidParams {
                message: format!("Invalid email format: '{}' must contain '@'", email),
            });
        }
        
        if email.len() > 255 {
            return Err(McpError::InvalidParams {
                message: format!("Email too long: {} characters (max 255)", email.len()),
            });
        }
        
        Ok(())
    }
    
    fn validate_arguments(&self, args: &HashMap<String, Value>) -> McpResult<ValidatedInput> {
        let email = args.get("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams {
                message: "Missing required field: 'email'".to_string(),
            })?;
        
        self.validate_email(email)?;
        
        let age = args.get("age")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams {
                message: "Field 'age' must be a positive integer".to_string(),
            })?;
        
        if age < 18 || age > 150 {
            return Err(McpError::InvalidParams {
                message: format!("Age {} is out of valid range (18-150)", age),
            });
        }
        
        Ok(ValidatedInput { email: email.to_string(), age })
    }
}
```

### Async Error Handling

Handle errors in async contexts:

```rust
use tokio::time::{timeout, Duration};

async fn with_timeout<T>(
    operation: impl Future<Output = McpResult<T>>,
    seconds: u64,
) -> McpResult<T> {
    match timeout(Duration::from_secs(seconds), operation).await {
        Ok(result) => result,
        Err(_) => Err(McpError::Timeout { seconds }),
    }
}

#[async_trait]
impl ToolHandler for TimeoutHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        // Apply timeout to operation
        let result = with_timeout(
            self.long_running_operation(arguments),
            30  // 30 second timeout
        ).await?;
        
        Ok(ToolResult::text(result))
    }
}
```

## Error Aggregation

Collect and report multiple errors:

```rust
struct ValidationErrors {
    errors: Vec<String>,
}

impl ValidationErrors {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }
    
    fn add(&mut self, error: String) {
        self.errors.push(error);
    }
    
    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    fn to_error(&self) -> McpError {
        McpError::InvalidParams {
            message: self.errors.join("; "),
        }
    }
}

fn validate_complex_input(input: &ComplexInput) -> McpResult<()> {
    let mut errors = ValidationErrors::new();
    
    if input.name.is_empty() {
        errors.add("Name cannot be empty".to_string());
    }
    
    if input.items.is_empty() {
        errors.add("At least one item is required".to_string());
    }
    
    for (index, item) in input.items.iter().enumerate() {
        if item.quantity == 0 {
            errors.add(format!("Item {} has invalid quantity", index));
        }
    }
    
    if errors.has_errors() {
        Err(errors.to_error())
    } else {
        Ok(())
    }
}
```

## Custom Error Types

Create domain-specific error types:

```rust
#[derive(Debug, thiserror::Error)]
enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Query failed: {query}")]
    QueryFailed { query: String },
    
    #[error("Record not found: {id}")]
    NotFound { id: String },
}

impl From<DatabaseError> for McpError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::NotFound { id } => 
                McpError::ResourceNotFound(format!("Database record: {}", id)),
            other => McpError::Other(other.to_string()),
        }
    }
}
```

## Error Logging

Structured error logging:

```rust
use log::{error, warn, debug};

#[async_trait]
impl ToolHandler for LoggingHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        debug!("Tool called with arguments: {:?}", arguments);
        
        match self.process(&arguments).await {
            Ok(result) => {
                debug!("Tool completed successfully");
                Ok(ToolResult::text(result))
            }
            Err(e) => {
                error!(
                    "Tool failed - Error: {}, Arguments: {:?}, Time: {}",
                    e, arguments, Utc::now()
                );
                
                // Return user-friendly error
                Ok(ToolResult::error(
                    "Operation failed. Please check the logs for details."
                ))
            }
        }
    }
}
```

## Testing Error Scenarios

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_missing_parameter() {
        let handler = MyHandler::new();
        let arguments = HashMap::new();  // Missing required parameters
        
        let result = handler.call(arguments).await;
        assert!(matches!(result, Err(McpError::InvalidParams { .. })));
    }
    
    #[tokio::test]
    async fn test_error_recovery() {
        let handler = ResilientHandler::new();
        let arguments = HashMap::from([
            ("force_error".to_string(), json!(true)),
        ]);
        
        let result = handler.call(arguments).await.unwrap();
        assert!(result.is_error == Some(false));  // Should recover with fallback
    }
    
    #[test]
    fn test_error_aggregation() {
        let input = ComplexInput {
            name: String::new(),  // Invalid
            items: vec![],        // Invalid
        };
        
        let result = validate_complex_input(&input);
        match result {
            Err(McpError::InvalidParams { message }) => {
                assert!(message.contains("Name cannot be empty"));
                assert!(message.contains("At least one item is required"));
            }
            _ => panic!("Expected validation error"),
        }
    }
}
```

## Best Practices

1. **Be Specific**: Use specific error variants rather than generic `Other`
2. **Add Context**: Include relevant information in error messages
3. **Log Appropriately**: Use appropriate log levels (error, warn, info, debug)
4. **User-Friendly Messages**: Return helpful messages to users, technical details to logs
5. **Test Error Paths**: Write tests for error conditions
6. **Document Errors**: Document which errors each function can return
7. **Graceful Degradation**: Provide fallbacks when possible

## Common Pitfalls

### Don't Panic

Avoid `unwrap()` and `expect()` in production code:

```rust
// Bad
let value = data.get("key").unwrap();

// Good
let value = data.get("key")
    .ok_or_else(|| McpError::InvalidParams {
        message: "Missing required key".to_string(),
    })?;
```

### Don't Swallow Errors

```rust
// Bad
let _ = risky_operation();  // Ignores errors

// Good
if let Err(e) = risky_operation() {
    log::warn!("Non-critical operation failed: {}", e);
}
```

### Don't Over-Catch

```rust
// Bad - catches everything
match operation() {
    Ok(result) => process(result),
    Err(_) => default_value(),  // Lost error information
}

// Good - handle specific errors
match operation() {
    Ok(result) => process(result),
    Err(McpError::ResourceNotFound(_)) => use_default(),
    Err(e) => return Err(e),  // Propagate other errors
}
```

## Further Reading

- [Production Error Handling Example](../../examples/production_error_handling_demo.rs)
- [Rust Error Handling Book](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [API Documentation](https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/error/)