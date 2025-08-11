#!/usr/bin/env python3
"""
Convert Rust documentation to structured markdown for /docs directory.
Extracts key API information and creates cross-referenced documentation.
"""

import os
import re
import json
from pathlib import Path

def extract_module_docs():
    """Extract module documentation from src files."""
    src_dir = Path('src')
    modules = {}
    
    for rust_file in src_dir.rglob('*.rs'):
        if rust_file.name == 'lib.rs' or rust_file.name == 'mod.rs':
            module_name = rust_file.parent.name if rust_file.name == 'mod.rs' else 'root'
            with open(rust_file, 'r') as f:
                content = f.read()
                # Extract module-level documentation
                doc_pattern = r'^//! (.+)$'
                docs = re.findall(doc_pattern, content, re.MULTILINE)
                if docs:
                    modules[module_name] = '\n'.join(docs)
    
    return modules

def generate_api_reference():
    """Generate API reference markdown."""
    modules = extract_module_docs()
    
    output = """# API Reference

> **Auto-generated from Rust source code**  
> Last updated: {date}

This is the complete API reference for the MCP Protocol SDK. For detailed type signatures and examples, see the [Rust API documentation](https://docs.rs/mcp-protocol-sdk).

## Modules

"""
    
    # Core modules
    core_modules = ['client', 'server', 'transport', 'protocol', 'core']
    
    for module in core_modules:
        if module in modules:
            output += f"### [{module}](https://docs.rs/mcp-protocol-sdk/latest/mcp_protocol_sdk/{module}/index.html)\n\n"
            output += f"{modules[module][:200]}...\n\n"
            output += f"[View full documentation →](https://docs.rs/mcp-protocol-sdk/latest/mcp_protocol_sdk/{module}/index.html)\n\n"
    
    # Save to docs directory
    with open('docs/api-reference.md', 'w') as f:
        f.write(output.format(date=__import__('datetime').datetime.now().strftime('%Y-%m-%d')))
    
    print("[x] Generated docs/api-reference.md")

if __name__ == '__main__':
    generate_api_reference()
