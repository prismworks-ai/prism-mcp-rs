// ! improved Tool Metadata System
// !
// ! Module provides complete metadata features for MCP tools including:
// ! - Tool behavior hints (readOnly, destructive, idempotent)
// ! - Tool categorization and tagging
// ! - Discovery and filtering capabilities
// ! - Performance metrics and tracking
// ! - Deprecation warnings and versioning

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Tool behavior hints for clients to understand tool characteristics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolBehaviorHints {
    /// Tool only reads data without making changes
    #[serde(rename = "readOnlyHint", skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    /// Tool makes destructive changes that cannot be easily undone
    #[serde(rename = "destructiveHint", skip_serializing_if = "Option::is_none")]
    pub destructive: Option<bool>,

    /// Tool produces the same output for the same input (no side effects)
    #[serde(rename = "idempotentHint", skip_serializing_if = "Option::is_none")]
    pub idempotent: Option<bool>,

    /// Tool requires authentication or special permissions
    #[serde(rename = "requiresAuthHint", skip_serializing_if = "Option::is_none")]
    pub requires_auth: Option<bool>,

    /// Tool may take a long time to execute
    #[serde(rename = "longRunningHint", skip_serializing_if = "Option::is_none")]
    pub long_running: Option<bool>,

    /// Tool may consume significant system resources
    #[serde(
        rename = "resourceIntensiveHint",
        skip_serializing_if = "Option::is_none"
    )]
    pub resource_intensive: Option<bool>,

    /// Tool provides cacheable results
    #[serde(rename = "cacheableHint", skip_serializing_if = "Option::is_none")]
    pub cacheable: Option<bool>,
}

impl ToolBehaviorHints {
    /// Create a new empty set of behavior hints
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark tool as read-only (no destructive changes)
    pub fn read_only(mut self) -> Self {
        self.read_only = Some(true);
        self
    }

    /// Mark tool as destructive (makes changes that cannot be easily undone)
    pub fn destructive(mut self) -> Self {
        self.destructive = Some(true);
        self
    }

    /// Mark tool as idempotent (same input produces same output)
    pub fn idempotent(mut self) -> Self {
        self.idempotent = Some(true);
        self
    }

    /// Mark tool as requiring authentication
    pub fn requires_auth(mut self) -> Self {
        self.requires_auth = Some(true);
        self
    }

    /// Mark tool as potentially long-running
    pub fn long_running(mut self) -> Self {
        self.long_running = Some(true);
        self
    }

    /// Mark tool as resource-intensive
    pub fn resource_intensive(mut self) -> Self {
        self.resource_intensive = Some(true);
        self
    }

    /// Mark tool results as cacheable
    pub fn cacheable(mut self) -> Self {
        self.cacheable = Some(true);
        self
    }
}

/// Tool categorization for organization and discovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCategory {
    /// Primary category (e.g., "file", "network", "data", "ai")
    pub primary: String,
    /// Secondary category (e.g., "read", "write", "analyze", "transform")
    pub secondary: Option<String>,
    /// Custom tags for configurable categorization
    pub tags: HashSet<String>,
}

impl ToolCategory {
    /// Create a new tool category
    pub fn new(primary: String) -> Self {
        Self {
            primary,
            secondary: None,
            tags: HashSet::new(),
        }
    }

    /// Set secondary category
    pub fn with_secondary(mut self, secondary: String) -> Self {
        self.secondary = Some(secondary);
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.insert(tag);
        self
    }

