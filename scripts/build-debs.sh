#!/bin/bash
set -euo pipefail

PACKAGES=(
    "guardian-gnome-desktop"
    "guardian-gnome-theme"
    "guardian-gnome-layouts"
    "guardian-auth-client"
    "guardian-device-agent"
    "guardian-parental"
    "guardian-heartbeat"
    "guardian-activate"
    "guardian-apps-base"
    "guardian-gaming-meta"
    "guardian-reflex"
    "guardian-reflex-models"
)

# Ensure assets exist
if [ ! -d "assets" ] || [ ! -f "assets/wallpaper.png" ]; then
    echo "Assets not found. Running fetch-assets.sh..."
    scripts/fetch-assets.sh
fi

# Build each package
for pkg in "${PACKAGES[@]}"; do
    if [ -d "packages/$pkg" ]; then
        echo "Building $pkg..."
        cd "packages/$pkg"
        
        # Copy assets where needed
        if [ "$pkg" = "guardian-gnome-theme" ]; then
            mkdir -p usr/share/backgrounds/guardian
            mkdir -p usr/share/pixmaps/guardian
            cp ../../assets/wallpaper.png usr/share/backgrounds/guardian/ || true
            cp ../../assets/logo-*.png usr/share/pixmaps/guardian/ || true
        fi
        
        # Build package
        dpkg-buildpackage -b -uc -us || {
            echo "Warning: Failed to build $pkg (may need dependencies)"
        }
        cd ../..
    else
        echo "Warning: Package directory $pkg not found"
    fi
done

# Move built packages to repo staging
mkdir -p repo/incoming
mv packages/*.deb repo/incoming/ 2>/dev/null || true

echo "Package build complete"
