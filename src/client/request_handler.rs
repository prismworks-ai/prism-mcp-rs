// ! Client-side request handling for server-initiated requests
// !
// ! Module provides the infrastructure for handling requests that the MCP server
// ! sends to the client, enabling true bidirectional communication as defined in the
// ! MCP 2025-06-18 specification
// !
// ! Key features:
// ! - Sampling/createMessage request handling (LLM integration)
// ! - Roots/list request handling (file system access)
// ! - Elicitation/create request handling (user input forms)
// ! - Ping request handling (connectivity testing)
// !
// ! # Example
// ! ```rust
// ! use prism_mcp_rs::client::{McpClient, InteractiveClientRequestHandler};
// !
// ! let mut client = McpClient::new("my-app".to_string(), "1.0.0".to_string());
// ! let handler = InteractiveClientRequestHandler::new("My Application")
// ! .add_root("file:///home/user", Some("Home Directory"))
// ! .auto_accept_elicitation(true);
// ! client.set_request_handler(handler);
// ! ```

use async_trait::async_trait;
use std::collections::HashMap;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, info};

use crate::core::error::{McpError, McpResult};
use crate::protocol::messages::*;
use crate::protocol::types::*;

/// Trait for handling server-initiated requests
///
/// Trait defines the interface for processing requests that the MCP server
/// sends to the client. Each method corresponds to a specific MCP method that
/// servers can invoke on clients for bidirectional communication.
#[async_trait]
pub trait ClientRequestHandler: Send + Sync {
    /// Handle sampling/createMessage request from the server
    ///
    /// Method is called when the server wants the client to generate
    /// a message using an LLM (Large Language Model). This enables the server
    /// to leverage the client's AI capabilities.
    ///
    /// # Arguments
    /// * `params` - Parameters for message creation including conversation context
    ///
    /// # Returns
    /// Result containing the generated message or an error
    async fn handle_create_message(
        &self,
        params: CreateMessageParams,
    ) -> McpResult<CreateMessageResult>;

    /// Handle roots/list request from the server
    ///
    /// Method is called when the server wants to know what root directories
    /// or file system areas the client has access to. This is used for file system
    /// integration and resource discovery.
    ///
    /// # Arguments
    /// * `params` - Parameters for root listing (typically empty)
    ///
    /// # Returns
    /// Result containing available roots or an error
    async fn handle_list_roots(&self, params: ListRootsParams) -> McpResult<ListRootsResult>;

    /// Handle elicitation/create request from the server
    ///
    /// Method is called when the server wants to collect structured input
    /// from the user through a form-like interface. This enables interactive
    /// workflows where the server needs user confirmation or additional data.
    ///
    /// # Arguments
    /// * `params` - Parameters defining the form schema and message
    ///
    /// # Returns
    /// Result containing user's response (accept/decline/cancel) and form data
    async fn handle_elicit(&self, params: ElicitParams) -> McpResult<ElicitResult>;

    /// Handle ping request from the server
    ///
    /// Method is called when the server wants to test connectivity
    /// or measure latency to the client.
    ///
    /// # Arguments
    /// * `params` - Ping parameters (typically empty)
    ///
    /// # Returns
    /// Result containing ping response or an error
    async fn handle_ping(&self, params: PingParams) -> McpResult<PingResult>;
}

/// Default implementation that rejects all server requests
///
/// This is the default request handler that provides minimal responses
/// to all server-initiated requests. It's used when no custom handler
/// is configured.
pub struct DefaultClientRequestHandler;

#[async_trait]
impl ClientRequestHandler for DefaultClientRequestHandler {
    async fn handle_create_message(
        &self,
        _params: CreateMessageParams,
    ) -> McpResult<CreateMessageResult> {
        Err(McpError::Protocol(
            "LLM sampling not supported - no handler configured".to_string(),
        ))
    }

    async fn handle_list_roots(&self, _params: ListRootsParams) -> McpResult<ListRootsResult> {
        Ok(ListRootsResult {
            roots: vec![],
            meta: None,
        })
    }

    async fn handle_elicit(&self, _params: ElicitParams) -> McpResult<ElicitResult> {
        Err(McpError::Protocol(
            "User interaction not supported - no handler configured".to_string(),
        ))
    }

    async fn handle_ping(&self, _params: PingParams) -> McpResult<PingResult> {
        Ok(PingResult { meta: None })
    }
}

