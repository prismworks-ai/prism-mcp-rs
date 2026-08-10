//! Official MCP 2026 conformance adapter.
//!
//! This binary intentionally exposes the diagnostic fixtures expected by the
//! upstream `server-stateless` and Tasks extension scenarios. It is test
//! infrastructure, not an application template.

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::protocol::{InputRequiredResult, OperationResult, ProtocolMode};
use prism_mcp_rs::server::TaskContext;
use prism_mcp_rs::transport::HttpServerTransport;
use tokio::sync::mpsc;

struct MissingCapability;

#[async_trait]
impl MultiRoundToolHandler for MissingCapability {
    async fn call(&self, _call: MultiRoundToolCall) -> McpResult<OperationResult<ToolResult>> {
        let requests = HashMap::from([(
            "sample".to_string(),
            json!({
                "method": "sampling/createMessage",
                "params": {
                    "messages": [{"role": "user", "content": {"type": "text", "text": "test"}}],
                    "maxTokens": 1
                }
            }),
        )]);
        Ok(OperationResult::InputRequired(InputRequiredResult::new(
            requests, None,
        )?))
    }
}

struct StreamingElicitation;

#[async_trait]
impl MultiRoundToolHandler for StreamingElicitation {
    async fn call(&self, call: MultiRoundToolCall) -> McpResult<OperationResult<ToolResult>> {
        if call.input_responses.is_empty() {
            let requests = HashMap::from([(
                "confirm".to_string(),
                json!({
                    "method": "elicitation/create",
                    "params": {
                        "message": "Continue?",
                        "requestedSchema": {"type": "object"}
                    }
                }),
            )]);
            return Ok(OperationResult::InputRequired(InputRequiredResult::new(
                requests, None,
            )?));
        }
        Ok(OperationResult::Complete(ToolResult::text("complete")))
    }
}

struct ComposedTaskPreflight;

#[async_trait]
impl MultiRoundToolHandler for ComposedTaskPreflight {
    async fn call(&self, call: MultiRoundToolCall) -> McpResult<OperationResult<ToolResult>> {
        if call.input_responses.is_empty() {
            return Ok(OperationResult::InputRequired(InputRequiredResult::new(
                HashMap::from([(
                    "user_name".to_string(),
                    json!({
                        "method": "elicitation/create",
                        "params": {
                            "message": "What is your name?",
                            "requestedSchema": {"type": "object"}
                        }
                    }),
                )]),
                Some("tasks-composition".to_string()),
            )?));
        }
        Ok(OperationResult::Complete(ToolResult::text("ready")))
    }
}

fn task_tool(name: &str, description: &str) -> ToolInfo {
    ToolInfo {
        name: name.to_string(),
        description: Some(description.to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: HashMap::new(),
        },
        output_schema: None,
        annotations: None,
        title: None,
        icons: None,
        meta: None,
    }
}

#[derive(Clone, Copy)]
enum CatalogChange {
    Tools,
    Prompts,
}

