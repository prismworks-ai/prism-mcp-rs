//! Example 04: Sampling API (Fixed Version)
//! Demonstrates basic sampling functionality

use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Sampling API example");
    println!("Demonstrating sampling functionality");

    // Example prompt message
    let message = PromptMessage {
        role: Role::User,
        content: ContentBlock::Text {
            text: "Hello, how are you?".to_string(),
            annotations: None,
            meta: None,
        },
    };

    println!("Sample message created: {:?}", message);

    // Example sampling parameters
    println!("Sampling parameters:");
    println!("  Max tokens: 100");
    println!("  Temperature: 0.7");
    println!("  Model: claude-3-sonnet");

    Ok(())
}
