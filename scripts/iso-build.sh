#!/bin/bash
set -euo pipefail

ISO_VERSION="${ISO_VERSION:-1.0.0}"
ISO_ARCH="amd64"

echo "Building Guardian OS ISO ${ISO_VERSION}..."

# Check for live-build
if ! command -v lb &> /dev/null; then
    echo "Error: live-build not installed"
    echo "Install with: sudo apt install live-build"
    exit 1
fi

# Clean previous builds
rm -rf iso/chroot iso/binary iso/.build

# Initialize live-build
cd iso
lb config \
    --mode ubuntu \
    --distribution noble \
    --architectures "${ISO_ARCH}" \
    --linux-flavours generic \
    --iso-application "Guardian OS" \
    --iso-publisher "Game Guardian AI" \
    --iso-volume "Guardian OS ${ISO_VERSION}" \
    --binary-images iso-hybrid \
    --bootappend-live "boot=casper quiet splash" \
    --memtest none \
    --apt-indices false \
    --cache-indices true

# Add our repository
mkdir -p config/archives
cp ../repo/GPG-KEY-GUARDIAN.asc config/archives/guardian.key
cat > config/archives/guardian.list << SOURCES
deb [trusted=yes] file:///repo noble main
SOURCES

# Copy repo into chroot
mkdir -p includes.chroot/repo
cp -r ../repo/* includes.chroot/repo/

# Copy includes
cp -r includes.chroot config/ 2>/dev/null || true
cp -r includes.binary config/ 2>/dev/null || true

# Copy package lists
mkdir -p config/package-lists
cp package-lists/*.list.chroot config/package-lists/ 2>/dev/null || true

# Add hooks
mkdir -p config/hooks/live
cat > config/hooks/live/0100-remove-ubiquity.hook.chroot << 'HOOK'
#!/bin/sh
set -e
apt-get remove --purge -y ubiquity ubiquity-casper ubiquity-frontend-* || true
apt-get autoremove --purge -y
HOOK

cat > config/hooks/live/0200-install-calamares.hook.chroot << 'HOOK'
#!/bin/sh
set -e
apt-get update
apt-get install -y calamares calamares-settings-ubuntu python3-requests || true
# Copy our Calamares config
cp -r /calamares/* /etc/calamares/ 2>/dev/null || true
HOOK

cat > config/hooks/live/0300-configure-system.hook.chroot << 'HOOK'
#!/bin/sh
set -e
# Update dconf database
dconf update || true
# Create Guardian directories
mkdir -p /etc/guardian
chmod 700 /etc/guardian
HOOK

chmod +x config/hooks/live/*.hook.chroot

# Copy Calamares config
cp -r ../calamares includes.chroot/ 2>/dev/null || true

# Build ISO
lb build

cd ..

# Copy final ISO
cp "iso/live-image-${ISO_ARCH}.hybrid.iso" "guardian-os-${ISO_VERSION}-${ISO_ARCH}.iso" 2>/dev/null || {
    echo "ISO build completed (output location may vary based on live-build version)"
}

echo "ISO build process complete"