/// Interactive client request handler with user prompts
///
/// This handler provides interactive responses to server requests,
/// prompting the user for input when needed. It's suitable for
/// command-line applications and interactive environments.
///
/// # Features
/// - Interactive user prompts for elicitation requests
/// - Configurable root directories
/// - Optional auto-acceptance for testing
/// - Console-based user interaction
pub struct InteractiveClientRequestHandler {
    /// Application name for user prompts
    app_name: String,
    /// Available root directories
    roots: Vec<Root>,
    /// Whether to auto-accept elicitation requests (for testing)
    auto_accept_elicitation: bool,
    /// Whether to show verbose output
    verbose: bool,
}

impl InteractiveClientRequestHandler {
    /// Create a new interactive request handler
    ///
    /// # Arguments
    /// * `app_name` - Name of the application for user prompts
    ///
    /// # Example
    /// ```rust
    /// use prism_mcp_rs::client::InteractiveClientRequestHandler;
    ///
    /// let handler = InteractiveClientRequestHandler::new("My App");
    /// ```
    pub fn new<S: Into<String>>(app_name: S) -> Self {
        Self {
            app_name: app_name.into(),
            roots: Vec::new(),
            auto_accept_elicitation: false,
            verbose: false,
        }
    }

    /// Add a root directory that the client can access
    ///
    /// # Arguments
    /// * `uri` - URI of the root directory (e.g., "file:///home/user")
    /// * `name` - Optional display name for the root
    ///
    /// # Example
    /// ```rust
    /// use prism_mcp_rs::client::InteractiveClientRequestHandler;
    ///
    /// let handler = InteractiveClientRequestHandler::new("My App")
    /// .add_root("file:///home/user", Some("Home Directory"));
    /// ```
    pub fn add_root<S: Into<String>>(mut self, uri: S, name: Option<S>) -> Self {
        let mut root = Root::new(uri.into());
        if let Some(n) = name {
            root = root.with_name(n.into());
        }
        self.roots.push(root);
        self
    }

    /// Set whether to auto-accept elicitation requests
    ///
    /// When enabled, all elicitation requests are automatically accepted
    /// with empty form data. This is useful for testing and automation.
    ///
    /// # Arguments
    /// * `auto_accept` - Whether to auto-accept requests
    ///
    /// # Example
    /// ```rust
    /// use prism_mcp_rs::client::InteractiveClientRequestHandler;
    ///
    /// let handler = InteractiveClientRequestHandler::new("My App")
    /// .auto_accept_elicitation(true); // For testing
    /// ```
    pub fn auto_accept_elicitation(mut self, auto_accept: bool) -> Self {
        self.auto_accept_elicitation = auto_accept;
        self
    }

    /// Set verbose output mode
    ///
    /// When enabled, the handler will print detailed information about
    /// each request it processes.
    ///
    /// # Arguments
    /// * `verbose` - Whether to enable verbose output
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Add common root directories for the current platform
    ///
    /// Method adds platform-appropriate root directories like
    /// home directory, documents, etc.
    ///
    /// # Example
    /// ```rust
    /// use prism_mcp_rs::client::InteractiveClientRequestHandler;
    ///
    /// let handler = InteractiveClientRequestHandler::new("My App")
    /// .add_common_roots();
    /// ```
    /// Add common file system roots (requires dirs feature)
    #[cfg(feature = "dirs")]
    pub fn add_common_roots(mut self) -> Self {
        // Add home directory
        if let Some(home_dir) = dirs::home_dir() {
            let home_uri = format!("file://{}", home_dir.display());
            self.roots
                .push(Root::new(home_uri).with_name("Home Directory".to_string()));
        }

        // Add documents directory
        if let Some(docs_dir) = dirs::document_dir() {
            let docs_uri = format!("file://{}", docs_dir.display());
            self.roots
                .push(Root::new(docs_uri).with_name("Documents".to_string()));
        }

        // Add desktop directory
        if let Some(desktop_dir) = dirs::desktop_dir() {
            let desktop_uri = format!("file://{}", desktop_dir.display());
            self.roots
                .push(Root::new(desktop_uri).with_name("Desktop".to_string()));
        }

        self
    }

    /// Add common file system roots (no-op when dirs feature is disabled)
    #[cfg(not(feature = "dirs"))]
    pub fn add_common_roots(self) -> Self {
        // No-op when dirs feature is not enabled
        self
    }

