#!/bin/bash
# Guardian OS v1.1 ISO Build Script
# Creates a Guardian OS ISO from Pop!_OS base
#
# This script should be run on the build server

set -e

GUARDIAN_VERSION="1.1.0"
WORK_DIR="/opt/guardian-iso-build"
BUILD_DIR="/opt/guardian-build"

# Pop!_OS 24.04 LTS - NVIDIA version (more compatible)
POP_ISO_URL="https://iso.pop-os.org/24.04/amd64/nvidia/20/pop-os_24.04_amd64_nvidia_20.iso"
# Alternative: Intel version
# POP_ISO_URL="https://iso.pop-os.org/24.04/amd64/intel/20/pop-os_24.04_amd64_intel_20.iso"

ISO_MOUNT="${WORK_DIR}/iso-mount"
SQUASHFS_DIR="${WORK_DIR}/squashfs"
NEW_ISO_DIR="${WORK_DIR}/new-iso"
OUTPUT_DIR="${WORK_DIR}/output"
LOG_FILE="${WORK_DIR}/build.log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${CYAN}[$(date +%H:%M:%S)]${NC} $1" | tee -a "$LOG_FILE"; }
success() { echo -e "${GREEN}✓${NC} $1" | tee -a "$LOG_FILE"; }
error() { echo -e "${RED}✗${NC} $1" | tee -a "$LOG_FILE"; exit 1; }
warn() { echo -e "${YELLOW}⚠${NC} $1" | tee -a "$LOG_FILE"; }

cleanup() {
    log "Cleaning up mounts..."
    umount "${SQUASHFS_DIR}/run" 2>/dev/null || true
    umount "${SQUASHFS_DIR}/sys" 2>/dev/null || true
    umount "${SQUASHFS_DIR}/proc" 2>/dev/null || true
    umount "${SQUASHFS_DIR}/dev/pts" 2>/dev/null || true
    umount "${SQUASHFS_DIR}/dev" 2>/dev/null || true
    umount "${ISO_MOUNT}" 2>/dev/null || true
}

trap cleanup EXIT

echo -e "${CYAN}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════╗
║            Guardian OS ISO Builder v1.1                           ║
║            Building from Pop!_OS 24.04 Base                       ║
╚═══════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Initialize log
echo "Guardian OS Build Log - $(date)" > "$LOG_FILE"

# ============================================
# Step 0: Install dependencies
# ============================================
log "Installing build dependencies..."
apt-get update -qq
apt-get install -y \
    squashfs-tools \
    xorriso \
    isolinux \
    syslinux-common \
    wget \
    rsync \
    || error "Failed to install dependencies"
success "Dependencies installed"

# ============================================
# Step 1: Create directories
# ============================================
log "Creating work directories..."
mkdir -p "${WORK_DIR}" "${ISO_MOUNT}" "${OUTPUT_DIR}"
success "Directories created"

# ============================================
# Step 2: Download Pop!_OS ISO
# ============================================
if [ ! -f "${WORK_DIR}/pop-os-base.iso" ]; then
    log "Downloading Pop!_OS 24.04 ISO (this may take a while)..."
    wget --progress=bar:force -O "${WORK_DIR}/pop-os-base.iso" "${POP_ISO_URL}" \
        || error "Failed to download ISO"
    success "ISO downloaded"
else
    log "Using existing Pop!_OS ISO"
fi

# Verify ISO
log "Verifying ISO..."
file "${WORK_DIR}/pop-os-base.iso" | grep -q "ISO 9660" \
    || error "Downloaded file is not a valid ISO"
success "ISO verified"

# ============================================
# Step 3: Clean previous build
# ============================================
log "Cleaning previous build..."
rm -rf "${SQUASHFS_DIR}" "${NEW_ISO_DIR}"
mkdir -p "${NEW_ISO_DIR}"

# ============================================
# Step 4: Mount and extract ISO
# ============================================
log "Mounting ISO..."
umount "${ISO_MOUNT}" 2>/dev/null || true
mount -o loop,ro "${WORK_DIR}/pop-os-base.iso" "${ISO_MOUNT}" \
    || error "Failed to mount ISO"
success "ISO mounted"

log "Copying ISO structure (excluding squashfs)..."
rsync -a --exclude='casper/filesystem.squashfs' "${ISO_MOUNT}/" "${NEW_ISO_DIR}/" \
    || error "Failed to copy ISO structure"
success "ISO structure copied"

log "Extracting squashfs filesystem (this takes several minutes)..."
unsquashfs -d "${SQUASHFS_DIR}" "${ISO_MOUNT}/casper/filesystem.squashfs" \
    || error "Failed to extract squashfs"
success "Filesystem extracted"

