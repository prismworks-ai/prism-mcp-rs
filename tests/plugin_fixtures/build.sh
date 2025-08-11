#!/bin/bash
# Build test plugin fixtures for integration tests

set -e

echo "Building test plugin fixtures..."

# Build calc1 plugin
cd calc1
cargo build --release
cd ..

echo "Test plugins built successfully!"
