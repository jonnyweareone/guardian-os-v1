#!/bin/bash
# Build guardian-sync-server for Linux using Docker

set -e

cd /Users/davidsmith/Documents/GitHub/guardian-os-v1/guardian-sync-server

echo "Building guardian-sync-server for Linux..."

# Use a Rust builder image
docker run --rm \
  -v "$PWD":/app \
  -v cargo-cache:/usr/local/cargo/registry \
  -w /app \
  rust:1.83-slim-bookworm \
  bash -c "
    apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler && \
    cargo build --release
  "

echo "Build complete!"
echo "Binary at: target/release/guardian-sync-server"

# Copy to deployment folder
mkdir -p ../deployment/bin
cp target/release/guardian-sync-server ../deployment/bin/

echo "Copied to deployment/bin/"