#[tokio::main]
async fn main() -> McpResult<()> {
    let bind_address =
        std::env::var("PRISM_CONFORMANCE_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let mut server = McpServer::create("prism-mcp-rs-conformance", env!("CARGO_PKG_VERSION"));
    server.set_protocol_mode(ProtocolMode::ModernOnly);

    server
        .add_multi_round_tool(
            "test_missing_capability",
            Some("Requires sampling capability"),
            json!({"type": "object"}),
            MissingCapability,
        )
        .await?;
    server
        .add_multi_round_tool(
            "test_streaming_elicitation",
            Some("Returns a valid input_required result"),
            json!({"type": "object"}),
            StreamingElicitation,
        )
        .await?;
    server
        .add_tool_with_closure(
            "test_logging_tool",
            Some("Completes without unsolicited log notifications"),
            json!({"type": "object"}),
            |_| Ok(vec![ContentBlock::text("complete")]),
        )
        .await?;

    server
        .add_tool_with_closure(
            "greet",
            Some("Synchronous Tasks conformance fixture"),
            json!({"type": "object"}),
            |arguments| {
                Ok(vec![ContentBlock::text(format!(
                    "Hello, {}!",
                    arguments
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("world")
                ))])
            },
        )
        .await?;
    server
        .add_task_tool_with_fallback(
            task_tool("slow_compute", "Cancellable durable computation"),
            |arguments: HashMap<String, Value>, context: TaskContext| async move {
                let seconds = arguments
                    .get("seconds")
                    .and_then(Value::as_u64)
                    .unwrap_or_default();
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_secs(seconds)) => {
                        Ok(ToolResult::text("computation complete"))
                    }
                    _ = context.cancelled() => {
                        Err(McpError::Cancelled("task cancellation requested".to_string()))
                    }
                }
            },
            prism_mcp_rs::core::tool::ClosureWrapper(|_arguments: &HashMap<String, Value>| {
                Ok(vec![ContentBlock::text("synchronous computation")])
            }),
        )
        .await?;
    server
        .add_task_tool(
            task_tool("failing_job", "Task that returns a tool-level error"),
            |_arguments: HashMap<String, Value>, _context: TaskContext| async move {
                Ok(ToolResult {
                    content: vec![ContentBlock::text("job failed")],
                    is_error: Some(true),
                    meta: None,
                    structured_content: None,
                })
            },
        )
        .await?;
    server
        .add_task_tool(
            task_tool("protocol_error_job", "Task that returns a protocol error"),
            |_arguments: HashMap<String, Value>, _context: TaskContext| async move {
                Err(McpError::Internal("simulated protocol failure".to_string()))
            },
        )
        .await?;
    server
        .add_task_tool(
            task_tool("confirm_delete", "Task that requires one elicitation"),
            |_arguments: HashMap<String, Value>, context: TaskContext| async move {
                let responses = context
                    .require_input(
                        HashMap::from([(
                            "confirm".to_string(),
                            json!({
                                "method": "elicitation/create",
                                "params": {
                                    "message": "Confirm deletion",
                                    "requestedSchema": {"type": "object"}
                                }
                            }),
                        )]),
                        Some("Waiting for confirmation".to_string()),
                    )
                    .await?;
                Ok(ToolResult::text(format!(
                    "confirmation received: {}",
                    responses.contains_key("confirm")
                )))
            },
        )
        .await?;
    server
        .add_task_tool(
            task_tool("multi_input", "Task that requires two inputs"),
            |_arguments: HashMap<String, Value>, context: TaskContext| async move {
                context
                    .require_input(
                        HashMap::from([
                            (
                                "first".to_string(),
                                json!({
                                    "method": "elicitation/create",
                                    "params": {
                                        "message": "First input",
                                        "requestedSchema": {"type": "object"}
                                    }
                                }),
                            ),
                            (
                                "second".to_string(),
                                json!({
                                    "method": "elicitation/create",
                                    "params": {
                                        "message": "Second input",
                                        "requestedSchema": {"type": "object"}
                                    }
                                }),
                            ),
                        ]),
                        Some("Waiting for both inputs".to_string()),
                    )
                    .await?;
                Ok(ToolResult::text("both inputs received"))
            },
        )
        .await?;
    server
        .add_composed_task_tool(
            task_tool(
                "test_tool_with_task",
                "MRTR input followed by durable execution",
            ),
            ComposedTaskPreflight,
            |call: MultiRoundToolCall, _context: TaskContext| async move {
                let name = call.input_responses["user_name"]["content"]["name"]
                    .as_str()
                    .unwrap_or("unknown");
                Ok(ToolResult::text(format!("Hello, {name}")))
            },
        )
        .await?;

    let (change_tx, mut change_rx) = mpsc::unbounded_channel();
    let tool_tx = change_tx.clone();
    server
        .add_tool_with_closure(
            "test_trigger_tool_change",
            Some("Trigger a tools/list_changed notification"),
            json!({"type": "object"}),
            move |_| {
                let _ = tool_tx.send(CatalogChange::Tools);
                Ok(vec![ContentBlock::text("triggered")])
            },
        )
        .await?;
    server
        .add_tool_with_closure(
            "test_trigger_prompt_change",
            Some("Trigger a prompts/list_changed notification"),
            json!({"type": "object"}),
            move |_| {
                let _ = change_tx.send(CatalogChange::Prompts);
                Ok(vec![ContentBlock::text("triggered")])
            },
        )
        .await?;

    server.start(HttpServerTransport::new(bind_address)).await?;
    let server = Arc::new(server);
    let notifier = server.clone();
    tokio::spawn(async move {
        while let Some(change) = change_rx.recv().await {
            let result = match change {
                CatalogChange::Tools => notifier.notify_tools_list_changed().await,
                CatalogChange::Prompts => notifier.notify_prompts_list_changed().await,
            };
            if let Err(error) = result {
                tracing::warn!(%error, "conformance notification failed");
            }
        }
    });

    tokio::signal::ctrl_c().await.map_err(McpError::io)?;
    server.stop().await
}
