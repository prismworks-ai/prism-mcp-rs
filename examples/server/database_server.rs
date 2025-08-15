// ! Database Server Example
// !
// ! A more complex MCP server that provides database-like functionality
// ! including storage, retrieval, and query capabilities using an in-memory store.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use prism_mcp_rs::{
    core::{
        error::{McpError, McpResult},
        resource::ResourceHandler,
        tool::ToolHandler,
    },
    protocol::types::{Content, Resource as ResourceInfo, ResourceContents, ToolResult},
    server::McpServer,
    transport::stdio::StdioServerTransport,
};

/// In-memory database store
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseRecord {
    id: String,
    data: Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// Shared database state
type Database = Arc<RwLock<HashMap<String, DatabaseRecord>>>;

/// Database tool handler for storing records
struct StoreHandler {
    db: Database,
}

#[async_trait]
impl ToolHandler for StoreHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let id = arguments
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'id' parameter".to_string()))?;

        let data = arguments
            .get("data")
            .ok_or_else(|| McpError::Validation("Missing 'data' parameter".to_string()))?;

        let now = chrono::Utc::now();
        let record = DatabaseRecord {
            id: id.to_string(),
            data: data.clone(),
            created_at: now,
            updated_at: now,
        };

        let mut db = self.db.write().await;
        let was_update = db.contains_key(id);
        db.insert(id.to_string(), record);

        let message = if was_update {
            format!("Updated record with ID: {id}")
        } else {
            format!("Created record with ID: {id}")
        };

        Ok(ToolResult {
            content: vec![Content::text(message)],
            is_error: None,
            structured_content: None,
            meta: None,
        })
    }
}

/// Database tool handler for retrieving records
struct RetrieveHandler {
    db: Database,
}

#[async_trait]
impl ToolHandler for RetrieveHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let id = arguments
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'id' parameter".to_string()))?;

        let db = self.db.read().await;

        match db.get(id) {
            Some(record) => {
                let response = json!({
                    "id": record.id,
                    "data": record.data,
                    "created_at": record.created_at.to_rfc3339(),
                    "updated_at": record.updated_at.to_rfc3339()
                });

                Ok(ToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&response)?)],
                    is_error: None,
                    structured_content: None,
                    meta: None,
                })
            }
            None => Ok(ToolResult {
                content: vec![Content::text(format!("No record found with ID: {id}"))],
                is_error: Some(true),
                structured_content: None,
                meta: None,
            }),
        }
    }
}

/// Database tool handler for listing all records
struct ListHandler {
    db: Database,
}

#[async_trait]
impl ToolHandler for ListHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10)
            .min(100) as usize; // Cap at 100 records

        let db = self.db.read().await;
        let records: Vec<_> = db
            .values()
            .take(limit)
            .map(|record| {
                json!({
                    "id": record.id,
                    "data": record.data,
                    "created_at": record.created_at.to_rfc3339(),
                    "updated_at": record.updated_at.to_rfc3339()
                })
            })
            .collect();

        let response = json!({
            "total": db.len(),
            "returned": records.len(),
            "records": records
        });

        Ok(ToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&response)?)],
            is_error: None,
            structured_content: None,
            meta: None,
        })
    }
}

/// Database tool handler for deleting records
struct DeleteHandler {
    db: Database,
}

#[async_trait]
impl ToolHandler for DeleteHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let id = arguments
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'id' parameter".to_string()))?;

        let mut db = self.db.write().await;

        match db.remove(id) {
            Some(_) => Ok(ToolResult {
                content: vec![Content::text(format!("Deleted record with ID: {id}"))],
                is_error: None,
                structured_content: None,
                meta: None,
            }),
            None => Ok(ToolResult {
                content: vec![Content::text(format!("No record found with ID: {id}"))],
                is_error: Some(true),
                structured_content: None,
                meta: None,
            }),
        }
    }
}

/// Resource handler for accessing database contents
struct DatabaseResourceHandler {
    db: Database,
}