umount "${ISO_MOUNT}"

# ============================================
# Step 5: Setup chroot
# ============================================
log "Setting up chroot environment..."
mount --bind /dev "${SQUASHFS_DIR}/dev"
mount --bind /dev/pts "${SQUASHFS_DIR}/dev/pts"
mount --bind /proc "${SQUASHFS_DIR}/proc"
mount --bind /sys "${SQUASHFS_DIR}/sys"
mount --bind /run "${SQUASHFS_DIR}/run"

# Copy DNS config
cp /etc/resolv.conf "${SQUASHFS_DIR}/etc/resolv.conf"
success "Chroot environment ready"

# ============================================
# Step 6: Copy Guardian packages
# ============================================
log "Copying Guardian packages to chroot..."
mkdir -p "${SQUASHFS_DIR}/tmp/guardian-packages"

if [ -d "${BUILD_DIR}/packages" ] && ls "${BUILD_DIR}/packages"/*.deb 1>/dev/null 2>&1; then
    cp "${BUILD_DIR}/packages"/*.deb "${SQUASHFS_DIR}/tmp/guardian-packages/"
    success "Guardian packages copied"
else
    warn "No Guardian packages found in ${BUILD_DIR}/packages - continuing without custom packages"
fi

# ============================================
# Step 7: Install packages in chroot
# ============================================
log "Installing Guardian packages in chroot..."
chroot "${SQUASHFS_DIR}" /bin/bash << 'CHROOT_SCRIPT'
#!/bin/bash
set -e

# Update package lists
apt-get update -qq

# Install Guardian packages if present
if ls /tmp/guardian-packages/*.deb 1>/dev/null 2>&1; then
    echo "Installing Guardian packages..."
    dpkg -i /tmp/guardian-packages/*.deb 2>&1 || true
    apt-get install -f -y
    echo "Guardian packages installed"
fi

# Enable guardian-daemon if installed
if [ -f /lib/systemd/system/guardian-daemon.service ]; then
    systemctl enable guardian-daemon
    echo "guardian-daemon enabled"
fi

# Clean up
rm -rf /tmp/guardian-packages
apt-get clean
apt-get autoremove -y

echo "Chroot configuration complete"
CHROOT_SCRIPT

success "Chroot configuration complete"

# ============================================
# Step 8: Update branding
# ============================================
log "Applying Guardian OS branding..."

# Update os-release
cat > "${SQUASHFS_DIR}/etc/os-release" << OSREL
NAME="Guardian OS"
VERSION="${GUARDIAN_VERSION}"
ID=guardian
ID_LIKE="ubuntu pop"
PRETTY_NAME="Guardian OS ${GUARDIAN_VERSION}"
VERSION_ID="${GUARDIAN_VERSION}"
HOME_URL="https://gameguardian.ai"
SUPPORT_URL="https://gameguardian.ai/support"
BUG_REPORT_URL="https://github.com/jonnyweareone/guardian-os-v1/issues"
PRIVACY_POLICY_URL="https://gameguardian.ai/privacy"
VERSION_CODENAME=noble
UBUNTU_CODENAME=noble
OSREL

# Update issue files
echo "Guardian OS ${GUARDIAN_VERSION} \\n \\l" > "${SQUASHFS_DIR}/etc/issue"
echo "Guardian OS ${GUARDIAN_VERSION}" > "${SQUASHFS_DIR}/etc/issue.net"

# Update lsb-release if present
if [ -f "${SQUASHFS_DIR}/etc/lsb-release" ]; then
    cat > "${SQUASHFS_DIR}/etc/lsb-release" << LSB
DISTRIB_ID=Guardian
DISTRIB_RELEASE=${GUARDIAN_VERSION}
DISTRIB_CODENAME=noble
DISTRIB_DESCRIPTION="Guardian OS ${GUARDIAN_VERSION}"
LSB
fi

success "Branding applied"

# ============================================
# Step 9: Unmount chroot
# ============================================
log "Unmounting chroot..."
umount "${SQUASHFS_DIR}/run" 2>/dev/null || true
umount "${SQUASHFS_DIR}/sys" 2>/dev/null || true
umount "${SQUASHFS_DIR}/proc" 2>/dev/null || true
umount "${SQUASHFS_DIR}/dev/pts" 2>/dev/null || true
umount "${SQUASHFS_DIR}/dev" 2>/dev/null || true
success "Chroot unmounted"

# ============================================
# Step 10: Repack squashfs
# ============================================
log "Repacking squashfs filesystem (this takes several minutes)..."
rm -f "${NEW_ISO_DIR}/casper/filesystem.squashfs"
mksquashfs "${SQUASHFS_DIR}" "${NEW_ISO_DIR}/casper/filesystem.squashfs" \
    -comp xz -b 1M -Xdict-size 100% \
    || error "Failed to create squashfs"

# Calculate filesystem size
du -sx --block-size=1 "${SQUASHFS_DIR}" | cut -f1 > "${NEW_ISO_DIR}/casper/filesystem.size"
success "Squashfs repacked"

# ============================================
# Step 11: Update manifests
# ============================================
log "Updating filesystem manifests..."
chroot "${SQUASHFS_DIR}" dpkg-query -W --showformat='${Package} ${Version}\n' \
    > "${NEW_ISO_DIR}/casper/filesystem.manifest" 2>/dev/null || true
success "Manifests updated"

# ============================================
# Step 12: Generate checksums
# ============================================
log "Generating checksums..."
cd "${NEW_ISO_DIR}"
find . -type f -not -name 'md5sum.txt' -not -path './isolinux/*' -print0 | \
    xargs -0 md5sum > md5sum.txt 2>/dev/null || true
success "Checksums generated"

# ============================================
# Step 13: Create bootable ISO
# ============================================
log "Creating bootable ISO..."
OUTPUT_ISO="${OUTPUT_DIR}/guardian-os_${GUARDIAN_VERSION}_amd64.iso"

cd "${NEW_ISO_DIR}"

# Detect boot files
EFI_IMG=""
if [ -f "boot/grub/efi.img" ]; then
    EFI_IMG="boot/grub/efi.img"
elif [ -f "EFI/boot/efi.img" ]; then
    EFI_IMG="EFI/boot/efi.img"
fi

ISOLINUX_BIN=""
if [ -f "isolinux/isolinux.bin" ]; then
    ISOLINUX_BIN="isolinux/isolinux.bin"
elif [ -f "syslinux/isolinux.bin" ]; then
    ISOLINUX_BIN="syslinux/isolinux.bin"
fi

# Find MBR file
MBR_FILE="/usr/lib/ISOLINUX/isohdpfx.bin"
if [ ! -f "$MBR_FILE" ]; then
    MBR_FILE="/usr/lib/syslinux/mbr/isohdpfx.bin"
fi
if [ ! -f "$MBR_FILE" ]; then
    # Extract from original ISO
    log "Extracting MBR from original ISO..."
    dd if="${WORK_DIR}/pop-os-base.iso" bs=512 count=1 of="${WORK_DIR}/isohdpfx.bin" 2>/dev/null
    MBR_FILE="${WORK_DIR}/isohdpfx.bin"
fi

if [ -n "$ISOLINUX_BIN" ] && [ -n "$EFI_IMG" ] && [ -f "$MBR_FILE" ]; then
    log "Creating hybrid BIOS/UEFI ISO..."
    xorriso -as mkisofs \
        -volid "Guardian OS ${GUARDIAN_VERSION}" \
        -isohybrid-mbr "$MBR_FILE" \
        -c isolinux/boot.cat \
        -b "$ISOLINUX_BIN" \
        -no-emul-boot \
        -boot-load-size 4 \
        -boot-info-table \
        -eltorito-alt-boot \
        -e "$EFI_IMG" \
        -no-emul-boot \
        -isohybrid-gpt-basdat \
        -o "${OUTPUT_ISO}" \
        . 2>&1 | tee -a "$LOG_FILE"
elif [ -n "$EFI_IMG" ]; then
    log "Creating UEFI-only ISO..."
    xorriso -as mkisofs \
        -volid "Guardian OS ${GUARDIAN_VERSION}" \
        -e "$EFI_IMG" \
        -no-emul-boot \
        -o "${OUTPUT_ISO}" \
        . 2>&1 | tee -a "$LOG_FILE"
else
    error "Cannot find boot files to create bootable ISO"
fi

# ============================================
# Verify ISO
# ============================================
if [ -f "${OUTPUT_ISO}" ]; then
    ISO_SIZE=$(du -h "${OUTPUT_ISO}" | cut -f1)
    success "ISO created: ${OUTPUT_ISO} (${ISO_SIZE})"
else
    error "ISO creation failed"
fi

# ============================================
# Summary
# ============================================
echo ""
echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}Guardian OS ${GUARDIAN_VERSION} Build Complete!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo "ISO Location: ${OUTPUT_ISO}"
echo "ISO Size: ${ISO_SIZE}"
echo "Build Log: ${LOG_FILE}"
echo ""
echo "To test the ISO:"
echo "  - Upload to a VM provider"
echo "  - Boot from the ISO"
echo "  - Select 'Try or Install'"
echo ""
echo "To download:"
echo "  scp root@$(hostname -I | awk '{print $1}'):${OUTPUT_ISO} ."
