#!/bin/bash
# Build script for Guardian DNS
# Run on a system with Rust installed

set -e

cd "$(dirname "$0")"

echo "Building guardian-dns..."
cargo build --release

echo ""
echo "Build complete!"
echo "Binary: target/release/guardian-dns"
echo ""
echo "To install:"
echo "  sudo cp target/release/guardian-dns /usr/lib/guardian/"
echo "  sudo cp config/dns.toml /etc/guardian/"
echo ""
echo "To run (requires root for port 53):"
echo "  sudo ./target/release/guardian-dns"
