// ! Resource system for MCP servers
// !
// ! Module provides the abstraction for implementing and managing resources in MCP servers.
// ! Resources represent data that can be read by clients, such as files, database records, or API endpoints.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::core::error::{McpError, McpResult};
use crate::protocol::types::{Resource as ResourceInfo, ResourceContents};

/// Template for parameterized resources
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceTemplate {
    /// URI template with parameter placeholders
    pub uri_template: String,
    /// Name of the resource template
    pub name: String,
    /// Description of the resource template
    pub description: Option<String>,
    /// MIME type of resources created from this template
    pub mime_type: Option<String>,
}

/// Trait for implementing resource handlers
#[async_trait]
pub trait ResourceHandler: Send + Sync {
    /// Read the content of a resource
    ///
    /// # Arguments
    /// * `uri` - URI of the resource to read
    /// * `params` - Additional parameters for the resource
    ///
    /// # Returns
    /// Result containing the resource content or an error
    async fn read(
        &self,
        uri: &str,
        params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>>;

    /// List all available resources
    ///
    /// # Returns
    /// Result containing a list of available resources or an error
    async fn list(&self) -> McpResult<Vec<ResourceInfo>>;

    /// Subscribe to changes in a resource (optional)
    ///
    /// # Arguments
    /// * `uri` - URI of the resource to subscribe to
    ///
    /// # Returns
    /// Result indicating success or an error
    async fn subscribe(&self, uri: &str) -> McpResult<()> {
        // Default implementation - subscription not supported
        Err(McpError::protocol(format!(
            "Subscription not supported for resource: {uri}"
        )))
    }

    /// Unsubscribe from changes in a resource (optional)
    ///
    /// # Arguments
    /// * `uri` - URI of the resource to unsubscribe from
    ///
    /// # Returns
    /// Result indicating success or an error
    async fn unsubscribe(&self, uri: &str) -> McpResult<()> {
        // Default implementation - subscription not supported
        Err(McpError::protocol(format!(
            "Subscription not supported for resource: {uri}"
        )))
    }
}

/// Legacy trait for backward compatibility with existing tests
/// This should be used for simple text-based resources
#[async_trait]
pub trait LegacyResourceHandler: Send + Sync {
    /// Read the content of a resource as a string
    ///
    /// # Arguments
    /// * `uri` - URI of the resource to read
    ///
    /// # Returns
    /// Result containing the resource content as a string or an error
    async fn read(&self, uri: &str) -> McpResult<String>;

    /// List all available resources
    ///
    /// # Returns
    /// Result containing a list of available resources or an error
    async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
        // Default implementation returns empty list
        Ok(vec![])
    }
}

/// Adapter to convert LegacyResourceHandler to ResourceHandler
pub struct LegacyResourceAdapter<T> {
    inner: T,
}

impl<T> LegacyResourceAdapter<T>
where
    T: LegacyResourceHandler,
{
    pub fn new(handler: T) -> Self {
        Self { inner: handler }
    }
}

#[async_trait]
impl<T> ResourceHandler for LegacyResourceAdapter<T>
where
    T: LegacyResourceHandler + Send + Sync,
{
    async fn read(
        &self,
        uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        let content = self.inner.read(uri).await?;
        Ok(vec![ResourceContents::Text {
            uri: uri.to_string(),
            mime_type: Some("text/plain".to_string()),
            text: content,
            meta: None,
        }])
    }

    async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
        self.inner.list().await
    }
}

/// A registered resource with its handler
pub struct Resource {
    /// Information about the resource
    pub info: ResourceInfo,
    /// Handler that implements the resource's functionality
    pub handler: Box<dyn ResourceHandler>,
    /// Optional template for parameterized resources
    pub template: Option<ResourceTemplate>,
    /// Whether the resource is currently enabled
    pub enabled: bool,
}

impl Resource {
    /// Create a new static resource
    ///
    /// # Arguments
    /// * `info` - Information about the resource
    /// * `handler` - Implementation of the resource's functionality
    pub fn new<H>(info: ResourceInfo, handler: H) -> Self
    where
        H: ResourceHandler + 'static,
    {
        Self {
            info,
            handler: Box::new(handler),
            template: None,
            enabled: true,
        }
    }

    /// Create a new templated resource
    ///
    /// # Arguments
    /// * `template` - Template for the resource
    /// * `handler` - Implementation of the resource's functionality
    pub fn with_template<H>(template: ResourceTemplate, handler: H) -> Self
    where
        H: ResourceHandler + 'static,
    {
        let info = ResourceInfo {
            uri: template.uri_template.clone(),
            name: template.name.clone(),
            description: template.description.clone(),
            mime_type: template.mime_type.clone(),
            annotations: None,
            size: None,
            title: None,
            meta: None,
        };

        Self {
            info,
            handler: Box::new(handler),
            template: Some(template),
            enabled: true,
        }
    }