    /// Add multiple tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Check if category matches a filter
    pub fn matches_filter(&self, filter: &CategoryFilter) -> bool {
        // Check primary category
        if let Some(ref primary) = filter.primary {
            if !self.primary.contains(primary) {
                return false;
            }
        }

        // Check secondary category
        if let Some(ref secondary) = filter.secondary {
            match &self.secondary {
                Some(s) => {
                    if !s.contains(secondary) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Check tags (any match is sufficient)
        if !filter.tags.is_empty() && !filter.tags.iter().any(|tag| self.tags.contains(tag)) {
            return false;
        }

        true
    }
}

/// Filter for tool discovery based on categories
#[derive(Debug, Clone, Default)]
pub struct CategoryFilter {
    /// Filter by primary category (substring match)
    pub primary: Option<String>,
    /// Filter by secondary category (substring match)
    pub secondary: Option<String>,
    /// Filter by tags (any match)
    pub tags: HashSet<String>,
}

impl CategoryFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by primary category
    pub fn with_primary(mut self, primary: String) -> Self {
        self.primary = Some(primary);
        self
    }

    /// Filter by secondary category
    pub fn with_secondary(mut self, secondary: String) -> Self {
        self.secondary = Some(secondary);
        self
    }

    /// Filter by tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.insert(tag);
        self
    }

    /// Filter by multiple tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }
}

/// Performance metrics for tool execution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPerformanceMetrics {
    /// Total number of executions
    pub execution_count: u64,
    /// Total execution time across all calls
    pub total_execution_time: Duration,
    /// Average execution time
    pub average_execution_time: Duration,
    /// Minimum execution time recorded
    pub min_execution_time: Duration,
    /// Maximum execution time recorded
    pub max_execution_time: Duration,
    /// Number of successful executions
    pub success_count: u64,
    /// Number of failed executions
    pub error_count: u64,
    /// Success rate as percentage (0.0 to 100.0)
    pub success_rate: f64,
    /// Last execution timestamp
    pub last_execution: Option<DateTime<Utc>>,
    /// Recent execution times (last 10 executions)
    pub recent_execution_times: Vec<Duration>,
}

impl Default for ToolPerformanceMetrics {
    fn default() -> Self {
        Self {
            execution_count: 0,
            total_execution_time: Duration::from_secs(0),
            average_execution_time: Duration::from_secs(0),
            min_execution_time: Duration::from_secs(u64::MAX),
            max_execution_time: Duration::from_secs(0),
            success_count: 0,
            error_count: 0,
            success_rate: 0.0,
            last_execution: None,
            recent_execution_times: Vec::new(),
        }
    }
}

impl ToolPerformanceMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful execution
    pub fn record_success(&mut self, execution_time: Duration) {
        self.execution_count += 1;
        self.success_count += 1;
        self.record_execution_time(execution_time);
        self.update_success_rate();
        self.last_execution = Some(Utc::now());
    }

    /// Record a failed execution
    pub fn record_error(&mut self, execution_time: Duration) {
        self.execution_count += 1;
        self.error_count += 1;
        self.record_execution_time(execution_time);
        self.update_success_rate();
        self.last_execution = Some(Utc::now());
    }

    /// Record execution time and update statistics
    fn record_execution_time(&mut self, execution_time: Duration) {
        self.total_execution_time += execution_time;

        // Update min/max
        if execution_time < self.min_execution_time {
            self.min_execution_time = execution_time;
        }
        if execution_time > self.max_execution_time {
            self.max_execution_time = execution_time;
        }

        // Update average
        if self.execution_count > 0 {
            self.average_execution_time = self.total_execution_time / self.execution_count as u32;
        }

        // Update recent execution times (keep last 10)
        self.recent_execution_times.push(execution_time);
        if self.recent_execution_times.len() > 10 {
            self.recent_execution_times.remove(0);
        }
    }

    /// Update success rate percentage
    fn update_success_rate(&mut self) {
        if self.execution_count > 0 {
            self.success_rate = (self.success_count as f64 / self.execution_count as f64) * 100.0;
        }
    }

    /// Get recent average execution time (last 10 executions)
    pub fn recent_average_execution_time(&self) -> Duration {
        if self.recent_execution_times.is_empty() {
            Duration::from_secs(0)
        } else {
            let total: Duration = self.recent_execution_times.iter().sum();
            total / self.recent_execution_times.len() as u32
        }
    }
}

/// Tool deprecation information and versioning
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDeprecation {
    /// Whether the tool is deprecated
    pub deprecated: bool,
    /// Deprecation reason/message
    pub reason: Option<String>,
    /// Recommended replacement tool
    pub replacement: Option<String>,
    /// Date when tool was deprecated
    pub deprecated_date: Option<DateTime<Utc>>,
    /// Date when tool will be removed (if known)
    pub removal_date: Option<DateTime<Utc>>,
    /// Severity of deprecation warning
    pub severity: DeprecationSeverity,
}

