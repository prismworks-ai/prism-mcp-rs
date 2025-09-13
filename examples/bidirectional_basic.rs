//! Example demonstrating bidirectional communication concepts
//!
//! This example shows the data structures and concepts for bidirectional
//! communication between MCP servers and clients.

use prism_mcp_rs::prelude::*;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> McpResult<()> {
    println!("🚀 Bidirectional Communication Concepts Example");
    println!("==============================================\n");

    println!("📋 This example demonstrates the data structures used for:");
    println!("   - Sampling/CreateMessage (LLM generation requests)");
    println!("   - Elicitation (user input requests)");
    println!("   - Roots (file system access)\n");

    // Example 1: Sampling/CreateMessage Request
    println!("1️⃣  Sampling/CreateMessage Request Structure:");
    println!("   Used when a server needs to request LLM generation from the client.\n");

    let sampling_message = SamplingMessage {
        role: Role::User,
        content: SamplingContent::Text {
            text: "Hello, how are you?".to_string(),
            annotations: None,
            meta: None,
        },
    };
    println!(
        "   Example message: {}",
        serde_json::to_string_pretty(&sampling_message)?
    );
    println!();

    // Example 2: Elicitation Request
    println!("2️⃣  Elicitation Request Structure:");
    println!("   Used when a server needs to request user input from the client.\n");

    let mut properties = HashMap::new();
    properties.insert(
        "name".to_string(),
        PrimitiveSchemaDefinition::String {
            title: Some("Your Name".to_string()),
            description: Some("Please enter your full name".to_string()),
            min_length: None,
            max_length: None,
            format: None,
            enum_values: None,
            enum_names: None,
        },
    );

    let elicitation_schema = ElicitationSchema {
        schema_type: "object".to_string(),
        properties,
        required: Some(vec!["name".to_string()]),
    };

    let elicit_params = ElicitParams {
        message: "We need some information to continue:".to_string(),
        requested_schema: elicitation_schema,
        meta: None,
    };
    println!(
        "   Example params: {}",
        serde_json::to_string_pretty(&elicit_params)?
    );
    println!();

    // Example 3: Roots List
    println!("3️⃣  Roots List Response Structure:");
    println!("   Used when a server queries available file system roots from the client.\n");

    let roots = vec![
        Root {
            uri: "file:///home/user/projects".to_string(),
            name: Some("Projects".to_string()),
        },
        Root {
            uri: "file:///home/user/documents".to_string(),
            name: Some("Documents".to_string()),
        },
    ];

    let roots_result = ListRootsResult { roots, meta: None };
    println!(
        "   Example response: {}",
        serde_json::to_string_pretty(&roots_result)?
    );
    println!();

    // Example 4: Expected responses
    println!("4️⃣  Expected Response Types:\n");

    println!("   Sampling Response (CreateMessageResult):");
    let create_message_result = CreateMessageResult {
        role: Role::Assistant,
        content: SamplingContent::Text {
            text: "I'm doing well, thank you for asking!".to_string(),
            annotations: None,
            meta: None,
        },
        model: "claude-3".to_string(),
        stop_reason: Some(StopReason::EndTurn),
        meta: None,
    };
    println!(
        "   {}",
        serde_json::to_string_pretty(&create_message_result)?
    );
    println!();

    println!("   Elicitation Response (ElicitResult):");
    let mut content_map = HashMap::new();
    content_map.insert("name".to_string(), json!("John Doe"));

    let elicit_result = ElicitResult {
        action: ElicitationAction::Accept,
        content: Some(content_map),
        meta: None,
    };
    println!("   {}", serde_json::to_string_pretty(&elicit_result)?);
    println!();

    println!("📋 Summary:");
    println!("   These structures enable bidirectional communication where:");
    println!("   - Servers can request LLM completions from clients");
    println!("   - Servers can request user input via forms");
    println!("   - Servers can discover and access client file systems");
    println!("   - All communication follows the MCP protocol standards\n");

    println!("✅ Example completed successfully!");

    Ok(())
}
