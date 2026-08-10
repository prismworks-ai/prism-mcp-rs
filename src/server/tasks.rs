//! Production runtime for the MCP Tasks extension.

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::{broadcast, Mutex, Notify, RwLock};
use tokio_util::sync::CancellationToken;

use crate::{
    core::error::{McpError, McpResult},
    core::MultiRoundToolCall,
    protocol::{error_codes, JsonRpcNotification, Task, TaskStatus, ToolResult, TASKS_STATUS},
};

/// Application handler for a tool that executes as a durable MCP task.
#[async_trait]
pub trait TaskToolHandler: Send + Sync {
    async fn call(
        &self,
        arguments: HashMap<String, Value>,
        context: TaskContext,
    ) -> McpResult<ToolResult>;
}

#[async_trait]
impl<F, Fut> TaskToolHandler for F
where
    F: Fn(HashMap<String, Value>, TaskContext) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = McpResult<ToolResult>> + Send,
{
    async fn call(
        &self,
        arguments: HashMap<String, Value>,
        context: TaskContext,
    ) -> McpResult<ToolResult> {
        self(arguments, context).await
    }
}

/// Application handler for the durable phase of a tool that first completes
/// an MCP multi-round input exchange and then escalates to a Task.
#[async_trait]
pub trait ComposedTaskToolHandler: Send + Sync {
    async fn call(&self, call: MultiRoundToolCall, context: TaskContext) -> McpResult<ToolResult>;
}

#[async_trait]
impl<F, Fut> ComposedTaskToolHandler for F
where
    F: Fn(MultiRoundToolCall, TaskContext) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = McpResult<ToolResult>> + Send,
{
    async fn call(&self, call: MultiRoundToolCall, context: TaskContext) -> McpResult<ToolResult> {
        self(call, context).await
    }
}

struct TaskRecord {
    owner: String,
    task: RwLock<Task>,
    responses: Mutex<HashMap<String, Value>>,
    response_ready: Notify,
    cancellation: CancellationToken,
    expires_at: Option<tokio::time::Instant>,
}

/// Context supplied to a task tool. It supports cooperative cancellation and
/// task-native multi-round input without leaking `requestState` into Tasks.
#[derive(Clone)]
pub struct TaskContext {
    record: Arc<TaskRecord>,
    status_sender: broadcast::Sender<JsonRpcNotification>,
}

impl TaskContext {
    pub async fn task_id(&self) -> String {
        self.record.task.read().await.task_id.clone()
    }

    pub fn is_cancelled(&self) -> bool {
        self.record.cancellation.is_cancelled()
    }

    pub async fn cancelled(&self) {
        self.record.cancellation.cancelled().await;
    }

    /// Update the human-readable working status.
    pub async fn report_progress(&self, message: impl Into<String>) -> McpResult<()> {
        {
            let mut task = self.record.task.write().await;
            if task.status.is_terminal() {
                return Err(McpError::Cancelled("task is already terminal".to_string()));
            }
            task.status = TaskStatus::Working;
            task.status_message = Some(message.into());
            task.input_requests.clear();
            task.last_updated_at = chrono::Utc::now().to_rfc3339();
        }
        self.publish().await
    }

    /// Publish input requests and wait until all of them are answered or the
    /// task is cancelled. Keys remain unique for the task lifetime.
    pub async fn require_input(
        &self,
        requests: HashMap<String, Value>,
        message: Option<String>,
    ) -> McpResult<HashMap<String, Value>> {
        if requests.is_empty() {
            return Err(McpError::InvalidParams(
                "task inputRequests cannot be empty".to_string(),
            ));
        }
        {
            let mut task = self.record.task.write().await;
            if task.status.is_terminal() {
                return Err(McpError::Cancelled("task is already terminal".to_string()));
            }
            task.status = TaskStatus::InputRequired;
            task.status_message = message;
            task.input_requests = requests.clone();
            task.last_updated_at = chrono::Utc::now().to_rfc3339();
        }
        self.publish().await?;

        loop {
            if self.is_cancelled() {
                return Err(McpError::Cancelled(
                    "task cancellation requested".to_string(),
                ));
            }
            let mut responses = self.record.responses.lock().await;
            if requests.keys().all(|key| responses.contains_key(key)) {
                let values = requests
                    .keys()
                    .filter_map(|key| responses.remove(key).map(|value| (key.clone(), value)))
                    .collect();
                drop(responses);
                {
                    let mut task = self.record.task.write().await;
                    task.status = TaskStatus::Working;
                    task.input_requests.clear();
                    task.last_updated_at = chrono::Utc::now().to_rfc3339();
                }
                self.publish().await?;
                return Ok(values);
            }
            drop(responses);
            tokio::select! {
                _ = self.record.response_ready.notified() => {},
                _ = self.record.cancellation.cancelled() => {
                    return Err(McpError::Cancelled("task cancellation requested".to_string()));
                }
            }
        }
    }

    async fn publish(&self) -> McpResult<()> {
        let task = self.record.task.read().await;
        publish_task(&self.status_sender, &task).await
    }
}

/// In-memory task store with caller binding, TTL enforcement, status
/// notifications, input delivery, and cooperative cancellation.
#[derive(Clone)]
pub(crate) struct TaskRegistry {
    records: Arc<RwLock<HashMap<String, Arc<TaskRecord>>>>,
    status_sender: broadcast::Sender<JsonRpcNotification>,
    default_ttl: Option<Duration>,
    poll_interval_ms: u64,
}

impl Default for TaskRegistry {
    fn default() -> Self {
        let (status_sender, _) = broadcast::channel(1024);
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            status_sender,
            default_ttl: Some(Duration::from_secs(60 * 60)),
            poll_interval_ms: 1_000,
        }
    }
}

