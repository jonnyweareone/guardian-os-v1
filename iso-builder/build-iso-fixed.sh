#!/bin/bash
# Guardian OS v1.1 ISO Build Script - FIXED
# Key fix: Keep ID=pop in os-release to maintain installer compatibility

set -e

GUARDIAN_VERSION="1.1.0"
WORK_DIR="/opt/guardian-iso-build"
BUILD_DIR="/opt/guardian-build"
POP_ISO_URL="https://iso.pop-os.org/24.04/amd64/intel/20/pop-os_24.04_amd64_intel_20.iso"

ISO_MOUNT="${WORK_DIR}/iso-mount"
SQUASHFS_DIR="${WORK_DIR}/squashfs"
NEW_ISO_DIR="${WORK_DIR}/new-iso"
OUTPUT_DIR="${WORK_DIR}/output"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${CYAN}[$(date +%H:%M:%S)]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }

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
echo "║            Guardian OS v1.1 ISO Build - FIXED                     ║"
echo "║            Maintains Pop!_OS installer compatibility              ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Install dependencies
log "Installing dependencies..."
apt-get update -qq
apt-get install -y squashfs-tools xorriso isolinux syslinux-common wget rsync

# Create directories
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

# Copy ISO structure
log "Copying ISO structure..."
rsync -a --exclude='casper/filesystem.squashfs' "${ISO_MOUNT}/" "${NEW_ISO_DIR}/"
success "ISO structure copied"

# Extract squashfs
log "Extracting squashfs (takes several minutes)..."
unsquashfs -d "${SQUASHFS_DIR}" "${ISO_MOUNT}/casper/filesystem.squashfs"
success "Squashfs extracted"

umount "${ISO_MOUNT}"

# ============================================
# Setup chroot
# ============================================
log "Setting up chroot..."
mount --bind /dev "${SQUASHFS_DIR}/dev"
mount --bind /dev/pts "${SQUASHFS_DIR}/dev/pts"
mount --bind /proc "${SQUASHFS_DIR}/proc"
mount --bind /sys "${SQUASHFS_DIR}/sys"
mount --bind /run "${SQUASHFS_DIR}/run"
cp /etc/resolv.conf "${SQUASHFS_DIR}/etc/resolv.conf"
success "Chroot ready"

