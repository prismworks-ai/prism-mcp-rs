// ! Tool Discovery and Management System
// !
// ! Module provides complete tool discovery, filtering, and management capabilities
// ! based on the improved metadata system. It allows smart tool selection,
// ! categorization, performance monitoring, and lifecycle management.

use crate::core::error::{McpError, McpResult};
use crate::core::tool::Tool;
use crate::core::tool_metadata::{
    CategoryFilter, DeprecationSeverity, ImprovedToolMetadata, ToolBehaviorHints,
};
#[cfg(feature = "chrono")]
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

/// Tool discovery and management system
pub struct ToolRegistry {
    /// Registered tools indexed by name
    tools: HashMap<String, Tool>,
    /// Tool execution statistics
    global_stats: GlobalToolStats,
}

/// Global statistics across all tools
#[derive(Debug, Clone)]
pub struct GlobalToolStats {
    /// Total number of registered tools
    pub total_tools: usize,
    /// Number of deprecated tools
    pub deprecated_tools: usize,
    /// Number of disabled tools
    pub disabled_tools: usize,
    /// Total executions across all tools
    pub total_executions: u64,
    /// Total successful executions
    pub total_successes: u64,
    /// Overall success rate
    pub overall_success_rate: f64,
    /// Most frequently used tool
    pub most_used_tool: Option<String>,
    /// Most reliable tool (highest success rate)
    pub most_reliable_tool: Option<String>,
}

impl Default for GlobalToolStats {
    fn default() -> Self {
        Self {
            total_tools: 0,
            deprecated_tools: 0,
            disabled_tools: 0,
            total_executions: 0,
            total_successes: 0,
            overall_success_rate: 0.0,
            most_used_tool: None,
            most_reliable_tool: None,
        }
    }
}

/// Tool discovery result with ranking information
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// Tool name
    pub name: String,
    /// Match score (0.0 to 1.0, higher is better)
    pub match_score: f64,
    /// Reason for recommendation
    pub recommendation_reason: String,
    /// Tool metadata snapshot
    pub metadata: ImprovedToolMetadata,
    /// Whether tool is deprecated
    pub is_deprecated: bool,
    /// Whether tool is enabled
    pub is_enabled: bool,
}

