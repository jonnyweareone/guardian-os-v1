#!/bin/bash
# Guardian OS Branding Asset Converter
# Converts SVG assets to PNG for use in Plymouth, GRUB, etc.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🛡️  Guardian OS Branding Asset Converter"
echo "========================================="

# Check for required tools
command -v convert >/dev/null 2>&1 || { echo "❌ ImageMagick required. Install with: brew install imagemagick"; exit 1; }
command -v rsvg-convert >/dev/null 2>&1 || { echo "❌ librsvg required. Install with: brew install librsvg"; exit 1; }

# Create output directories
mkdir -p logo/png
mkdir -p wallpapers/png
mkdir -p plymouth/png
mkdir -p grub/png

echo ""
echo "📦 Converting logos..."
# Logo - various sizes
for size in 16 24 32 48 64 96 128 256 512; do
    rsvg-convert -w $size -h $size logo/guardian-shield.svg > logo/png/guardian-shield-${size}.png
    echo "  ✓ guardian-shield-${size}.png"
done

# Wordmark
rsvg-convert -w 400 logo/guardian-os-wordmark.svg > logo/png/guardian-os-wordmark.png
echo "  ✓ guardian-os-wordmark.png"

echo ""
echo "🖼️  Converting wallpapers..."
# Wallpapers - 1080p and 4K
rsvg-convert -w 1920 -h 1080 wallpapers/guardian-dark.svg > wallpapers/png/guardian-dark-1080p.png
rsvg-convert -w 3840 -h 2160 wallpapers/guardian-dark.svg > wallpapers/png/guardian-dark-4k.png
rsvg-convert -w 1920 -h 1080 wallpapers/guardian-shield.svg > wallpapers/png/guardian-shield-1080p.png
rsvg-convert -w 3840 -h 2160 wallpapers/guardian-shield.svg > wallpapers/png/guardian-shield-4k.png
echo "  ✓ guardian-dark-1080p.png"
echo "  ✓ guardian-dark-4k.png"
echo "  ✓ guardian-shield-1080p.png"
echo "  ✓ guardian-shield-4k.png"

echo ""
echo "🚀 Converting Plymouth boot splash..."
rsvg-convert -w 200 -h 240 plymouth/guardian-logo.svg > plymouth/png/guardian-logo.png
rsvg-convert -w 300 -h 50 plymouth/guardian-wordmark.svg > plymouth/png/guardian-wordmark.png
rsvg-convert -w 300 -h 8 plymouth/progress-bg.svg > plymouth/png/progress-bg.png
rsvg-convert -w 296 -h 4 plymouth/progress-fill.svg > plymouth/png/progress-fill.png
echo "  ✓ guardian-logo.png"
echo "  ✓ guardian-wordmark.png"
echo "  ✓ progress-bg.png"
echo "  ✓ progress-fill.png"

echo ""
echo "🔧 Converting GRUB theme..."
rsvg-convert -w 1920 -h 1080 grub/background.svg > grub/png/background.png
rsvg-convert -w 100 -h 120 plymouth/guardian-logo.svg > grub/png/guardian-logo.png
echo "  ✓ background.png"
echo "  ✓ guardian-logo.png"

# Create selection highlight for GRUB menu
convert -size 500x40 xc:'#00E5FF10' -fill '#00E5FF20' -draw "roundrectangle 0,0 499,39 5,5" grub/png/select_c.png
echo "  ✓ select_c.png (menu highlight)"

echo ""
echo "✅ All assets converted successfully!"
echo ""
echo "Output locations:"
echo "  • logo/png/          - Logo files"
echo "  • wallpapers/png/    - Desktop wallpapers"
echo "  • plymouth/png/      - Boot splash images"
echo "  • grub/png/          - GRUB theme images"
