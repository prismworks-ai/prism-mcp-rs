// ! Bidirectional Communication Demo
// !
// ! This example demonstrates the bidirectional communication capabilities
// ! of the MCP 2025-06-18 protocol, where servers can initiate requests
// ! to clients for sampling, elicitation, and root access.
// !
// ! Key features demonstrated:
// ! - Server-to-client sampling requests (LLM integration)
// ! - User input collection through elicitation
// ! - Root directory access from client
// ! - Interactive client request handling
// !
// ! Run with: cargo run --example bidirectional_communication_demo --features stdio,dirs

use prism_mcp_rs::prelude::*;
use std::collections::HashMap;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting bidirectional communication demo...");
    info!("==============================================\n");

    // Run different demo scenarios
    demo_elicitation_workflow().await?;
    demo_roots_access().await?;
    demo_completion_api().await?;

    info!("\n=======================================\n");
    info!("Bidirectional communication demo completed!");
    info!("In a real implementation, you would:");
    info!("1. Integrate with actual LLM services (OpenAI, Anthropic, etc.)");
    info!("2. Connect server and client over network transports");
    info!("3. Implement custom elicitation forms for your use case");
    info!("4. Add real file system operations for root access");

    Ok(())
}

/// Demonstrate elicitation workflow for user input collection
async fn demo_elicitation_workflow() -> McpResult<()> {
    info!("## Demo 1: Elicitation Workflow (User Input Collection)");
    info!("======================================================");

    // Create a client with interactive request handler
    let mut client = McpClient::new("demo-client".to_string(), "1.0.0".to_string());

    // Set up interactive handler that auto-accepts for demo
    let handler = InteractiveClientRequestHandler::new("Bidirectional Demo Client")
        .auto_accept_elicitation(true) // Auto-accept for demo
        .verbose(true);

    client.set_request_handler(handler);

    // Simulate server requesting user input
    info!("Note: Server wants to collect user preferences...");

    // Create elicitation schema
    let mut properties = HashMap::new();
    properties.insert(
        "notification_level".to_string(),
        PrimitiveSchemaDefinition::String {
            title: Some("Notification Level".to_string()),
            description: Some("How verbose should notifications be?".to_string()),
            min_length: None,
            max_length: None,
            format: None,
            enum_values: Some(vec![
                "quiet".to_string(),
                "normal".to_string(),
                "verbose".to_string(),
            ]),
            enum_names: Some(vec![
                "Quiet".to_string(),
                "Normal".to_string(),
                "Verbose".to_string(),
            ]),
        },
    );
    properties.insert(
        "auto_save".to_string(),
        PrimitiveSchemaDefinition::Boolean {
            title: Some("Auto Save".to_string()),
            description: Some("Should changes be saved automatically?".to_string()),
            default: Some(true),
        },
    );

    let elicit_params = ElicitParams {
        message: "Please configure your application preferences. This will help customize your experience.".to_string(),
        requested_schema: ElicitationSchema {
            schema_type: "object".to_string(),
            properties,
            required: Some(vec!["notification_level".to_string()]),
        },
        meta: None,
    };

    // Simulate server sending elicitation request to client
    match client
        .handle_server_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Number(serde_json::Number::from(1)),
            method: "elicitation/create".to_string(),
            params: Some(serde_json::to_value(elicit_params)?),
        })
        .await
    {
        Ok(response) => {
            info!("[x] Elicitation completed successfully!");
            if let Some(result) = response.result {
                let elicit_result: ElicitResult = serde_json::from_value(result)?;
                match elicit_result.action {
                    ElicitationAction::Accept => {
                        info!(
                            "👤 User accepted and provided data: {:?}",
                            elicit_result.content
                        );
                    }
                    ElicitationAction::Decline => {
                        warn!("👤 User declined to provide information");
                    }
                    ElicitationAction::Cancel => {
                        warn!("👤 User cancelled the request");
                    }
                }
            }
        }
        Err(e) => {
            warn!("[!] Elicitation failed: {}", e);
        }
    }

    info!("");
    Ok(())
}

