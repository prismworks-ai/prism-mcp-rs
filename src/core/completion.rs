//! Completion handling for MCP servers
//!
//! This module provides the foundation for implementing autocompletion features
//! in MCP servers, supporting the completion API introduced in MCP 2025-06-18.

use crate::core::error::McpResult;
use crate::protocol::messages::{CompletionArgument, CompletionReference};
use crate::protocol::types::*;
use async_trait::async_trait;
use std::collections::HashMap;

/// Trait for handling completion requests
///
/// Implement Trait to provide smart autocompletion for prompts,
/// tools, and resources in your MCP server.
#[async_trait]
pub trait CompletionHandler: Send + Sync {
    /// Generate completion suggestions
    ///
    /// Method is called when a client requests autocompletion for an
    /// argument value. The implementation should analyze the context and
    /// return relevant completion suggestions.
    ///
    /// # Arguments
    /// * `reference` - The item being completed (prompt, tool, or resource)
    /// * `argument` - The argument being completed with current value
    /// * `context` - Optional additional context for completion
    ///
    /// # Returns
    /// List of completion suggestions
    async fn complete(
        &self,
        reference: &CompletionReference,
        argument: &CompletionArgument,
        context: Option<&CompletionContext>,
    ) -> McpResult<Vec<String>>;
}

/// Default prompt completion handler with fuzzy matching
///
/// This handler provides basic completion for prompt names by matching
/// against a predefined list of available prompts.
///
/// # Example
/// ```rust
/// use prism_mcp_rs::core::completion::PromptCompletionHandler;
///
/// let prompts = vec![
/// "analyze_data".to_string(),
/// "analyze_text".to_string(),
/// "create_report".to_string(),
/// ];
/// let handler = PromptCompletionHandler::new(prompts);
/// ```
pub struct PromptCompletionHandler {
    prompts: Vec<String>,
    argument_completions: HashMap<String, HashMap<String, Vec<String>>>,
}

impl PromptCompletionHandler {
    /// Create a new prompt completion handler
    ///
    /// # Arguments
    /// * `prompts` - List of available prompt names
    pub fn new(prompts: Vec<String>) -> Self {
        Self {
            prompts,
            argument_completions: HashMap::new(),
        }
    }

    /// Add a prompt to the completion list
    pub fn add_prompt<S: Into<String>>(&mut self, name: S) {
        self.prompts.push(name.into());
    }

    /// Remove a prompt from the completion list
    pub fn remove_prompt(&mut self, name: &str) {
        self.prompts.retain(|p| p != name);
    }

    /// Get all available prompts
    pub fn prompts(&self) -> &[String] {
        &self.prompts
    }

    /// Add argument completions for a specific prompt
    pub fn add_argument_completions(
        &mut self,
        prompt_name: &str,
        argument_name: &str,
        completions: Vec<String>,
    ) {
        let prompt_args = self
            .argument_completions
            .entry(prompt_name.to_string())
            .or_default();
        prompt_args.insert(argument_name.to_string(), completions);
    }

