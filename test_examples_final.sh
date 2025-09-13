#!/bin/bash

# Test all examples with proper features
cd /Users/rishirandhawa/prismworks-ai/prism-mcp-rs

echo "Testing MCP-RS Examples Build Status"
echo "====================================="
echo ""

# Examples without special features
for ex in 01_mcp_tool_macro 09_configuration; do
    echo -n "$ex: "
    if cargo build --example $ex 2>&1 | grep -q "Finished"; then
        echo "✅ PASS"
    else
        echo "❌ FAIL"
    fi
done

# Examples with features
echo -n "05_http_transport: "
if cargo build --example 05_http_transport --features http 2>&1 | grep -q "Finished"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

echo -n "06_websocket_transport: "
if cargo build --example 06_websocket_transport --features websocket 2>&1 | grep -q "Finished"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

echo -n "07_authentication: "
if cargo build --example 07_authentication --features http,auth 2>&1 | grep -q "Finished"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

echo -n "10_plugin_system: "
if cargo build --example 10_plugin_system --features stdio,plugin 2>&1 | grep -q "Finished"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

# Test existing working examples
echo ""
echo "Existing Examples:"
for ex in bidirectional_basic closure_handlers custom_transport minimal_working; do
    echo -n "$ex: "
    if cargo build --example $ex 2>&1 | grep -q "Finished"; then
        echo "✅ PASS"
    else
        echo "❌ FAIL"
    fi
done

echo ""
echo "Build test complete!"