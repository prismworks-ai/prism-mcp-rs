//! Tool system for MCP servers
//!
//! This module provides the abstraction for implementing and managing tools in MCP servers.
//! Tools are functions that can be called by clients to perform specific operations,
//! enhanced with complete parameter validation, type checking, and metadata support.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

use crate::core::error::{McpError, McpResult};
use crate::core::tool_metadata::{
    CategoryFilter, ImprovedToolMetadata, ToolBehaviorHints, ToolCategory, ToolDeprecation,
};
use crate::core::validation::{ParameterValidator, ValidationConfig};
use crate::protocol::types::{ContentBlock, ToolInfo, ToolInputSchema, ToolResult};

/// Trait for implementing tool handlers
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Execute the tool with the given arguments
    ///
    /// # Arguments
    /// * `arguments` - Tool arguments as key-value pairs
    ///
    /// # Returns
    /// Result containing the tool execution result or an error
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult>;
}

/// A registered tool with its handler, validation, and improved metadata
pub struct Tool {
    /// Information about the tool
    pub info: ToolInfo,
    /// Handler that implements the tool's functionality
    pub handler: Box<dyn ToolHandler>,
    /// Whether the tool is currently enabled
    pub enabled: bool,
    /// Parameter validator for input validation
    pub validator: Option<ParameterValidator>,
    /// improved metadata for tool behavior, categorization, and performance
    pub improved_metadata: ImprovedToolMetadata,
}

impl Tool {
    /// Create a new tool with the given information and handler
    ///
    /// # Arguments
    /// * `name` - Name of the tool
    /// * `description` - Optional description of the tool
    /// * `input_schema` - JSON schema describing the input parameters
    /// * `handler` - Implementation of the tool's functionality
    pub fn new<H>(
        name: String,
        description: Option<String>,
        input_schema: Value,
        handler: H,
    ) -> Self
    where
        H: ToolHandler + 'static,
    {
        // Create validator from schema
        let validator = if input_schema.is_object() {
            Some(ParameterValidator::new(input_schema.clone()))
        } else {
            None
        };

        Self {
            info: ToolInfo {
                name,
                description,
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties: input_schema
                        .get("properties")
                        .and_then(|p| p.as_object())
                        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
                    required: input_schema
                        .get("required")
                        .and_then(|r| r.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        }),
                    additional_properties: input_schema
                        .as_object()
                        .unwrap_or(&serde_json::Map::new())
                        .iter()
                        .filter(|(k, _)| !["type", "properties", "required"].contains(&k.as_str()))
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                },
                output_schema: None,
                annotations: None,
                title: None,
                meta: None,
            },
            handler: Box::new(handler),
            enabled: true,
            validator,
            improved_metadata: ImprovedToolMetadata::new(),
        }
    }

    /// Create a new tool with custom validation configuration
    pub fn with_validation<H>(
        name: String,
        description: Option<String>,
        input_schema: Value,
        handler: H,
        validation_config: ValidationConfig,
    ) -> Self
    where
        H: ToolHandler + 'static,
    {
        let mut tool = Self::new(name, description, input_schema.clone(), handler);
        if input_schema.is_object() {
            tool.validator = Some(ParameterValidator::with_config(
                input_schema,
                validation_config,
            ));
        }
        tool
    }

    /// Enable the tool
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the tool
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if the tool is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Execute the tool if it's enabled with parameter validation and performance tracking
    ///
    /// # Arguments
    /// * `arguments` - Tool arguments as key-value pairs
    ///
    /// # Returns
    /// Result containing the tool execution result or an error
    pub async fn call(&self, mut arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        if !self.enabled {
            return Err(McpError::validation(format!(
                "Tool '{}' is disabled",
                self.info.name
            )));
        }

        // Check for deprecation warning
        if let Some(warning) = self.improved_metadata.deprecation_warning() {
            eprintln!("Warning: {warning}");
        }

        // Validate and coerce parameters if validator is present
        if let Some(ref validator) = self.validator {
            validator.validate_and_coerce(&mut arguments).map_err(|e| {
                McpError::validation(format!(
                    "Tool '{}' parameter validation failed: {}",
                    self.info.name, e
                ))
            })?;
        }

        // Track execution time and outcome
        let start_time = Instant::now();
        let result = self.handler.call(arguments).await;
        let execution_time = start_time.elapsed();

        // Update performance metrics using interior mutability
        match &result {
            Ok(_) => self.improved_metadata.record_success(execution_time),
            Err(_) => self.improved_metadata.record_error(execution_time),
        }

        result
    }

    /// Execute the tool without validation or performance tracking (for specialized use cases)
    pub async fn call_unchecked(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        if !self.enabled {
            return Err(McpError::validation(format!(
                "Tool '{}' is disabled",
                self.info.name
            )));
        }

        self.handler.call(arguments).await
    }

    /// Validate parameters without executing the tool
    pub fn validate_parameters(&self, arguments: &mut HashMap<String, Value>) -> McpResult<()> {
        if let Some(ref validator) = self.validator {
            validator.validate_and_coerce(arguments).map_err(|e| {
                McpError::validation(format!(
                    "Tool '{}' parameter validation failed: {}",
                    self.info.name, e
                ))
            })
        } else {
            Ok(())
        }
    }

    // improved Metadata Management Methods

    /// Set behavior hints for the tool
    pub fn set_behavior_hints(&mut self, hints: ToolBehaviorHints) {
        self.improved_metadata.behavior_hints = hints;
    }

    /// Get behavior hints for the tool
    pub fn behavior_hints(&self) -> &ToolBehaviorHints {
        &self.improved_metadata.behavior_hints
    }

    /// Set category for the tool
    pub fn set_category(&mut self, category: ToolCategory) {
        self.improved_metadata.category = Some(category);
    }

    /// Get category for the tool
    pub fn category(&self) -> Option<&ToolCategory> {
        self.improved_metadata.category.as_ref()
    }

    /// Set version for the tool
    pub fn set_version(&mut self, version: String) {
        self.improved_metadata.version = Some(version);
    }

    /// Get version of the tool
    pub fn version(&self) -> Option<&String> {
        self.improved_metadata.version.as_ref()
    }

    /// Set author for the tool
    pub fn set_author(&mut self, author: String) {
        self.improved_metadata.author = Some(author);
    }

    /// Get author of the tool
    pub fn author(&self) -> Option<&String> {
        self.improved_metadata.author.as_ref()
    }

    /// Mark tool as deprecated
    pub fn deprecate(&mut self, deprecation: ToolDeprecation) {
        self.improved_metadata.deprecation = Some(deprecation);
    }

    /// Check if tool is deprecated
    pub fn is_deprecated(&self) -> bool {
        self.improved_metadata.is_deprecated()
    }

    /// Get deprecation warning if tool is deprecated
    pub fn deprecation_warning(&self) -> Option<String> {
        self.improved_metadata.deprecation_warning()
    }

    /// Get performance metrics for the tool
    pub fn performance_metrics(&self) -> crate::core::tool_metadata::ToolPerformanceMetrics {
        self.improved_metadata.get_performance_snapshot()
    }

    /// Add custom metadata field
    pub fn add_custom_metadata(&mut self, key: String, value: serde_json::Value) {
        self.improved_metadata.custom.insert(key, value);
    }

    /// Get custom metadata field
    pub fn get_custom_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.improved_metadata.custom.get(key)
    }

    /// Check if tool matches a category filter
    pub fn matches_category_filter(&self, filter: &CategoryFilter) -> bool {
        if let Some(ref category) = self.improved_metadata.category {
            category.matches_filter(filter)
        } else {
            // If no category set, only match empty filters
            filter.primary.is_none() && filter.secondary.is_none() && filter.tags.is_empty()
        }
    }

    /// Check if tool is suitable for caching based on behavior hints
    pub fn is_cacheable(&self) -> bool {
        self.improved_metadata
            .behavior_hints
            .cacheable
            .unwrap_or(false)
            || (self
                .improved_metadata
                .behavior_hints
                .read_only
                .unwrap_or(false)
                && self
                    .improved_metadata
                    .behavior_hints
                    .idempotent
                    .unwrap_or(false))
    }

    /// Check if tool is destructive
    pub fn is_destructive(&self) -> bool {
        self.improved_metadata
            .behavior_hints
            .destructive
            .unwrap_or(false)
    }

    /// Check if tool is read-only
    pub fn is_read_only(&self) -> bool {
        self.improved_metadata
            .behavior_hints
            .read_only
            .unwrap_or(false)
    }

    /// Check if tool is idempotent
    pub fn is_idempotent(&self) -> bool {
        self.improved_metadata
            .behavior_hints
            .idempotent
            .unwrap_or(false)
    }

    /// Check if tool requires authentication
    pub fn requires_auth(&self) -> bool {
        self.improved_metadata
            .behavior_hints
            .requires_auth
            .unwrap_or(false)
    }
}