    /// Enable the resource
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the resource
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if the resource is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Read the resource if it's enabled
    ///
    /// # Arguments
    /// * `uri` - URI of the resource to read
    /// * `params` - Additional parameters for the resource
    ///
    /// # Returns
    /// Result containing the resource content or an error
    pub async fn read(
        &self,
        uri: &str,
        params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        if !self.enabled {
            let name = self.info.name.as_str();
            return Err(McpError::validation(format!(
                "Resource '{name}' is disabled"
            )));
        }

        self.handler.read(uri, params).await
    }

    /// List resources from this handler
    pub async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
        if !self.enabled {
            return Ok(vec![]);
        }

        self.handler.list().await
    }

    /// Subscribe to resource changes
    pub async fn subscribe(&self, uri: &str) -> McpResult<()> {
        if !self.enabled {
            let name = self.info.name.as_str();
            return Err(McpError::validation(format!(
                "Resource '{name}' is disabled"
            )));
        }

        self.handler.subscribe(uri).await
    }

    /// Unsubscribe from resource changes
    pub async fn unsubscribe(&self, uri: &str) -> McpResult<()> {
        if !self.enabled {
            let name = self.info.name.as_str();
            return Err(McpError::validation(format!(
                "Resource '{name}' is disabled"
            )));
        }

        self.handler.unsubscribe(uri).await
    }

    /// Check if this resource matches the given URI
    pub fn matches_uri(&self, uri: &str) -> bool {
        if let Some(template) = &self.template {
            // Simple template matching - in a real implementation,
            // you'd want more complete URI template matching
            uri.starts_with(&template.uri_template.replace("{id}", "").replace("{*}", ""))
        } else {
            self.info.uri == uri
        }
    }
}

impl std::fmt::Debug for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("info", &self.info)
            .field("template", &self.template)
            .field("enabled", &self.enabled)
            .finish()
    }
}

// Common resource implementations

/// Simple text resource
pub struct TextResource {
    content: String,
    mime_type: String,
}

impl TextResource {
    /// Create a new text resource
    pub fn new(content: String, mime_type: Option<String>) -> Self {
        Self {
            content,
            mime_type: mime_type.unwrap_or_else(|| "text/plain".to_string()),
        }
    }
}

#[async_trait]
impl ResourceHandler for TextResource {
    async fn read(
        &self,
        uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        Ok(vec![ResourceContents::Text {
            uri: uri.to_string(),
            mime_type: Some(self.mime_type.clone()),
            text: self.content.clone(),
            meta: None,
        }])
    }

    async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
        // Static resources don't provide dynamic listing
        Ok(vec![])
    }
}

/// File system resource handler
pub struct FileSystemResource {
    base_path: std::path::PathBuf,
    allowed_extensions: Option<Vec<String>>,
}

impl FileSystemResource {
    /// Create a new file system resource handler
    pub fn new<P: AsRef<std::path::Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            allowed_extensions: None,
        }
    }

    /// Set allowed file extensions
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_extensions = Some(extensions);
        self
    }

    fn is_allowed_file(&self, path: &std::path::Path) -> bool {
        if let Some(ref allowed) = self.allowed_extensions {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                return allowed.contains(&ext.to_lowercase());
            }
            return false;
        }
        true
    }

    fn get_mime_type(&self, path: &std::path::Path) -> String {
        match path.extension().and_then(|e| e.to_str()) {
            Some("txt") => "text/plain".to_string(),
            Some("json") => "application/json".to_string(),
            Some("html") => "text/html".to_string(),
            Some("css") => "text/css".to_string(),
            Some("js") => "application/javascript".to_string(),
            Some("md") => "text/markdown".to_string(),
            Some("xml") => "application/xml".to_string(),
            Some("yaml") | Some("yml") => "application/yaml".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }
}

#[async_trait]
impl ResourceHandler for FileSystemResource {
    async fn read(
        &self,
        uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        // Extract file path from URI (assuming file://scheme or relative path)
        let file_path = if uri.starts_with("file://") {
            uri.strip_prefix("file://").unwrap_or(uri)
        } else {
            uri
        };

        let full_path = self.base_path.join(file_path);

        // Security check - ensure path is within base directory
        let canonical_base = self.base_path.canonicalize().map_err(McpError::io)?;
        let canonical_target = full_path
            .canonicalize()
            .map_err(|_| McpError::ResourceNotFound(uri.to_string()))?;

        if !canonical_target.starts_with(&canonical_base) {
            return Err(McpError::validation("Path outside of allowed directory"));
        }

        if !self.is_allowed_file(&canonical_target) {
            return Err(McpError::validation("File type not allowed"));
        }

        let content = tokio::fs::read_to_string(&canonical_target)
            .await
            .map_err(|_| McpError::ResourceNotFound(uri.to_string()))?;

        let mime_type = self.get_mime_type(&canonical_target);

        Ok(vec![ResourceContents::Text {
            uri: uri.to_string(),
            mime_type: Some(mime_type),
            text: content,
            meta: None,
        }])
    }

