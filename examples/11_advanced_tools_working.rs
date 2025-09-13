//! Example 11: Advanced Tools (Fixed Version)
//! Demonstrates advanced tool patterns

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Advanced file system tool
struct FileSystemTool;

#[async_trait]
impl ToolHandler for FileSystemTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let operation = arguments
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("list");

        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        match operation {
            "list" => {
                let content = format!("Listing files in: {}", path);
                Ok(ToolResult {
                    content: vec![ContentBlock::Text {
                        text: content,
                        annotations: None,
                        meta: None,
                    }],
                    is_error: Some(false),
                    structured_content: Some(json!({
                        "operation": "list",
                        "path": path,
                        "files": ["file1.txt", "file2.txt", "file3.txt"]
                    })),
                    meta: None,
                })
            }
            "read" => {
                let content = format!("Reading file: {}", path);
                Ok(ToolResult {
                    content: vec![ContentBlock::Text {
                        text: content,
                        annotations: None,
                        meta: None,
                    }],
                    is_error: Some(false),
                    structured_content: Some(json!({
                        "operation": "read",
                        "path": path,
                        "content": "Sample file content here..."
                    })),
                    meta: None,
                })
            }
            _ => Err(McpError::validation(format!(
                "Unsupported operation: {}",
                operation
            ))),
        }
    }
}

/// Advanced calculation tool with validation
struct AdvancedCalculatorTool;

#[async_trait]
impl ToolHandler for AdvancedCalculatorTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let expression = arguments
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::validation("Missing 'expression' parameter"))?;

        // Simple expression evaluation (in real implementation, use a proper parser)
        let result = match expression {
            "2+2" => 4.0,
            "10*5" => 50.0,
            "100/4" => 25.0,
            "3^2" => 9.0,
            _ => {
                return Err(McpError::validation(format!(
                    "Unsupported expression: {}",
                    expression
                )))
            }
        };

        Ok(ToolResult {
            content: vec![ContentBlock::Text {
                text: format!("{} = {}", expression, result),
                annotations: None,
                meta: None,
            }],
            is_error: Some(false),
            structured_content: Some(json!({
                "expression": expression,
                "result": result,
                "type": "calculation"
            })),
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Advanced Tools Example");
    println!("=====================");

    let fs_tool = FileSystemTool;
    let calc_tool = AdvancedCalculatorTool;

    // Test file system tool
    let mut fs_args = HashMap::new();
    fs_args.insert("operation".to_string(), json!("list"));
    fs_args.insert("path".to_string(), json!("/home/user"));

    match fs_tool.call(fs_args).await {
        Ok(result) => println!("FS Tool Result: {:?}", result),
        Err(e) => println!("FS Tool Error: {:?}", e),
    }

    // Test calculator tool
    let mut calc_args = HashMap::new();
    calc_args.insert("expression".to_string(), json!("2+2"));

    match calc_tool.call(calc_args).await {
        Ok(result) => println!("Calc Tool Result: {:?}", result),
        Err(e) => println!("Calc Tool Error: {:?}", e),
    }

    println!("Advanced tools example completed");

    Ok(())
}
