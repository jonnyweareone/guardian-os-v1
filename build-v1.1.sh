#!/bin/bash
# Guardian OS v1.1 Build Script
# Run this locally to sync to build server and trigger ISO build
#
# Usage: ./build-v1.1.sh
#
# Prerequisites:
#   - SSH key access to build server
#   - Build server: 136.244.71.108

set -e

# Configuration
BUILD_SERVER="136.244.71.108"
BUILD_USER="root"
GUARDIAN_VERSION="1.1.0"
REPO_URL="https://github.com/jonnyweareone/guardian-os-v1.git"
BUILD_DIR="/opt/guardian-build"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${CYAN}[$(date +%H:%M:%S)]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

echo -e "${CYAN}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════╗
║             Guardian OS v1.1.0 - Build & Deploy                   ║
║             AI Powered Protection For Families                    ║
╚═══════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Step 1: Push local changes
log "Step 1: Pushing local changes to GitHub..."
git add -A
git commit -m "Guardian OS v${GUARDIAN_VERSION} - Pre-build commit" 2>/dev/null || true
git push origin main
success "Changes pushed to GitHub"

# Step 2: Sync to build server and build
log "Step 2: Connecting to build server and starting build..."

ssh ${BUILD_USER}@${BUILD_SERVER} << REMOTE_SCRIPT
#!/bin/bash
set -e

echo "========================================="
echo "Guardian OS v${GUARDIAN_VERSION} Build"
echo "Build Server: \$(hostname)"
echo "========================================="

# Install dependencies if needed
if ! command -v cargo &>/dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

apt-get update -qq
apt-get install -y git squashfs-tools xorriso wget curl build-essential \
    pkg-config libssl-dev libsqlite3-dev libdbus-1-dev libxkbcommon-dev \
    libwayland-dev libinput-dev libudev-dev libgbm-dev libseat-dev \
    libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libexpat1-dev libfontconfig-dev libfreetype-dev clang just

# Clone or update repo
if [ -d "${BUILD_DIR}" ]; then
    echo "Updating existing repo..."
    cd ${BUILD_DIR}
    git fetch origin
    git reset --hard origin/main
    git submodule update --init --recursive
else
    echo "Cloning repo..."
    git clone --recursive ${REPO_URL} ${BUILD_DIR}
    cd ${BUILD_DIR}
fi

# Update version
echo "${GUARDIAN_VERSION}" > VERSION

# ==========================================
# Build guardian-installer
# ==========================================
echo ""
echo "========================================="
echo "Building guardian-installer..."
echo "========================================="

cd ${BUILD_DIR}/guardian-installer
source ~/.cargo/env

# Fetch cargo.just if missing
if [ ! -f cargo.just ]; then
    curl --proto '=https' --tlsv1.2 -sSf -o cargo.just https://raw.githubusercontent.com/pop-os/cosmic-justfiles/master/cargo.just
fi

# Build
cargo build --release

# Create .deb package
DEB_DIR="${BUILD_DIR}/packages/guardian-installer_${GUARDIAN_VERSION}_amd64"
rm -rf "\${DEB_DIR}"
mkdir -p "\${DEB_DIR}/DEBIAN"
mkdir -p "\${DEB_DIR}/usr/bin"
mkdir -p "\${DEB_DIR}/usr/share/applications"
mkdir -p "\${DEB_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "\${DEB_DIR}/etc/xdg/autostart"

cp target/release/cosmic-initial-setup "\${DEB_DIR}/usr/bin/guardian-installer"
chmod 755 "\${DEB_DIR}/usr/bin/guardian-installer"

# Also link as cosmic-initial-setup for compatibility
ln -sf guardian-installer "\${DEB_DIR}/usr/bin/cosmic-initial-setup"

# Copy desktop files
cp res/com.system76.CosmicInitialSetup.desktop "\${DEB_DIR}/usr/share/applications/" 2>/dev/null || true
cp res/com.system76.CosmicInitialSetup.Autostart.desktop "\${DEB_DIR}/etc/xdg/autostart/" 2>/dev/null || true
cp res/icon.svg "\${DEB_DIR}/usr/share/icons/hicolor/scalable/apps/com.system76.CosmicInitialSetup.svg" 2>/dev/null || true

