#!/bin/bash

# Build script for NEAR AI contracts

set -e

# Change to the directory where the script is located
cd "$(dirname "$0")"

echo "Building NEAR contracts..."

# Build Model Registry Contract
echo "Building Model Registry Contract..."
cd near-ai-model-registry
cargo build --target wasm32-unknown-unknown --release
cd ..

# Build Inference Service Contract
echo "Building Inference Service Contract..."
cd near-ai-inference
cargo build --target wasm32-unknown-unknown --release
cd ..

# Build Philanthropy Agent Contract
echo "Building Philanthropy Agent Contract..."
cd near-philanthropy-agent
cargo build --target wasm32-unknown-unknown --release
cd ..

# Build Silence Bridge Intent Registry
echo "Building Silence Bridge Intent Registry..."
cd silence-bridge
cargo build --target wasm32-unknown-unknown --release
cd ..

echo "Build complete!"
echo "Compiled contracts:"
echo "  - near-ai-model-registry/target/wasm32-unknown-unknown/release/near_ai_model_registry.wasm"
echo "  - near-ai-inference/target/wasm32-unknown-unknown/release/near_ai_inference.wasm"
echo "  - near-philanthropy-agent/target/wasm32-unknown-unknown/release/philanthropy_agent.wasm"
echo "  - silence-bridge/target/wasm32-unknown-unknown/release/silence_bridge_intent_registry.wasm"
