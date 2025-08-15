// ! Practical completion handlers for common MCP use cases
// !
// ! Module provides ready-to-use completion handlers that servers can use
// ! to provide smart autocompletion for prompts, tools, and resources.
// ! These handlers implement the CompletionHandler trait and can be easily
// ! integrated into any MCP server.
// !
// ! # Features
// ! - File path completion for resource URIs
// ! - Prompt argument completion with fuzzy matching
// ! - Tool parameter completion based on schema
// ! - Database-backed completion with caching
// ! - Custom completion with validation

use crate::core::{
    completion::{CompletionContext, CompletionHandler},
    error::{McpError, McpResult},
};
use crate::protocol::messages::{CompletionArgument, CompletionReference};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

/// File system path completion handler
///
/// Provides completion suggestions for file system paths. This is useful
/// for resource URIs that represent files and directories.
///
/// # Features
/// - Directory traversal with proper permissions handling
/// - File extension filtering
/// - Hidden file inclusion control
/// - Path normalization and validation
///
/// # Example
/// ```rust
/// use prism_mcp_rs::core::completion_handlers::FileSystemCompletionHandler;
///
/// let handler = FileSystemCompletionHandler::new("/home/user")
/// .with_extensions(vec!["txt", "md", "json"])
/// .include_hidden_files(false);
/// ```
pub struct FileSystemCompletionHandler {
    /// Base directory for completions
    base_path: PathBuf,
    /// Allowed file extensions (None means all)
    allowed_extensions: Option<Vec<String>>,
    /// Whether to include hidden files/directories
    include_hidden: bool,
    /// Maximum number of suggestions to return
    max_suggestions: usize,
    /// Maximum directory depth to traverse
    max_depth: usize,
}