    /// Perform fuzzy matching on a list of strings
    pub fn fuzzy_match(&self, items: &[String], query: &str) -> Vec<String> {
        let mut matches: Vec<(String, f32)> = items
            .iter()
            .filter_map(|item| {
                let score = self.calculate_match_score(item, query);
                if score > 0.0 {
                    Some((item.clone(), score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (descending)
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        matches.into_iter().map(|(item, _)| item).collect()
    }

    /// Calculate fuzzy match score
    fn calculate_match_score(&self, item: &str, query: &str) -> f32 {
        if item == query {
            return 1.0;
        }
        if item.starts_with(query) {
            return 0.8;
        }
        if item.contains(query) {
            return 0.6;
        }
        0.0
    }

    /// Get supported reference types
    pub fn supported_reference_types(&self) -> Vec<&str> {
        vec!["ref/prompt"]
    }
}

#[async_trait]
impl CompletionHandler for PromptCompletionHandler {
    async fn complete(
        &self,
        reference: &CompletionReference,
        argument: &CompletionArgument,
        _context: Option<&CompletionContext>,
    ) -> McpResult<Vec<String>> {
        match reference {
            CompletionReference::Prompt { name } => {
                if argument.name == "name" {
                    // Return prompt names that start with the current value
                    Ok(self
                        .prompts
                        .iter()
                        .filter(|prompt_name| prompt_name.starts_with(&argument.value))
                        .take(10) // Limit to 10 suggestions
                        .cloned()
                        .collect())
                } else {
                    // Check for argument completions
                    if let Some(prompt_args) = self.argument_completions.get(name) {
                        if let Some(values) = prompt_args.get(&argument.name) {
                            return Ok(values
                                .iter()
                                .filter(|value| value.starts_with(&argument.value))
                                .take(10)
                                .cloned()
                                .collect());
                        }
                    }
                    Ok(vec![])
                }
            }
            _ => Ok(vec![]), // Only handle prompt references
        }
    }
}

/// Resource URI completion handler
///
/// This handler provides completion for resource URIs based on resource
/// templates. It can suggest URI completions by analyzing URI patterns
/// and available resources.
///
/// # Example
/// ```rust
/// use prism_mcp_rs::core::completion::ResourceUriCompletionHandler;
/// use prism_mcp_rs::protocol::types::ResourceTemplate;
///
/// let templates = vec![
/// ResourceTemplate::new("file:///docs/{category}/{filename}".to_string(), "Documentation".to_string()),
/// ResourceTemplate::new("db:/// {table}/{id}".to_string(), "Database Records".to_string()),
/// ];
/// let handler = ResourceUriCompletionHandler::new(templates);
/// ```
pub struct ResourceUriCompletionHandler {
    templates: Vec<ResourceTemplate>,
    static_resources: Vec<String>,
}

impl ResourceUriCompletionHandler {
    /// Create a new resource URI completion handler
    ///
    /// # Arguments
    /// * `templates` - List of resource templates to use for completion
    pub fn new(templates: Vec<ResourceTemplate>) -> Self {
        Self {
            templates,
            static_resources: Vec::new(),
        }
    }

    /// Add a static resource URI for completion
    ///
    /// # Arguments
    /// * `uri` - Static resource URI to add to completions
    pub fn add_static_resource<S: Into<String>>(&mut self, uri: S) {
        self.static_resources.push(uri.into());
    }

    /// Generate URI completions based on templates
    async fn generate_uri_completions(
        &self,
        uri_template: &str,
        current_value: &str,
        _context: Option<&CompletionContext>,
    ) -> McpResult<Vec<String>> {
        let mut completions = Vec::new();

        // Simple template variable completion
        if uri_template.contains("{category}") && current_value.contains("/docs/") {
            // Example categories for documentation
            let categories = ["api", "guides", "tutorials", "reference"];
            for category in categories {
                let suggestion = uri_template.replace("{category}", category);
                if suggestion.starts_with(current_value) {
                    completions.push(suggestion);
                }
            }
        }

        if uri_template.contains("{filename}") {
            // Example filenames
            let filenames = [
                "overview.md",
                "getting-started.md",
                "reference.md",
                "examples.md",
            ];
            for filename in filenames {
                let suggestion = uri_template.replace("{filename}", filename);
                if suggestion.starts_with(current_value) {
                    completions.push(suggestion);
                }
            }
        }

        if uri_template.contains("{table}") {
            // Example database tables
            let tables = ["users", "products", "orders", "categories"];
            for table in tables {
                let suggestion = uri_template.replace("{table}", table);
                if suggestion.starts_with(current_value) {
                    completions.push(suggestion);
                }
            }
        }

        if uri_template.contains("{id}") {
            // Example IDs
            let ids = ["1", "2", "3", "latest", "featured"];
            for id in ids {
                let suggestion = uri_template.replace("{id}", id);
                if suggestion.starts_with(current_value) {
                    completions.push(suggestion);
                }
            }
        }

        Ok(completions)
    }

    /// Extract template variables from a URI template
    pub fn extract_template_variables(&self, template: &str) -> Vec<String> {
        let mut variables = Vec::new();
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut var_name = String::new();
                for ch in chars.by_ref() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                if !var_name.is_empty() {
                    variables.push(var_name);
                }
            }
        }

        variables
    }

    /// Get supported reference types
    pub fn supported_reference_types(&self) -> Vec<&str> {
        vec!["ref/resource"]
    }
}

#[async_trait]
impl CompletionHandler for ResourceUriCompletionHandler {
    async fn complete(
        &self,
        reference: &CompletionReference,
        argument: &CompletionArgument,
        context: Option<&CompletionContext>,
    ) -> McpResult<Vec<String>> {
        match reference {
            CompletionReference::Resource { uri: _ } => {
                // Match against URI templates and suggest completions
                let mut suggestions = Vec::new();

                for template in &self.templates {
                    if let Ok(completions) = self
                        .generate_uri_completions(&template.uri_template, &argument.value, context)
                        .await
                    {
                        suggestions.extend(completions);
                    }
                }

                // Add static resource completions
                for resource_uri in &self.static_resources {
                    if resource_uri.starts_with(&argument.value) {
                        suggestions.push(resource_uri.clone());
                    }
                }

                Ok(suggestions.into_iter().take(10).collect())
            }
            _ => Ok(vec![]), // Only handle resource references
        }
    }
}

/// Tool argument completion handler
///
/// This handler provides completion for tool arguments based on the tool's
/// input schema and available values.
///
/// # Example
/// ```rust
/// use prism_mcp_rs::core::completion::ToolCompletionHandler;
/// use std::collections::HashMap;
///
/// let mut completions = HashMap::new();
/// completions.insert("file_reader".to_string(), vec![
/// ("path".to_string(), vec!["/home/user/file1.txt".to_string(), "/home/user/file2.txt".to_string()]),
/// ]);
/// let handler = ToolCompletionHandler::new(completions);
/// ```
pub struct ToolCompletionHandler {
    tool_completions: HashMap<String, Vec<(String, Vec<String>)>>,
}

impl ToolCompletionHandler {
    /// Create a new tool completion handler
    ///
    /// # Arguments
    /// * `tool_completions` - Map of tool names to argument completions
    pub fn new(tool_completions: HashMap<String, Vec<(String, Vec<String>)>>) -> Self {
        Self { tool_completions }
    }

    /// Add completion values for a tool argument
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool
    /// * `argument_name` - Name of the argument
    /// * `values` - List of possible values for completion
    pub fn add_tool_argument_completions<S: Into<String>>(
        &mut self,
        tool_name: S,
        argument_name: S,
        values: Vec<String>,
    ) {
        let tool_name = tool_name.into();
        let argument_name = argument_name.into();

        let tool_entry = self.tool_completions.entry(tool_name).or_default();

        // Remove existing entry for this argument if it exists
        tool_entry.retain(|(name, _)| name != &argument_name);

        // Add new entry
        tool_entry.push((argument_name, values));
    }

    /// Remove completions for a tool
    pub fn remove_tool_completions(&mut self, tool_name: &str) {
        self.tool_completions.remove(tool_name);
    }

    /// Get supported reference types
    pub fn supported_reference_types(&self) -> Vec<&str> {
        vec!["ref/tool"]
    }
}

#[async_trait]
impl CompletionHandler for ToolCompletionHandler {
    async fn complete(
        &self,
        reference: &CompletionReference,
        argument: &CompletionArgument,
        _context: Option<&CompletionContext>,
    ) -> McpResult<Vec<String>> {
        match reference {
            CompletionReference::Tool { name } => {
                if let Some(tool_args) = self.tool_completions.get(name) {
                    for (arg_name, values) in tool_args {
                        if arg_name == &argument.name {
                            // Return values that start with the current value
                            return Ok(values
                                .iter()
                                .filter(|value| value.starts_with(&argument.value))
                                .take(10)
                                .cloned()
                                .collect());
                        }
                    }
                }
                Ok(vec![])
            }
            _ => Ok(vec![]), // Only handle tool references
        }
    }
}

/// Combined completion handler that delegates to different handlers based on reference type
///
/// This handler allows you to combine multiple completion handlers and route
/// completion requests to the appropriate handler based on the reference type.
///
/// # Example
/// ```rust
/// use prism_mcp_rs::core::completion::*;
/// use std::collections::HashMap;
///
/// let prompt_handler = PromptCompletionHandler::new(vec!["analyze".to_string()]);
/// let resource_handler = ResourceUriCompletionHandler::new(vec![]);
/// let tool_handler = ToolCompletionHandler::new(HashMap::new());
///
/// let combined = CompositeCompletionHandler::new()
/// .with_prompt_handler(prompt_handler)
/// .with_resource_handler(resource_handler)
/// .with_tool_handler(tool_handler);
/// ```
pub struct CompositeCompletionHandler {
    prompt_handler: Option<Box<dyn CompletionHandler>>,
    resource_handler: Option<Box<dyn CompletionHandler>>,
    tool_handler: Option<Box<dyn CompletionHandler>>,
}

impl CompositeCompletionHandler {
    /// Create a new combined completion handler
    pub fn new() -> Self {
        Self {
            prompt_handler: None,
            resource_handler: None,
            tool_handler: None,
        }
    }

    /// Add a handler for a specific reference type
    ///
    /// # Arguments
    /// * `reference_type` - The reference type this handler supports (e.g., "ref/prompt")
    /// * `handler` - The completion handler
    pub fn add_handler<H>(&mut self, reference_type: String, handler: H)
    where
        H: CompletionHandler + 'static,
    {
        match reference_type.as_str() {
            "ref/prompt" => self.prompt_handler = Some(Box::new(handler)),
            "ref/resource" => self.resource_handler = Some(Box::new(handler)),
            "ref/tool" => self.tool_handler = Some(Box::new(handler)),
            _ => {} // Ignore unknown types
        }
    }

    /// Set the prompt completion handler
    pub fn with_prompt_handler<H: CompletionHandler + 'static>(mut self, handler: H) -> Self {
        self.prompt_handler = Some(Box::new(handler));
        self
    }

    /// Set the resource completion handler
    pub fn with_resource_handler<H: CompletionHandler + 'static>(mut self, handler: H) -> Self {
        self.resource_handler = Some(Box::new(handler));
        self
    }

    /// Set the tool completion handler
    pub fn with_tool_handler<H: CompletionHandler + 'static>(mut self, handler: H) -> Self {
        self.tool_handler = Some(Box::new(handler));
        self
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
        match reference {
            CompletionReference::Prompt { .. } => {
                if let Some(handler) = &self.prompt_handler {
                    handler.complete(reference, argument, context).await
                } else {
                    Ok(vec![])
                }
            }
            CompletionReference::Resource { .. } => {
                if let Some(handler) = &self.resource_handler {
                    handler.complete(reference, argument, context).await
                } else {
                    Ok(vec![])
                }
            }
            CompletionReference::Tool { .. } => {
                if let Some(handler) = &self.tool_handler {
                    handler.complete(reference, argument, context).await
                } else {
                    Ok(vec![])
                }
            }
        }
    }
}