/// Severity levels for deprecation warnings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum DeprecationSeverity {
    /// Tool is deprecated but still fully functional
    #[default]
    Low,
    /// Tool may have reduced functionality or support
    Medium,
    /// Tool will be removed soon or has significant issues
    High,
    /// Tool is disabled or non-functional
    Critical,
}

impl ToolDeprecation {
    /// Create a new deprecation notice
    pub fn new(reason: String) -> Self {
        Self {
            deprecated: true,
            reason: Some(reason),
            replacement: None,
            deprecated_date: Some(Utc::now()),
            removal_date: None,
            severity: DeprecationSeverity::Low,
        }
    }

    /// Set replacement tool
    pub fn with_replacement(mut self, replacement: String) -> Self {
        self.replacement = Some(replacement);
        self
    }

    /// Set removal date
    pub fn with_removal_date(mut self, removal_date: DateTime<Utc>) -> Self {
        self.removal_date = Some(removal_date);
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: DeprecationSeverity) -> Self {
        self.severity = severity;
        self
    }
}

/// complete improved metadata for tools
#[derive(Debug, Clone)]
pub struct ImprovedToolMetadata {
    /// Tool behavior hints for client understanding
    pub behavior_hints: ToolBehaviorHints,
    /// Tool categorization for organization
    pub category: Option<ToolCategory>,
    /// Performance tracking metrics (using thread-safe interior mutability)
    pub performance: Arc<RwLock<ToolPerformanceMetrics>>,
    /// Deprecation information
    pub deprecation: Option<ToolDeprecation>,
    /// Tool version information
    pub version: Option<String>,
    /// Author/maintainer information
    pub author: Option<String>,
    /// Custom metadata fields
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for ImprovedToolMetadata {
    fn default() -> Self {
        Self {
            behavior_hints: ToolBehaviorHints::default(),
            category: None,
            performance: Arc::new(RwLock::new(ToolPerformanceMetrics::default())),
            deprecation: None,
            version: None,
            author: None,
            custom: HashMap::new(),
        }
    }
}

impl ImprovedToolMetadata {
    /// Create new improved metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Set behavior hints
    pub fn with_behavior_hints(mut self, hints: ToolBehaviorHints) -> Self {
        self.behavior_hints = hints;
        self
    }

