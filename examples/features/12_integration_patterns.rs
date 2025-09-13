//! Example 12: Integration Patterns (Fixed Version)
//! Demonstrates integration patterns

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Data processing tool
struct DataProcessorTool;

#[async_trait]
impl ToolHandler for DataProcessorTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let data_type = arguments
            .get("data_type")
            .and_then(|v| v.as_str())
            .unwrap_or("json");

        let default_data = json!({});
        let input_data = arguments.get("data").unwrap_or(&default_data);

        let processed_result = match data_type {
            "json" => {
                json!({
                    "original": input_data,
                    "processed": true,
                    "timestamp": "2024-01-01T00:00:00Z",
                    "type": "json_processing"
                })
            }
            "csv" => {
                json!({
                    "rows_processed": 150,
                    "columns_found": 5,
                    "data_type": "csv",
                    "status": "completed"
                })
            }
            "xml" => {
                json!({
                    "elements_parsed": 42,
                    "attributes_found": 18,
                    "validation_status": "valid",
                    "data_type": "xml"
                })
            }
            _ => {
                return Err(McpError::validation(format!(
                    "Unsupported data type: {}",
                    data_type
                )))
            }
        };

        Ok(ToolResult {
            content: vec![ContentBlock::Text {
                text: format!("Processed {} data successfully", data_type),
                annotations: None,
                meta: None,
            }],
            is_error: Some(false),
            structured_content: Some(processed_result),
            meta: None,
        })
    }
}

/// API integration tool
struct ApiIntegrationTool;

#[async_trait]
impl ToolHandler for ApiIntegrationTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let api_endpoint = arguments
            .get("endpoint")
            .and_then(|v| v.as_str())
            .unwrap_or("/api/default");

        let method = arguments
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");

        // Simulate API call
        let response = json!({
            "endpoint": api_endpoint,
            "method": method,
            "status_code": 200,
            "response_time_ms": 150,
            "data": {
                "message": "API call successful",
                "timestamp": "2024-01-01T00:00:00Z"
            }
        });

        Ok(ToolResult {
            content: vec![ContentBlock::Text {
                text: format!("{} request to {} completed", method, api_endpoint),
                annotations: None,
                meta: None,
            }],
            is_error: Some(false),
            structured_content: Some(response),
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Integration Patterns Example");
    println!("===========================");

    let data_tool = DataProcessorTool;
    let api_tool = ApiIntegrationTool;

    // Test data processing
    let mut data_args = HashMap::new();
    data_args.insert("data_type".to_string(), json!("json"));
    data_args.insert("data".to_string(), json!({"name": "test", "value": 42}));

    match data_tool.call(data_args).await {
        Ok(result) => println!("Data Processing Result: {:?}", result),
        Err(e) => println!("Data Processing Error: {:?}", e),
    }

    // Test API integration
    let mut api_args = HashMap::new();
    api_args.insert("endpoint".to_string(), json!("/api/users"));
    api_args.insert("method".to_string(), json!("GET"));

    match api_tool.call(api_args).await {
        Ok(result) => println!("API Integration Result: {:?}", result),
        Err(e) => println!("API Integration Error: {:?}", e),
    }

    println!("Integration patterns example completed");

    Ok(())
}