/// Completion context for additional information
///
/// Struct can be extended to provide additional context for completion
/// requests, such as current file paths, user preferences, etc.
#[derive(Debug, Clone, Default)]
pub struct CompletionContext {
    /// Additional context arguments
    pub arguments: Option<HashMap<String, String>>,
    /// Current working directory or context path
    pub context_path: Option<String>,
    /// User preferences for completion
    pub preferences: Option<HashMap<String, serde_json::Value>>,
}

impl CompletionContext {
    /// Create a new completion context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set context arguments
    pub fn with_arguments(mut self, arguments: HashMap<String, String>) -> Self {
        self.arguments = Some(arguments);
        self
    }

    /// Set context path
    pub fn with_context_path<S: Into<String>>(mut self, path: S) -> Self {
        self.context_path = Some(path.into());
        self
    }

    /// Set user preferences
    pub fn with_preferences(mut self, preferences: HashMap<String, serde_json::Value>) -> Self {
        self.preferences = Some(preferences);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prompt_completion() {
        let handler = PromptCompletionHandler::new(vec![
            "analyze_data".to_string(),
            "analyze_text".to_string(),
            "create_report".to_string(),
        ]);

        let reference = CompletionReference::Prompt {
            name: "test".to_string(),
        };

        let argument = CompletionArgument {
            name: "name".to_string(),
            value: "ana".to_string(),
        };

        let results = handler.complete(&reference, &argument, None).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"analyze_data".to_string()));
        assert!(results.contains(&"analyze_text".to_string()));
    }

    #[tokio::test]
    async fn test_prompt_completion_with_arguments() {
        let mut handler = PromptCompletionHandler::new(vec!["analyze".to_string()]);
        handler.add_argument_completions(
            "analyze",
            "format",
            vec!["json".to_string(), "xml".to_string(), "yaml".to_string()],
        );

        let reference = CompletionReference::Prompt {
            name: "analyze".to_string(),
        };

        let argument = CompletionArgument {
            name: "format".to_string(),
            value: "j".to_string(),
        };

        let results = handler.complete(&reference, &argument, None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&"json".to_string()));
    }

    #[tokio::test]
    async fn test_resource_uri_completion_handler() {
        let templates = vec![ResourceTemplate::new(
            "file:///docs/{category}/{filename}".to_string(),
            "Documentation".to_string(),
        )];
        let handler = ResourceUriCompletionHandler::new(templates);

        let reference = CompletionReference::Resource {
            uri: "file:///docs/".to_string(),
        };

        let argument = CompletionArgument {
            name: "uri".to_string(),
            value: "file:///docs/".to_string(),
        };

        let results = handler.complete(&reference, &argument, None).await.unwrap();
        // Results should contain expanded template examples
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_composite_completion_handler() {
        let mut composite = CompositeCompletionHandler::new();

        let prompt_handler = PromptCompletionHandler::new(vec!["test_prompt".to_string()]);

        composite.add_handler("ref/prompt".to_string(), prompt_handler);

        let reference = CompletionReference::Prompt {
            name: "test".to_string(),
        };

        let argument = CompletionArgument {
            name: "name".to_string(),
            value: "test".to_string(),
        };

        let results = composite
            .complete(&reference, &argument, None)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&"test_prompt".to_string()));
    }

    #[test]
    fn test_fuzzy_matching() {
        let handler = PromptCompletionHandler::new(vec![
            "analyze_data".to_string(),
            "create_report".to_string(),
            "data_analysis".to_string(),
        ]);

        // Test exact match
        let results = handler.fuzzy_match(&handler.prompts, "analyze_data");
        assert_eq!(results[0], "analyze_data");

        // Test prefix match
        let results = handler.fuzzy_match(&handler.prompts, "ana");
        assert!(results.contains(&"analyze_data".to_string()));

        // Test substring match
        let results = handler.fuzzy_match(&handler.prompts, "data");
        assert!(results.len() >= 2); // Should match both "analyze_data" and "data_analysis"
    }

    #[test]
    fn test_template_variable_extraction() {
        let handler = ResourceUriCompletionHandler::new(vec![]);

        let template = "file:///docs/{category}/{filename}";
        let variables = handler.extract_template_variables(template);

        assert_eq!(variables.len(), 2);
        assert!(variables.contains(&"category".to_string()));
        assert!(variables.contains(&"filename".to_string()));
    }

    #[test]
    fn test_supported_reference_types() {
        let prompt_handler = PromptCompletionHandler::new(vec![]);
        assert_eq!(
            prompt_handler.supported_reference_types(),
            vec!["ref/prompt"]
        );

        let resource_handler = ResourceUriCompletionHandler::new(vec![]);
        assert_eq!(
            resource_handler.supported_reference_types(),
            vec!["ref/resource"]
        );

        let tool_handler = ToolCompletionHandler::new(HashMap::new());
        assert_eq!(tool_handler.supported_reference_types(), vec!["ref/tool"]);
    }
}