impl std::fmt::Debug for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tool")
            .field("info", &self.info)
            .field("enabled", &self.enabled)
            .field("has_validator", &self.validator.is_some())
            .field("deprecated", &self.is_deprecated())
            .field("category", &self.improved_metadata.category)
            .field("version", &self.improved_metadata.version)
            .field("execution_count", &self.improved_metadata.execution_count())
            .field("success_rate", &self.improved_metadata.success_rate())
            .finish()
    }
}

/// Helper macro for creating tools with schema validation
///
/// # Examples
/// ```rust
/// use prism_mcp_rs::{tool, core::tool::ToolHandler};
/// use serde_json::json;
///
/// struct MyHandler;
/// #[async_trait::async_trait]
/// impl ToolHandler for MyHandler {
/// async fn call(&self, _args: std::collections::HashMap<String, serde_json::Value>) -> prism_mcp_rs::McpResult<prism_mcp_rs::protocol::types::ToolResult> {
/// // Implementation here
/// todo!()
/// }
/// }
///
/// let tool = tool!(
/// "my_tool",
/// "A sample tool",
/// json!({
/// "type": "object",
/// "properties": {
/// "input": { "type": "string" }
/// }
/// }),
/// MyHandler
/// );
/// ```
#[macro_export]
macro_rules! tool {
    ($name:expr_2021, $schema:expr_2021, $handler:expr_2021) => {
        $crate::core::tool::Tool::new($name.to_string(), None, $schema, $handler)
    };
    ($name:expr_2021, $description:expr_2021, $schema:expr_2021, $handler:expr_2021) => {
        $crate::core::tool::Tool::new(
            $name.to_string(),
            Some($description.to_string()),
            $schema,
            $handler,
        )
    };
}

// Common tool implementations

/// Simple echo tool for testing
pub struct EchoTool;

#[async_trait]
impl ToolHandler for EchoTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello, World!");

        Ok(ToolResult {
            content: vec![ContentBlock::Text {
                text: message.to_string(),
                annotations: None,
                meta: None,
            }],
            is_error: None,
            structured_content: None,
            meta: None,
        })
    }
}

