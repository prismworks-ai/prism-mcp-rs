//! Official MCP 2026 client-conformance adapter.
//!
//! The upstream runner supplies the scenario name through
//! `MCP_CONFORMANCE_SCENARIO` and appends its mock-server URL.

use std::collections::HashMap;

use prism_mcp_rs::prelude::*;
use prism_mcp_rs::protocol::{
    ElicitationCapability, ProtocolMode, RootsCapability, SamplingCapability,
};

#[tokio::main]
async fn main() -> McpResult<()> {
    let server_url = std::env::args()
        .nth(1)
        .ok_or_else(|| McpError::InvalidParams("missing conformance server URL".to_string()))?;
    let scenario = std::env::var("MCP_CONFORMANCE_SCENARIO").unwrap_or_default();
    let protocol = std::env::var("MCP_CONFORMANCE_PROTOCOL_VERSION").unwrap_or_default();

    let capabilities = ClientCapabilities {
        sampling: Some(SamplingCapability::default()),
        roots: Some(RootsCapability::default()),
        elicitation: Some(ElicitationCapability::default()),
        ..Default::default()
    };
    let mut client = McpClient::new(
        "prism-mcp-rs-conformance".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    );
    client.set_capabilities(capabilities);
    if protocol == "2026-07-28" || protocol.is_empty() {
        client.set_protocol_mode(ProtocolMode::ModernOnly);
    } else {
        client.set_protocol_mode(ProtocolMode::LegacyOnly);
    }
    client.connect_with_http(&server_url, None).await?;

    match scenario.as_str() {
        "tools_call" | "tools-call" => call_first_tool(&client).await?,
        "request-metadata" => {
            client.list_tools(None).await?;
        }
        "http-standard-headers" => exercise_standard_headers(&client).await?,
        "json-schema-ref-no-deref" => {
            client.list_tools(None).await?;
        }
        "json-schema-2020-12-preservation" => preserve_schema(&client).await?,
        other => {
            return Err(McpError::InvalidParams(format!(
                "unsupported client conformance scenario: {other}"
            )))
        }
    }

    client.disconnect().await
}

async fn call_first_tool(client: &McpClient) -> McpResult<()> {
    let tools = client.list_tools(None).await?.tools;
    if let Some(tool) = tools.first() {
        client
            .call_tool(
                tool.name.clone(),
                Some(HashMap::from([
                    ("a".to_string(), json!(2)),
                    ("b".to_string(), json!(3)),
                ])),
            )
            .await?;
    }
    Ok(())
}

async fn exercise_standard_headers(client: &McpClient) -> McpResult<()> {
    call_first_tool(client).await?;
    let resources = client.list_resources(None).await?.resources;
    if let Some(resource) = resources.first() {
        client.read_resource(resource.uri.clone()).await?;
    }
    let prompts = client.list_prompts(None).await?.prompts;
    if let Some(prompt) = prompts.first() {
        client.get_prompt(prompt.name.clone(), None).await?;
    }
    Ok(())
}

async fn preserve_schema(client: &McpClient) -> McpResult<()> {
    let tools = client.list_tools(None).await?.tools;
    let schema = tools
        .iter()
        .find(|tool| tool.name == "json_schema_2020_12_tool")
        .map(|tool| serde_json::to_value(&tool.input_schema))
        .transpose()?
        .ok_or_else(|| McpError::ToolNotFound("json_schema_2020_12_tool".to_string()))?;
    client
        .call_tool(
            "json_schema_echo".to_string(),
            Some(HashMap::from([("schema".to_string(), schema)])),
        )
        .await?;
    Ok(())
}
