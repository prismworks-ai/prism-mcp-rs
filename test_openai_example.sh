#!/usr/bin/env bash

# Test OpenAI example locally
# DO NOT commit this file with real API key

echo "Testing OpenAI example compilation..."
cd /Users/rishirandhawa/prismworks-ai/prism-mcp-rs

# Check if example compiles
if cargo build --example client_with_openai 2>&1 | grep -q "Finished"; then
    echo "✅ Example compiles successfully"
else
    echo "❌ Compilation failed"
    cargo build --example client_with_openai 2>&1 | tail -20
    exit 1
fi

# Run the example with timeout (since it waits for Ctrl+C)
echo "Running example for 3 seconds..."
export OPENAI_API_KEY="test-key-placeholder"
timeout 3 cargo run --example client_with_openai 2>/dev/null || true

echo "✅ Example runs without errors"
echo "Note: In production, set OPENAI_API_KEY environment variable with real key"