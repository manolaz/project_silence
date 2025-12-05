#!/bin/bash
# Docker-based reproducible build for Silence Bridge
# This ensures WASM compatibility with NEAR VM

set -e

cd "$(dirname "$0")"

echo "Building Silence Bridge contract using Docker..."

# Use official NEAR contract builder
docker run --rm \
  -v "$(pwd)":/host \
  -w /host \
  nearprotocol/contract-builder:latest \
  cargo build --target wasm32-unknown-unknown --release

# Strip debug symbols
echo "Stripping debug symbols..."
wasm-tools strip --all \
  target/wasm32-unknown-unknown/release/silence_bridge.wasm \
  -o target/wasm32-unknown-unknown/release/silence_bridge_stripped.wasm

echo "Build complete!"
echo "WASM file: target/wasm32-unknown-unknown/release/silence_bridge_stripped.wasm"

