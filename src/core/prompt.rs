//! Prompt system for MCP servers
//!
//! This module provides the abstraction for implementing and managing prompts in MCP servers.
//! Prompts are templates that can be used to generate messages for language models.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::core::error::{McpError, McpResult};
use crate::protocol::types::{
    Content, GetPromptResult as PromptResult, Prompt as PromptInfo, PromptArgument, PromptMessage,
    Role,
};

/// Trait for implementing prompt handlers
#[async_trait]
pub trait PromptHandler: Send + Sync {
    /// Generate prompt messages with the given arguments
    ///
    /// # Arguments
    /// * `arguments` - Prompt arguments as key-value pairs
    ///
    /// # Returns
    /// Result containing the generated prompt messages or an error
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<PromptResult>;
}

/// A registered prompt with its handler
pub struct Prompt {
    /// Information about the prompt
    pub info: PromptInfo,
    /// Handler that implements the prompt's functionality
    pub handler: Box<dyn PromptHandler>,
    /// Whether the prompt is currently enabled
    pub enabled: bool,
}

impl Prompt {
    /// Create a new prompt with the given information and handler
    ///
    /// # Arguments
    /// * `info` - Information about the prompt
    /// * `handler` - Implementation of the prompt's functionality
    pub fn new<H>(info: PromptInfo, handler: H) -> Self
    where
        H: PromptHandler + 'static,
    {
        Self {
            info,
            handler: Box::new(handler),
            enabled: true,
        }
    }

    /// Enable the prompt
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the prompt
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if the prompt is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Execute the prompt if it's enabled
    ///
    /// # Arguments
    /// * `arguments` - Prompt arguments as key-value pairs
    ///
    /// # Returns
    /// Result containing the prompt result or an error
    pub async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<PromptResult> {
        if !self.enabled {
            return Err(McpError::validation(format!(
                "Prompt '{}' is disabled",
                self.info.name
            )));
        }

        // Validate required arguments
        if let Some(ref args) = self.info.arguments {
            for arg in args {
                if arg.required.unwrap_or(false) && !arguments.contains_key(&arg.name) {
                    return Err(McpError::validation(format!(
                        "Required argument '{}' missing for prompt '{}'",
                        arg.name, self.info.name
                    )));
                }
            }
        }

        self.handler.get(arguments).await
    }
}

impl std::fmt::Debug for Prompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Prompt")
            .field("info", &self.info)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl PromptMessage {
    /// Create a system message
    pub fn system<S: Into<String>>(content: S) -> Self {
        Self {
            role: Role::User, // Note: 2025-06-18 only has User and Assistant roles
            content: Content::text(content.into()),
        }
    }

    /// Create a user message
    pub fn user<S: Into<String>>(content: S) -> Self {
        Self {
            role: Role::User,
            content: Content::text(content.into()),
        }
    }

    /// Create an assistant message
    pub fn assistant<S: Into<String>>(content: S) -> Self {
        Self {
            role: Role::Assistant,
            content: Content::text(content.into()),
        }
    }

    /// Create a message with custom role
    pub fn with_role(role: Role, content: Content) -> Self {
        Self { role, content }
    }
}

// Common prompt implementations

/// Simple greeting prompt
pub struct GreetingPrompt;

#[async_trait]
impl PromptHandler for GreetingPrompt {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<PromptResult> {
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");

        Ok(PromptResult {
            description: Some("A friendly greeting".to_string()),
            messages: vec![
                PromptMessage::system("You are a friendly assistant."),
                PromptMessage::user(format!("Hello, {name}!")),
            ],
            meta: None,
        })
    }
}

/// Code review prompt
pub struct CodeReviewPrompt;

#[async_trait]
impl PromptHandler for CodeReviewPrompt {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<PromptResult> {
        let code = arguments
            .get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::validation("Missing 'code' argument"))?;

        let language = arguments
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let focus = arguments
            .get("focus")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let system_prompt = format!(
            "You are an expert code reviewer. Focus on {focus} aspects of the code. \
             Provide constructive feedback and suggestions for improvement."
        );

        let user_prompt =
            format!("Please review this {language} code:\n\n```{language}\n{code}\n```");

        Ok(PromptResult {
            description: Some("Code review prompt".to_string()),
            messages: vec![
                PromptMessage::system(system_prompt),
                PromptMessage::user(user_prompt),
            ],
            meta: None,
        })
    }
}

/// SQL query generation prompt
pub struct SqlQueryPrompt;

#[async_trait]
impl PromptHandler for SqlQueryPrompt {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<PromptResult> {
        let request = arguments
            .get("request")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::validation("Missing 'request' argument"))?;

        let schema = arguments
            .get("schema")
            .and_then(|v| v.as_str())
            .unwrap_or("No schema provided");

        let dialect = arguments
            .get("dialect")
            .and_then(|v| v.as_str())
            .unwrap_or("PostgreSQL");

        let system_prompt = format!(
            "You are an expert SQL developer. Generate efficient and safe {dialect} queries. \
             Always use proper escaping and avoid SQL injection vulnerabilities."
        );

        let user_prompt = format!(
            "Database Schema:\n{schema}\n\nRequest: {request}\n\nPlease generate a {dialect} query for this request."
        );

        Ok(PromptResult {
            description: Some("SQL query generation prompt".to_string()),
            messages: vec![
                PromptMessage::system(system_prompt),
                PromptMessage::user(user_prompt),
            ],
            meta: None,
        })
    }
}

