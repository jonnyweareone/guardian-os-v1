#!/bin/bash
# Deploy and build guardian-sync-server on VPS

set -e

VPS_HOST="192.248.163.171"
VPS_USER="root"
REMOTE_DIR="/opt/guardian-sync-server"

echo "=== Guardian Sync Server Deployment ==="

# Create remote directory
ssh ${VPS_USER}@${VPS_HOST} "mkdir -p ${REMOTE_DIR}"

# Sync source files (excluding target directory)
echo "Syncing source files..."
rsync -avz --delete \
  --exclude 'target/' \
  --exclude '.git/' \
  --exclude '*.lock' \
  /Users/davidsmith/Documents/GitHub/guardian-os-v1/guardian-sync-server/ \
  ${VPS_USER}@${VPS_HOST}:${REMOTE_DIR}/

# Install Rust and build on VPS
echo "Building on VPS..."
ssh ${VPS_USER}@${VPS_HOST} << 'ENDSSH'
set -e
cd /opt/guardian-sync-server

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Install build dependencies
apt-get update
apt-get install -y pkg-config libssl-dev protobuf-compiler

# Build release
source ~/.cargo/env
cargo build --release

echo "Build complete!"
ls -la target/release/guardian-sync-server
ENDSSH

echo "=== Deployment Complete ==="
