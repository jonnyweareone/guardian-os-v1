#!/bin/bash
set -euo pipefail

echo "Fetching Guardian OS brand assets..."

# Create assets directory if it doesn't exist
mkdir -p assets

# Download assets with proper names
curl -fL "https://gameguardian.ai/lovable-uploads/guardian-wallpaper-desktop.png" \
    -o assets/wallpaper.png || {
    echo "Failed to download wallpaper"
    exit 1
}

curl -fL "https://gameguardian.ai/lovable-uploads/guardian-logo-shield-text-dark.png" \
    -o assets/logo-dark.png || {
    echo "Failed to download dark logo"
    exit 1
}

curl -fL "https://gameguardian.ai/lovable-uploads/guardian-logo2-transparent.png" \
    -o assets/logo-transparent.png || {
    echo "Failed to download transparent logo"
    exit 1
}

echo "Assets downloaded successfully:"
ls -la assets/
