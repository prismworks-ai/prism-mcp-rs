//! MCP 2026-07-28 notification subscription types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Notifications selected by a `subscriptions/listen` request.
///
/// Every field is opt-in. The `taskIds` field belongs to the official Tasks
/// extension and is accepted only when that extension was declared by the
/// request's client capabilities.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionFilter {
    #[serde(rename = "toolsListChanged", skip_serializing_if = "Option::is_none")]
    pub tools_list_changed: Option<bool>,
    #[serde(rename = "promptsListChanged", skip_serializing_if = "Option::is_none")]
    pub prompts_list_changed: Option<bool>,
    #[serde(
        rename = "resourcesListChanged",
        skip_serializing_if = "Option::is_none"
    )]
    pub resources_list_changed: Option<bool>,
    #[serde(
        rename = "resourceSubscriptions",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub resource_subscriptions: Vec<String>,
    #[serde(rename = "taskIds", default, skip_serializing_if = "Vec::is_empty")]
    pub task_ids: Vec<String>,
}

impl SubscriptionFilter {
    /// Return true when this filter permits a notification.
    pub fn matches(&self, method: &str, params: Option<&Value>) -> bool {
        match method {
            "notifications/tools/list_changed" => self.tools_list_changed == Some(true),
            "notifications/prompts/list_changed" => self.prompts_list_changed == Some(true),
            "notifications/resources/list_changed" => self.resources_list_changed == Some(true),
            "notifications/resources/updated" => params
                .and_then(|value| value.get("uri"))
                .and_then(Value::as_str)
                .is_some_and(|uri| self.resource_subscriptions.iter().any(|item| item == uri)),
            "notifications/tasks" => params
                .and_then(|value| value.get("taskId"))
                .and_then(Value::as_str)
                .is_some_and(|task_id| self.task_ids.iter().any(|item| item == task_id)),
            _ => false,
        }
    }

    /// Whether the Tasks extension is needed to honor this filter.
    pub fn requests_tasks(&self) -> bool {
        !self.task_ids.is_empty()
    }
}

/// Parameters for `subscriptions/listen`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscriptionsListenParams {
    pub notifications: SubscriptionFilter,
    #[serde(rename = "_meta", default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, Value>,
}

/// Parameters for the first notification on a subscription stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscriptionsAcknowledgedParams {
    pub notifications: SubscriptionFilter,
    #[serde(rename = "_meta")]
    pub meta: HashMap<String, Value>,
}

/// Result sent only when a subscription is closed gracefully.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscriptionsListenResult {
    #[serde(rename = "resultType")]
    pub result_type: String,
    #[serde(rename = "_meta")]
    pub meta: HashMap<String, Value>,
}

/// Reserved metadata key that correlates stream messages with the listen request.
pub const SUBSCRIPTION_ID_META_KEY: &str = "io.modelcontextprotocol/subscriptionId";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_are_strictly_opt_in() {
        let filter = SubscriptionFilter {
            tools_list_changed: Some(true),
            resource_subscriptions: vec!["file:///a".to_string()],
            task_ids: vec!["task-a".to_string()],
            ..Default::default()
        };
        assert!(filter.matches("notifications/tools/list_changed", None));
        assert!(!filter.matches("notifications/prompts/list_changed", None));
        assert!(filter.matches(
            "notifications/resources/updated",
            Some(&serde_json::json!({"uri":"file:///a"}))
        ));
        assert!(!filter.matches(
            "notifications/tasks",
            Some(&serde_json::json!({"taskId":"task-b"}))
        ));
    }
}