/// Tool for adding two numbers
pub struct AdditionTool;

#[async_trait]
impl ToolHandler for AdditionTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let a = arguments
            .get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::validation("Missing or invalid 'a' parameter"))?;

        let b = arguments
            .get("b")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::validation("Missing or invalid 'b' parameter"))?;

        let result = a + b;

        Ok(ToolResult {
            content: vec![ContentBlock::Text {
                text: result.to_string(),
                annotations: None,
                meta: None,
            }],
            is_error: None,
            structured_content: None,
            meta: None,
        })
    }
}

/// Tool for getting current timestamp
pub struct TimestampTool;

#[async_trait]
impl ToolHandler for TimestampTool {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| McpError::internal(e.to_string()))?
            .as_secs();

        Ok(ToolResult {
            content: vec![ContentBlock::Text {
                text: timestamp.to_string(),
                annotations: None,
                meta: None,
            }],
            is_error: None,
            structured_content: None,
            meta: None,
        })
    }
}

/// Builder for creating tools with fluent API, validation, and enhanced metadata
pub struct ToolBuilder {
    name: String,
    description: Option<String>,
    input_schema: Option<Value>,
    validation_config: Option<ValidationConfig>,
    title: Option<String>,
    behavior_hints: ToolBehaviorHints,
    category: Option<ToolCategory>,
    version: Option<String>,
    author: Option<String>,
    deprecation: Option<ToolDeprecation>,
    custom_metadata: HashMap<String, serde_json::Value>,
}

impl ToolBuilder {
    /// Create a new tool builder with the given name
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema: None,
            validation_config: None,
            title: None,
            behavior_hints: ToolBehaviorHints::new(),
            category: None,
            version: None,
            author: None,
            deprecation: None,
            custom_metadata: HashMap::new(),
        }
    }

    /// Set the tool description
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the tool title (for UI display)
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the input schema
    pub fn schema(mut self, schema: Value) -> Self {
        self.input_schema = Some(schema);
        self
    }

    /// Set custom validation configuration
    pub fn validation_config(mut self, config: ValidationConfig) -> Self {
        self.validation_config = Some(config);
        self
    }

    /// Enable strict validation (no additional properties, strict types)
    pub fn strict_validation(mut self) -> Self {
        self.validation_config = Some(ValidationConfig {
            allow_additional: false,
            coerce_types: false,
            detailed_errors: true,
            max_string_length: Some(1000),
            max_array_length: Some(100),
            max_object_properties: Some(50),
        });
        self
    }

    /// Enable permissive validation (allow additional properties, type coercion)
    pub fn permissive_validation(mut self) -> Self {
        self.validation_config = Some(ValidationConfig {
            allow_additional: true,
            coerce_types: true,
            detailed_errors: false,
            max_string_length: None,
            max_array_length: None,
            max_object_properties: None,
        });
        self
    }

    // improved Metadata Builder Methods

    /// Set behavior hints for the tool
    pub fn behavior_hints(mut self, hints: ToolBehaviorHints) -> Self {
        self.behavior_hints = hints;
        self
    }

    /// Mark tool as read-only
    pub fn read_only(mut self) -> Self {
        self.behavior_hints = self.behavior_hints.read_only();
        self
    }

    /// Mark tool as destructive
    pub fn destructive(mut self) -> Self {
        self.behavior_hints = self.behavior_hints.destructive();
        self
    }

    /// Mark tool as idempotent
    pub fn idempotent(mut self) -> Self {
        self.behavior_hints = self.behavior_hints.idempotent();
        self
    }

    /// Mark tool as requiring authentication
    pub fn requires_auth(mut self) -> Self {
        self.behavior_hints = self.behavior_hints.requires_auth();
        self
    }

    /// Mark tool as potentially long-running
    pub fn long_running(mut self) -> Self {
        self.behavior_hints = self.behavior_hints.long_running();
        self
    }

    /// Mark tool as resource-intensive
    pub fn resource_intensive(mut self) -> Self {
        self.behavior_hints = self.behavior_hints.resource_intensive();
        self
    }

    /// Mark tool results as cacheable
    pub fn cacheable(mut self) -> Self {
        self.behavior_hints = self.behavior_hints.cacheable();
        self
    }

    /// Set tool category
    pub fn category(mut self, category: ToolCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set tool category with primary and secondary classification
    pub fn category_simple(mut self, primary: String, secondary: Option<String>) -> Self {
        let mut cat = ToolCategory::new(primary);
        if let Some(sec) = secondary {
            cat = cat.with_secondary(sec);
        }
        self.category = Some(cat);
        self
    }

    /// Add category tag
    pub fn tag(mut self, tag: String) -> Self {
        if let Some(ref mut category) = self.category {
            category.tags.insert(tag);
        } else {
            let mut cat = ToolCategory::new("general".to_string());
            cat.tags.insert(tag);
            self.category = Some(cat);
        }
        self
    }

    /// Set tool version
    pub fn version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set tool author
    pub fn author<S: Into<String>>(mut self, author: S) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Mark tool as deprecated
    pub fn deprecated(mut self, deprecation: ToolDeprecation) -> Self {
        self.deprecation = Some(deprecation);
        self
    }

    /// Mark tool as deprecated with simple reason
    pub fn deprecated_simple<S: Into<String>>(mut self, reason: S) -> Self {
        self.deprecation = Some(ToolDeprecation::new(reason.into()));
        self
    }

    /// Add custom metadata field
    pub fn custom_metadata<S: Into<String>>(mut self, key: S, value: serde_json::Value) -> Self {
        self.custom_metadata.insert(key.into(), value);
        self
    }

    /// Build the tool with the given handler
    pub fn build<H>(self, handler: H) -> McpResult<Tool>
    where
        H: ToolHandler + 'static,
    {
        let schema = self.input_schema.unwrap_or_else(|| {
            serde_json::json!({
                "type": "object",
                "properties": {},
                "additionalProperties": true
            })
        });

        let mut tool = if let Some(config) = self.validation_config {
            Tool::with_validation(self.name, self.description, schema, handler, config)
        } else {
            Tool::new(self.name, self.description, schema, handler)
        };

        // Set title if provided
        if let Some(title) = self.title {
            tool.info.title = Some(title);
        }

        // Apply improved metadata
        let mut improved_metadata =
            ImprovedToolMetadata::new().with_behavior_hints(self.behavior_hints);

        if let Some(category) = self.category {
            improved_metadata = improved_metadata.with_category(category);
        }

        if let Some(version) = self.version {
            improved_metadata = improved_metadata.with_version(version);
        }

        if let Some(author) = self.author {
            improved_metadata = improved_metadata.with_author(author);
        }

        if let Some(deprecation) = self.deprecation {
            improved_metadata = improved_metadata.deprecated(deprecation);
        }

        // Add custom metadata fields
        for (key, value) in self.custom_metadata {
            improved_metadata = improved_metadata.with_custom_field(key, value);
        }

        tool.improved_metadata = improved_metadata;

        Ok(tool)
    }

    /// Build the tool with validation chain - allows chaining parameter validation
    pub fn build_with_validation_chain<H>(
        self,
        handler: H,
        validation_fn: impl Fn(&mut HashMap<String, Value>) -> McpResult<()> + Send + Sync + 'static,
    ) -> McpResult<ValidationChainTool>
    where
        H: ToolHandler + 'static,
    {
        let tool = self.build(handler)?;
        Ok(ValidationChainTool {
            tool,
            custom_validator: Box::new(validation_fn),
        })
    }
}