    async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
        let mut resources = Vec::new();
        let mut stack = vec![self.base_path.clone()];

        while let Some(dir_path) = stack.pop() {
            let mut dir = tokio::fs::read_dir(&dir_path).await.map_err(McpError::io)?;

            while let Some(entry) = dir.next_entry().await.map_err(McpError::io)? {
                let path = entry.path();

                if path.is_dir() {
                    stack.push(path);
                } else if self.is_allowed_file(&path) {
                    let relative_path = path
                        .strip_prefix(&self.base_path)
                        .map_err(|_| McpError::internal("Path computation error"))?;

                    let path_display = relative_path.display();
                    let uri = format!("file://{path_display}");
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unnamed")
                        .to_string();

                    resources.push(ResourceInfo {
                        uri,
                        name,
                        description: None,
                        mime_type: Some(self.get_mime_type(&path)),
                        annotations: None,
                        size: None,
                        title: None,
                        meta: None,
                    });
                }
            }
        }

        Ok(resources)
    }
}

/// Builder for creating resources with fluent API
pub struct ResourceBuilder {
    uri: String,
    name: String,
    description: Option<String>,
    mime_type: Option<String>,
}

impl ResourceBuilder {
    /// Create a new resource builder
    pub fn new<S: Into<String>>(uri: S, name: S) -> Self {
        Self {
            uri: uri.into(),
            name: name.into(),
            description: None,
            mime_type: None,
        }
    }

    /// Set the resource description
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the MIME type
    pub fn mime_type<S: Into<String>>(mut self, mime_type: S) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Build the resource with the given handler
    pub fn build<H>(self, handler: H) -> Resource
    where
        H: ResourceHandler + 'static,
    {
        let info = ResourceInfo {
            uri: self.uri,
            name: self.name,
            description: self.description,
            mime_type: self.mime_type,
            annotations: None,
            size: None,
            title: None,
            meta: None,
        };

        Resource::new(info, handler)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_text_resource() {
        let resource =
            TextResource::new("Hello, World!".to_string(), Some("text/plain".to_string()));
        let params = HashMap::new();

        let content = resource.read("test://resource", &params).await.unwrap();
        assert_eq!(content.len(), 1);
        match &content[0] {
            ResourceContents::Text {
                text, mime_type, ..
            } => {
                assert_eq!(*text, "Hello, World!".to_string());
                assert_eq!(*mime_type, Some("text/plain".to_string()));
            }
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_resource_creation() {
        let info = ResourceInfo {
            uri: "test://resource".to_string(),
            name: "Test Resource".to_string(),
            description: Some("A test resource".to_string()),
            mime_type: Some("text/plain".to_string()),
            annotations: None,
            size: None,
            title: None,
            meta: None,
        };

        let resource = Resource::new(info.clone(), TextResource::new("test".to_string(), None));
        assert_eq!(resource.info, info);
        assert!(resource.is_enabled());
    }

    #[test]
    fn test_resource_template() {
        let template = ResourceTemplate {
            uri_template: "test://resource/{id}".to_string(),
            name: "Test Template".to_string(),
            description: Some("A test template".to_string()),
            mime_type: Some("text/plain".to_string()),
        };

        let resource = Resource::with_template(
            template.clone(),
            TextResource::new("test".to_string(), None),
        );
        assert_eq!(resource.template, Some(template));
    }

    #[test]
    fn test_resource_uri_matching() {
        let template = ResourceTemplate {
            uri_template: "test://resource/{id}".to_string(),
            name: "Test Template".to_string(),
            description: None,
            mime_type: None,
        };

        let resource =
            Resource::with_template(template, TextResource::new("test".to_string(), None));

        // Simple test - real implementation would need proper URI template matching
        assert!(resource.matches_uri("test://resource/123"));
        assert!(!resource.matches_uri("other://resource/123"));
    }

    #[test]
    fn test_resource_builder() {
        let resource = ResourceBuilder::new("test://resource", "Test Resource")
            .description("A test resource")
            .mime_type("text/plain")
            .build(TextResource::new("test".to_string(), None));

        assert_eq!(resource.info.uri, "test://resource");
        assert_eq!(resource.info.name, "Test Resource");
        assert_eq!(
            resource.info.description,
            Some("A test resource".to_string())
        );
        assert_eq!(resource.info.mime_type, Some("text/plain".to_string()));
    }
}
