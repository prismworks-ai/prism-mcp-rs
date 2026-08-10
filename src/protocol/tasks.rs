//! `io.modelcontextprotocol/tasks` extension types (SEP-2663).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Official extension identifier.
pub const TASKS_EXTENSION_ID: &str = "io.modelcontextprotocol/tasks";

/// Lifecycle state of a durable MCP task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Working,
    InputRequired,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// Complete task state returned by `tasks/get` and `notifications/tasks`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    #[serde(rename = "taskId")]
    pub task_id: String,
    pub status: TaskStatus,
    #[serde(rename = "statusMessage", skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "lastUpdatedAt")]
    pub last_updated_at: String,
    #[serde(rename = "ttlMs")]
    pub ttl_ms: Option<u64>,
    #[serde(rename = "pollIntervalMs", skip_serializing_if = "Option::is_none")]
    pub poll_interval_ms: Option<u64>,
    #[serde(
        rename = "inputRequests",
        default,
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub input_requests: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

impl Task {
    /// Verify status-specific fields before a task is put on the wire.
    pub fn validate(&self) -> Result<(), String> {
        match self.status {
            TaskStatus::InputRequired if self.input_requests.is_empty() => {
                Err("input_required task must include inputRequests".to_string())
            }
            TaskStatus::Completed if self.result.is_none() => {
                Err("completed task must include result".to_string())
            }
            TaskStatus::Failed if self.error.is_none() => {
                Err("failed task must include error".to_string())
            }
            TaskStatus::Working | TaskStatus::Cancelled
                if !self.input_requests.is_empty()
                    || self.result.is_some()
                    || self.error.is_some() =>
            {
                Err("task contains fields that do not match its status".to_string())
            }
            _ => Ok(()),
        }
    }
}

/// Task-shaped result returned in place of an immediate tool result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateTaskResult {
    #[serde(rename = "resultType")]
    pub result_type: String,
    #[serde(flatten)]
    pub task: Task,
    #[serde(rename = "_meta", default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, Value>,
}

/// Result of `tasks/get`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTaskResult {
    #[serde(rename = "resultType")]
    pub result_type: String,
    #[serde(flatten)]
    pub task: Task,
    #[serde(rename = "_meta", default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTaskParams {
    #[serde(rename = "taskId")]
    pub task_id: String,
    #[serde(rename = "_meta", default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateTaskParams {
    #[serde(rename = "taskId")]
    pub task_id: String,
    #[serde(rename = "inputResponses")]
    pub input_responses: HashMap<String, Value>,
    #[serde(rename = "_meta", default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelTaskParams {
    #[serde(rename = "taskId")]
    pub task_id: String,
    #[serde(rename = "_meta", default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, Value>,
}

/// Empty acknowledgement returned by task updates and cancellation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAcknowledgement {
    #[serde(rename = "resultType")]
    pub result_type: String,
    #[serde(rename = "_meta", default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, Value>,
}

pub fn has_tasks_extension(capabilities: &super::ClientCapabilities) -> bool {
    capabilities
        .extensions
        .as_ref()
        .is_some_and(|extensions| extensions.contains_key(TASKS_EXTENSION_ID))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_wire_fields_are_flat() {
        let task = Task {
            task_id: "t".to_string(),
            status: TaskStatus::Completed,
            status_message: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            last_updated_at: "2026-01-01T00:00:01Z".to_string(),
            ttl_ms: Some(60_000),
            poll_interval_ms: Some(100),
            input_requests: HashMap::new(),
            result: Some(serde_json::json!({"content":[]})),
            error: None,
        };
        task.validate().unwrap();
        let value = serde_json::to_value(GetTaskResult {
            result_type: "complete".to_string(),
            task,
            meta: HashMap::new(),
        })
        .unwrap();
        assert_eq!(value["taskId"], "t");
        assert_eq!(value["resultType"], "complete");
        assert!(value.get("task").is_none());
    }
}