/// Tool wrapper that supports custom validation chains
/// Type alias for validation function to reduce complexity
type ValidationFunction = Box<dyn Fn(&mut HashMap<String, Value>) -> McpResult<()> + Send + Sync>;

pub struct ValidationChainTool {
    tool: Tool,
    custom_validator: ValidationFunction,
}

#[async_trait]
impl ToolHandler for ValidationChainTool {
    async fn call(&self, mut arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        // Run custom validation first
        (self.custom_validator)(&mut arguments)?;

        // Then run the tool's built-in validation and execution
        self.tool.call(arguments).await
    }
}

// ============================================================================
// improved Tool Creation Helpers and Macros
// ============================================================================

/// Create a validated tool with typed parameters
#[macro_export]
macro_rules! validated_tool {
    (
        name: $name:expr_2021,
        description: $desc:expr_2021,
        parameters: {
            $( $param_name:ident: $param_type:ident $( ( $( $constraint:ident: $value:expr_2021 ),* ) )? ),*
        },
        handler: $handler:expr_2021
    ) => {{
        use $crate::core::validation::{create_tool_schema, param_schema};

        let params = vec![
            $(
                {
                    let base_schema = param_schema!($param_type stringify!($param_name));
                    // Apply constraints if any
                    $(
                        // This would need more complex macro expansion for constraints
                        // For now, we'll use the base schema
                    )?
                    base_schema
                }
            ),*
        ];

        let required = vec![ $( stringify!($param_name) ),* ];
        let schema = create_tool_schema(params, required);

        $crate::core::tool::Tool::new(
            $name.to_string(),
            Some($desc.to_string()),
            schema,
            $handler
        )
    }};
}

/// Helper function to create a simple string parameter tool
pub fn create_string_tool<H>(
    name: &str,
    description: &str,
    param_name: &str,
    param_description: &str,
    handler: H,
) -> Tool
where
    H: ToolHandler + 'static,
{
    use serde_json::json;

    let schema = json!({
        "type": "object",
        "properties": {
            param_name: {
                "type": "string",
                "description": param_description
            }
        },
        "required": [param_name]
    });

    Tool::new(
        name.to_string(),
        Some(description.to_string()),
        schema,
        handler,
    )
}

/// Helper function to create a tool with multiple typed parameters
pub fn create_typed_tool<H>(
    name: &str,
    description: &str,
    parameters: Vec<(&str, &str, Value)>, // (name, description, schema)
    required: Vec<&str>,
    handler: H,
) -> Tool
where
    H: ToolHandler + 'static,
{
    use serde_json::{Map, json};

    let mut properties = Map::new();
    for (param_name, param_desc, param_schema) in parameters {
        let mut schema_with_desc = param_schema;
        if let Some(obj) = schema_with_desc.as_object_mut() {
            obj.insert("description".to_string(), json!(param_desc));
        }
        properties.insert(param_name.to_string(), schema_with_desc);
    }

    let schema = json!({
        "type": "object",
        "properties": properties,
        "required": required
    });

    Tool::new(
        name.to_string(),
        Some(description.to_string()),
        schema,
        handler,
    )
}

/// Trait for tools that can provide their own parameter validation
pub trait ValidatedToolHandler: ToolHandler {
    /// Get the JSON schema for this tool's parameters
    fn parameter_schema() -> Value;

    /// Get validation configuration for this tool
    fn validation_config() -> ValidationConfig {
        ValidationConfig::default()
    }

