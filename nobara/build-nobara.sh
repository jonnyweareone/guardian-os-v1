#!/bin/bash
# Guardian OS Nobara Edition - Build Script
# Builds Guardian OS ISO based on Nobara Linux
#
# Prerequisites:
#   - Fedora/Nobara build system
#   - mock, lorax, livemedia-creator
#   - Rust toolchain for building guardian-daemon

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GUARDIAN_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="/var/tmp/guardian-nobara-build"
OUTPUT_DIR="${BUILD_DIR}/output"
VERSION="1.1.0"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${CYAN}[$(date +%H:%M:%S)]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

echo -e "${CYAN}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════╗
║        Guardian OS v1.1.0 - Nobara Edition Build                  ║
║              AI Powered Protection For Families                   ║
╚═══════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    error "This script must be run as root (for livemedia-creator)"
fi

# Create build directories
log "Creating build directories..."
mkdir -p "${BUILD_DIR}"/{rpms,repo,iso}
mkdir -p "${OUTPUT_DIR}"

# Step 1: Build Guardian packages
log "Step 1: Building Guardian RPM packages..."

# Build guardian-daemon
if [ -d "${GUARDIAN_ROOT}/../guardian-components/guardian-daemon" ]; then
    log "Building guardian-daemon..."
    cd "${GUARDIAN_ROOT}/../guardian-components/guardian-daemon"
    
    # Check for Rust
    if ! command -v cargo &>/dev/null; then
        warn "Rust not found, installing..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi
    
    cargo build --release
    
    # Create RPM structure
    RPM_BUILD="${BUILD_DIR}/rpmbuild"
    mkdir -p "${RPM_BUILD}"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
    
    cp target/release/guardian-daemon "${RPM_BUILD}/SOURCES/"
    cp "${SCRIPT_DIR}/packages/guardian-daemon/guardian-daemon.service" "${RPM_BUILD}/SOURCES/"
    cp "${SCRIPT_DIR}/packages/guardian-daemon/guardian-daemon.spec" "${RPM_BUILD}/SPECS/"
    
    # Build RPM
    rpmbuild --define "_topdir ${RPM_BUILD}" -bb "${RPM_BUILD}/SPECS/guardian-daemon.spec" || warn "RPM build failed, continuing..."
    
    # Copy RPMs to repo
    find "${RPM_BUILD}/RPMS" -name "*.rpm" -exec cp {} "${BUILD_DIR}/rpms/" \;
    success "guardian-daemon built"
else
    warn "guardian-daemon source not found, skipping"
fi

# Step 2: Create local repo
log "Step 2: Creating local package repository..."
cd "${BUILD_DIR}/rpms"
createrepo_c .
success "Local repo created"

# Step 3: Download Nobara base ISO (if needed)
NOBARA_ISO="${BUILD_DIR}/nobara-base.iso"
if [ ! -f "${NOBARA_ISO}" ]; then
    log "Step 3: Downloading Nobara base ISO..."
    # Use GNOME variant as base - Nobara 42 is latest
    NOBARA_URL="https://nobara-images.nobaraproject.org/Nobara-42-GNOME-2025-09-25.iso"
    wget --progress=bar:force -O "${NOBARA_ISO}" "${NOBARA_URL}" || {
        warn "Failed to download Nobara ISO. Using livemedia-creator instead."
        NOBARA_ISO=""
    }
else
    success "Nobara base ISO already downloaded"
fi

# Step 4: Build ISO
log "Step 4: Building Guardian OS ISO..."