/// Demonstrate roots access for file system integration
async fn demo_roots_access() -> McpResult<()> {
    info!(" Demo 2: Roots Access (File System Integration)");
    info!("===================================================");

    // Create a client with roots configured
    let mut client = McpClient::new("demo-client".to_string(), "1.0.0".to_string());

    let handler = InteractiveClientRequestHandler::new("File System Demo Client")
        .add_root("file:///home/user/documents", Some("Documents"))
        .add_root("file:///home/user/projects", Some("Projects"))
        .add_common_roots() // Add platform-specific roots
        .verbose(true);

    client.set_request_handler(handler);

    info!("🗂️  Server requesting available file system roots...");

    // Simulate server requesting roots from client
    match client
        .handle_server_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Number(serde_json::Number::from(2)),
            method: "roots/list".to_string(),
            params: Some(serde_json::to_value(ListRootsParams { meta: None })?),
        })
        .await
    {
        Ok(response) => {
            info!("[x] Roots access completed successfully!");
            if let Some(result) = response.result {
                let roots_result: ListRootsResult = serde_json::from_value(result)?;
                info!(" Available roots:");
                for root in roots_result.roots {
                    info!(
                        "   • {} ({})",
                        root.uri,
                        root.name.unwrap_or("Unnamed".to_string())
                    );
                }
            }
        }
        Err(e) => {
            warn!("[!] Roots access failed: {}", e);
        }
    }

    info!("");
    Ok(())
}

/// Demonstrate completion API for smart autocompletion
async fn demo_completion_api() -> McpResult<()> {
    info!("🧠 Demo 3: Completion API (smart Autocompletion)");
    info!("======================================================");

    // Create a server with completion handlers
    let server = McpServer::new("demo-server".to_string(), "1.0.0".to_string());

    // Add prompt completion handler
    let prompt_handler = PromptCompletionHandler::new(vec![
        "analyze_data".to_string(),
        "analyze_text".to_string(),
        "analyze_code".to_string(),
        "create_report".to_string(),
        "generate_summary".to_string(),
    ]);

    server
        .add_completion_handler("ref/prompt".to_string(), prompt_handler)
        .await?;

    // Add resource template completion handler
    let templates = vec![
        ResourceTemplate::new(
            "file:///docs/{category}/{filename}".to_string(),
            "Documentation".to_string(),
        ),
        ResourceTemplate::new(
            "https://api.example.com/{version}/{resource}".to_string(),
            "API Endpoints".to_string(),
        ),
    ];
    let resource_handler = ResourceUriCompletionHandler::new(templates);

    server
        .add_completion_handler("ref/resource".to_string(), resource_handler)
        .await?;

    info!("🔤 Testing prompt name completion...");

    // Test prompt completion
    let reference = CompletionReference::Prompt {
        name: "test".to_string(),
    };
    let argument = CompletionArgument {
        name: "name".to_string(),
        value: "ana".to_string(),
    };

    match server.handle_completion(&reference, &argument, None).await {
        Ok(completions) => {
            info!("[x] Completion successful!");
            info!("Note: Input: 'ana'");
            info!("Note: Suggestions: {:?}", completions);
        }
        Err(e) => {
            warn!("[!] Completion failed: {}", e);
        }
    }

    info!("");
    info!("🔗 Testing resource URI completion...");

    // Test resource completion
    let resource_ref = CompletionReference::Resource {
        uri: "file:///docs/".to_string(),
    };
    let uri_arg = CompletionArgument {
        name: "uri".to_string(),
        value: "file:///docs/".to_string(),
    };

    match server
        .handle_completion(&resource_ref, &uri_arg, None)
        .await
    {
        Ok(completions) => {
            info!("[x] Resource completion successful!");
            info!("Note: Input: 'file:///docs/'");
            info!("Note: Suggestions: {:?}", completions);
        }
        Err(e) => {
            warn!("[!] Resource completion failed: {}", e);
        }
    }

    info!("");
    Ok(())
}