    /// Create a tool instance with built-in validation
    fn create_tool(name: String, description: Option<String>, handler: Self) -> Tool
    where
        Self: Sized + 'static,
    {
        Tool::with_validation(
            name,
            description,
            Self::parameter_schema(),
            handler,
            Self::validation_config(),
        )
    }
}

// ============================================================================
// improved Built-in Tool Examples
// ============================================================================

/// Calculator tool with comprehensive validation
pub struct CalculatorTool;

#[async_trait]
impl ToolHandler for CalculatorTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let operation = arguments
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::validation("Missing 'operation' parameter"))?;

        let a = arguments
            .get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::validation("Missing or invalid 'a' parameter"))?;

        let b = arguments
            .get("b")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::validation("Missing or invalid 'b' parameter"))?;

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Ok(ToolResult {
                        content: vec![ContentBlock::Text {
                            text: "Error: Division by zero".to_string(),
                            annotations: None,
                            meta: None,
                        }],
                        is_error: Some(true),
                        structured_content: Some(serde_json::json!({
                            "error": "division_by_zero",
                            "message": "Cannot divide by zero"
                        })),
                        meta: None,
                    });
                }
                a / b
            }
            _ => {
                return Err(McpError::validation(format!(
                    "Unsupported operation: {operation}"
                )));
            }
        };

        Ok(ToolResult {
            content: vec![ContentBlock::Text {
                text: result.to_string(),
                annotations: None,
                meta: None,
            }],
            is_error: None,
            structured_content: Some(serde_json::json!({
                "operation": operation,
                "operands": [a, b],
                "result": result
            })),
            meta: None,
        })
    }
}

impl ValidatedToolHandler for CalculatorTool {
    fn parameter_schema() -> Value {
        use crate::core::validation::create_tool_schema;
        use crate::param_schema;

        create_tool_schema(
            vec![
                param_schema!(enum "operation", values: ["add", "subtract", "multiply", "divide"]),
                param_schema!(number "a", min: -1000000, max: 1000000),
                param_schema!(number "b", min: -1000000, max: 1000000),
            ],
            vec!["operation", "a", "b"],
        )
    }

    fn validation_config() -> ValidationConfig {
        ValidationConfig {
            allow_additional: false,
            coerce_types: true,
            detailed_errors: true,
            max_string_length: Some(20),
            max_array_length: Some(10),
            max_object_properties: Some(10),
        }
    }
}

/// Text processing tool with string validation
pub struct TextProcessorTool;

#[async_trait]
impl ToolHandler for TextProcessorTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::validation("Missing 'text' parameter"))?;

        let operation = arguments
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("uppercase");

        let result = match operation {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "reverse" => text.chars().rev().collect(),
            "word_count" => text.split_whitespace().count().to_string(),
            "char_count" => text.len().to_string(),
            _ => {
                return Err(McpError::validation(format!(
                    "Unsupported operation: {operation}"
                )));
            }
        };

        Ok(ToolResult {
            content: vec![ContentBlock::Text {
                text: result.clone(),
                annotations: None,
                meta: None,
            }],
            is_error: None,
            structured_content: Some(serde_json::json!({
                "original_text": text,
                "operation": operation,
                "result": result,
                "length": text.len()
            })),
            meta: None,
        })
    }
}