#[async_trait]
impl ResourceHandler for DatabaseResourceHandler {
    async fn read(
        &self,
        uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        match uri {
            "db:/// all" => {
                let db = self.db.read().await;
                let records: Vec<_> = db.values().collect();

                let content = serde_json::to_string_pretty(&records)?;

                Ok(vec![ResourceContents::Text {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: content,
                    meta: None,
                }])
            }
            "db:/// schema" => {
                let schema = json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Unique identifier for the record"
                        },
                        "data": {
                            "description": "The stored data (can be any JSON value)"
                        },
                        "created_at": {
                            "type": "string",
                            "format": "date-time",
                            "description": "When the record was created"
                        },
                        "updated_at": {
                            "type": "string",
                            "format": "date-time",
                            "description": "When the record was last updated"
                        }
                    }
                });

                Ok(vec![ResourceContents::Text {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: serde_json::to_string_pretty(&schema)?,
                    meta: None,
                }])
            }
            _ if uri.starts_with("db:/// record/") => {
                let id = uri.strip_prefix("db:/// record/").unwrap();
                let db = self.db.read().await;

                match db.get(id) {
                    Some(record) => {
                        let content = serde_json::to_string_pretty(record)?;
                        Ok(vec![ResourceContents::Text {
                            uri: uri.to_string(),
                            mime_type: Some("application/json".to_string()),
                            text: content,
                            meta: None,
                        }])
                    }
                    None => Err(McpError::ResourceNotFound(uri.to_string())),
                }
            }
            _ => Err(McpError::ResourceNotFound(uri.to_string())),
        }
    }

    async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
        let db = self.db.read().await;
        let mut resources = vec![
            ResourceInfo {
                uri: "db:/// all".to_string(),
                name: "All Records".to_string(),
                description: Some("All records in the database".to_string()),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                size: None,
                title: Some("All Records".to_string()),
                meta: None,
            },
            ResourceInfo {
                uri: "db:/// schema".to_string(),
                name: "Database Schema".to_string(),
                description: Some("JSON schema for database records".to_string()),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                size: None,
                title: Some("Database Schema".to_string()),
                meta: None,
            },
        ];

        // Add individual record resources
        for id in db.keys() {
            resources.push(ResourceInfo {
                uri: format!("db:/// record/{id}"),
                name: format!("Record: {id}"),
                description: Some(format!("Individual database record with ID: {id}")),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                size: None,
                title: Some(format!("Record: {id}")),
                meta: None,
            });
        }

        Ok(resources)
    }

    async fn subscribe(&self, _uri: &str) -> McpResult<()> {
        // In a real implementation, this would set up change notifications
        Ok(())
    }

    async fn unsubscribe(&self, _uri: &str) -> McpResult<()> {
        // In a real implementation, this would remove change notifications
        Ok(())
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    #[cfg(feature = "tracing-subscriber")]
    tracing_subscriber::fmt::init();

    let mut server = McpServer::new("database-server".to_string(), "1.0.0".to_string());

    // Create shared database
    let db: Database = Arc::new(RwLock::new(HashMap::new()));

    // Add tools
    tracing::info!("Adding database tools...");

    server
        .add_tool(
            "store".to_string(),
            Some("Store a record in the database".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Unique identifier for the record"
                    },
                    "data": {
                        "description": "The data to store (can be any JSON value)"
                    }
                },
                "required": ["id", "data"]
            }),
            StoreHandler { db: db.clone() },
        )
        .await?;

    server
        .add_tool(
            "retrieve".to_string(),
            Some("Retrieve a record from the database".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Unique identifier of the record to retrieve"
                    }
                },
                "required": ["id"]
            }),
            RetrieveHandler { db: db.clone() },
        )
        .await?;

    server.add_tool(
        "list".to_string(),
        Some("List all records in the database".to_string()),
        json!({
            "type": "object",
            "properties": {
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of records to return (default: 10, max: 100)",
                    "minimum": 1,
                    "maximum": 100,
                    "default": 10
                }
            }
        }),
        ListHandler { db: db.clone() },
    ).await?;

    server
        .add_tool(
            "delete".to_string(),
            Some("Delete a record from the database".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Unique identifier of the record to delete"
                    }
                },
                "required": ["id"]
            }),
            DeleteHandler { db: db.clone() },
        )
        .await?;

    // Add database resource
    tracing::info!("Adding database resource...");

    server
        .add_resource_detailed(
            ResourceInfo {
                uri: "db:/// ".to_string(),
                name: "Database".to_string(),
                description: Some("In-memory database with JSON records".to_string()),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                size: None,
                title: Some("Database".to_string()),
                meta: None,
            },
            DatabaseResourceHandler { db: db.clone() },
        )
        .await?;

    // Insert some sample data
    tracing::info!("Inserting sample data...");
    {
        let mut db_guard = db.write().await;
        let now = chrono::Utc::now();

        db_guard.insert(
            "user1".to_string(),
            DatabaseRecord {
                id: "user1".to_string(),
                data: json!({
                    "name": "Alice",
                    "email": "alice@example.com",
                    "age": 30
                }),
                created_at: now,
                updated_at: now,
            },
        );

        db_guard.insert(
            "user2".to_string(),
            DatabaseRecord {
                id: "user2".to_string(),
                data: json!({
                    "name": "Bob",
                    "email": "bob@example.com",
                    "age": 25
                }),
                created_at: now,
                updated_at: now,
            },
        );
    }

    // Start the server
    tracing::info!("Starting database server...");
    let transport = StdioServerTransport::new();
    server.start(transport).await?;

    tracing::info!("Database server is running! Try these tools:");
    tracing::info!("  - store: Store a new record");
    tracing::info!("  - retrieve: Get a record by ID");
    tracing::info!("  - list: List all records");
    tracing::info!("  - delete: Remove a record");

    // Keep running until interrupted
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c");
    server.stop().await?;

    Ok(())
}
