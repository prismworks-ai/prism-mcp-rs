//! Example demonstrating the new ServerBuilder API and error convenience methods
//!
//! This example shows how to use the improved APIs introduced to address
//! developer feedback for the prism-mcp-rs library.

use prism_mcp_rs::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> McpResult<()> {
    println!("🚀 Prism MCP-RS Server Builder Demo\n");

    // ========================================================================
    // FEATURE 1: ServerBuilder Pattern
    // ========================================================================
    println!("1️⃣ Using the new ServerBuilder pattern:\n");

    // Create a server using the fluent builder API
    let server = ServerBuilder::new()
        .name("demo-server")
        .version("1.0.0")
        .with_prompts()
        .with_resources()
        .with_tools()
        .with_sampling()
        .with_logging()
        .with_completions()
        .max_concurrent_requests(200)
        .request_timeout_ms(60000)
        .validate_requests(true)
        .enable_logging(true)
        .with_experimental("custom_feature", json!(true))
        .build();

    println!("✅ Server created with name: {}", server.name());
    println!("✅ Server version: {}", server.version());
    println!("✅ Server capabilities configured via builder\n");

    // Alternative: Using try_build for error handling
    let result = ServerBuilder::new()
        .name("another-server")
        .version("2.0.0")
        .try_build();

    match result {
        Ok(server) => println!("✅ Second server created: {}", server.name()),
        Err(e) => println!("❌ Error creating server: {}", e),
    }

    // ========================================================================
    // FEATURE 2: Error Response Convenience Methods
    // ========================================================================
    println!("\n2️⃣ Using error response convenience methods:\n");

    let request_id = json!("req-123");

    // Standard JSON-RPC errors
    let parse_error = JsonRpcError::parse_error(request_id.clone());
    println!("Parse error: {:?}", parse_error.error.message);

    let method_not_found = JsonRpcError::method_not_found(request_id.clone());
    println!("Method not found: {:?}", method_not_found.error.message);

    // Method not found with details
    let method_error_detailed =
        JsonRpcError::method_not_found_with_name(request_id.clone(), "unknown_method");
    println!(
        "Detailed method error: {:?}",
        method_error_detailed.error.message
    );

    // Invalid params with custom message
    let invalid_params = JsonRpcError::invalid_params_with_message(
        request_id.clone(),
        "Field 'name' is required but was not provided",
    );
    println!("Invalid params: {:?}", invalid_params.error.message);

    // Internal error
    let internal_error =
        JsonRpcError::internal_error_with_message(request_id.clone(), "Database connection failed");
    println!("Internal error: {:?}", internal_error.error.message);

    // MCP-specific errors
    let tool_not_found = JsonRpcError::tool_not_found(request_id.clone(), "my_tool");
    println!("Tool not found: {:?}", tool_not_found.error.message);

    let resource_not_found =
        JsonRpcError::resource_not_found(request_id.clone(), "file:///missing.txt");
    println!("Resource not found: {:?}", resource_not_found.error.message);

    let prompt_not_found = JsonRpcError::prompt_not_found(request_id.clone(), "test_prompt");
    println!("Prompt not found: {:?}", prompt_not_found.error.message);

    // Custom errors with data
    let custom_error = JsonRpcError::custom_with_data(
        request_id.clone(),
        -32099,
        "Custom validation error",
        json!({
            "field": "email",
            "reason": "Invalid email format",
            "provided": "not-an-email"
        }),
    );
    println!(
        "Custom error: {:?} with data: {:?}",
        custom_error.error.message, custom_error.error.data
    );

    // ========================================================================
    // FEATURE 3: Converting Errors to JsonRpcMessage
    // ========================================================================
    println!("\n3️⃣ Converting errors to JsonRpcMessage:\n");

    // Using From trait
    let message: JsonRpcMessage = JsonRpcError::method_not_found(request_id.clone()).into();
    match message {
        JsonRpcMessage::Error(e) => {
            println!("✅ Error converted to JsonRpcMessage: {}", e.error.message);
        }
        _ => println!("❌ Unexpected message type"),
    }

    // Using IntoJsonRpcMessage trait
    let error = JsonRpcError::invalid_params(request_id.clone());
    let message = error.into_message();
    match message {
        JsonRpcMessage::Error(e) => {
            println!("✅ Error converted via trait: {}", e.error.message);
        }
        _ => println!("❌ Unexpected message type"),
    }

    // ========================================================================
    // FEATURE 4: Success Response Helpers
    // ========================================================================
    println!("\n4️⃣ Creating success responses:\n");

    // Using the fallible success method
    let success_response = JsonRpcResponse::success(
        request_id.clone(),
        json!({
            "status": "ok",
            "data": "Hello, World!"
        }),
    )?;
    println!(
        "✅ Success response created with result: {:?}",
        success_response.result
    );

    // Using the infallible success_unchecked method
    let unchecked_response = JsonRpcResponse::success_unchecked(
        request_id.clone(),
        json!({
            "status": "completed",
            "count": 42
        }),
    );
    println!(
        "✅ Unchecked success response: {:?}",
        unchecked_response.result
    );

    // ========================================================================
    // FEATURE 5: Alternative Server Creation Methods
    // ========================================================================
    println!("\n5️⃣ Alternative server creation methods:\n");

    // Method 1: Direct construction with fluent configuration
    let server_direct = McpServer::new("direct-server".to_string(), "1.0.0".to_string())
        .with_capabilities(ServerCapabilities {
            prompts: Some(PromptsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            ..Default::default()
        });
    println!(
        "✅ Server created directly with fluent API: {}",
        server_direct.name()
    );

    // Method 2: Using builder() static method
    let server_from_builder = McpServer::builder()
        .name("builder-method-server")
        .version("1.0.0")
        .with_tools()
        .build();
    println!(
        "✅ Server created via builder() method: {}",
        server_from_builder.name()
    );

    // Method 3: With custom config
    let custom_config = ServerConfig {
        max_concurrent_requests: 500,
        request_timeout_ms: 120000,
        validate_requests: false,
        enable_logging: true,
    };

    let server_with_config = McpServer::with_config(
        "config-server".to_string(),
        "1.0.0".to_string(),
        custom_config,
    );
    println!(
        "✅ Server created with custom config: {}",
        server_with_config.name()
    );

    // ========================================================================
    // FEATURE 6: Using Error Codes Constants
    // ========================================================================
    println!("\n6️⃣ Using error code constants:\n");

    println!("Standard JSON-RPC error codes:");
    println!("  PARSE_ERROR: {}", error_codes::PARSE_ERROR);
    println!("  INVALID_REQUEST: {}", error_codes::INVALID_REQUEST);
    println!("  METHOD_NOT_FOUND: {}", error_codes::METHOD_NOT_FOUND);
    println!("  INVALID_PARAMS: {}", error_codes::INVALID_PARAMS);
    println!("  INTERNAL_ERROR: {}", error_codes::INTERNAL_ERROR);

    println!("\nMCP-specific error codes:");
    println!("  TOOL_NOT_FOUND: {}", error_codes::TOOL_NOT_FOUND);
    println!("  RESOURCE_NOT_FOUND: {}", error_codes::RESOURCE_NOT_FOUND);
    println!("  PROMPT_NOT_FOUND: {}", error_codes::PROMPT_NOT_FOUND);

    // Using error codes in custom error creation
    let custom_with_code = JsonRpcError::new(
        request_id.clone(),
        error_codes::METHOD_NOT_FOUND,
        "The requested method is not available",
    );
    println!(
        "\n✅ Error created with constant: code={}, message={}",
        custom_with_code.error.code, custom_with_code.error.message
    );

    // ========================================================================
    // SUMMARY
    // ========================================================================
    println!("\n🎉 Demo completed successfully!\n");
    println!("This demo showcased:");
    println!("  ✅ ServerBuilder pattern for fluent server configuration");
    println!("  ✅ Convenient error response creation methods");
    println!("  ✅ Error to JsonRpcMessage conversion");
    println!("  ✅ Success response helpers (fallible and infallible)");
    println!("  ✅ Multiple server creation patterns");
    println!("  ✅ Error code constants for consistency");

    Ok(())
}
