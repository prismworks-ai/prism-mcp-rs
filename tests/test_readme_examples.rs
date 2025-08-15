// ! Test file to verify that README examples actually compile and work
// !
// ! This file contains all the examples from the README to ensure they are
// ! accurate and functional

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use serde_json::{Value, json};
use std::collections::HashMap;

// =============================================================================
// Test 1: Basic Server Example from README
// =============================================================================

/// Calculator handler from README example
struct CalculatorHandler;

#[async_trait]
impl ToolHandler for CalculatorHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let a = arguments
            .get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::Validation("Missing 'a' parameter".to_string()))?;

        let b = arguments
            .get("b")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::Validation("Missing 'b' parameter".to_string()))?;

        let result = a + b;

        Ok(ToolResult {
            content: vec![Content::text(result.to_string())],
            is_error: None,
            structured_content: Some(json!({
                "operation": "addition",
                "operands": [a, b],
                "result": result
            })),
            meta: None,
        })
    }
}

// Test function to verify server creation works
fn test_server_creation() -> Result<(), Box<dyn std::error::Error>> {
    // This should compile without errors
    let _server = McpServer::new("my-calculator".to_string(), "1.0.0".to_string());
    Ok(())
}

// =============================================================================
// Test 2: ToolBuilder Example from README
// =============================================================================

fn test_tool_builder() -> McpResult<()> {
    use prism_mcp_rs::core::tool::ToolBuilder;

    // This should compile and create a tool successfully
    let _tool = ToolBuilder::new("improved_calculator")
        .description("complete calculator with validation")
        .version("1.0.0")
        .schema(json!({
            "type": "object",
            "properties": {
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["a", "b"]
        }))
        .strict_validation()
        .read_only()
        .idempotent()
        .cacheable()
        .build(CalculatorHandler)?;

    Ok(())
}

// =============================================================================
// Test 3: Client Configuration Example
// =============================================================================

#[cfg(feature = "http")]
fn test_client_config() -> Result<(), Box<dyn std::error::Error>> {
    use prism_mcp_rs::client::McpClient;
    use prism_mcp_rs::transport::traits::TransportConfig;

    // This should compile without errors
    let _config = TransportConfig {
        connect_timeout_ms: Some(5_000),
        read_timeout_ms: Some(30_000),
        write_timeout_ms: Some(30_000),
        max_message_size: Some(1024 * 1024), // 1MB
        keep_alive_ms: Some(60_000),         // 1 minute
        compression: true,
        headers: std::collections::HashMap::new(),
    };

    let _client = McpClient::new("my-client".to_string(), "1.0.0".to_string());

    Ok(())
}

// =============================================================================
// Test 4: Echo Handler Example
// =============================================================================

struct EchoHandler;

#[async_trait]
impl ToolHandler for EchoHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello, World!");

        Ok(ToolResult {
            content: vec![Content::text(message.to_string())],
            is_error: None,
            structured_content: None,
            meta: None,
        })
    }
}

// Test async server setup (compilation test)
async fn test_async_server_setup() -> McpResult<()> {
    let server = McpServer::new("test-server".to_string(), "1.0.0".to_string());

    // Test adding a tool with the actual API
    server
        .add_tool(
            "echo".to_string(),
            Some("Echo a message".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string"}
                },
                "required": ["message"]
            }),
            EchoHandler,
        )
        .await?;

    Ok(())
}

// =============================================================================
// Test 5: Schema Validation Example
// =============================================================================

fn test_schema_types() -> Result<(), Box<dyn std::error::Error>> {
    use prism_mcp_rs::protocol::types::*;

    // Test that schema types can be created (from README example)
    let tool_info = ToolInfo {
        name: "calculator".to_string(),
        description: Some("Performs mathematical operations".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: Some(std::collections::HashMap::new()),
            required: Some(vec!["a".to_string(), "b".to_string()]),
            additional_properties: std::collections::HashMap::new(),
        },
        output_schema: None,
        annotations: None,
        title: None,
        meta: None,
    };

    // Test that it can be serialized to JSON
    let _json = serde_json::to_value(&tool_info)?;

    Ok(())
}

// =============================================================================
// Main function (for compilation test)
// =============================================================================

fn main() {
    println!("All README examples compile successfully!");

    // Run basic tests
    test_server_creation().expect("Server creation test failed");
    test_tool_builder().expect("ToolBuilder test failed");
    test_schema_types().expect("Schema types test failed");

    #[cfg(feature = "http")]
    test_client_config().expect("Client config test failed");

    println!("[x] All README examples are working!");
}

// =============================================================================
// Test Runner
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_readme_examples_compile() {
        // Test 1: Server creation
        test_server_creation().expect("Server creation should work");

        // Test 2: ToolBuilder
        test_tool_builder().expect("ToolBuilder should work");

        // Test 3: Client config (only if http feature enabled)
        #[cfg(feature = "http")]
        test_client_config().expect("Client config should work");

        // Test 4: Schema types
        test_schema_types().expect("Schema types should work");
    }

    #[tokio::test]
    async fn test_async_examples() {
        // Test async server setup
        test_async_server_setup()
            .await
            .expect("Async server setup should work");
    }

    #[tokio::test]
    async fn test_tool_handler_execution() {
        let handler = CalculatorHandler;

        let mut args = HashMap::new();
        args.insert("a".to_string(), json!(5.0));
        args.insert("b".to_string(), json!(3.0));

        let result = handler
            .call(args)
            .await
            .expect("Tool should execute successfully");

        // Verify the result
        assert_eq!(result.content.len(), 1);
        if let Content::Text { text, .. } = &result.content[0] {
            assert_eq!(text, "8");
        } else {
            panic!("Expected text content");
        }
    }
}