impl ValidatedToolHandler for TextProcessorTool {
    fn parameter_schema() -> Value {
        use crate::core::validation::create_tool_schema;
        use crate::param_schema;

        create_tool_schema(
            vec![
                param_schema!(string "text", min: 1, max: 10000),
                param_schema!(enum "operation", values: ["uppercase", "lowercase", "reverse", "word_count", "char_count"]),
            ],
            vec!["text"],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Content;
    use serde_json::json;

    #[tokio::test]
    async fn test_echo_tool() {
        let tool = EchoTool;
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!("test message"));

        let result = tool.call(args).await.unwrap();
        match &result.content[0] {
            Content::Text { text, .. } => assert_eq!(text, "test message"),
            _ => panic!("Expected text content"),
        }
    }

    #[tokio::test]
    async fn test_addition_tool() {
        let tool = AdditionTool;
        let mut args = HashMap::new();
        args.insert("a".to_string(), json!(5.0));
        args.insert("b".to_string(), json!(3.0));

        let result = tool.call(args).await.unwrap();
        match &result.content[0] {
            Content::Text { text, .. } => assert_eq!(text, "8"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_tool_creation() {
        let tool = Tool::new(
            "test_tool".to_string(),
            Some("Test tool".to_string()),
            json!({"type": "object"}),
            EchoTool,
        );

        assert_eq!(tool.info.name, "test_tool");
        assert_eq!(tool.info.description, Some("Test tool".to_string()));
        assert!(tool.is_enabled());
    }

    #[test]
    fn test_tool_enable_disable() {
        let mut tool = Tool::new(
            "test_tool".to_string(),
            None,
            json!({"type": "object"}),
            EchoTool,
        );

        assert!(tool.is_enabled());

        tool.disable();
        assert!(!tool.is_enabled());

        tool.enable();
        assert!(tool.is_enabled());
    }

    #[tokio::test]
    async fn test_disabled_tool() {
        let mut tool = Tool::new(
            "test_tool".to_string(),
            None,
            json!({"type": "object"}),
            EchoTool,
        );

        tool.disable();

        let result = tool.call(HashMap::new()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::Validation(msg) => assert!(msg.contains("disabled")),
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_tool_builder() {
        let tool = ToolBuilder::new("test")
            .description("A test tool")
            .schema(json!({"type": "object", "properties": {"x": {"type": "number"}}}))
            .build(EchoTool)
            .unwrap();

        assert_eq!(tool.info.name, "test");
        assert_eq!(tool.info.description, Some("A test tool".to_string()));
        assert!(tool.validator.is_some());
    }

    #[test]
    fn test_improved_tool_builder() {
        let tool = ToolBuilder::new("improved_test")
            .title("improved Test Tool")
            .description("A test tool with improved features")
            .strict_validation()
            .schema(json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "minLength": 2},
                    "age": {"type": "integer", "minimum": 0}
                },
                "required": ["name"]
            }))
            .build(EchoTool)
            .unwrap();

        assert_eq!(tool.info.name, "improved_test");
        assert_eq!(tool.info.title, Some("improved Test Tool".to_string()));
        assert!(tool.validator.is_some());
    }

    #[tokio::test]
    async fn test_parameter_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "minLength": 2},
                "age": {"type": "integer", "minimum": 0, "maximum": 150}
            },
            "required": ["name", "age"]
        });

        let tool = Tool::new(
            "validation_test".to_string(),
            Some("Test validation".to_string()),
            schema,
            EchoTool,
        );

        // Valid parameters
        let mut valid_args = HashMap::new();
        valid_args.insert("name".to_string(), json!("Alice"));
        valid_args.insert("age".to_string(), json!(25));
        assert!(tool.validate_parameters(&mut valid_args).is_ok());

        // Missing required parameter
        let mut invalid_args = HashMap::new();
        invalid_args.insert("name".to_string(), json!("Bob"));
        assert!(tool.validate_parameters(&mut invalid_args).is_err());

        // Invalid parameter type with coercion
        let mut coercible_args = HashMap::new();
        coercible_args.insert("name".to_string(), json!("Charlie"));
        coercible_args.insert("age".to_string(), json!("30")); // String that can be coerced to number
        assert!(tool.validate_parameters(&mut coercible_args).is_ok());
        // After validation, should be coerced to number
        assert_eq!(coercible_args.get("age").unwrap().as_i64(), Some(30));
    }

    #[tokio::test]
    async fn test_calculator_tool() {
        let tool = CalculatorTool::create_tool(
            "calculator".to_string(),
            Some("complete calculator".to_string()),
            CalculatorTool,
        );

        // Test addition
        let mut args = HashMap::new();
        args.insert("operation".to_string(), json!("add"));
        args.insert("a".to_string(), json!(5));
        args.insert("b".to_string(), json!(3));

        let result = tool.call(args).await.unwrap();
        assert_eq!(
            result.content[0],
            ContentBlock::Text {
                text: "8".to_string(),
                annotations: None,
                meta: None,
            }
        );
        assert!(result.structured_content.is_some());

        // Test division by zero
        let mut args = HashMap::new();
        args.insert("operation".to_string(), json!("divide"));
        args.insert("a".to_string(), json!(10));
        args.insert("b".to_string(), json!(0));

        let result = tool.call(args).await.unwrap();
        assert_eq!(result.is_error, Some(true));
        if let ContentBlock::Text { text, .. } = &result.content[0] {
            assert!(text.contains("Division by zero"));
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_text_processor_tool() {
        let tool = TextProcessorTool::create_tool(
            "text_processor".to_string(),
            Some("Text processing utility".to_string()),
            TextProcessorTool,
        );

        // Test uppercase
        let mut args = HashMap::new();
        args.insert("text".to_string(), json!("hello world"));
        args.insert("operation".to_string(), json!("uppercase"));

        let result = tool.call(args.clone()).await.unwrap();
        assert_eq!(
            result.content[0],
            ContentBlock::Text {
                text: "HELLO WORLD".to_string(),
                annotations: None,
                meta: None,
            }
        );

        // Test word count
        args.insert("operation".to_string(), json!("word_count"));
        let result = tool.call(args).await.unwrap();
        assert_eq!(
            result.content[0],
            ContentBlock::Text {
                text: "2".to_string(),
                annotations: None,
                meta: None,
            }
        );
    }

    #[test]
    fn test_create_typed_tool() {
        let tool = create_typed_tool(
            "typed_test",
            "A typed parameter test tool",
            vec![
                (
                    "username",
                    "User's name",
                    json!({"type": "string", "minLength": 3}),
                ),
                (
                    "age",
                    "User's age",
                    json!({"type": "integer", "minimum": 0}),
                ),
                (
                    "active",
                    "Whether user is active",
                    json!({"type": "boolean"}),
                ),
            ],
            vec!["username", "age"],
            EchoTool,
        );

        assert_eq!(tool.info.name, "typed_test");
        assert!(tool.validator.is_some());

        // Check that schema was built correctly
        let schema = &tool.info.input_schema;
        assert!(schema.properties.is_some());
        let props = schema.properties.as_ref().unwrap();
        assert!(props.contains_key("username"));
        assert!(props.contains_key("age"));
        assert!(props.contains_key("active"));
    }

    #[test]
    fn test_validation_config_options() {
        // Test strict validation
        let strict_tool = ToolBuilder::new("strict")
            .strict_validation()
            .build(EchoTool)
            .unwrap();
        assert!(strict_tool.validator.is_some());

        // Test permissive validation
        let permissive_tool = ToolBuilder::new("permissive")
            .permissive_validation()
            .build(EchoTool)
            .unwrap();
        assert!(permissive_tool.validator.is_some());
    }
}

// ============================================================================
// Extension Trait for Better Ergonomics
// ============================================================================

/// Extension trait for HashMap to make parameter extraction easier
pub trait ParameterExt {
    /// Extract a required string parameter
    fn get_string(&self, key: &str) -> McpResult<&str>;

    /// Extract an optional string parameter
    fn get_optional_string(&self, key: &str) -> Option<&str>;

    /// Extract a required number parameter
    fn get_number(&self, key: &str) -> McpResult<f64>;

    /// Extract an optional number parameter
    fn get_optional_number(&self, key: &str) -> Option<f64>;

    /// Extract a required integer parameter
    fn get_integer(&self, key: &str) -> McpResult<i64>;

    /// Extract an optional integer parameter
    fn get_optional_integer(&self, key: &str) -> Option<i64>;

    /// Extract a required boolean parameter
    fn get_boolean(&self, key: &str) -> McpResult<bool>;

    /// Extract an optional boolean parameter
    fn get_optional_boolean(&self, key: &str) -> Option<bool>;
}

impl ParameterExt for HashMap<String, Value> {
    fn get_string(&self, key: &str) -> McpResult<&str> {
        self.get(key).and_then(|v| v.as_str()).ok_or_else(|| {
            McpError::validation(format!("Missing or invalid string parameter: {key}"))
        })
    }

    fn get_optional_string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    fn get_number(&self, key: &str) -> McpResult<f64> {
        self.get(key).and_then(|v| v.as_f64()).ok_or_else(|| {
            McpError::validation(format!("Missing or invalid number parameter: {key}"))
        })
    }

    fn get_optional_number(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_f64())
    }

    fn get_integer(&self, key: &str) -> McpResult<i64> {
        self.get(key).and_then(|v| v.as_i64()).ok_or_else(|| {
            McpError::validation(format!("Missing or invalid integer parameter: {key}"))
        })
    }

    fn get_optional_integer(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_i64())
    }

    fn get_boolean(&self, key: &str) -> McpResult<bool> {
        self.get(key).and_then(|v| v.as_bool()).ok_or_else(|| {
            McpError::validation(format!("Missing or invalid boolean parameter: {key}"))
        })
    }

    fn get_optional_boolean(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }
}

