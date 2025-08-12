# Plugin Component Types Reference

## Overview

The Prism MCP SDK plugin system supports four distinct component types that plugins can provide. This document details each component type, its purpose, interface, and implementation patterns.

## Related Documentation

- [Plugin Development Guide](PLUGIN_GUIDE.md) - Complete guide to creating and distributing plugins
- [API Documentation](https://docs.rs/prism-mcp-rs) - Full API documentation (available after publication)
- [Example Plugins](../examples/README.md#plugin-development) - Working examples

## Component Type Matrix

| Component | Purpose | Handler Trait | Primary Use Cases |
|-----------|---------|--------------|-------------------|
| **Tool** | Execute operations | `ToolHandler` | Data processing, calculations, API calls |
| **Resource** | Provide data access | `ResourceHandler` | Database queries, file access, configuration |
| **Prompt** | Generate LLM templates | `PromptHandler` | Conversation starters, query builders |
| **Completion** | Provide autocomplete | `CompletionHandler` | Parameter hints, path suggestions |

## Detailed Component Specifications

### Tools

#### Definition
Tools are executable functions that accept arguments and return results. They represent actions or operations that can be performed by the MCP server.

#### Interface
```rust
#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult>;
}
```

#### Characteristics
- **Stateless or Stateful**: Can maintain internal state if needed
- **Asynchronous**: All operations are async by default
- **Typed Arguments**: Arguments are validated against schemas
- **Rich Results**: Can return text, structured data, or errors

#### Implementation Example
```rust
pub struct CalculatorTool;

#[async_trait]
impl ToolHandler for CalculatorTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let a = arguments.get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::InvalidParams {
                message: "Missing parameter 'a'".to_string(),
            })?;
        
        let b = arguments.get("b")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::InvalidParams {
                message: "Missing parameter 'b'".to_string(),
            })?;
        
        let operation = arguments.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("add");
        
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" if b != 0.0 => a / b,
            _ => return Err(McpError::InvalidParams {
                message: "Invalid operation".to_string(),
            }),
        };
        
        Ok(ToolResult {
            content: vec![ContentBlock::text(format!("{:.2}", result))],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}
```

#### Common Patterns
- **CRUD Operations**: Create, read, update, delete
- **Data Transformation**: Format conversion, encoding/decoding
- **External Integration**: API calls, service interactions
- **Computation**: Mathematical operations, analysis

### Resources

#### Definition
Resources provide read-only access to data through URI-based addressing. They represent queryable data sources that can be accessed with parameters.

#### Interface
```rust
#[async_trait]
pub trait ResourceHandler: Send + Sync {
    async fn read(
        &self,
        uri: &str,
        params: &HashMap<String, String>
    ) -> McpResult<Vec<ResourceContents>>;
}
```

#### Characteristics
- **URI-Based**: Access through standardized URI patterns
- **Parameterized**: Support query parameters for filtering
- **Read-Only**: Resources are not modified through this interface
- **Multiple Formats**: Can return text, binary, or structured data

#### Implementation Example
```rust
pub struct ConfigResource {
    config_store: Arc<RwLock<HashMap<String, Value>>>,
}

#[async_trait]
impl ResourceHandler for ConfigResource {
    async fn read(
        &self,
        uri: &str,
        params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        if uri == "config://all" {
            let store = self.config_store.read().await;
            let json = serde_json::to_string_pretty(&*store)?;
            
            Ok(vec![ResourceContents::Text {
                uri: uri.to_string(),
                mime_type: Some("application/json".to_string()),
                text: json,
                meta: None,
            }])
        } else if let Some(key) = uri.strip_prefix("config://") {
            let store = self.config_store.read().await;
            
            match store.get(key) {
                Some(value) => {
                    Ok(vec![ResourceContents::Text {
                        uri: uri.to_string(),
                        mime_type: Some("application/json".to_string()),
                        text: serde_json::to_string_pretty(value)?,
                        meta: None,
                    }])
                }
                None => Err(McpError::ResourceNotFound(uri.to_string()))
            }
        } else {
            Err(McpError::ResourceNotFound(uri.to_string()))
        }
    }
}
```

#### URI Patterns
- `scheme://path` - Basic pattern
- `db://table/users` - Database table access
- `file:///path/to/file` - File system access
- `api://endpoint/data` - API endpoint access
- `config://key` - Configuration access

### Prompts

#### Definition
Prompts generate structured message templates for LLM interactions. They create conversation contexts with system and user messages.

#### Interface
```rust
#[async_trait]
pub trait PromptHandler: Send + Sync {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<PromptResult>;
}
```

#### Characteristics
- **Template Generation**: Create dynamic prompts from arguments
- **Role-Based Messages**: System, user, and assistant roles
- **Context Building**: Establish conversation context
- **Reusable Templates**: Consistent prompt patterns

#### Implementation Example
```rust
pub struct CodeReviewPrompt;

#[async_trait]
impl PromptHandler for CodeReviewPrompt {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<PromptResult> {
        let code = arguments.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams {
                message: "Missing 'code' parameter".to_string(),
            })?;
        
        let language = arguments.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let focus = arguments.get("focus")
            .and_then(|v| v.as_str())
            .unwrap_or("general");
        
        let messages = vec![
            PromptMessage {
                role: Role::System,
                content: Content::text(format!(
                    "You are an expert {} code reviewer focusing on {}.",
                    language, focus
                )),
            },
            PromptMessage {
                role: Role::User,
                content: Content::text(format!(
                    "Please review this code:\n\n```{}\n{}\n```",
                    language, code
                )),
            },
        ];
        
        Ok(PromptResult {
            description: Some(format!("{} code review prompt", language)),
            messages,
        })
    }
}
```

#### Common Templates
- **Analysis Prompts**: Data analysis, code review, document review
- **Generation Prompts**: Content creation, code generation
- **Query Builders**: SQL queries, API requests
- **Conversation Starters**: Support chat, tutoring

### Completions

#### Definition
Completions provide intelligent autocomplete suggestions for tool arguments, resource URIs, and prompt parameters.

#### Interface
```rust
#[async_trait]
pub trait CompletionHandler: Send + Sync {
    async fn complete(
        &self,
        reference: &CompletionReference,
        argument: &CompletionArgument,
        context: Option<&CompletionContext>
    ) -> McpResult<Vec<String>>;
}
```

#### Characteristics
- **Context-Aware**: Suggestions based on current context
- **Type-Specific**: Different completions for different types
- **Partial Matching**: Complete from partial input
- **Dynamic Generation**: Real-time suggestion generation

#### Implementation Example
```rust
pub struct FilePathCompletion {
    root_directory: PathBuf,
}

#[async_trait]
impl CompletionHandler for FilePathCompletion {
    async fn complete(
        &self,
        reference: &CompletionReference,
        argument: &CompletionArgument,
        _context: Option<&CompletionContext>,
    ) -> McpResult<Vec<String>> {
        if let CompletionReference::Tool { name } = reference {
            if name == "read_file" && argument.name == "path" {
                let partial_path = &argument.value;
                let suggestions = self.find_matching_paths(partial_path).await?;
                return Ok(suggestions);
            }
        }
        Ok(vec![])
    }
    
    async fn find_matching_paths(&self, partial: &str) -> McpResult<Vec<String>> {
        let mut matches = Vec::new();
        let search_dir = if partial.contains('/') {
            self.root_directory.join(partial.rsplit_once('/').unwrap().0)
        } else {
            self.root_directory.clone()
        };
        
        if search_dir.exists() {
            let entries = tokio::fs::read_dir(search_dir).await?;
            // Collect matching entries
            // ... implementation details ...
        }
        
        Ok(matches)
    }
}
```

#### Completion Types
- **Path Completions**: File paths, directory paths
- **Schema Completions**: Database tables, columns
- **Command Completions**: Available commands, options
- **Value Completions**: Enum values, configuration options

## Component Interaction

Components within a plugin can interact and share state:

```rust
pub struct IntegratedPlugin {
    data_store: Arc<RwLock<DataStore>>,
}

impl IntegratedPlugin {
    pub fn new() -> Self {
        Self {
            data_store: Arc::new(RwLock::new(DataStore::new())),
        }
    }
    
    pub fn create_tool(&self) -> impl ToolHandler {
        DataTool {
            store: self.data_store.clone(),
        }
    }
    
    pub fn create_resource(&self) -> impl ResourceHandler {
        DataResource {
            store: self.data_store.clone(),
        }
    }
    
    pub fn create_completion(&self) -> impl CompletionHandler {
        DataCompletion {
            store: self.data_store.clone(),
        }
    }
}
```

## Best Practices

### General Guidelines

1. **Single Responsibility**: Each component should have a clear, focused purpose
2. **Error Handling**: Provide descriptive error messages
3. **Validation**: Validate all inputs thoroughly
4. **Documentation**: Document expected arguments and return values
5. **Testing**: Write comprehensive tests for each component

### Performance Considerations

1. **Async Operations**: Use async for I/O operations
2. **Resource Pooling**: Share expensive resources (connections, clients)
3. **Caching**: Cache frequently accessed data when appropriate
4. **Lazy Loading**: Load resources only when needed

### Security Considerations

1. **Input Sanitization**: Always sanitize user inputs
2. **Access Control**: Implement proper authorization checks
3. **Resource Limits**: Enforce limits on resource consumption
4. **Audit Logging**: Log significant operations

## Component Selection Guide

| If you need to... | Use this component |
|-------------------|--------------------|
| Execute an operation or action | Tool |
| Provide access to data | Resource |
| Generate LLM conversation templates | Prompt |
| Provide autocomplete suggestions | Completion |
| Process and return data | Tool |
| Read configuration or state | Resource |
| Create reusable conversation patterns | Prompt |
| Enhance user input experience | Completion |

## Examples by Use Case

### Database Plugin
- **Tools**: insert_record, update_record, delete_record
- **Resources**: db://tables, db://schema
- **Prompts**: sql_query_builder, migration_generator
- **Completions**: table_names, column_names

### File System Plugin
- **Tools**: create_file, delete_file, move_file
- **Resources**: file:///path/to/files
- **Prompts**: file_operation_assistant
- **Completions**: file_paths, directory_paths

### API Integration Plugin
- **Tools**: api_request, api_post
- **Resources**: api://endpoints
- **Prompts**: api_query_builder
- **Completions**: endpoint_urls, parameter_names

## Further Reading

- [Plugin Development Guide](PLUGIN_GUIDE.md)
- [API Reference](https://docs.rs/prism-mcp-rs/latest/)
- [Example Plugins](https://github.com/prismworks-ai/plugin-examples)