cat > "\${DEB_DIR}/DEBIAN/control" << CONTROL
Package: guardian-installer
Version: ${GUARDIAN_VERSION}
Section: admin
Priority: optional
Architecture: amd64
Depends: libssl3, libxkbcommon0, libwayland-client0, libfontconfig1
Replaces: cosmic-initial-setup
Provides: cosmic-initial-setup
Conflicts: cosmic-initial-setup
Maintainer: Guardian OS Team <support@gameguardian.ai>
Description: Guardian OS Initial Setup Wizard
 First-boot wizard for Guardian OS with parent authentication,
 child profile selection, and optional sync enrollment.
 Built on COSMIC Initial Setup.
CONTROL

dpkg-deb --build "\${DEB_DIR}"
echo "✓ guardian-installer.deb built"

# ==========================================
# Build guardian-daemon
# ==========================================
echo ""
echo "========================================="
echo "Building guardian-daemon..."
echo "========================================="

cd ${BUILD_DIR}/guardian-components/guardian-daemon
cargo build --release || echo "guardian-daemon build skipped"

if [ -f target/release/guardian-daemon ] || [ -f target/release/guardian_daemon ]; then
    DEB_DIR="${BUILD_DIR}/packages/guardian-daemon_${GUARDIAN_VERSION}_amd64"
    rm -rf "\${DEB_DIR}"
    mkdir -p "\${DEB_DIR}/DEBIAN"
    mkdir -p "\${DEB_DIR}/usr/bin"
    mkdir -p "\${DEB_DIR}/lib/systemd/system"
    
    BINARY=\$(ls target/release/guardian-daemon target/release/guardian_daemon 2>/dev/null | head -1)
    cp "\${BINARY}" "\${DEB_DIR}/usr/bin/guardian-daemon"
    chmod 755 "\${DEB_DIR}/usr/bin/guardian-daemon"
    
    cat > "\${DEB_DIR}/lib/systemd/system/guardian-daemon.service" << SERVICE
[Unit]
Description=Guardian OS Safety Daemon
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/guardian-daemon
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
SERVICE

    cat > "\${DEB_DIR}/DEBIAN/control" << CONTROL
Package: guardian-daemon
Version: ${GUARDIAN_VERSION}
Section: misc
Priority: optional
Architecture: amd64
Depends: libssl3, libsqlite3-0
Maintainer: Guardian OS Team <support@gameguardian.ai>
Description: Guardian OS Safety Daemon
 Core safety enforcement service for Guardian OS.
 Monitors activity and enforces parental controls.
CONTROL

    cat > "\${DEB_DIR}/DEBIAN/postinst" << POSTINST
#!/bin/bash
systemctl daemon-reload
systemctl enable guardian-daemon
systemctl start guardian-daemon || true
POSTINST
    chmod 755 "\${DEB_DIR}/DEBIAN/postinst"

    dpkg-deb --build "\${DEB_DIR}"
    echo "✓ guardian-daemon.deb built"
fi

# ==========================================
# Build guardian-wizard
# ==========================================
echo ""
echo "========================================="
echo "Building guardian-wizard..."
echo "========================================="

cd ${BUILD_DIR}/guardian-components/guardian-wizard
cargo build --release || echo "guardian-wizard build skipped"

if [ -f target/release/guardian-wizard ] || [ -f target/release/guardian_wizard ]; then
    DEB_DIR="${BUILD_DIR}/packages/guardian-wizard_${GUARDIAN_VERSION}_amd64"
    rm -rf "\${DEB_DIR}"
    mkdir -p "\${DEB_DIR}/DEBIAN"
    mkdir -p "\${DEB_DIR}/usr/bin"
    
    BINARY=\$(ls target/release/guardian-wizard target/release/guardian_wizard 2>/dev/null | head -1)
    cp "\${BINARY}" "\${DEB_DIR}/usr/bin/guardian-wizard"
    chmod 755 "\${DEB_DIR}/usr/bin/guardian-wizard"

    cat > "\${DEB_DIR}/DEBIAN/control" << CONTROL
