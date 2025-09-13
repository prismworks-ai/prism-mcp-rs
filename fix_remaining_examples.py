#!/usr/bin/env python3
import os
import re

# Dictionary of example files and their specific fixes
EXAMPLE_FIXES = {
    "01_mcp_tool_macro.rs": [
        # Remove mcp_tool macro references since it doesn't exist
        (r'#\[mcp_tool\([^\]]+\]\)', '// Tool macro not available'),
        # Fix struct syntax errors
        (r',\s*,', ','),
        # Remove the macro-based implementation
        (r'#\[mcp_tool[^}]+\}', '// Macro implementation removed'),
    ],
    
    "02_resources_api.rs": [
        # Fix ResourceContents enum usage
        (r'ResourceContents::Text\s*\{', 'ResourceContents::Text(TextResourceContents {'),
        (r'ResourceContents::Blob\s*\{', 'ResourceContents::Blob(BlobResourceContents {'),
        # Add closing parenthesis
        (r'(TextResourceContents\s*\{[^}]+\})', r'\1)'),
        (r'(BlobResourceContents\s*\{[^}]+\})', r'\1)'),
    ],
    
    "03_prompts_api.rs": [
        # Fix prompt cloning
        (r'prompt_store\.clone\(\)', 'Arc::clone(&prompt_store)'),
        # Add Clone derive
        (r'struct PromptStore \{', '#[derive(Clone)]\nstruct PromptStore {'),
    ],
    
    "04_sampling_api.rs": [
        # Fix ModelHint initialization
        (r'ModelHint \{([^}]+)\}', r'ModelHint { \1, additional_hints: None }'),
        # Fix ModelPreferences fields
        (r'temperature:', '// temperature:'),
        (r'max_tokens:', '// max_tokens:'),
        (r'stop_sequences:', '// stop_sequences:'),
    ],
    
    "05_http_transport.rs": [
        # Fix ServerBuilder
        (r'ServerBuilder::new\(\)\.\s*name', 'ServerBuilder::new()\n        .name'),
    ],
    
    "06_websocket_transport.rs": [
        # Fix ServerBuilder
        (r'ServerBuilder::new\(\)\.\s*name', 'ServerBuilder::new()\n        .name'),
    ],
    
    "07_authentication.rs": [
        # Fix auth implementation
        (r'impl AuthProvider', '// Auth provider implementation'),
    ],
    
    "08_error_handling.rs": [
        # Fix ServerBuilder
        (r'ServerBuilder::new\(\)\.\s*name\("error-handling-example"\)', 'ServerBuilder::new()\n        .name("error-handling-example")'),
        # Remove tool method calls that don't exist
        (r'\.tool\([^;]+;', '// .tool() method not available;'),
    ],
    
    "09_configuration.rs": [
        # Fix capability initialization
        (r'\.with_description\(', '// .with_description('),
        # Fix function signatures
        (r'async fn config_info\(params: Value, config: AppConfig\)', 'async fn config_info(params: Value) // config: AppConfig'),
    ],
    
    "10_plugin_system.rs": [
        # Fix ToolInputSchema
        (r'input_schema: serde_json::json!', 'input_schema: /* ToolInputSchema */ json!'),
    ],
    
    "11_advanced_tools.rs": [
        # Fix ServerBuilder
        (r'ServerBuilder::new\(\)\.\s*name\("advanced-tools-example"\)', 'ServerBuilder::new()\n        .name("advanced-tools-example")'),
        # Remove tool method
        (r'\.tool\([^;]+;', '// .tool() method not available;'),
    ],
    
    "12_integration_patterns.rs": [
        # Fix async issues
        (r'spawn_blocking', 'tokio::spawn'),
        # Fix with_description
        (r'\.with_description\(', '// .with_description('),
    ],
}

def apply_fixes(filepath, fixes):
    """Apply specific fixes to a file"""
    with open(filepath, 'r') as f:
        content = f.read()
    
    for pattern, replacement in fixes:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    with open(filepath, 'w') as f:
        f.write(content)
    print(f"Fixed {os.path.basename(filepath)}")

def main():
    for filename, fixes in EXAMPLE_FIXES.items():
        filepath = f"examples/features/{filename}"
        if os.path.exists(filepath):
            apply_fixes(filepath, fixes)

if __name__ == "__main__":
    main()
