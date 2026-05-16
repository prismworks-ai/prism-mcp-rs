#!/usr/bin/env python3
"""Fix all examples to match the actual library API"""

import re
import glob

# Define the fixes needed
FIXES = [
    # Fix imports - use prelude
    (r'use prism_mcp_rs::\{[^}]+\};', 'use prism_mcp_rs::prelude::*;'),
    
    # Remove standalone transport imports
    (r'use prism_mcp_rs::transport::stdio::StdioTransport;', ''),
    (r'transport::stdio::StdioTransport', ''),
    
    # Fix ServerBuilder API
    (r'ServerBuilder::new\("([^"]+)"\)', r'ServerBuilder::new()\n        .name("\1")'),
    
    # Fix CallToolResult struct initialization
    (r'CallToolResult \{\s*content: ([^,]+),\s*is_error: ([^}]+)\s*\}',
     r'CallToolResult { content: \1, is_error: \2, meta: None, structured_content: None }'),
    
    # Fix ToolResult usage (different from CallToolResult)
    (r'ToolResult \{\s*content: ([^,]+),\s*is_error: ([^}]+)\s*\}',
     r'ToolResult { content: \1, is_error: \2, meta: None, structured_content: None }'),
     
    # Fix CreateMessageResult
    (r'CreateMessageResult \{\s*model:([^,]+),\s*role:([^,]+),\s*content:([^}]+)\s*\}',
     r'CreateMessageResult { model:\1, role:\2, content:\3, meta: None }'),
    
    # Fix Resource struct
    (r'Resource \{\s*uri:([^,]+),\s*name:([^,]+),\s*description:([^,]+),\s*mime_type:([^}]+)\s*\}',
     r'Resource { uri:\1, name:\2, description:\3, mime_type:\4, annotations: None, meta: None, size: None, title: None }'),
    
    # Fix Prompt struct
    (r'Prompt \{\s*name:([^,]+),\s*description:([^,]+),\s*arguments:([^}]+)\s*\}',
     r'Prompt { name:\1, description:\2, arguments:\3, meta: None, title: None }'),
    
    # Fix PromptArgument
    (r'PromptArgument \{\s*name:([^,]+),\s*description:([^,]+),\s*required:([^}]+)\s*\}',
     r'PromptArgument { name:\1, description:\2, required:\3, title: None }'),
    
    # Fix ResourceTemplate  
    (r'ResourceTemplate \{\s*uri_template:([^,]+),\s*name:([^,]+),\s*description:([^,]+),\s*mime_type:([^}]+)\s*\}',
     r'ResourceTemplate { uri_template:\1, name:\2, description:\3, mime_type:\4, annotations: None, meta: None, title: None }'),
    
    # Fix imports that don't exist
    (r'ErrorCode', 'JsonRpcError'),
    (r'ListResourcesRequest', 'ListResourcesParams'),
    (r'ReadResourceRequest', 'ReadResourceParams'),
    (r'ListPromptsRequest', 'ListPromptsParams'),
    (r'GetPromptRequest', 'GetPromptParams'),
    (r'CreateMessageRequest', 'CreateMessageParams'),
    
    # Fix method names
    (r'\.description\(', '.with_description('),
    (r'server\.run_with_stdio\(\)', '// server.run_with_stdio() - method not available'),
    
    # Fix capability types
    (r'capabilities\.tools = Some\(true\)', 'capabilities.tools = Some(ToolsCapability { list_changed: Some(true) })'),
    (r'capabilities\.resources = Some\(true\)', 'capabilities.resources = Some(ResourcesCapability { subscribe: Some(true), list_changed: Some(true) })'),
    (r'capabilities\.prompts = Some\(true\)', 'capabilities.prompts = Some(PromptsCapability { list_changed: Some(true) })'),
    (r'capabilities\.sampling = Some\(true\)', 'capabilities.sampling = Some(SamplingCapability {})'),
]

def fix_file(filepath):
    """Apply fixes to a single file"""
    print(f"Fixing {filepath}...")
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    original = content
    for pattern, replacement in FIXES:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        print(f"  ✓ Fixed {filepath}")
        return True
    else:
        print(f"  - No changes needed for {filepath}")
        return False

def main():
    """Fix all example files"""
    examples_dir = "examples"
    
    # Find all .rs files in examples directory
    rust_files = glob.glob(f"{examples_dir}/**/*.rs", recursive=True)
    
    fixed_count = 0
    for filepath in rust_files:
        if fix_file(filepath):
            fixed_count += 1
    
    print(f"\nFixed {fixed_count}/{len(rust_files)} files")

if __name__ == "__main__":
    main()