/// Tool discovery criteria
#[derive(Debug, Clone, Default)]
pub struct DiscoveryCriteria {
    /// Category filter
    pub category_filter: Option<CategoryFilter>,
    /// Required behavior hints
    pub required_hints: ToolBehaviorHints,
    /// Preferred behavior hints (for ranking)
    pub preferred_hints: ToolBehaviorHints,
    /// Exclude deprecated tools
    pub exclude_deprecated: bool,
    /// Exclude disabled tools
    pub exclude_disabled: bool,
    /// Minimum success rate (0.0 to 1.0)
    pub min_success_rate: Option<f64>,
    /// Maximum average execution time
    pub max_execution_time: Option<Duration>,
    /// Text search in name/description
    pub text_search: Option<String>,
    /// Minimum number of executions (for reliability filtering)
    pub min_executions: Option<u64>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            global_stats: GlobalToolStats::default(),
        }
    }

    /// Register a tool in the registry
    pub fn register_tool(&mut self, tool: Tool) -> McpResult<()> {
        let name = tool.info.name.clone();

        if self.tools.contains_key(&name) {
            return Err(McpError::validation(format!(
                "Tool '{name}' is already registered"
            )));
        }

        self.tools.insert(name, tool);
        self.update_global_stats();
        Ok(())
    }

    /// Unregister a tool from the registry
    pub fn unregister_tool(&mut self, name: &str) -> McpResult<Tool> {
        let tool = self
            .tools
            .remove(name)
            .ok_or_else(|| McpError::validation(format!("Tool '{name}' not found")))?;

        self.update_global_stats();
        Ok(tool)
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// Get a mutable reference to a tool by name
    pub fn get_tool_mut(&mut self, name: &str) -> Option<&mut Tool> {
        self.tools.get_mut(name)
    }

    /// List all tool names
    pub fn list_tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Discover tools based on criteria
    pub fn discover_tools(&self, criteria: &DiscoveryCriteria) -> Vec<DiscoveryResult> {
        let mut results = Vec::new();

        for (name, tool) in &self.tools {
            if let Some(result) = self.evaluate_tool_match(name, tool, criteria) {
                results.push(result);
            }
        }

        // Sort by match score (descending)
        results.sort_by(|a, b| {
            b.match_score
                .partial_cmp(&a.match_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Get tools by category
    pub fn get_tools_by_category(&self, filter: &CategoryFilter) -> Vec<String> {
        self.tools
            .iter()
            .filter(|(_, tool)| tool.matches_category_filter(filter))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get deprecated tools
    pub fn get_deprecated_tools(&self) -> Vec<String> {
        self.tools
            .iter()
            .filter(|(_, tool)| tool.is_deprecated())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get disabled tools
    pub fn get_disabled_tools(&self) -> Vec<String> {
        self.tools
            .iter()
            .filter(|(_, tool)| !tool.is_enabled())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get performance report for all tools
    pub fn get_performance_report(
        &self,
    ) -> HashMap<String, crate::core::tool_metadata::ToolPerformanceMetrics> {
        self.tools
            .iter()
            .map(|(name, tool)| (name.clone(), tool.performance_metrics()))
            .collect()
    }

    /// Get global statistics
    pub fn get_global_stats(&self) -> &GlobalToolStats {
        &self.global_stats
    }

    /// Recommend best tool for a specific use case
    pub fn recommend_tool(
        &self,
        use_case: &str,
        criteria: &DiscoveryCriteria,
    ) -> Option<DiscoveryResult> {
        let mut improved_criteria = criteria.clone();

        // Add text search based on use case
        improved_criteria.text_search = Some(use_case.to_string());

        let results = self.discover_tools(&improved_criteria);
        results.into_iter().next()
    }

    /// Clean up deprecated tools based on policy
    pub fn cleanup_deprecated_tools(&mut self, policy: &DeprecationCleanupPolicy) -> Vec<String> {
        let mut removed_tools = Vec::new();

        #[cfg(feature = "chrono")]
        let current_time = Utc::now();

        let tools_to_remove: Vec<String> = self
            .tools
            .iter()
            .filter(|(_, tool)| {
                if let Some(ref deprecation) = tool.improved_metadata.deprecation {
                    if !deprecation.deprecated {
                        return false;
                    }

                    // Check severity-based removal
                    if matches!(deprecation.severity, DeprecationSeverity::Critical) {
                        return true;
                    }

                    #[cfg(feature = "chrono")]
                    {
                        // Check time-based removal
                        if let Some(removal_date) = deprecation.removal_date {
                            if current_time >= removal_date {
                                return true;
                            }
                        }

                        // Check age-based removal
                        if let Some(deprecated_date) = deprecation.deprecated_date {
                            let age = current_time.signed_duration_since(deprecated_date);
                            if age.num_days() > policy.max_deprecated_days as i64 {
                                return true;
                            }
                        }
                    }
                    #[cfg(not(feature = "chrono"))]
                    {
                        // Without chrono, we can't check time-based removal
                        let _ = policy; // Suppress unused warning
                    }
                }
                false
            })
            .map(|(name, _)| name.clone())
            .collect();

        for name in tools_to_remove {
            if self.tools.remove(&name).is_some() {
                removed_tools.push(name);
            }
        }

        if !removed_tools.is_empty() {
            self.update_global_stats();
        }

        removed_tools
    }

    /// Update global statistics
    fn update_global_stats(&mut self) {
        let mut stats = GlobalToolStats {
            total_tools: self.tools.len(),
            ..Default::default()
        };

        let mut max_executions = 0u64;
        let mut max_success_rate = 0.0f64;
        let mut most_used = None;
        let mut most_reliable = None;

        for (name, tool) in &self.tools {
            let metrics = tool.performance_metrics();

            if tool.is_deprecated() {
                stats.deprecated_tools += 1;
            }

            if !tool.is_enabled() {
                stats.disabled_tools += 1;
            }

            stats.total_executions += metrics.execution_count;
            stats.total_successes += metrics.success_count;

            // Track most used tool
            if metrics.execution_count > max_executions {
                max_executions = metrics.execution_count;
                most_used = Some(name.clone());
            }

            // Track most reliable tool (with minimum executions)
            if metrics.execution_count >= 5 && metrics.success_rate > max_success_rate {
                max_success_rate = metrics.success_rate;
                most_reliable = Some(name.clone());
            }
        }

        if stats.total_executions > 0 {
            stats.overall_success_rate =
                (stats.total_successes as f64 / stats.total_executions as f64) * 100.0;
        }

        stats.most_used_tool = most_used;
        stats.most_reliable_tool = most_reliable;
        self.global_stats = stats;
    }

    /// Evaluate how well a tool matches the discovery criteria
    fn evaluate_tool_match(
        &self,
        name: &str,
        tool: &Tool,
        criteria: &DiscoveryCriteria,
    ) -> Option<DiscoveryResult> {
        let mut score = 0.0f64;
        let mut reasons = Vec::new();

        // Filter out tools that don't meet basic criteria
        if criteria.exclude_deprecated && tool.is_deprecated() {
            return None;
        }

        if criteria.exclude_disabled && !tool.is_enabled() {
            return None;
        }

        let metrics = tool.performance_metrics();

        // Filter by minimum success rate
        if let Some(min_rate) = criteria.min_success_rate {
            if metrics.execution_count > 0 && metrics.success_rate < min_rate * 100.0 {
                return None;
            }
        }

        // Filter by maximum execution time
        if let Some(max_time) = criteria.max_execution_time {
            if metrics.execution_count > 0 && metrics.average_execution_time > max_time {
                return None;
            }
        }

        // Filter by minimum executions
        if let Some(min_execs) = criteria.min_executions {
            if metrics.execution_count < min_execs {
                return None;
            }
        }

        // Category matching
        if let Some(ref filter) = criteria.category_filter {
            if tool.matches_category_filter(filter) {
                score += 0.3;
                reasons.push("matches category criteria".to_string());
            } else {
                return None;
            }
        }

        // Text search matching
        if let Some(ref search_text) = criteria.text_search {
            let search_lower = search_text.to_lowercase();
            let name_match = name.to_lowercase().contains(&search_lower);
            let desc_match = tool
                .info
                .description
                .as_ref()
                .map(|d| d.to_lowercase().contains(&search_lower))
                .unwrap_or(false);

            if name_match || desc_match {
                score += if name_match { 0.4 } else { 0.2 };
                reasons.push("matches text search".to_string());
            } else {
                // If text search is specified but doesn't match, exclude this tool
                return None;
            }
        }

        // Behavior hints matching - check required hints first
        let hints = tool.behavior_hints();

        // Filter out tools that don't meet required hints
        if criteria.required_hints.read_only.unwrap_or(false) && !hints.read_only.unwrap_or(false) {
            return None;
        }
        if criteria.required_hints.idempotent.unwrap_or(false) && !hints.idempotent.unwrap_or(false)
        {
            return None;
        }
        if criteria.required_hints.cacheable.unwrap_or(false) && !hints.cacheable.unwrap_or(false) {
            return None;
        }
        if criteria.required_hints.destructive.unwrap_or(false)
            && !hints.destructive.unwrap_or(false)
        {
            return None;
        }
        if criteria.required_hints.requires_auth.unwrap_or(false)
            && !hints.requires_auth.unwrap_or(false)
        {
            return None;
        }

        // Add score bonuses for meeting required hints
        if criteria.required_hints.read_only.unwrap_or(false) && hints.read_only.unwrap_or(false) {
            score += 0.2;
            reasons.push("read-only as required".to_string());
        }
        if criteria.required_hints.idempotent.unwrap_or(false) && hints.idempotent.unwrap_or(false)
        {
            score += 0.2;
            reasons.push("idempotent as required".to_string());
        }
        if criteria.required_hints.cacheable.unwrap_or(false) && hints.cacheable.unwrap_or(false) {
            score += 0.15;
            reasons.push("cacheable as required".to_string());
        }

        // Preferred hints bonus
        if criteria.preferred_hints.read_only.unwrap_or(false) && hints.read_only.unwrap_or(false) {
            score += 0.1;
            reasons.push("preferred: read-only".to_string());
        }
        if criteria.preferred_hints.idempotent.unwrap_or(false) && hints.idempotent.unwrap_or(false)
        {
            score += 0.1;
            reasons.push("preferred: idempotent".to_string());
        }

        // Performance-based scoring
        if metrics.execution_count > 0 {
            // Success rate bonus
            let success_bonus = (metrics.success_rate / 100.0) * 0.2;
            score += success_bonus;

            // Usage frequency bonus (logarithmic scale)
            let usage_bonus = (metrics.execution_count as f64).ln() * 0.05;
            score += usage_bonus.min(0.15);

            if metrics.success_rate > 95.0 {
                reasons.push("high reliability".to_string());
            }
            if metrics.execution_count > 100 {
                reasons.push("well-tested".to_string());
            }
        }

        // Deprecation penalty
        if tool.is_deprecated() {
            score *= 0.5;
            reasons.push("deprecated (reduced score)".to_string());
        }

        // Disabled penalty
        if !tool.is_enabled() {
            score *= 0.1;
            reasons.push("disabled (reduced score)".to_string());
        }

        Some(DiscoveryResult {
            name: name.to_string(),
            match_score: score.min(1.0),
            recommendation_reason: reasons.join(", "),
            metadata: tool.improved_metadata.clone(),
            is_deprecated: tool.is_deprecated(),
            is_enabled: tool.is_enabled(),
        })
    }
}

/// Policy for cleaning up deprecated tools
#[derive(Debug, Clone)]
pub struct DeprecationCleanupPolicy {
    /// Maximum number of days to keep deprecated tools
    pub max_deprecated_days: u32,
    /// Remove tools marked as critical immediately
    pub remove_critical_immediately: bool,
}

impl Default for DeprecationCleanupPolicy {
    fn default() -> Self {
        Self {
            max_deprecated_days: 90,
            remove_critical_immediately: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tool::{ToolBuilder, ToolHandler};
    use crate::core::tool_metadata::*;
    use async_trait::async_trait;
    use serde_json::Value;
    use std::collections::HashMap;

    struct MockHandler {
        result: String,
    }

    #[async_trait]
    impl ToolHandler for MockHandler {
        async fn call(
            &self,
            _args: HashMap<String, Value>,
        ) -> McpResult<crate::protocol::types::ToolResult> {
            Ok(crate::protocol::types::ToolResult {
                content: vec![crate::protocol::types::ContentBlock::Text {
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

    #[test]
    fn test_tool_registry_basic_operations() {
        let mut registry = ToolRegistry::new();

        let tool = ToolBuilder::new("test_tool")
            .description("A test tool")
            .build(MockHandler {
                result: "test".to_string(),
            })
            .unwrap();

        // Register tool
        registry.register_tool(tool).unwrap();
        assert_eq!(registry.list_tool_names().len(), 1);
        assert!(registry.get_tool("test_tool").is_some());

        // Try to register duplicate - should fail
        let duplicate_tool = ToolBuilder::new("test_tool")
            .build(MockHandler {
                result: "duplicate".to_string(),
            })
            .unwrap();
        assert!(registry.register_tool(duplicate_tool).is_err());

        // Unregister tool
        let removed = registry.unregister_tool("test_tool").unwrap();
        assert_eq!(removed.info.name, "test_tool");
        assert_eq!(registry.list_tool_names().len(), 0);
    }

    #[test]
    fn test_tool_discovery_by_category() {
        let mut registry = ToolRegistry::new();

        // Add tools with different categories
        let file_tool = ToolBuilder::new("file_reader")
            .category_simple("file".to_string(), Some("read".to_string()))
            .tag("filesystem".to_string())
            .build(MockHandler {
                result: "file".to_string(),
            })
            .unwrap();

        let network_tool = ToolBuilder::new("http_client")
            .category_simple("network".to_string(), Some("http".to_string()))
            .tag("client".to_string())
            .build(MockHandler {
                result: "network".to_string(),
            })
            .unwrap();

        registry.register_tool(file_tool).unwrap();
        registry.register_tool(network_tool).unwrap();

        // Test category filtering
        let file_filter = CategoryFilter::new().with_primary("file".to_string());
        let file_tools = registry.get_tools_by_category(&file_filter);
        assert_eq!(file_tools.len(), 1);
        assert!(file_tools.contains(&"file_reader".to_string()));

        let network_filter = CategoryFilter::new().with_primary("network".to_string());
        let network_tools = registry.get_tools_by_category(&network_filter);
        assert_eq!(network_tools.len(), 1);
        assert!(network_tools.contains(&"http_client".to_string()));
    }

    #[test]
    fn test_tool_discovery_criteria() {
        let mut registry = ToolRegistry::new();

        // Add tools with different characteristics
        let read_only_tool = ToolBuilder::new("reader")
            .description("Reads data")
            .read_only()
            .idempotent()
            .cacheable()
            .build(MockHandler {
                result: "read".to_string(),
            })
            .unwrap();

        let destructive_tool = ToolBuilder::new("deleter")
            .description("Deletes data")
            .destructive()
            .build(MockHandler {
                result: "delete".to_string(),
            })
            .unwrap();

        let deprecated_tool = ToolBuilder::new("old_tool")
            .description("Old tool")
            .deprecated_simple("Use new_tool instead")
            .build(MockHandler {
                result: "old".to_string(),
            })
            .unwrap();

        registry.register_tool(read_only_tool).unwrap();
        registry.register_tool(destructive_tool).unwrap();
        registry.register_tool(deprecated_tool).unwrap();

        // Test discovery with read-only requirement
        let criteria = DiscoveryCriteria {
            required_hints: ToolBehaviorHints::new().read_only(),
            exclude_deprecated: false,
            exclude_disabled: false,
            ..Default::default()
        };

        let results = registry.discover_tools(&criteria);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "reader");

        // Test discovery excluding deprecated
        let criteria = DiscoveryCriteria {
            exclude_deprecated: true,
            ..Default::default()
        };

        let results = registry.discover_tools(&criteria);
        assert_eq!(results.len(), 2); // Should exclude deprecated tool
        assert!(!results.iter().any(|r| r.name == "old_tool"));

        // Test text search
        let criteria = DiscoveryCriteria {
            text_search: Some("delete".to_string()),
            exclude_deprecated: false,
            ..Default::default()
        };

        let results = registry.discover_tools(&criteria);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "deleter");
    }

    #[test]
    fn test_global_statistics() {
        let mut registry = ToolRegistry::new();

        let tool1 = ToolBuilder::new("tool1")
            .build(MockHandler {
                result: "1".to_string(),
            })
            .unwrap();

        let tool2 = ToolBuilder::new("tool2")
            .deprecated_simple("Old tool")
            .build(MockHandler {
                result: "2".to_string(),
            })
            .unwrap();

        registry.register_tool(tool1).unwrap();
        registry.register_tool(tool2).unwrap();

        let stats = registry.get_global_stats();
        assert_eq!(stats.total_tools, 2);
        assert_eq!(stats.deprecated_tools, 1);
        assert_eq!(stats.disabled_tools, 0);
    }

    #[test]
    fn test_tool_recommendation() {
        let mut registry = ToolRegistry::new();

        let file_tool = ToolBuilder::new("file_processor")
            .description("Processes files efficiently")
            .category_simple("file".to_string(), Some("process".to_string()))
            .read_only()
            .build(MockHandler {
                result: "processed".to_string(),
            })
            .unwrap();

        let network_tool = ToolBuilder::new("network_handler")
            .description("Handles network requests")
            .category_simple("network".to_string(), None)
            .build(MockHandler {
                result: "handled".to_string(),
            })
            .unwrap();

        registry.register_tool(file_tool).unwrap();
        registry.register_tool(network_tool).unwrap();

        // Recommend tool for file processing
        let criteria = DiscoveryCriteria::default();
        let recommendation = registry.recommend_tool("file", &criteria);

        assert!(recommendation.is_some());
        let result = recommendation.unwrap();
        assert_eq!(result.name, "file_processor");
        assert!(result.match_score > 0.0);
        assert!(result.recommendation_reason.contains("matches text search"));
    }

    #[test]
    fn test_deprecation_cleanup() {
        let mut registry = ToolRegistry::new();

        // Add tools with different deprecation states
        let normal_tool = ToolBuilder::new("normal")
            .build(MockHandler {
                result: "normal".to_string(),
            })
            .unwrap();

        let deprecated_tool = ToolBuilder::new("deprecated")
            .deprecated(
                ToolDeprecation::new("Old version".to_string())
                    .with_severity(DeprecationSeverity::Low),
            )
            .build(MockHandler {
                result: "deprecated".to_string(),
            })
            .unwrap();

        let critical_tool = ToolBuilder::new("critical")
            .deprecated(
                ToolDeprecation::new("Security issue".to_string())
                    .with_severity(DeprecationSeverity::Critical),
            )
            .build(MockHandler {
                result: "critical".to_string(),
            })
            .unwrap();

        registry.register_tool(normal_tool).unwrap();
        registry.register_tool(deprecated_tool).unwrap();
        registry.register_tool(critical_tool).unwrap();

        assert_eq!(registry.list_tool_names().len(), 3);

        // Clean up with default policy (should remove critical tools)
        let policy = DeprecationCleanupPolicy::default();
        let removed = registry.cleanup_deprecated_tools(&policy);

        assert_eq!(removed.len(), 1);
        assert!(removed.contains(&"critical".to_string()));
        assert_eq!(registry.list_tool_names().len(), 2);
    }
}