    /// Prompt user for yes/no input
    async fn prompt_yes_no(&self, message: &str) -> McpResult<bool> {
        println!("\n[{}] {}", self.app_name, message);
        print!("Continue? (y/N): ");
        io::stdout().flush().await.map_err(McpError::io)?;

        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut input = String::new();

        reader.read_line(&mut input).await.map_err(McpError::io)?;
        let input = input.trim().to_lowercase();

        Ok(input == "y" || input == "yes")
    }

    /// Collect form data from user based on schema
    async fn collect_form_data(
        &self,
        schema: &ElicitationSchema,
    ) -> McpResult<HashMap<String, serde_json::Value>> {
        let mut form_data = HashMap::new();

        println!(
            "\n[{}] Please provide the following information:",
            self.app_name
        );

        for (field_name, field_def) in &schema.properties {
            let prompt = match field_def {
                PrimitiveSchemaDefinition::String {
                    title, description, ..
                } => {
                    format!(
                        "{} ({}): ",
                        title.as_ref().unwrap_or(field_name),
                        description.as_ref().unwrap_or(&"string".to_string())
                    )
                }
                PrimitiveSchemaDefinition::Number {
                    title, description, ..
                } => {
                    format!(
                        "{} ({}): ",
                        title.as_ref().unwrap_or(field_name),
                        description.as_ref().unwrap_or(&"number".to_string())
                    )
                }
                PrimitiveSchemaDefinition::Integer {
                    title, description, ..
                } => {
                    format!(
                        "{} ({}): ",
                        title.as_ref().unwrap_or(field_name),
                        description.as_ref().unwrap_or(&"integer".to_string())
                    )
                }
                PrimitiveSchemaDefinition::Boolean {
                    title, description, ..
                } => {
                    format!(
                        "{} ({}) (y/N): ",
                        title.as_ref().unwrap_or(field_name),
                        description.as_ref().unwrap_or(&"yes/no".to_string())
                    )
                }
            };

            print!("{prompt}");
            io::stdout().flush().await.map_err(McpError::io)?;

            let stdin = io::stdin();
            let mut reader = BufReader::new(stdin);
            let mut input = String::new();

            reader.read_line(&mut input).await.map_err(McpError::io)?;
            let input = input.trim();

            // Parse input based on field type
            let value = match field_def {
                PrimitiveSchemaDefinition::String { .. } => {
                    serde_json::Value::String(input.to_string())
                }
                PrimitiveSchemaDefinition::Number { .. } => {
                    if let Ok(num) = input.parse::<f64>() {
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(num)
                                .unwrap_or_else(|| serde_json::Number::from(0)),
                        )
                    } else {
                        return Err(McpError::validation(format!("Invalid number: {input}")));
                    }
                }
                PrimitiveSchemaDefinition::Integer { .. } => {
                    if let Ok(num) = input.parse::<i64>() {
                        serde_json::Value::Number(serde_json::Number::from(num))
                    } else {
                        return Err(McpError::validation(format!("Invalid integer: {input}")));
                    }
                }
                PrimitiveSchemaDefinition::Boolean { .. } => {
                    let input_lower = input.to_lowercase();
                    let value = input_lower == "y" || input_lower == "yes" || input_lower == "true";
                    serde_json::Value::Bool(value)
                }
            };

            form_data.insert(field_name.clone(), value);
        }

        Ok(form_data)
    }
}

#[async_trait]
impl ClientRequestHandler for InteractiveClientRequestHandler {
    async fn handle_create_message(
        &self,
        params: CreateMessageParams,
    ) -> McpResult<CreateMessageResult> {
        if self.verbose {
            info!(
                "Server requested LLM sampling with {} messages, max_tokens: {}",
                params.messages.len(),
                params.max_tokens
            );
        }

        // For now, we don't have LLM integration, so we provide a helpful error
        // In a real implementation, this would integrate with OpenAI, Anthropic, etc.
        Err(McpError::Protocol(
            "LLM sampling not implemented - this would require integration with an AI service like OpenAI or Anthropic".to_string()
        ))
    }

    async fn handle_list_roots(&self, _params: ListRootsParams) -> McpResult<ListRootsResult> {
        if self.verbose {
            info!(
                "Server requested roots list - returning {} configured roots",
                self.roots.len()
            );
        }

        Ok(ListRootsResult {
            roots: self.roots.clone(),
            meta: None,
        })
    }

