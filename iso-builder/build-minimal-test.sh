#!/bin/bash
# Guardian OS v1.1 MINIMAL ISO Build Script
# Only changes branding - NO package modifications
# This is to test if the base repack process works

set -e

GUARDIAN_VERSION="1.1.0"
WORK_DIR="/opt/guardian-iso-build"
POP_ISO_URL="https://iso.pop-os.org/24.04/amd64/intel/20/pop-os_24.04_amd64_intel_20.iso"

ISO_MOUNT="${WORK_DIR}/iso-mount"
SQUASHFS_DIR="${WORK_DIR}/squashfs"
NEW_ISO_DIR="${WORK_DIR}/new-iso"
OUTPUT_DIR="${WORK_DIR}/output"

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${CYAN}[$(date +%H:%M:%S)]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

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
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║       Guardian OS MINIMAL ISO Build (Branding Only)               ║"
echo "║       Testing if base repack works without package changes        ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Install dependencies
log "Installing dependencies..."
apt-get update -qq
apt-get install -y squashfs-tools xorriso isolinux wget rsync

# Create directories
log "Creating directories..."
mkdir -p "${WORK_DIR}" "${ISO_MOUNT}" "${OUTPUT_DIR}"

# Download ISO if needed
if [ ! -f "${WORK_DIR}/pop-os-base.iso" ]; then
    log "Downloading Pop!_OS 24.04 ISO..."
    wget --progress=bar:force -O "${WORK_DIR}/pop-os-base.iso" "${POP_ISO_URL}"
fi

# Clean previous
rm -rf "${SQUASHFS_DIR}" "${NEW_ISO_DIR}"
mkdir -p "${NEW_ISO_DIR}"

# Mount ISO
log "Mounting ISO..."
umount "${ISO_MOUNT}" 2>/dev/null || true
mount -o loop,ro "${WORK_DIR}/pop-os-base.iso" "${ISO_MOUNT}"
success "ISO mounted"

# Copy ISO structure (KEEP EVERYTHING including squashfs initially)
log "Copying ISO structure..."
rsync -a "${ISO_MOUNT}/" "${NEW_ISO_DIR}/"
success "ISO structure copied"

umount "${ISO_MOUNT}"

# Extract squashfs for modification
log "Extracting squashfs..."
rm -rf "${SQUASHFS_DIR}"
unsquashfs -d "${SQUASHFS_DIR}" "${NEW_ISO_DIR}/casper/filesystem.squashfs"
success "Squashfs extracted"

# ============================================
# MINIMAL CHANGES - BRANDING ONLY
# ============================================
log "Applying MINIMAL branding changes..."

# Update /etc/os-release
cat > "${SQUASHFS_DIR}/etc/os-release" << 'OSREL'
NAME="Guardian OS"
VERSION="1.1.0"
ID=pop
ID_LIKE="ubuntu debian"
PRETTY_NAME="Guardian OS 1.1.0"
VERSION_ID="24.04"
HOME_URL="https://gameguardian.ai"
SUPPORT_URL="https://gameguardian.ai/support"
BUG_REPORT_URL="https://github.com/jonnyweareone/guardian-os-v1/issues"
PRIVACY_POLICY_URL="https://gameguardian.ai/privacy"
VERSION_CODENAME=noble
UBUNTU_CODENAME=noble
LOGO=pop-logo
OSREL

# Update /etc/issue
echo "Guardian OS 1.1.0 \\n \\l" > "${SQUASHFS_DIR}/etc/issue"
echo "Guardian OS 1.1.0" > "${SQUASHFS_DIR}/etc/issue.net"

# Update lsb-release
cat > "${SQUASHFS_DIR}/etc/lsb-release" << 'LSB'
DISTRIB_ID=Pop
DISTRIB_RELEASE=24.04
DISTRIB_CODENAME=noble
DISTRIB_DESCRIPTION="Guardian OS 1.1.0"
LSB

success "Branding applied (NO package changes)"

# ============================================
# Repack squashfs
# ============================================
log "Repacking squashfs (this takes several minutes)..."
rm -f "${NEW_ISO_DIR}/casper/filesystem.squashfs"
mksquashfs "${SQUASHFS_DIR}" "${NEW_ISO_DIR}/casper/filesystem.squashfs" \
    -comp xz -b 1M -Xdict-size 100%
success "Squashfs repacked"

# Update filesystem.size
du -sx --block-size=1 "${SQUASHFS_DIR}" | cut -f1 > "${NEW_ISO_DIR}/casper/filesystem.size"

# ============================================
# Update ISO boot menu text
# ============================================
log "Updating boot menu..."

# Update GRUB config
if [ -f "${NEW_ISO_DIR}/boot/grub/grub.cfg" ]; then
    sed -i 's/Pop!_OS/Guardian OS/g' "${NEW_ISO_DIR}/boot/grub/grub.cfg"
fi

# Update isolinux config  
if [ -f "${NEW_ISO_DIR}/isolinux/isolinux.cfg" ]; then
    sed -i 's/Pop!_OS/Guardian OS/g' "${NEW_ISO_DIR}/isolinux/isolinux.cfg"
fi

# Update any txt.cfg
find "${NEW_ISO_DIR}" -name "*.cfg" -exec sed -i 's/Pop!_OS/Guardian OS/g' {} \;

success "Boot menu updated"

# ============================================
# Regenerate md5sums
# ============================================
log "Regenerating checksums..."
cd "${NEW_ISO_DIR}"
find . -type f -not -name 'md5sum.txt' -not -path './isolinux/*' -print0 | \
    xargs -0 md5sum 2>/dev/null > md5sum.txt || true
success "Checksums generated"

# ============================================
# Create ISO
# ============================================
log "Creating ISO..."
OUTPUT_ISO="${OUTPUT_DIR}/guardian-os_${GUARDIAN_VERSION}_minimal_amd64.iso"

cd "${NEW_ISO_DIR}"

# Use the exact same method as Pop!_OS
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
    -o "${OUTPUT_ISO}" \
    .

if [ -f "${OUTPUT_ISO}" ]; then
    ISO_SIZE=$(du -h "${OUTPUT_ISO}" | cut -f1)
    success "ISO created: ${OUTPUT_ISO} (${ISO_SIZE})"
else
    error "ISO creation failed!"
fi

echo ""
echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}Guardian OS MINIMAL ISO Build Complete!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo "ISO: ${OUTPUT_ISO}"
echo "Size: ${ISO_SIZE}"
echo ""
echo "This ISO has ONLY branding changes, no package modifications."
echo "If this installs successfully, the issue is in our package installation."
echo "If this ALSO fails, the issue is in the squashfs repack process."