impl TaskRegistry {
    pub fn subscribe(&self) -> broadcast::Receiver<JsonRpcNotification> {
        self.status_sender.subscribe()
    }

    pub async fn create(
        &self,
        owner: String,
        arguments: HashMap<String, Value>,
        handler: Arc<dyn TaskToolHandler>,
    ) -> McpResult<Task> {
        self.create_with(owner, move |context| async move {
            handler.call(arguments, context).await
        })
        .await
    }

    pub async fn create_composed(
        &self,
        owner: String,
        call: MultiRoundToolCall,
        handler: Arc<dyn ComposedTaskToolHandler>,
    ) -> McpResult<Task> {
        self.create_with(owner, move |context| async move {
            handler.call(call, context).await
        })
        .await
    }

    async fn create_with<F, Fut>(&self, owner: String, run: F) -> McpResult<Task>
    where
        F: FnOnce(TaskContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = McpResult<ToolResult>> + Send + 'static,
    {
        let now = chrono::Utc::now().to_rfc3339();
        let task = Task {
            task_id: uuid::Uuid::new_v4().to_string(),
            status: TaskStatus::Working,
            status_message: Some("Task accepted".to_string()),
            created_at: now.clone(),
            last_updated_at: now,
            ttl_ms: self.default_ttl.map(|value| value.as_millis() as u64),
            poll_interval_ms: Some(self.poll_interval_ms),
            input_requests: HashMap::new(),
            result: None,
            error: None,
        };
        let record = Arc::new(TaskRecord {
            owner,
            task: RwLock::new(task.clone()),
            responses: Mutex::new(HashMap::new()),
            response_ready: Notify::new(),
            cancellation: CancellationToken::new(),
            expires_at: self
                .default_ttl
                .map(|ttl| tokio::time::Instant::now() + ttl),
        });
        // Strong creation consistency: insert before returning the handle.
        self.records
            .write()
            .await
            .insert(task.task_id.clone(), record.clone());

        let sender = self.status_sender.clone();
        tokio::spawn(async move {
            let context = TaskContext {
                record: record.clone(),
                status_sender: sender.clone(),
            };
            let outcome = run(context).await;
            let mut state = record.task.write().await;
            if record.cancellation.is_cancelled() && !state.status.is_terminal() {
                state.status = TaskStatus::Cancelled;
                state.status_message = Some("Task cancellation accepted".to_string());
                state.input_requests.clear();
            } else {
                match outcome {
                    Ok(result) => {
                        state.status = TaskStatus::Completed;
                        state.status_message = Some("Task completed".to_string());
                        state.input_requests.clear();
                        state.result = serde_json::to_value(result).ok();
                    }
                    Err(error) => {
                        state.status = TaskStatus::Failed;
                        state.status_message = Some(error.to_string());
                        state.input_requests.clear();
                        state.error = Some(serde_json::json!({
                            "code": error_codes::INTERNAL_ERROR,
                            "message": error.to_string()
                        }));
                    }
                }
            }
            state.last_updated_at = chrono::Utc::now().to_rfc3339();
            let _ = publish_task(&sender, &state).await;
        });
        Ok(task)
    }

    async fn record(&self, task_id: &str, owner: &str) -> McpResult<Arc<TaskRecord>> {
        let record = self
            .records
            .read()
            .await
            .get(task_id)
            .cloned()
            .ok_or_else(|| McpError::InvalidParams("Task not found".to_string()))?;
        if record.owner != owner {
            // Do not disclose whether a handle belongs to another caller.
            return Err(McpError::InvalidParams("Task not found".to_string()));
        }
        if record
            .expires_at
            .is_some_and(|deadline| tokio::time::Instant::now() >= deadline)
        {
            self.records.write().await.remove(task_id);
            return Err(McpError::InvalidParams("Task has expired".to_string()));
        }
        Ok(record)
    }

    pub async fn get(&self, task_id: &str, owner: &str) -> McpResult<Task> {
        let record = self.record(task_id, owner).await?;
        let task = record.task.read().await.clone();
        task.validate().map_err(McpError::Internal)?;
        Ok(task)
    }

    pub async fn update(
        &self,
        task_id: &str,
        owner: &str,
        responses: HashMap<String, Value>,
    ) -> McpResult<()> {
        let record = self.record(task_id, owner).await?;
        let mut task = record.task.write().await;
        let mut accepted = record.responses.lock().await;
        for (key, value) in responses {
            if task.input_requests.contains_key(&key) && !accepted.contains_key(&key) {
                accepted.insert(key.clone(), value);
                task.input_requests.remove(&key);
            }
        }
        drop(accepted);
        if task.status == TaskStatus::InputRequired && task.input_requests.is_empty() {
            task.status = TaskStatus::Working;
        }
        task.last_updated_at = chrono::Utc::now().to_rfc3339();
        let snapshot = task.clone();
        drop(task);
        publish_task(&self.status_sender, &snapshot).await?;
        record.response_ready.notify_waiters();
        Ok(())
    }

    pub async fn cancel(&self, task_id: &str, owner: &str) -> McpResult<()> {
        let record = self.record(task_id, owner).await?;
        record.cancellation.cancel();
        Ok(())
    }
}

async fn publish_task(
    sender: &broadcast::Sender<JsonRpcNotification>,
    task: &Task,
) -> McpResult<()> {
    task.validate().map_err(McpError::Internal)?;
    let notification =
        JsonRpcNotification::new(TASKS_STATUS.to_string(), Some(serde_json::to_value(task)?))?;
    let _ = sender.send(notification);
    Ok(())
}