    /// Set category
    pub fn with_category(mut self, category: ToolCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set version
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Set author
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    /// Add custom metadata field
    pub fn with_custom_field(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom.insert(key, value);
        self
    }

    /// Deprecate the tool
    pub fn deprecated(mut self, deprecation: ToolDeprecation) -> Self {
        self.deprecation = Some(deprecation);
        self
    }

    /// Check if tool is deprecated
    pub fn is_deprecated(&self) -> bool {
        self.deprecation.as_ref().is_some_and(|d| d.deprecated)
    }

    /// Get deprecation warning message
    pub fn deprecation_warning(&self) -> Option<String> {
        self.deprecation.as_ref().and_then(|d| {
            if d.deprecated {
                let mut warning = "Tool is deprecated".to_string();
                if let Some(ref reason) = d.reason {
                    warning.push_str(&format!(": {reason}"));
                }
                if let Some(ref replacement) = d.replacement {
                    warning.push_str(&format!(". Use '{replacement}' instead"));
                }
                Some(warning)
            } else {
                None
            }
        })
    }

    /// Record a successful execution (with thread-safe interior mutability)
    pub fn record_success(&self, execution_time: Duration) {
        if let Ok(mut perf) = self.performance.write() {
            perf.record_success(execution_time);
        }
    }

    /// Record a failed execution (with thread-safe interior mutability)
    pub fn record_error(&self, execution_time: Duration) {
        if let Ok(mut perf) = self.performance.write() {
            perf.record_error(execution_time);
        }
    }

    /// Get performance metrics snapshot
    pub fn get_performance_snapshot(&self) -> ToolPerformanceMetrics {
        self.performance
            .read()
            .map(|p| p.clone())
            .unwrap_or_default()
    }

    /// Get execution count
    pub fn execution_count(&self) -> u64 {
        self.performance
            .read()
            .map(|p| p.execution_count)
            .unwrap_or(0)
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        self.performance
            .read()
            .map(|p| p.success_rate)
            .unwrap_or(0.0)
    }

    /// Get average execution time
    pub fn average_execution_time(&self) -> Duration {
        self.performance
            .read()
            .map(|p| p.average_execution_time)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_behavior_hints() {
        let hints = ToolBehaviorHints::new()
            .read_only()
            .idempotent()
            .cacheable();

        assert_eq!(hints.read_only, Some(true));
        assert_eq!(hints.idempotent, Some(true));
        assert_eq!(hints.cacheable, Some(true));
        assert_eq!(hints.destructive, None);
    }

    #[test]
    fn test_tool_category() {
        let category = ToolCategory::new("file".to_string())
            .with_secondary("read".to_string())
            .with_tag("filesystem".to_string())
            .with_tag("utility".to_string());

        assert_eq!(category.primary, "file");
        assert_eq!(category.secondary, Some("read".to_string()));
        assert!(category.tags.contains("filesystem"));
        assert!(category.tags.contains("utility"));
    }

    #[test]
    fn test_category_filter() {
        let category = ToolCategory::new("file".to_string())
            .with_secondary("read".to_string())
            .with_tag("filesystem".to_string());

        let filter = CategoryFilter::new().with_primary("file".to_string());

        assert!(category.matches_filter(&filter));

        let filter = CategoryFilter::new().with_primary("network".to_string());

        assert!(!category.matches_filter(&filter));

        let filter = CategoryFilter::new().with_tag("filesystem".to_string());

        assert!(category.matches_filter(&filter));
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = ToolPerformanceMetrics::new();

        metrics.record_success(Duration::from_millis(100));
        metrics.record_success(Duration::from_millis(200));
        metrics.record_error(Duration::from_millis(150));

        assert_eq!(metrics.execution_count, 3);
        assert_eq!(metrics.success_count, 2);
        assert_eq!(metrics.error_count, 1);
        assert!((metrics.success_rate - 66.66666666666667).abs() < 0.001);
        assert_eq!(metrics.min_execution_time, Duration::from_millis(100));
        assert_eq!(metrics.max_execution_time, Duration::from_millis(200));
    }

    #[test]
    fn test_tool_deprecation() {
        let deprecation = ToolDeprecation::new("Tool is no longer maintained".to_string())
            .with_replacement("new_tool".to_string())
            .with_severity(DeprecationSeverity::High);

        assert!(deprecation.deprecated);
        assert_eq!(
            deprecation.reason,
            Some("Tool is no longer maintained".to_string())
        );
        assert_eq!(deprecation.replacement, Some("new_tool".to_string()));
        assert_eq!(deprecation.severity, DeprecationSeverity::High);
    }

    #[test]
    fn test_improved_metadata() {
        let hints = ToolBehaviorHints::new().read_only().cacheable();
        let category = ToolCategory::new("data".to_string()).with_tag("analysis".to_string());

        let metadata = ImprovedToolMetadata::new()
            .with_behavior_hints(hints)
            .with_category(category)
            .with_version("1.0.0".to_string())
            .with_author("Test Author".to_string());

        assert_eq!(metadata.behavior_hints.read_only, Some(true));
        assert_eq!(metadata.behavior_hints.cacheable, Some(true));
        assert!(metadata.category.is_some());
        assert_eq!(metadata.version, Some("1.0.0".to_string()));
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert!(!metadata.is_deprecated());
    }

    #[test]
    fn test_deprecation_warning() {
        let deprecation = ToolDeprecation::new("Old implementation".to_string())
            .with_replacement("better_tool".to_string());

        let metadata = ImprovedToolMetadata::new().deprecated(deprecation);

        assert!(metadata.is_deprecated());
        let warning = metadata.deprecation_warning().unwrap();
        assert!(warning.contains("deprecated"));
        assert!(warning.contains("Old implementation"));
        assert!(warning.contains("better_tool"));
    }
}
