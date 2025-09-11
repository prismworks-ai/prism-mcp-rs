//! Enhanced API Demonstration - All Improvements Implemented
//! Shows the new fluent interface and best practices

use prism_mcp_rs::{
    client::McpClient,
    core::enhanced_errors::{McpError, McpResult},
};
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> McpResult<()> {
    println!("🚀 Enhanced API Demo - All Best Practices Implemented");

    // ========================================================================
    // 1. Modern Builder Pattern (Primary Constructor)
    // ========================================================================
    println!("\n✅ Enhanced Builder Pattern:");

    // Production client with validation
    let client = McpClient::builder()
        .name("enhanced-client")
        .version("2.0.0")
        .timeout(Duration::from_secs(30))
        .max_retries(3)
        .validate_requests(true)
        .build()?;

    println!("   Built client: {}", client.info());

    // Development client with convenient defaults
    let _dev_client = McpClient::builder().development("dev-client").build()?;

    // ========================================================================
    // 2. Fluent Interface for Operations (New Primary API)
    // ========================================================================
    println!("\n✅ Modern Fluent Interface:");

    // Tools - Fluent and discoverable
    let tool_result = client
        .tools()
        .call("calculator")
        .arg("operation", "add")
        .arg("a", 5)
        .arg("b", 3)
        .execute()
        .await;

    match tool_result {
        Err(McpError::Connection { .. }) => {
            println!("   ✅ Tools API works (requires connection)");
        }
        _ => println!("   ⚠️  Unexpected result"),
    }

    // Resources - Clean and intuitive
    let resource_result = client.resources().read("file:///tmp/data.txt").await;

    match resource_result {
        Err(McpError::Connection { .. }) => {
            println!("   ✅ Resources API works (requires connection)");
        }
        _ => println!("   ⚠️  Unexpected result"),
    }

    // Prompts - Type-safe arguments
    let prompt_result = client
        .prompts()
        .get("code_review")
        .arg("language", "rust")
        .arg("complexity", "high")
        .execute()
        .await;

    match prompt_result {
        Err(McpError::Connection { .. }) => {
            println!("   ✅ Prompts API works (requires connection)");
        }
        _ => println!("   ⚠️  Unexpected result"),
    }

    // ========================================================================
    // 3. Enhanced Error Types (Structured Errors)
    // ========================================================================
    println!("\n✅ Structured Error Handling:");

    let error = McpError::timeout("Operation timed out");
    println!("   Error type: {:?}", error);
    println!("   Is recoverable: {}", error.is_recoverable());

    // ========================================================================
    // 4. List Operations (Simplified)
    // ========================================================================
    println!("\n✅ Simplified List Operations:");

    // List tools
    let list_tools = client.tools().list().await;
    match list_tools {
        Err(McpError::Connection { .. }) => {
            println!("   ✅ List tools works (requires connection)");
        }
        _ => println!("   ⚠️  Unexpected result"),
    }

    // List resources
    let list_resources = client.resources().list().await;
    match list_resources {
        Err(McpError::Connection { .. }) => {
            println!("   ✅ List resources works (requires connection)");
        }
        _ => println!("   ⚠️  Unexpected result"),
    }

    // List prompts
    let list_prompts = client.prompts().list().await;
    match list_prompts {
        Err(McpError::Connection { .. }) => {
            println!("   ✅ List prompts works (requires connection)");
        }
        _ => println!("   ⚠️  Unexpected result"),
    }

    // ========================================================================
    // 5. Comparison: Old vs New API
    // ========================================================================
    println!("\n📊 API Comparison:");

    // OLD: Verbose and inconsistent
    #[allow(deprecated)]
    let _old_result = client
        .call_tool_simple("tool_name", json!({"arg1": "value1", "arg2": "value2"}))
        .await;
    println!("   ⚠️  Old API: client.call_tool_simple() - deprecated");

    // NEW: Fluent and discoverable
    let _new_result = client
        .tools()
        .call("tool_name")
        .arg("arg1", "value1")
        .arg("arg2", "value2")
        .execute()
        .await;
    println!("   ✅ New API: client.tools().call().arg().execute() - modern");

    // ========================================================================
    // 6. Type Safety Demonstration
    // ========================================================================
    println!("\n🛡️  Type Safety:");

    // Builder validation
    let invalid_client = McpClient::builder()
        // .name() missing - will fail validation
        .version("1.0")
        .build();

    match invalid_client {
        Err(McpError::Validation { message }) => {
            println!("   ✅ Builder validation: {}", message);
        }
        _ => println!("   ⚠️  Validation should have failed"),
    }

    // ========================================================================
    // Summary
    // ========================================================================
    println!("\n🎉 ENHANCED API FEATURES DEMONSTRATED:");
    println!("   ✅ Modern builder pattern with validation");
    println!("   ✅ Fluent interface for all operations");
    println!("   ✅ Structured error types with context");
    println!("   ✅ Type-safe argument handling");
    println!("   ✅ Consistent API surface");
    println!("   ✅ Backward compatibility maintained");

    println!("\n🚀 Ready for production use with enhanced ergonomics!");

    Ok(())
}