# ============================================
# Install Guardian packages (if available)
# ============================================
if [ -d "${BUILD_DIR}/packages" ] && ls "${BUILD_DIR}/packages"/*.deb 1>/dev/null 2>&1; then
    log "Installing Guardian packages..."
    mkdir -p "${SQUASHFS_DIR}/tmp/guardian-packages"
    cp "${BUILD_DIR}/packages"/*.deb "${SQUASHFS_DIR}/tmp/guardian-packages/"
    
    chroot "${SQUASHFS_DIR}" /bin/bash << 'CHROOT_INSTALL'
#!/bin/bash
set -e
cd /tmp/guardian-packages

# Install packages
for pkg in *.deb; do
    echo "Installing $pkg..."
    dpkg -i "$pkg" 2>&1 || true
done

# Fix any dependency issues
apt-get install -f -y

# Enable guardian-daemon if installed
if [ -f /lib/systemd/system/guardian-daemon.service ]; then
    systemctl enable guardian-daemon 2>/dev/null || true
fi

# Cleanup
rm -rf /tmp/guardian-packages
apt-get clean
CHROOT_INSTALL
    success "Guardian packages installed"
else
    warn "No Guardian packages found - creating branding-only ISO"
fi

# ============================================
# Apply branding - KEEP ID=pop for compatibility!
# ============================================
log "Applying Guardian branding..."

# CRITICAL: Keep ID=pop so distinst/kernelstub work correctly!
cat > "${SQUASHFS_DIR}/etc/os-release" << OSREL
NAME="Guardian OS"
VERSION="${GUARDIAN_VERSION}"
ID=pop
ID_LIKE="ubuntu debian"
PRETTY_NAME="Guardian OS ${GUARDIAN_VERSION}"
VERSION_ID="24.04"
HOME_URL="https://gameguardian.ai"
SUPPORT_URL="https://gameguardian.ai/support"
BUG_REPORT_URL="https://github.com/jonnyweareone/guardian-os-v1/issues"
PRIVACY_POLICY_URL="https://gameguardian.ai/privacy"
VERSION_CODENAME=noble
UBUNTU_CODENAME=noble
LOGO=pop-logo
OSREL

# Update display files
echo "Guardian OS ${GUARDIAN_VERSION} \\n \\l" > "${SQUASHFS_DIR}/etc/issue"
echo "Guardian OS ${GUARDIAN_VERSION}" > "${SQUASHFS_DIR}/etc/issue.net"

# Keep Pop in lsb-release for package compatibility
cat > "${SQUASHFS_DIR}/etc/lsb-release" << LSB
DISTRIB_ID=Pop
DISTRIB_RELEASE=24.04
DISTRIB_CODENAME=noble
DISTRIB_DESCRIPTION="Guardian OS ${GUARDIAN_VERSION}"
LSB

success "Branding applied (ID=pop preserved for installer compatibility)"

# ============================================
# Unmount chroot
# ============================================
log "Unmounting chroot..."
umount "${SQUASHFS_DIR}/run" 2>/dev/null || true
umount "${SQUASHFS_DIR}/sys" 2>/dev/null || true
umount "${SQUASHFS_DIR}/proc" 2>/dev/null || true
umount "${SQUASHFS_DIR}/dev/pts" 2>/dev/null || true
umount "${SQUASHFS_DIR}/dev" 2>/dev/null || true
success "Chroot unmounted"

# ============================================
# Repack squashfs
# ============================================
log "Repacking squashfs (takes several minutes)..."
rm -f "${NEW_ISO_DIR}/casper/filesystem.squashfs"
mksquashfs "${SQUASHFS_DIR}" "${NEW_ISO_DIR}/casper/filesystem.squashfs" \
    -comp xz -b 1M -Xdict-size 100%
success "Squashfs repacked"

# Update filesystem.size
du -sx --block-size=1 "${SQUASHFS_DIR}" | cut -f1 > "${NEW_ISO_DIR}/casper/filesystem.size"

# ============================================
# Update boot menu
# ============================================
log "Updating boot menu text..."

# Update GRUB config
if [ -f "${NEW_ISO_DIR}/boot/grub/grub.cfg" ]; then
    sed -i 's/Pop!_OS/Guardian OS/g' "${NEW_ISO_DIR}/boot/grub/grub.cfg"
    sed -i "s/Install Pop/Install Guardian OS/g" "${NEW_ISO_DIR}/boot/grub/grub.cfg"
fi

if [ -f "${NEW_ISO_DIR}/boot/grub/loopback.cfg" ]; then
    sed -i 's/Pop!_OS/Guardian OS/g' "${NEW_ISO_DIR}/boot/grub/loopback.cfg"
fi

# Update isolinux
for cfg in "${NEW_ISO_DIR}"/isolinux/*.cfg; do
    if [ -f "$cfg" ]; then
        sed -i 's/Pop!_OS/Guardian OS/g' "$cfg"
    fi
done

success "Boot menu updated"

# ============================================
# Update installer title
# ============================================
log "Updating installer branding..."

# The installer reads from casper and displays OS name
# We update the grub entries which control what appears during install

# Find and update any Pop!_OS references in boot configs
find "${NEW_ISO_DIR}/boot" -type f -name "*.cfg" -exec sed -i 's/Pop!_OS/Guardian OS/g' {} \;

success "Installer branding updated"

# ============================================
# Regenerate checksums
# ============================================
log "Regenerating checksums..."
cd "${NEW_ISO_DIR}"
rm -f md5sum.txt
find . -type f -not -name 'md5sum.txt' -not -path './isolinux/*' -print0 | \
    xargs -0 md5sum 2>/dev/null > md5sum.txt || true
success "Checksums generated"

# ============================================
# Create bootable ISO
# ============================================
log "Creating bootable ISO..."
OUTPUT_ISO="${OUTPUT_DIR}/guardian-os_${GUARDIAN_VERSION}_amd64.iso"

cd "${NEW_ISO_DIR}"

# Find MBR file
MBR_FILE=""
for path in /usr/lib/ISOLINUX/isohdpfx.bin /usr/lib/syslinux/mbr/isohdpfx.bin; do
    if [ -f "$path" ]; then
        MBR_FILE="$path"
        break
    fi
done

if [ -z "$MBR_FILE" ]; then
    warn "isohdpfx.bin not found, extracting from original ISO..."
    dd if="${WORK_DIR}/pop-os-base.iso" bs=512 count=1 of="${WORK_DIR}/isohdpfx.bin" 2>/dev/null
    MBR_FILE="${WORK_DIR}/isohdpfx.bin"
fi

# Create hybrid BIOS/UEFI ISO
xorriso -as mkisofs \
    -volid "Guardian OS ${GUARDIAN_VERSION}" \
    -isohybrid-mbr "${MBR_FILE}" \
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
    success "ISO created successfully!"
else
    error "ISO creation failed!"
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
echo ""
echo "Key fixes in this build:"
echo "  ✓ ID=pop preserved in /etc/os-release"
echo "  ✓ Installer compatibility maintained"
echo "  ✓ Guardian branding applied to display names"
echo ""
echo "Upload to Vultr or test with:"
echo "  scp ${OUTPUT_ISO} user@host:~/"