    async fn handle_elicit(&self, params: ElicitParams) -> McpResult<ElicitResult> {
        if self.verbose {
            info!("Server requested user input: {}", params.message);
        }

        // Auto-accept mode for testing
        if self.auto_accept_elicitation {
            debug!("Auto-accepting elicitation request");
            return Ok(ElicitResult {
                action: ElicitationAction::Accept,
                content: Some(HashMap::new()),
                meta: None,
            });
        }

        // Show the message to the user
        println!("\n=== Server Request ===");
        println!("{}", params.message);
        println!("======================");

        // Ask if user wants to proceed
        let proceed = self
            .prompt_yes_no("The server is requesting your input.")
            .await?;

        if !proceed {
            return Ok(ElicitResult {
                action: ElicitationAction::Decline,
                content: None,
                meta: None,
            });
        }

        // Collect form data
        let form_data = match self.collect_form_data(&params.requested_schema).await {
            Ok(data) => data,
            Err(_) => {
                return Ok(ElicitResult {
                    action: ElicitationAction::Cancel,
                    content: None,
                    meta: None,
                });
            }
        };

        Ok(ElicitResult {
            action: ElicitationAction::Accept,
            content: Some(form_data),
            meta: None,
        })
    }

    async fn handle_ping(&self, _params: PingParams) -> McpResult<PingResult> {
        if self.verbose {
            debug!("Server ping received - responding");
        }

        Ok(PingResult { meta: None })
    }
}

/// Simple request handler for headless/automated environments
///
/// This handler provides non-interactive responses suitable for
/// automated environments, testing, and headless applications.
/// It never prompts the user and provides sensible defaults.
pub struct AutomatedClientRequestHandler {
    /// Available root directories
    roots: Vec<Root>,
    /// Default responses for elicitation requests
    default_responses: HashMap<String, serde_json::Value>,
}

impl AutomatedClientRequestHandler {
    /// Create a new automated request handler
    ///
    /// # Example
    /// ```rust
    /// use prism_mcp_rs::client::AutomatedClientRequestHandler;
    ///
    /// let handler = AutomatedClientRequestHandler::new();
    /// ```
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            default_responses: HashMap::new(),
        }
    }

    /// Add a root directory
    ///
    /// # Arguments
    /// * `uri` - URI of the root directory
    /// * `name` - Optional display name
    pub fn add_root<S: Into<String>>(mut self, uri: S, name: Option<S>) -> Self {
        let mut root = Root::new(uri.into());
        if let Some(n) = name {
            root = root.with_name(n.into());
        }
        self.roots.push(root);
        self
    }

    /// Set a default response for elicitation fields
    ///
    /// # Arguments
    /// * `field_name` - Name of the form field
    /// * `value` - Default value to provide
    pub fn set_default_response<S: Into<String>>(
        mut self,
        field_name: S,
        value: serde_json::Value,
    ) -> Self {
        self.default_responses.insert(field_name.into(), value);
        self
    }
}

impl Default for AutomatedClientRequestHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ClientRequestHandler for AutomatedClientRequestHandler {
    async fn handle_create_message(
        &self,
        _params: CreateMessageParams,
    ) -> McpResult<CreateMessageResult> {
        // Automated environments can't provide LLM sampling
        Err(McpError::Protocol(
            "LLM sampling not available in automated mode".to_string(),
        ))
    }

    async fn handle_list_roots(&self, _params: ListRootsParams) -> McpResult<ListRootsResult> {
        Ok(ListRootsResult {
            roots: self.roots.clone(),
            meta: None,
        })
    }

    async fn handle_elicit(&self, params: ElicitParams) -> McpResult<ElicitResult> {
        // Generate form data using defaults or empty values
        let mut form_data = HashMap::new();

        for (field_name, field_def) in &params.requested_schema.properties {
            let value = if let Some(default_value) = self.default_responses.get(field_name) {
                default_value.clone()
            } else {
                // Provide sensible defaults based on field type
                match field_def {
                    PrimitiveSchemaDefinition::String { .. } => {
                        serde_json::Value::String(String::new())
                    }
                    PrimitiveSchemaDefinition::Number { .. } => {
                        serde_json::Value::Number(serde_json::Number::from(0))
                    }
                    PrimitiveSchemaDefinition::Integer { .. } => {
                        serde_json::Value::Number(serde_json::Number::from(0))
                    }
                    PrimitiveSchemaDefinition::Boolean { default, .. } => {
                        serde_json::Value::Bool(default.unwrap_or(false))
                    }
                }
            };

            form_data.insert(field_name.clone(), value);
        }

        Ok(ElicitResult {
            action: ElicitationAction::Accept,
            content: Some(form_data),
            meta: None,
        })
    }