/// Builder for creating prompts with fluent API
pub struct PromptBuilder {
    name: String,
    description: Option<String>,
    arguments: Vec<PromptArgument>,
}

impl PromptBuilder {
    /// Create a new prompt builder with the given name
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            description: None,
            arguments: Vec::new(),
        }
    }

    /// Set the prompt description
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a required argument
    pub fn required_arg<S: Into<String>>(mut self, name: S, description: Option<S>) -> Self {
        self.arguments.push(PromptArgument {
            name: name.into(),
            description: description.map(|d| d.into()),
            required: Some(true),
            title: None,
        });
        self
    }

    /// Add an optional argument
    pub fn optional_arg<S: Into<String>>(mut self, name: S, description: Option<S>) -> Self {
        self.arguments.push(PromptArgument {
            name: name.into(),
            description: description.map(|d| d.into()),
            required: Some(false),
            title: None,
        });
        self
    }

    /// Build the prompt with the given handler
    pub fn build<H>(self, handler: H) -> Prompt
    where
        H: PromptHandler + 'static,
    {
        let info = PromptInfo {
            name: self.name,
            description: self.description,
            arguments: if self.arguments.is_empty() {
                None
            } else {
                Some(self.arguments)
            },
            title: None,
            meta: None,
        };

        Prompt::new(info, handler)
    }
}

/// Utility for creating prompt arguments
pub fn required_arg<S: Into<String>>(name: S, description: Option<S>) -> PromptArgument {
    PromptArgument {
        name: name.into(),
        description: description.map(|d| d.into()),
        required: Some(true),
        title: None,
    }
}

/// Utility for creating optional prompt arguments
pub fn optional_arg<S: Into<String>>(name: S, description: Option<S>) -> PromptArgument {
    PromptArgument {
        name: name.into(),
        description: description.map(|d| d.into()),
        required: Some(false),
        title: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_greeting_prompt() {
        let prompt = GreetingPrompt;
        let mut args = HashMap::new();
        args.insert("name".to_string(), json!("Alice"));

        let result = prompt.get(args).await.unwrap();
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0].role, Role::User);
        assert_eq!(result.messages[1].role, Role::User);

        match &result.messages[1].content {
            Content::Text { text, .. } => assert!(text.contains("Alice")),
            _ => panic!("Expected text content"),
        }
    }

    #[tokio::test]
    async fn test_code_review_prompt() {
        let prompt = CodeReviewPrompt;
        let mut args = HashMap::new();
        args.insert(
            "code".to_string(),
            json!("function hello() { console.log('Hello'); }"),
        );
        args.insert("language".to_string(), json!("javascript"));
        args.insert("focus".to_string(), json!("performance"));

        let result = prompt.get(args).await.unwrap();
        assert_eq!(result.messages.len(), 2);

        match &result.messages[1].content {
            Content::Text { text, .. } => {
                assert!(text.contains("javascript"));
                assert!(text.contains("console.log"));
            }
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_prompt_creation() {
        let info = PromptInfo {
            name: "test_prompt".to_string(),
            description: Some("Test prompt".to_string()),
            arguments: Some(vec![PromptArgument {
                name: "arg1".to_string(),
                description: Some("First argument".to_string()),
                required: Some(true),
                title: None,
            }]),
            title: None,
            meta: None,
        };

        let prompt = Prompt::new(info.clone(), GreetingPrompt);
        assert_eq!(prompt.info, info);
        assert!(prompt.is_enabled());
    }

    #[tokio::test]
    async fn test_prompt_validation() {
        let info = PromptInfo {
            name: "test_prompt".to_string(),
            description: None,
            arguments: Some(vec![PromptArgument {
                name: "required_arg".to_string(),
                description: None,
                required: Some(true),
                title: None,
            }]),
            title: None,
            meta: None,
        };

        let prompt = Prompt::new(info, GreetingPrompt);

        // Test missing required argument
        let result = prompt.get(HashMap::new()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::Validation(msg) => assert!(msg.contains("required_arg")),
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_prompt_builder() {
        let prompt = PromptBuilder::new("test")
            .description("A test prompt")
            .required_arg("input", Some("Input text"))
            .optional_arg("format", Some("Output format"))
            .build(GreetingPrompt);

        assert_eq!(prompt.info.name, "test");
        assert_eq!(prompt.info.description, Some("A test prompt".to_string()));

        let args = prompt.info.arguments.unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].name, "input");
        assert_eq!(args[0].required, Some(true));
        assert_eq!(args[1].name, "format");
        assert_eq!(args[1].required, Some(false));
    }

    #[test]
    fn test_prompt_message_creation() {
        let system_msg = PromptMessage::system("You are a helpful assistant");
        assert_eq!(system_msg.role, Role::User);

        let user_msg = PromptMessage::user("Hello!");
        assert_eq!(user_msg.role, Role::User);

        let assistant_msg = PromptMessage::assistant("Hi there!");
        assert_eq!(assistant_msg.role, Role::Assistant);
    }

    #[test]
    fn test_prompt_content_creation() {
        let text_content = Content::text("Hello, world!");
        match text_content {
            Content::Text { text, .. } => {
                assert_eq!(text, "Hello, world!");
            }
            _ => panic!("Expected text content"),
        }

        let image_content = Content::image("base64data", "image/png");
        match image_content {
            Content::Image {
                data, mime_type, ..
            } => {
                assert_eq!(data, "base64data");
                assert_eq!(mime_type, "image/png");
            }
            _ => panic!("Expected image content"),
        }
    }
}