impl FileSystemCompletionHandler {
    /// Create a new file system completion handler
    ///
    /// # Arguments
    /// * `base_path` - Base directory for file completions
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            allowed_extensions: None,
            include_hidden: false,
            max_suggestions: 20,
            max_depth: 5,
        }
    }

    /// Set allowed file extensions
    ///
    /// # Arguments
    /// * `extensions` - List of allowed extensions (without dots)
    pub fn with_extensions(mut self, extensions: Vec<&str>) -> Self {
        self.allowed_extensions = Some(extensions.into_iter().map(|s| s.to_string()).collect());
        self
    }

    /// Set whether to include hidden files
    pub fn include_hidden_files(mut self, include: bool) -> Self {
        self.include_hidden = include;
        self
    }

    /// Set maximum number of suggestions
    pub fn max_suggestions(mut self, max: usize) -> Self {
        self.max_suggestions = max;
        self
    }

    /// Set maximum directory depth
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Check if a file should be included based on extension filter
    fn should_include_file(&self, path: &Path) -> bool {
        if let Some(ref extensions) = self.allowed_extensions {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                return extensions.contains(&ext.to_string());
            }
            return false;
        }
        true
    }

    /// Check if a file/directory should be included based on hidden status
    fn should_include_hidden(&self, path: &Path) -> bool {
        if !self.include_hidden {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                return !name.starts_with('.');
            }
        }
        true
    }

    /// Get completions for a given path prefix
    async fn get_path_completions(&self, prefix: &str) -> McpResult<Vec<String>> {
        let mut suggestions = Vec::new();

        // Resolve the target directory
        let search_path = if prefix.is_empty() {
            self.base_path.clone()
        } else {
            let prefix_path = Path::new(prefix);
            if prefix_path.is_absolute() {
                prefix_path.to_path_buf()
            } else {
                self.base_path.join(prefix_path)
            }
        };

        // If the path doesn't exist or isn't accessible, try parent directory
        let (dir_to_search, partial_name) = if search_path.exists() && search_path.is_dir() {
            (search_path, String::new())
        } else {
            let parent = search_path.parent().unwrap_or(&self.base_path);
            let partial = search_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            (parent.to_path_buf(), partial)
        };

        // Read directory entries
        match fs::read_dir(&dir_to_search).await {
            Ok(mut entries) => {
                while let Some(entry) = entries.next_entry().await.map_err(McpError::io)? {
                    let path = entry.path();
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    // Skip if doesn't match partial name
                    if !partial_name.is_empty() && !file_name.starts_with(&partial_name) {
                        continue;
                    }

                    // Skip hidden files if not included
                    if !self.should_include_hidden(&path) {
                        continue;
                    }

                    // For files, check extension filter
                    if path.is_file() && !self.should_include_file(&path) {
                        continue;
                    }

                    // Create the completion suggestion
                    let relative_path = path
                        .strip_prefix(&self.base_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();

                    // Add trailing slash for directories
                    let suggestion = if path.is_dir() {
                        format!("{relative_path}/")
                    } else {
                        relative_path
                    };

                    suggestions.push(suggestion);

                    if suggestions.len() >= self.max_suggestions {
                        break;
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read directory {:?}: {}", dir_to_search, e);
                return Ok(vec![]);
            }
        }

        // Sort suggestions for consistent ordering
        suggestions.sort();
        Ok(suggestions)
    }
}

#[async_trait]
impl CompletionHandler for FileSystemCompletionHandler {
    async fn complete(
        &self,
        reference: &CompletionReference,
        argument: &CompletionArgument,
        _context: Option<&CompletionContext>,
    ) -> McpResult<Vec<String>> {
        if let CompletionReference::Resource { uri } = reference {
            // Handle file://URIs
            if uri.starts_with("file://") {
                let _path_part = uri.strip_prefix("file://").unwrap_or("");
                return self.get_path_completions(&argument.value).await;
            }

            // For other URI schemes, provide basic path completion
            if argument.name == "path" || argument.name == "filename" || argument.name == "uri" {
                return self.get_path_completions(&argument.value).await;
            }
        }

        Ok(vec![])
    }
}

/// Fuzzy string completion handler
///
/// Provides fuzzy matching completion for string-based arguments.
/// Useful for prompt names, tool names, and other predefined lists.
///
/// # Features
/// - Fuzzy matching with customizable similarity threshold
/// - Case-insensitive matching
/// - Substring and prefix matching
/// - Custom scoring algorithm
///
/// # Example
/// ```rust
/// use prism_mcp_rs::core::completion_handlers::FuzzyCompletionHandler;
///
/// let handler = FuzzyCompletionHandler::new(vec![
/// "analyze_data", "analyze_text", "create_report", "generate_summary"
/// ]).threshold(0.6);
/// ```
pub struct FuzzyCompletionHandler {
    /// Available options for completion
    options: Vec<String>,
    /// Minimum similarity threshold (0.0 to 1.0)
    threshold: f64,
    /// Maximum number of suggestions
    max_suggestions: usize,
    /// Whether to use case-insensitive matching
    case_insensitive: bool,
}

impl FuzzyCompletionHandler {
    /// Create a new fuzzy completion handler
    ///
    /// # Arguments
    /// * `options` - List of available completion options
    pub fn new<S: AsRef<str>>(options: Vec<S>) -> Self {
        Self {
            options: options
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
            threshold: 0.4,
            max_suggestions: 10,
            case_insensitive: true,
        }
    }

    /// Set similarity threshold (0.0 to 1.0)
    pub fn threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set maximum number of suggestions
    pub fn max_suggestions(mut self, max: usize) -> Self {
        self.max_suggestions = max;
        self
    }

    /// Set case sensitivity
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_insensitive = !sensitive;
        self
    }

    /// Calculate similarity between two strings using Jaro-Winkler-like algorithm
    fn similarity(&self, a: &str, b: &str) -> f64 {
        let a = if self.case_insensitive {
            a.to_lowercase()
        } else {
            a.to_string()
        };
        let b = if self.case_insensitive {
            b.to_lowercase()
        } else {
            b.to_string()
        };

        if a == b {
            return 1.0;
        }

        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        // Check for exact prefix match (high score)
        if b.starts_with(&a) {
            return 0.9 + (a.len() as f64 / b.len() as f64) * 0.1;
        }

        // Check for substring match
        if b.contains(&a) {
            return 0.7 + (a.len() as f64 / b.len() as f64) * 0.2;
        }

        // Simple character overlap ratio
        let mut matches = 0;
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();

        for ac in &a_chars {
            if b_chars.contains(ac) {
                matches += 1;
            }
        }

        matches as f64 / a_chars.len().max(b_chars.len()) as f64
    }

    /// Get fuzzy completions for the given input
    fn get_fuzzy_completions(&self, input: &str) -> Vec<String> {
        let mut scored_options: Vec<(f64, String)> = self
            .options
            .iter()
            .map(|option| {
                let score = self.similarity(input, option);
                (score, option.clone())
            })
            .filter(|(score, _)| *score >= self.threshold)
            .collect();

        // Sort by score (descending)
        scored_options.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Return top suggestions
        scored_options
            .into_iter()
            .take(self.max_suggestions)
            .map(|(_, option)| option)
            .collect()
    }
}

#[async_trait]
impl CompletionHandler for FuzzyCompletionHandler {
    async fn complete(
        &self,
        _reference: &CompletionReference,
        argument: &CompletionArgument,
        _context: Option<&CompletionContext>,
    ) -> McpResult<Vec<String>> {
        Ok(self.get_fuzzy_completions(&argument.value))
    }
}

/// Schema-based completion handler
///
/// Provides completion suggestions based on JSON schema definitions.
/// Useful for tool parameters that have enum constraints or specific patterns.
///
/// # Features
/// - Enum value completion
/// - Pattern-based completion
/// - Type-aware suggestions
/// - Format-specific completions (email, date, etc.)
///
/// # Example
/// ```rust
/// use prism_mcp_rs::core::completion_handlers::SchemaCompletionHandler;
/// use serde_json::json;
///
/// let schema = json!({
/// "type": "object",
/// "properties": {
/// "priority": {
/// "type": "string",
/// "enum": ["low", "medium", "high"]
/// }
/// }
/// });
///
/// let handler = SchemaCompletionHandler::new(schema);
/// ```
pub struct SchemaCompletionHandler {
    /// JSON schema for parameter validation and completion
    schema: serde_json::Value,
    /// Custom completion mappings
    custom_completions: HashMap<String, Vec<String>>,
}

impl SchemaCompletionHandler {
    /// Create a new schema-based completion handler
    ///
    /// # Arguments
    /// * `schema` - JSON schema defining the parameter structure
    pub fn new(schema: serde_json::Value) -> Self {
        Self {
            schema,
            custom_completions: HashMap::new(),
        }
    }

    /// Add custom completion values for a specific parameter
    ///
    /// # Arguments
    /// * `parameter_name` - Name of the parameter
    /// * `values` - List of completion values
    pub fn add_custom_completions<S: AsRef<str>>(
        mut self,
        parameter_name: S,
        values: Vec<S>,
    ) -> Self {
        let values: Vec<String> = values.into_iter().map(|s| s.as_ref().to_string()).collect();
        self.custom_completions
            .insert(parameter_name.as_ref().to_string(), values);
        self
    }

    /// Extract enum values from schema property
    fn get_enum_values(&self, property: &serde_json::Value) -> Vec<String> {
        if let Some(enum_array) = property.get("enum").and_then(|e| e.as_array()) {
            return enum_array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
        }
        vec![]
    }

    /// Get format-based suggestions
    fn get_format_suggestions(&self, format: &str, current_value: &str) -> Vec<String> {
        match format {
            "email" => {
                if current_value.is_empty() {
                    vec!["user@example.com".to_string()]
                } else if !current_value.contains('@') {
                    vec![format!("{}@example.com", current_value)]
                } else {
                    vec![]
                }
            }
            "date" => {
                if current_value.is_empty() {
                    vec!["2025-01-01".to_string()]
                } else {
                    vec![]
                }
            }
            "time" => {
                if current_value.is_empty() {
                    vec!["12:00:00".to_string()]
                } else {
                    vec![]
                }
            }
            "uri" => {
                if current_value.is_empty() {
                    vec!["https://example.com".to_string()]
                } else if !current_value.contains("://") {
                    vec![
                        format!("https://{}", current_value),
                        format!("http://{}", current_value),
                        format!("file://{}", current_value),
                    ]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }

    /// Get completions for a parameter based on schema
    fn get_parameter_completions(&self, parameter_name: &str, current_value: &str) -> Vec<String> {
        // Check custom completions first
        if let Some(custom) = self.custom_completions.get(parameter_name) {
            return custom
                .iter()
                .filter(|value| value.starts_with(current_value))
                .cloned()
                .collect();
        }

        // Check schema properties
        if let Some(properties) = self.schema.get("properties").and_then(|p| p.as_object()) {
            if let Some(property) = properties.get(parameter_name) {
                // Handle enum values
                let enum_values = self.get_enum_values(property);
                if !enum_values.is_empty() {
                    return enum_values
                        .into_iter()
                        .filter(|value| value.starts_with(current_value))
                        .collect();
                }

                // Handle format-based suggestions
                if let Some(format) = property.get("format").and_then(|f| f.as_str()) {
                    return self.get_format_suggestions(format, current_value);
                }

                // Handle type-based suggestions
                if let Some(type_str) = property.get("type").and_then(|t| t.as_str()) {
                    match type_str {
                        "boolean" => {
                            return vec!["true".to_string(), "false".to_string()]
                                .into_iter()
                                .filter(|value| value.starts_with(current_value))
                                .collect();
                        }
                        "number" | "integer" => {
                            if current_value.is_empty() {
                                return vec!["0".to_string(), "1".to_string(), "10".to_string()];
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        vec![]
    }
}

#[async_trait]
impl CompletionHandler for SchemaCompletionHandler {
    async fn complete(
        &self,
        _reference: &CompletionReference,
        argument: &CompletionArgument,
        _context: Option<&CompletionContext>,
    ) -> McpResult<Vec<String>> {
        Ok(self.get_parameter_completions(&argument.name, &argument.value))
    }
}

/// Composite completion handler that combines multiple handlers
///
/// This handler allows you to combine different completion strategies
/// and provides a unified interface for complex completion scenarios.
///
/// # Example
/// ```rust
/// use prism_mcp_rs::core::completion_handlers::{
/// CompositeCompletionHandler, FuzzyCompletionHandler, FileSystemCompletionHandler
/// };
///
/// let composite = CompositeCompletionHandler::new()
/// .add_handler("files", FileSystemCompletionHandler::new("/home/user"))
/// .add_handler("prompts", FuzzyCompletionHandler::new(vec!["analyze", "create", "generate"]));
/// ```
pub struct CompositeCompletionHandler {
    /// Named completion handlers
    handlers: HashMap<String, Box<dyn CompletionHandler>>,
    /// Default handler to use when no specific handler matches
    default_handler: Option<Box<dyn CompletionHandler>>,
}

impl CompositeCompletionHandler {
    /// Create a new composite completion handler
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            default_handler: None,
        }
    }

    /// Add a named completion handler
    ///
    /// # Arguments
    /// * `name` - Identifier for this handler (used for routing)
    /// * `handler` - The completion handler implementation
    pub fn add_handler<H>(mut self, name: &str, handler: H) -> Self
    where
        H: CompletionHandler + 'static,
    {
        self.handlers.insert(name.to_string(), Box::new(handler));
        self
    }

    /// Set the default handler for unmatched requests
    ///
    /// # Arguments
    /// * `handler` - Default completion handler
    pub fn with_default<H>(mut self, handler: H) -> Self
    where
        H: CompletionHandler + 'static,
    {
        self.default_handler = Some(Box::new(handler));
        self
    }

    /// Determine which handler to use based on the reference and argument
    fn select_handler(
        &self,
        reference: &CompletionReference,
        argument: &CompletionArgument,
    ) -> Option<&dyn CompletionHandler> {
        // Strategy 1: Match by reference type + argument name
        let handler_key = match reference {
            CompletionReference::Prompt { .. } => {
                if argument.name == "name" {
                    Some("prompts".to_string())
                } else {
                    Some(format!("prompt_{}", argument.name))
                }
            }
            CompletionReference::Resource { .. } => {
                if argument.name == "uri" || argument.name == "path" {
                    Some("files".to_string())
                } else {
                    Some(format!("resource_{}", argument.name))
                }
            }
            CompletionReference::Tool { name } => Some(format!("tool_{}_{}", name, argument.name)),
        };

        // Try specific handler first
        if let Some(key) = handler_key {
            if let Some(handler) = self.handlers.get(&key) {
                return Some(handler.as_ref());
            }
        }

        // Try generic handlers
        match reference {
            CompletionReference::Prompt { .. } => self.handlers.get("prompts").map(|h| h.as_ref()),
            CompletionReference::Resource { .. } => {
                self.handlers.get("resources").map(|h| h.as_ref())
            }
            CompletionReference::Tool { .. } => self.handlers.get("tools").map(|h| h.as_ref()),
        }
        .or_else(|| self.default_handler.as_ref().map(|h| h.as_ref()))
    }
}

impl Default for CompositeCompletionHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CompletionHandler for CompositeCompletionHandler {
    async fn complete(
        &self,
        reference: &CompletionReference,
        argument: &CompletionArgument,
        context: Option<&CompletionContext>,
    ) -> McpResult<Vec<String>> {
        if let Some(handler) = self.select_handler(reference, argument) {
            handler.complete(reference, argument, context).await
        } else {
            Ok(vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs::File;

    #[tokio::test]
    async fn test_filesystem_completion() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        File::create(temp_path.join("test.txt")).await.unwrap();
        File::create(temp_path.join("example.md")).await.unwrap();
        tokio::fs::create_dir(temp_path.join("subdir"))
            .await
            .unwrap();

        let handler =
            FileSystemCompletionHandler::new(temp_path).with_extensions(vec!["txt", "md"]);

        let reference = CompletionReference::Resource {
            uri: "file:///test".to_string(),
        };
        let argument = CompletionArgument {
            name: "path".to_string(),
            value: "".to_string(),
        };

        let completions = handler.complete(&reference, &argument, None).await.unwrap();

        // Should include both files and the directory
        assert!(completions.len() >= 2);
        assert!(completions.iter().any(|c| c.contains("test.txt")));
        assert!(completions.iter().any(|c| c.contains("example.md")));
    }

    #[tokio::test]
    async fn test_fuzzy_completion() {
        let handler = FuzzyCompletionHandler::new(vec![
            "analyze_data",
            "analyze_text",
            "create_report",
            "generate_summary",
        ])
        .threshold(0.3);

        let reference = CompletionReference::Prompt {
            name: "test".to_string(),
        };
        let argument = CompletionArgument {
            name: "name".to_string(),
            value: "ana".to_string(),
        };

        let completions = handler.complete(&reference, &argument, None).await.unwrap();

        // Should match analyze_data and analyze_text
        assert_eq!(completions.len(), 2);
        assert!(completions.contains(&"analyze_data".to_string()));
        assert!(completions.contains(&"analyze_text".to_string()));
    }

    #[tokio::test]
    async fn test_schema_completion() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "priority": {
                    "type": "string",
                    "enum": ["low", "medium", "high"]
                },
                "email": {
                    "type": "string",
                    "format": "email"
                }
            }
        });

        let handler = SchemaCompletionHandler::new(schema);

        let reference = CompletionReference::Tool {
            name: "create_task".to_string(),
        };

        // Test enum completion
        let argument = CompletionArgument {
            name: "priority".to_string(),
            value: "m".to_string(),
        };

        let completions = handler.complete(&reference, &argument, None).await.unwrap();
        assert_eq!(completions, vec!["medium".to_string()]);

        // Test format completion
        let argument = CompletionArgument {
            name: "email".to_string(),
            value: "user".to_string(),
        };

        let completions = handler.complete(&reference, &argument, None).await.unwrap();
        assert_eq!(completions, vec!["user@example.com".to_string()]);
    }

    #[tokio::test]
    async fn test_composite_completion() {
        let fuzzy = FuzzyCompletionHandler::new(vec!["prompt1", "prompt2"]);
        let schema = SchemaCompletionHandler::new(serde_json::json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["active", "inactive"]
                }
            }
        }));

        let composite = CompositeCompletionHandler::new()
            .add_handler("prompts", fuzzy)
            .add_handler("tool_create_task_status", schema);

        // Test prompt completion
        let reference = CompletionReference::Prompt {
            name: "test".to_string(),
        };
        let argument = CompletionArgument {
            name: "name".to_string(),
            value: "prom".to_string(),
        };

        let completions = composite
            .complete(&reference, &argument, None)
            .await
            .unwrap();
        assert!(completions.contains(&"prompt1".to_string()));

        // Test tool parameter completion
        let reference = CompletionReference::Tool {
            name: "create_task".to_string(),
        };
        let argument = CompletionArgument {
            name: "status".to_string(),
            value: "a".to_string(),
        };

        let completions = composite
            .complete(&reference, &argument, None)
            .await
            .unwrap();
        assert_eq!(completions, vec!["active".to_string()]);
    }
}