    async fn handle_ping(&self, _params: PingParams) -> McpResult<PingResult> {
        Ok(PingResult { meta: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_handler() {
        let handler = DefaultClientRequestHandler;

        // Test ping (should succeed)
        let ping_result = handler.handle_ping(PingParams { meta: None }).await;
        assert!(ping_result.is_ok());

        // Test roots list (should return empty)
        let roots_result = handler
            .handle_list_roots(ListRootsParams { meta: None })
            .await;
        assert!(roots_result.is_ok());
        assert!(roots_result.unwrap().roots.is_empty());

        // Test create message (should fail)
        let create_params = CreateMessageParams {
            messages: vec![],
            max_tokens: 100,
            system_prompt: None,
            include_context: None,
            temperature: None,
            stop_sequences: None,
            model_preferences: None,
            metadata: None,
            meta: None,
        };
        let create_result = handler.handle_create_message(create_params).await;
        assert!(create_result.is_err());

        // Test elicit (should fail)
        let elicit_params = ElicitParams {
            message: "Test message".to_string(),
            requested_schema: ElicitationSchema {
                schema_type: "object".to_string(),
                properties: HashMap::new(),
                required: None,
            },
            meta: None,
        };
        let elicit_result = handler.handle_elicit(elicit_params).await;
        assert!(elicit_result.is_err());
    }

    #[tokio::test]
    async fn test_interactive_handler_builder() {
        let handler = InteractiveClientRequestHandler::new("Test App")
            .add_root("file:///test", Some("Test Root"))
            .auto_accept_elicitation(true)
            .verbose(true);

        // Test roots list
        let roots_result = handler
            .handle_list_roots(ListRootsParams { meta: None })
            .await;
        assert!(roots_result.is_ok());
        let roots = roots_result.unwrap().roots;
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].uri, "file:///test");
        assert_eq!(roots[0].name, Some("Test Root".to_string()));

        // Test elicit with auto-accept
        let elicit_params = ElicitParams {
            message: "Test message".to_string(),
            requested_schema: ElicitationSchema {
                schema_type: "object".to_string(),
                properties: HashMap::new(),
                required: None,
            },
            meta: None,
        };
        let elicit_result = handler.handle_elicit(elicit_params).await;
        assert!(elicit_result.is_ok());
        let result = elicit_result.unwrap();
        assert!(matches!(result.action, ElicitationAction::Accept));
        assert!(result.content.is_some());
    }

    #[tokio::test]
    async fn test_automated_handler() {
        let handler = AutomatedClientRequestHandler::new()
            .add_root("file:///automated", Some("Automated Root"))
            .set_default_response(
                "test_field",
                serde_json::Value::String("test_value".to_string()),
            );

        // Test roots list
        let roots_result = handler
            .handle_list_roots(ListRootsParams { meta: None })
            .await;
        assert!(roots_result.is_ok());
        let roots = roots_result.unwrap().roots;
        assert_eq!(roots.len(), 1);

        // Test elicit with default responses
        let mut properties = HashMap::new();
        properties.insert(
            "test_field".to_string(),
            PrimitiveSchemaDefinition::String {
                title: Some("Test Field".to_string()),
                description: None,
                min_length: None,
                max_length: None,
                format: None,
                enum_values: None,
                enum_names: None,
            },
        );

        let elicit_params = ElicitParams {
            message: "Test message".to_string(),
            requested_schema: ElicitationSchema {
                schema_type: "object".to_string(),
                properties,
                required: None,
            },
            meta: None,
        };

        let elicit_result = handler.handle_elicit(elicit_params).await;
        assert!(elicit_result.is_ok());
        let result = elicit_result.unwrap();
        assert!(matches!(result.action, ElicitationAction::Accept));

        let content = result.content.unwrap();
        assert_eq!(
            content.get("test_field"),
            Some(&serde_json::Value::String("test_value".to_string()))
        );
    }
}