Package: guardian-wizard
Version: ${GUARDIAN_VERSION}
Section: misc
Priority: optional
Architecture: amd64
Depends: libssl3, libxkbcommon0, libwayland-client0, libfontconfig1
Maintainer: Guardian OS Team <support@gameguardian.ai>
Description: Guardian OS Setup Wizard
 First-boot wizard for Guardian OS family setup.
CONTROL

    dpkg-deb --build "\${DEB_DIR}"
    echo "✓ guardian-wizard.deb built"
fi

# ==========================================
# List built packages
# ==========================================
echo ""
echo "========================================="
echo "Built Packages:"
echo "========================================="
ls -la ${BUILD_DIR}/packages/*.deb

# ==========================================
# Build ISO
# ==========================================
echo ""
echo "========================================="
echo "Building Guardian OS ISO..."
echo "========================================="

cd ${BUILD_DIR}

# Update build script to use local packages
cat > /tmp/build-iso-v1.1.sh << 'ISOSCRIPT'
#!/bin/bash
set -e

GUARDIAN_VERSION="${GUARDIAN_VERSION}"
WORK_DIR="/opt/guardian-iso-build"
BUILD_DIR="/opt/guardian-build"
POP_ISO_URL="https://iso.pop-os.org/24.04/amd64/intel/20/pop-os_24.04_amd64_intel_20.iso"

ISO_MOUNT="\${WORK_DIR}/iso-mount"
SQUASHFS_DIR="\${WORK_DIR}/squashfs"
NEW_ISO_DIR="\${WORK_DIR}/new-iso"
OUTPUT_DIR="\${WORK_DIR}/output"

echo "Creating work directories..."
mkdir -p "\${WORK_DIR}" "\${ISO_MOUNT}" "\${OUTPUT_DIR}"

# Download Pop!_OS ISO if not present
if [ ! -f "\${WORK_DIR}/pop-os-base.iso" ]; then
    echo "Downloading Pop!_OS 24.04 ISO..."
    wget --progress=bar:force -O "\${WORK_DIR}/pop-os-base.iso" "\${POP_ISO_URL}"
fi

# Clean previous build
rm -rf "\${SQUASHFS_DIR}" "\${NEW_ISO_DIR}"
mkdir -p "\${NEW_ISO_DIR}"

# Mount ISO
umount "\${ISO_MOUNT}" 2>/dev/null || true
mount -o loop,ro "\${WORK_DIR}/pop-os-base.iso" "\${ISO_MOUNT}"
echo "✓ ISO mounted"

# Copy ISO structure
echo "Copying ISO structure..."
rsync -a --exclude='casper/filesystem.squashfs' "\${ISO_MOUNT}/" "\${NEW_ISO_DIR}/"

# Extract squashfs
echo "Extracting filesystem (this takes a while)..."
unsquashfs -d "\${SQUASHFS_DIR}" "\${ISO_MOUNT}/casper/filesystem.squashfs"
echo "✓ Filesystem extracted"

umount "\${ISO_MOUNT}"

# Mount for chroot
mount --bind /dev "\${SQUASHFS_DIR}/dev"
mount --bind /dev/pts "\${SQUASHFS_DIR}/dev/pts"
mount --bind /proc "\${SQUASHFS_DIR}/proc"
mount --bind /sys "\${SQUASHFS_DIR}/sys"
mount --bind /run "\${SQUASHFS_DIR}/run"
cp /etc/resolv.conf "\${SQUASHFS_DIR}/etc/resolv.conf"

# Copy packages
mkdir -p "\${SQUASHFS_DIR}/tmp/guardian-packages"
cp ${BUILD_DIR}/packages/*.deb "\${SQUASHFS_DIR}/tmp/guardian-packages/"

# Install in chroot
echo "Installing Guardian packages..."
chroot "\${SQUASHFS_DIR}" /bin/bash << 'CHROOT'
dpkg -i /tmp/guardian-packages/*.deb || apt-get install -f -y
systemctl enable guardian-daemon 2>/dev/null || true
rm -rf /tmp/guardian-packages
apt-get clean
CHROOT

# Update branding
cat > "\${SQUASHFS_DIR}/etc/os-release" << OSREL
NAME="Guardian OS"
VERSION="${GUARDIAN_VERSION}"
ID=guardian
ID_LIKE=ubuntu pop
PRETTY_NAME="Guardian OS ${GUARDIAN_VERSION}"
VERSION_ID="${GUARDIAN_VERSION}"
HOME_URL="https://gameguardian.ai"
SUPPORT_URL="https://gameguardian.ai/support"
UBUNTU_CODENAME=noble
OSREL

# Unmount chroot
umount "\${SQUASHFS_DIR}/run" 2>/dev/null || true
umount "\${SQUASHFS_DIR}/sys" 2>/dev/null || true
umount "\${SQUASHFS_DIR}/proc" 2>/dev/null || true
umount "\${SQUASHFS_DIR}/dev/pts" 2>/dev/null || true
umount "\${SQUASHFS_DIR}/dev" 2>/dev/null || true

# Repack squashfs
echo "Repacking filesystem..."
rm -f "\${NEW_ISO_DIR}/casper/filesystem.squashfs"
mksquashfs "\${SQUASHFS_DIR}" "\${NEW_ISO_DIR}/casper/filesystem.squashfs" -comp xz -b 1M
du -sx --block-size=1 "\${SQUASHFS_DIR}" | cut -f1 > "\${NEW_ISO_DIR}/casper/filesystem.size"

# Create ISO
echo "Creating ISO..."
OUTPUT_ISO="\${OUTPUT_DIR}/guardian-os_${GUARDIAN_VERSION}_amd64.iso"

cd "\${NEW_ISO_DIR}"
find . -type f -not -name 'md5sum.txt' -print0 | xargs -0 md5sum > md5sum.txt

xorriso -as mkisofs \
    -volid "Guardian OS ${GUARDIAN_VERSION}" \
    -isohybrid-mbr /usr/lib/ISOLINUX/isohdpfx.bin \
    -c isolinux/boot.cat \
    -b isolinux/isolinux.bin \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    -eltorito-alt-boot \
    -e boot/grub/efi.img \
    -no-emul-boot \
    -isohybrid-gpt-basdat \
    -o "\${OUTPUT_ISO}" \
    . 2>/dev/null || xorriso -as mkisofs -volid "Guardian OS ${GUARDIAN_VERSION}" -o "\${OUTPUT_ISO}" .

echo ""
echo "========================================="
echo "✓ BUILD COMPLETE!"
echo "========================================="
echo "ISO: \${OUTPUT_ISO}"
echo "Size: \$(du -h "\${OUTPUT_ISO}" | cut -f1)"
ISOSCRIPT

chmod +x /tmp/build-iso-v1.1.sh
bash /tmp/build-iso-v1.1.sh

echo ""
echo "========================================="
echo "Guardian OS v${GUARDIAN_VERSION} Build Complete!"
echo "========================================="
echo "ISO Location: /opt/guardian-iso-build/output/"
ls -la /opt/guardian-iso-build/output/

REMOTE_SCRIPT

success "Build complete on server!"

echo ""
echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}Guardian OS v${GUARDIAN_VERSION} Build Complete!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo "To download the ISO:"
echo "  scp ${BUILD_USER}@${BUILD_SERVER}:/opt/guardian-iso-build/output/guardian-os_${GUARDIAN_VERSION}_amd64.iso ."
echo ""
echo "To test on VPS:"
echo "  ssh ${BUILD_USER}@${BUILD_SERVER}"
echo "  ls -la /opt/guardian-iso-build/output/"
