#!/bin/bash

# Fix all examples to use correct imports and API

echo "Fixing example imports and API usage..."

# Fix all imports to use prelude
find examples -name "*.rs" -type f | while read -r file; do
    echo "Processing: $file"
    
    # Replace complex imports with prelude
    sed -i.bak 's/use prism_mcp_rs::{[^}]*};/use prism_mcp_rs::prelude::*;/g' "$file"
    
    # Remove StdioTransport imports (not in prelude)
    sed -i.bak '/transport::stdio::StdioTransport/d' "$file"
    
    # Fix ServerBuilder::new() calls (no longer takes string argument)
    sed -i.bak 's/ServerBuilder::new("[^"]*")/ServerBuilder::new()/g' "$file"
    
    # Fix CallToolResult initialization - add missing fields
    perl -i.bak -pe 's/CallToolResult\s*{\s*content:\s*([^,]+),\s*is_error:\s*([^}]+)\s*}/CallToolResult { content: $1, is_error: $2, meta: None, structured_content: None }/g' "$file"
    
    # Fix server builder pattern
    perl -i.bak -0pe 's/ServerBuilder::new\(\)\s*\.version/ServerBuilder::new()\n        .name("example")\n        .version/g' "$file"
    
    # Remove backup files
    rm -f "${file}.bak"
done

echo "Fixing complete. Building examples..."
cargo build --examples 2>&1 | tee build_results.txt

echo "Build complete. Check build_results.txt for details."
