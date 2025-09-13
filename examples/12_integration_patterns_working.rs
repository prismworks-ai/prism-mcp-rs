//! Example 12: Integration Patterns Working (Fixed Version)
//! Demonstrates integration patterns

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Database integration tool
struct DatabaseTool;

#[async_trait]
impl ToolHandler for DatabaseTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let query_type = arguments
            .get("query_type")
            .and_then(|v| v.as_str())
            .unwrap_or("select");
            
        let table = arguments
            .get("table")
            .and_then(|v| v.as_str())
            .unwrap_or("users");
        
        let result = match query_type {
            "select" => {
                json!({
                    "query_type": "select",
                    "table": table,
                    "rows_returned": 25,
                    "execution_time_ms": 45,
                    "data": [
                        {"id": 1, "name": "Alice", "email": "alice@example.com"},
                        {"id": 2, "name": "Bob", "email": "bob@example.com"}
                    ]
                })
            },
            "insert" => {
                json!({
                    "query_type": "insert",
                    "table": table,
                    "rows_affected": 1,
                    "execution_time_ms": 12,
                    "last_insert_id": 123
                })
            },
            "update" => {
                json!({
                    "query_type": "update",
                    "table": table,
                    "rows_affected": 3,
                    "execution_time_ms": 18
                })
            },
            _ => {
                return Err(McpError::validation(format!(
                    "Unsupported query type: {}", query_type
                )))
            }
        };
        
        Ok(ToolResult {
            content: vec![ContentBlock::Text {
                text: format!("Executed {} query on {} table", query_type, table),
                annotations: None,
                meta: None,
            }],
            is_error: Some(false),
            structured_content: Some(result),
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Integration Patterns Working Example");
    println!("===================================");
    
    let db_tool = DatabaseTool;
    
    // Test database query
    let mut db_args = HashMap::new();
    db_args.insert("query_type".to_string(), json!("select"));
    db_args.insert("table".to_string(), json!("users"));
    
    match db_tool.call(db_args).await {
        Ok(result) => println!("Database Result: {:?}", result),
        Err(e) => println!("Database Error: {:?}", e),
    }
    
    println!("Integration patterns working example completed");
    
    Ok(())
}