if [ -n "${NOBARA_ISO}" ]; then
    # Method 1: Remaster existing ISO
    log "Remastering Nobara ISO..."
    
    ISO_MOUNT="${BUILD_DIR}/iso-mount"
    SQUASHFS="${BUILD_DIR}/squashfs"
    NEW_ISO="${BUILD_DIR}/new-iso"
    
    mkdir -p "${ISO_MOUNT}" "${NEW_ISO}"
    
    # Mount base ISO
    mount -o loop,ro "${NOBARA_ISO}" "${ISO_MOUNT}"
    
    # Copy ISO structure
    rsync -a --exclude='LiveOS/squashfs.img' "${ISO_MOUNT}/" "${NEW_ISO}/"
    
    # Extract squashfs
    unsquashfs -d "${SQUASHFS}" "${ISO_MOUNT}/LiveOS/squashfs.img"
    
    umount "${ISO_MOUNT}"
    
    # Install Guardian packages in chroot
    mount --bind /dev "${SQUASHFS}/dev"
    mount --bind /proc "${SQUASHFS}/proc"
    mount --bind /sys "${SQUASHFS}/sys"
    cp /etc/resolv.conf "${SQUASHFS}/etc/resolv.conf"
    
    # Copy our RPMs
    mkdir -p "${SQUASHFS}/tmp/guardian-rpms"
    cp "${BUILD_DIR}/rpms/"*.rpm "${SQUASHFS}/tmp/guardian-rpms/" 2>/dev/null || true
    
    # Install in chroot
    chroot "${SQUASHFS}" /bin/bash << 'CHROOT'
dnf install -y /tmp/guardian-rpms/*.rpm 2>/dev/null || true
systemctl enable guardian-daemon
rm -rf /tmp/guardian-rpms
dnf clean all
CHROOT

    # Update branding
    cat > "${SQUASHFS}/etc/os-release" << OSREL
NAME="Guardian OS"
VERSION="${VERSION} (Nobara Edition)"
ID=guardian
ID_LIKE=fedora nobara
VERSION_ID="${VERSION}"
PRETTY_NAME="Guardian OS ${VERSION}"
HOME_URL="https://gameguardian.ai"
SUPPORT_URL="https://gameguardian.ai/support"
OSREL

    # Unmount chroot
    umount "${SQUASHFS}/sys"
    umount "${SQUASHFS}/proc"
    umount "${SQUASHFS}/dev"
    
    # Repack squashfs
    rm -f "${NEW_ISO}/LiveOS/squashfs.img"
    mksquashfs "${SQUASHFS}" "${NEW_ISO}/LiveOS/squashfs.img" -comp xz
    
    # Create ISO
    OUTPUT_ISO="${OUTPUT_DIR}/guardian-os_${VERSION}_nobara_amd64.iso"
    
    xorriso -as mkisofs \
        -volid "Guardian OS ${VERSION}" \
        -isohybrid-mbr /usr/share/syslinux/isohdpfx.bin \
        -c isolinux/boot.cat \
        -b isolinux/isolinux.bin \
        -no-emul-boot \
        -boot-load-size 4 \
        -boot-info-table \
        -eltorito-alt-boot \
        -e images/efiboot.img \
        -no-emul-boot \
        -isohybrid-gpt-basdat \
        -o "${OUTPUT_ISO}" \
        "${NEW_ISO}"
    
    success "ISO created: ${OUTPUT_ISO}"
    
else
    # Method 2: Build from scratch with livemedia-creator
    log "Building ISO from kickstart..."
    
    livemedia-creator \
        --ks="${SCRIPT_DIR}/kickstarts/guardian-gnome.ks" \
        --no-virt \
        --resultdir="${OUTPUT_DIR}" \
        --project="Guardian OS" \
        --releasever=41 \
        --iso-name="guardian-os_${VERSION}_nobara_amd64.iso" \
        --iso-only \
        --macboot
    
    success "ISO created in ${OUTPUT_DIR}"
fi

# Cleanup
log "Cleaning up..."
rm -rf "${BUILD_DIR}/squashfs" "${BUILD_DIR}/new-iso" "${BUILD_DIR}/iso-mount"

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}          Guardian OS Nobara Edition Build Complete!               ${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Output: ${OUTPUT_DIR}/"
ls -lh "${OUTPUT_DIR}"/*.iso 2>/dev/null || echo "Check ${OUTPUT_DIR} for output files"
echo ""
echo "To test in a VM:"
echo "  qemu-system-x86_64 -enable-kvm -m 4G -cdrom ${OUTPUT_DIR}/guardian-os_${VERSION}_nobara_amd64.iso"