#[cfg(test)]
mod improved_tests {
    use super::*;
    use crate::core::tool_metadata::*;
    use crate::prelude::ToolHandler;
    use std::time::Duration;
    use tokio;

    // Test handler for basic tool functionality
    struct TestHandler {
        result: String,
        should_fail: bool,
    }

    #[async_trait]
    impl ToolHandler for TestHandler {
        async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
            if self.should_fail {
                Err(McpError::validation("Test error".to_string()))
            } else {
                Ok(ToolResult {
                    content: vec![ContentBlock::Text {
                        text: self.result.clone(),
                        annotations: None,
                        meta: None,
                    }],
                    is_error: None,
                    structured_content: None,
                    meta: None,
                })
            }
        }
    }

    #[tokio::test]
    async fn test_improved_tool_builder() {
        let handler = TestHandler {
            result: "test result".to_string(),
            should_fail: false,
        };

        let tool = ToolBuilder::new("test_tool")
            .description("A test tool")
            .title("Test Tool")
            .version("1.0.0")
            .author("Test Author")
            .read_only()
            .idempotent()
            .cacheable()
            .category_simple("data".to_string(), Some("analysis".to_string()))
            .tag("testing".to_string())
            .tag("utility".to_string())
            .custom_metadata("priority".to_string(), serde_json::Value::from("high"))
            .build(handler)
            .expect("Failed to build tool");

        assert_eq!(tool.info.name, "test_tool");
        assert_eq!(tool.info.description, Some("A test tool".to_string()));
        assert_eq!(tool.info.title, Some("Test Tool".to_string()));
        assert_eq!(tool.version(), Some(&"1.0.0".to_string()));
        assert_eq!(tool.author(), Some(&"Test Author".to_string()));
        assert!(tool.is_read_only());
        assert!(tool.is_idempotent());
        assert!(tool.is_cacheable());
        assert!(!tool.is_destructive());
        assert!(!tool.requires_auth());

        let category = tool.category().unwrap();
        assert_eq!(category.primary, "data");
        assert_eq!(category.secondary, Some("analysis".to_string()));
        assert!(category.tags.contains("testing"));
        assert!(category.tags.contains("utility"));

        let custom_priority = tool.get_custom_metadata("priority");
        assert_eq!(custom_priority, Some(&serde_json::Value::from("high")));
    }

    #[tokio::test]
    async fn test_performance_tracking() {
        let handler = TestHandler {
            result: "success".to_string(),
            should_fail: false,
        };

        let tool = ToolBuilder::new("performance_test")
            .build(handler)
            .expect("Failed to build tool");

        // Initial state
        let metrics = tool.performance_metrics();
        assert_eq!(metrics.execution_count, 0);
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.error_count, 0);

        // Execute tool successfully
        let result = tool.call(HashMap::new()).await;
        assert!(result.is_ok());

        // Check updated metrics
        let metrics = tool.performance_metrics();
        assert_eq!(metrics.execution_count, 1);
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.success_rate, 100.0);
        assert!(metrics.average_execution_time > Duration::from_nanos(0));
    }

    #[tokio::test]
    async fn test_performance_tracking_with_errors() {
        let handler = TestHandler {
            result: "".to_string(),
            should_fail: true,
        };

        let tool = ToolBuilder::new("error_test")
            .build(handler)
            .expect("Failed to build tool");

        // Execute tool with error
        let result = tool.call(HashMap::new()).await;
        assert!(result.is_err());

        // Check error metrics
        let metrics = tool.performance_metrics();
        assert_eq!(metrics.execution_count, 1);
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.success_rate, 0.0);
    }

    #[tokio::test]
    async fn test_deprecation_warning() {
        let handler = TestHandler {
            result: "deprecated result".to_string(),
            should_fail: false,
        };

        let deprecation = ToolDeprecation::new("This tool is outdated".to_string())
            .with_replacement("new_tool".to_string())
            .with_severity(DeprecationSeverity::High);

        let tool = ToolBuilder::new("deprecated_tool")
            .deprecated(deprecation)
            .build(handler)
            .expect("Failed to build tool");

        assert!(tool.is_deprecated());
        let warning = tool.deprecation_warning().unwrap();
        assert!(warning.contains("deprecated"));
        assert!(warning.contains("outdated"));
        assert!(warning.contains("new_tool"));
    }

    #[tokio::test]
    async fn test_category_filtering() {
        let category = ToolCategory::new("file".to_string())
            .with_secondary("read".to_string())
            .with_tag("filesystem".to_string())
            .with_tag("utility".to_string());

        let handler = TestHandler {
            result: "filtered result".to_string(),
            should_fail: false,
        };

        let tool = ToolBuilder::new("filterable_tool")
            .category(category)
            .build(handler)
            .expect("Failed to build tool");

        // Test primary category filter
        let filter = CategoryFilter::new().with_primary("file".to_string());
        assert!(tool.matches_category_filter(&filter));

        let filter = CategoryFilter::new().with_primary("network".to_string());
        assert!(!tool.matches_category_filter(&filter));

        // Test tag filter
        let filter = CategoryFilter::new().with_tag("filesystem".to_string());
        assert!(tool.matches_category_filter(&filter));

        let filter = CategoryFilter::new().with_tag("nonexistent".to_string());
        assert!(!tool.matches_category_filter(&filter));

        // Test secondary category filter
        let filter = CategoryFilter::new().with_secondary("read".to_string());
        assert!(tool.matches_category_filter(&filter));

        let filter = CategoryFilter::new().with_secondary("write".to_string());
        assert!(!tool.matches_category_filter(&filter));
    }

    #[tokio::test]
    async fn test_behavior_hints() {
        let hints = ToolBehaviorHints::new()
            .read_only()
            .idempotent()
            .cacheable()
            .requires_auth()
            .long_running()
            .resource_intensive();

        let handler = TestHandler {
            result: "hints result".to_string(),
            should_fail: false,
        };

        let tool = ToolBuilder::new("hints_tool")
            .behavior_hints(hints)
            .build(handler)
            .expect("Failed to build tool");

        assert!(tool.is_read_only());
        assert!(tool.is_idempotent());
        assert!(tool.is_cacheable());
        assert!(tool.requires_auth());
        assert!(!tool.is_destructive());

        let behavior_hints = tool.behavior_hints();
        assert_eq!(behavior_hints.read_only, Some(true));
        assert_eq!(behavior_hints.idempotent, Some(true));
        assert_eq!(behavior_hints.cacheable, Some(true));
        assert_eq!(behavior_hints.requires_auth, Some(true));
        assert_eq!(behavior_hints.long_running, Some(true));
        assert_eq!(behavior_hints.resource_intensive, Some(true));
        assert_eq!(behavior_hints.destructive, None);
    }

    #[tokio::test]
    async fn test_tool_enabling_disabling() {
        let handler = TestHandler {
            result: "enabled result".to_string(),
            should_fail: false,
        };

        let mut tool = ToolBuilder::new("enable_test")
            .build(handler)
            .expect("Failed to build tool");

        assert!(tool.is_enabled());

        // Disable tool
        tool.disable();
        assert!(!tool.is_enabled());

        // Try to call disabled tool
        let result = tool.call(HashMap::new()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disabled"));

        // Re-enable tool
        tool.enable();
        assert!(tool.is_enabled());

        // Should work again
        let result = tool.call(HashMap::new()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_custom_metadata() {
        let handler = TestHandler {
            result: "metadata result".to_string(),
            should_fail: false,
        };

        let mut tool = ToolBuilder::new("metadata_tool")
            .custom_metadata("priority".to_string(), serde_json::Value::from("high"))
            .custom_metadata("team".to_string(), serde_json::Value::from("backend"))
            .build(handler)
            .expect("Failed to build tool");

        assert_eq!(
            tool.get_custom_metadata("priority"),
            Some(&serde_json::Value::from("high"))
        );
        assert_eq!(
            tool.get_custom_metadata("team"),
            Some(&serde_json::Value::from("backend"))
        );
        assert_eq!(tool.get_custom_metadata("nonexistent"), None);

        // Add metadata after creation
        tool.add_custom_metadata(
            "environment".to_string(),
            serde_json::Value::from("production"),
        );
        assert_eq!(
            tool.get_custom_metadata("environment"),
            Some(&serde_json::Value::from("production"))
        );
    }

    #[test]
    fn test_tool_debug_format() {
        let handler = TestHandler {
            result: "debug result".to_string(),
            should_fail: false,
        };

        let tool = ToolBuilder::new("debug_tool")
            .version("2.0.0")
            .category_simple("debug".to_string(), None)
            .build(handler)
            .expect("Failed to build tool");

        let debug_str = format!("{tool:?}");
        assert!(debug_str.contains("debug_tool"));
        assert!(debug_str.contains("enabled"));
        assert!(debug_str.contains("execution_count"));
        assert!(debug_str.contains("success_rate"));
    }
}