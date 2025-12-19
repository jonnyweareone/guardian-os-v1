#!/bin/bash
# Guardian OS - Pre-install Hook
# Runs before package installation in chroot

set -e

echo "🛡️  Guardian OS Pre-Install Hook"

# Add Guardian PPA (when available)
# add-apt-repository -y ppa:guardian-os/stable

# For now, we'll copy .deb files from overlay
echo "  ✓ Pre-install complete"
