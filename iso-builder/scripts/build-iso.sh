#!/bin/bash
# Guardian OS ISO Builder v2.0
# Remasters Pop!_OS 24.04 COSMIC with Guardian components
# 
# This script:
# 1. Downloads the official Pop!_OS 24.04 ISO
# 2. Extracts the filesystem
# 3. Injects Guardian packages and branding
# 4. Repacks into a new ISO

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Configuration
POP_VERSION="24.04"
POP_ISO_URL="https://iso.pop-os.org/24.04/amd64/intel/20/pop-os_24.04_amd64_intel_20.iso"
GUARDIAN_VERSION="1.0.0"
GITHUB_RELEASE_URL="https://github.com/jonnyweareone/guardian-os-v1/releases/download/v${GUARDIAN_VERSION}"

# Directories
WORK_DIR="/opt/guardian-iso-build"
ISO_MOUNT="${WORK_DIR}/iso-mount"
SQUASHFS_DIR="${WORK_DIR}/squashfs"
NEW_ISO_DIR="${WORK_DIR}/new-iso"
OUTPUT_DIR="${WORK_DIR}/output"

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

print_banner() {
    echo -e "${CYAN}"
    cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║   ██████╗ ██╗   ██╗ █████╗ ██████╗ ██████╗ ██╗ █████╗ ███╗   ██╗  ║
║  ██╔════╝ ██║   ██║██╔══██╗██╔══██╗██╔══██╗██║██╔══██╗████╗  ██║  ║
║  ██║  ███╗██║   ██║███████║██████╔╝██║  ██║██║███████║██╔██╗ ██║  ║
║  ██║   ██║██║   ██║██╔══██║██╔══██╗██║  ██║██║██╔══██║██║╚██╗██║  ║
║  ╚██████╔╝╚██████╔╝██║  ██║██║  ██║██████╔╝██║██║  ██║██║ ╚████║  ║
║   ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝  ║
║                         OS  v1.0.0                                ║
║                                                                   ║
║            AI Powered Protection For Families                     ║
║            Built on Pop!_OS 24.04 LTS + COSMIC                    ║
╚═══════════════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

check_root() {
    if [ "$EUID" -ne 0 ]; then
        error "Please run as root: sudo $0"
    fi
}

check_deps() {
    log "Checking dependencies..."
    
    local deps=(wget unsquashfs mksquashfs xorriso mount umount)
    local missing=()
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &>/dev/null; then
            missing+=("$dep")
        fi
    done
    
    if [ ${#missing[@]} -ne 0 ]; then
        log "Installing missing dependencies..."
        apt-get update -qq
        apt-get install -y squashfs-tools xorriso wget curl
    fi
    
    success "All dependencies ready"
}

download_pop_iso() {
    log "Downloading Pop!_OS ${POP_VERSION} ISO..."
    
    local iso_file="${WORK_DIR}/pop-os-base.iso"
    
    if [ -f "$iso_file" ]; then
        log "ISO already downloaded, verifying..."
        if file "$iso_file" | grep -q "ISO 9660"; then
            success "Using cached ISO"
            return
        else
            warn "Cached ISO corrupted, re-downloading..."
            rm -f "$iso_file"
        fi
    fi
    
    mkdir -p "$WORK_DIR"
    wget --progress=bar:force:noscroll -O "$iso_file" "$POP_ISO_URL"
    
    if [ ! -f "$iso_file" ]; then
        error "Failed to download Pop!_OS ISO"
    fi
    
    success "Pop!_OS ISO downloaded ($(du -h "$iso_file" | cut -f1))"
}

download_guardian_packages() {
    log "Downloading Guardian component packages..."
    
    local pkg_dir="${WORK_DIR}/packages"
    mkdir -p "$pkg_dir"
    
    local packages=(guardian-daemon guardian-wizard)
    
    for pkg in "${packages[@]}"; do
        local deb="${pkg}_${GUARDIAN_VERSION}_amd64.deb"
        if [ ! -f "${pkg_dir}/${deb}" ]; then
            log "  Downloading ${pkg}..."
            wget -q "${GITHUB_RELEASE_URL}/${deb}" -O "${pkg_dir}/${deb}" || {
                warn "Could not download ${pkg} from release"
            }
        else
            success "  ${pkg} already downloaded"
        fi
    done
    
    # List what we got
    log "Available packages:"
    ls -la "${pkg_dir}/"*.deb 2>/dev/null || warn "No .deb packages found"
}

extract_iso() {
    log "Extracting Pop!_OS ISO..."
    
    local iso_file="${WORK_DIR}/pop-os-base.iso"
    
    # Create directories
    mkdir -p "$ISO_MOUNT" "$SQUASHFS_DIR" "$NEW_ISO_DIR"
    
    # Unmount if already mounted
    umount "$ISO_MOUNT" 2>/dev/null || true
    
    # Mount ISO
    mount -o loop,ro "$iso_file" "$ISO_MOUNT"
    success "ISO mounted"
    
    # Copy ISO contents (excluding squashfs)
    log "Copying ISO structure..."
    rsync -a --exclude='casper/filesystem.squashfs' "$ISO_MOUNT/" "$NEW_ISO_DIR/"
    success "ISO structure copied"
    
    # Extract squashfs
    log "Extracting filesystem (this takes a while)..."
    rm -rf "$SQUASHFS_DIR"
    unsquashfs -d "$SQUASHFS_DIR" "$ISO_MOUNT/casper/filesystem.squashfs"
    success "Filesystem extracted"
    
    # Unmount ISO
    umount "$ISO_MOUNT"
}

inject_guardian() {
    log "Injecting Guardian OS components..."
    
    local pkg_dir="${WORK_DIR}/packages"
    local root="$SQUASHFS_DIR"
    
    # Copy packages into chroot
    mkdir -p "${root}/tmp/guardian-packages"
    cp "${pkg_dir}"/*.deb "${root}/tmp/guardian-packages/" 2>/dev/null || true
    
    # Mount required filesystems for chroot
    mount --bind /dev "${root}/dev"
    mount --bind /dev/pts "${root}/dev/pts"
    mount --bind /proc "${root}/proc"
    mount --bind /sys "${root}/sys"
    mount --bind /run "${root}/run"
    
    # Resolve DNS in chroot
    cp /etc/resolv.conf "${root}/etc/resolv.conf"
    
    # Install packages inside chroot
    log "Installing Guardian packages in chroot..."
    chroot "$root" /bin/bash << 'CHROOT_SCRIPT'
#!/bin/bash
set -e

# Install any .deb packages
if ls /tmp/guardian-packages/*.deb 1>/dev/null 2>&1; then
    dpkg -i /tmp/guardian-packages/*.deb || apt-get install -f -y
fi

# Enable guardian-daemon service if it exists
if [ -f /lib/systemd/system/guardian-daemon.service ]; then
    systemctl enable guardian-daemon || true
fi

# Clean up
rm -rf /tmp/guardian-packages
apt-get clean
rm -rf /var/lib/apt/lists/*

echo "Guardian components installed"
CHROOT_SCRIPT
    
    success "Guardian packages installed"
    
    # Unmount chroot filesystems
    umount "${root}/run" 2>/dev/null || true
    umount "${root}/sys" 2>/dev/null || true
    umount "${root}/proc" 2>/dev/null || true
    umount "${root}/dev/pts" 2>/dev/null || true
    umount "${root}/dev" 2>/dev/null || true
}

apply_branding() {
    log "Applying Guardian OS branding..."
    
    local root="$SQUASHFS_DIR"
    local branding="${PROJECT_ROOT}/branding"
    
    # Copy wallpaper
    if [ -f "${branding}/wallpapers/guardian-wallpaper.png" ]; then
        cp "${branding}/wallpapers/guardian-wallpaper.png" "${root}/usr/share/backgrounds/"
        success "Wallpaper installed"
    fi
    
    # Copy logo
    if [ -d "${branding}/logo" ]; then
        mkdir -p "${root}/usr/share/icons/guardian"
        cp "${branding}/logo"/*.png "${root}/usr/share/icons/guardian/" 2>/dev/null || true
        success "Logo installed"
    fi
    
    # Update OS release info
    cat > "${root}/etc/os-release" << EOF
NAME="Guardian OS"
VERSION="${GUARDIAN_VERSION}"
ID=guardian
ID_LIKE=ubuntu pop
PRETTY_NAME="Guardian OS ${GUARDIAN_VERSION}"
VERSION_ID="${GUARDIAN_VERSION}"
HOME_URL="https://gameguardian.ai"
SUPPORT_URL="https://gameguardian.ai/support"
BUG_REPORT_URL="https://github.com/jonnyweareone/guardian-os-v1/issues"
PRIVACY_POLICY_URL="https://gameguardian.ai/privacy"
VERSION_CODENAME=guardian
UBUNTU_CODENAME=noble
EOF
    success "OS branding updated"
    
    # Create first-boot wizard autostart
    mkdir -p "${root}/etc/skel/.config/autostart"
    cat > "${root}/etc/skel/.config/autostart/guardian-wizard.desktop" << EOF
[Desktop Entry]
Type=Application
Name=Guardian OS Setup
Comment=Configure your family-safe system
Exec=/usr/bin/guardian-wizard
Icon=guardian
Terminal=false
Categories=System;Settings;
X-GNOME-Autostart-enabled=true
OnlyShowIn=COSMIC;GNOME;
EOF
    success "First-boot wizard configured"
}

configure_live_session() {
    log "Configuring live session for COSMIC..."
    
    local root="$SQUASHFS_DIR"
    
    # COSMIC uses greetd, not GDM
    # For live session, we configure auto-login to bypass cosmic-greeter issues in VMs
    mkdir -p "${root}/etc/greetd"
    
    # Create live session auto-login config
    cat > "${root}/etc/greetd/config.toml" << 'EOF'
[terminal]
vt = 1

[default_session]
command = "cosmic-session"
user = "liveuser"
EOF
    
    # Create cosmic-greeter config for live session
    cat > "${root}/etc/greetd/cosmic-greeter.toml" << 'EOF'
# Live session - auto login to bypass greeter rendering issues in VMs
[initial_session]
command = "cosmic-session"
user = "liveuser"
EOF
    success "Greetd live session configured"
    
    # Create liveuser in chroot
    log "Creating live session user..."
    chroot "$root" /bin/bash << 'LIVEUSER_SCRIPT'
#!/bin/bash
# Create liveuser for live session
if ! id liveuser &>/dev/null; then
    useradd -m -s /bin/bash -G sudo,adm,cdrom,dip,plugdev,lpadmin liveuser
    echo "liveuser:live" | chpasswd
    # Allow passwordless sudo for live session
    echo "liveuser ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/liveuser
    chmod 440 /etc/sudoers.d/liveuser
fi
LIVEUSER_SCRIPT
    success "Live session user created"
    
    # Add kernel parameters for better VM compatibility
    log "Adding VM-friendly kernel parameters..."
    
    # Update GRUB for live session
    if [ -f "${root}/etc/default/grub" ]; then
        sed -i 's/GRUB_CMDLINE_LINUX_DEFAULT="[^"]*"/GRUB_CMDLINE_LINUX_DEFAULT="quiet splash nomodeset"/' "${root}/etc/default/grub"
    fi
    
    # Also update isolinux/grub boot entries in the ISO
    if [ -f "${NEW_ISO_DIR}/boot/grub/grub.cfg" ]; then
        sed -i 's/quiet splash/quiet splash nomodeset/g' "${NEW_ISO_DIR}/boot/grub/grub.cfg"
        success "GRUB boot parameters updated"
    fi
    
    if [ -f "${NEW_ISO_DIR}/isolinux/isolinux.cfg" ]; then
        sed -i 's/quiet splash/quiet splash nomodeset/g' "${NEW_ISO_DIR}/isolinux/isolinux.cfg"
        success "ISOLINUX boot parameters updated"
    fi
    
    # Update casper boot parameters in all config files
    for cfg in "${NEW_ISO_DIR}"/boot/grub/*.cfg "${NEW_ISO_DIR}"/EFI/boot/*.cfg "${NEW_ISO_DIR}"/EFI/BOOT/*.cfg; do
        if [ -f "$cfg" ]; then
            sed -i 's/quiet splash/quiet splash nomodeset/g' "$cfg" 2>/dev/null || true
        fi
    done
    
    success "Live session configuration complete"
}

repack_squashfs() {
    log "Repacking filesystem (this takes a while)..."
    
    local squashfs_out="${NEW_ISO_DIR}/casper/filesystem.squashfs"
    
    rm -f "$squashfs_out"
    mksquashfs "$SQUASHFS_DIR" "$squashfs_out" \
        -comp xz \
        -b 1M \
        -Xdict-size 100% \
        -no-recovery
    
    # Update filesystem size
    du -sx --block-size=1 "$SQUASHFS_DIR" | cut -f1 > "${NEW_ISO_DIR}/casper/filesystem.size"
    
    success "Filesystem repacked ($(du -h "$squashfs_out" | cut -f1))"
}

create_iso() {
    log "Creating Guardian OS ISO..."
    
    mkdir -p "$OUTPUT_DIR"
    local output_iso="${OUTPUT_DIR}/guardian-os_${GUARDIAN_VERSION}_amd64.iso"
    
    # Generate MD5 checksums
    cd "$NEW_ISO_DIR"
    find . -type f -not -name 'md5sum.txt' -print0 | xargs -0 md5sum > md5sum.txt
    
    # Create ISO
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
        -o "$output_iso" \
        "$NEW_ISO_DIR" 2>/dev/null || {
            # Fallback for different ISO structure
            xorriso -as mkisofs \
                -volid "Guardian OS ${GUARDIAN_VERSION}" \
                -o "$output_iso" \
                "$NEW_ISO_DIR"
        }
    
    success "ISO created: $output_iso"
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                    🎉 BUILD COMPLETE!                             ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "Output: ${CYAN}${output_iso}${NC}"
    echo -e "Size:   ${CYAN}$(du -h "$output_iso" | cut -f1)${NC}"
    echo ""
    echo "To test: Write to USB with 'dd' or use in a VM"
    echo "  dd if=${output_iso} of=/dev/sdX bs=4M status=progress"
}

cleanup() {
    log "Cleaning up..."
    umount "$ISO_MOUNT" 2>/dev/null || true
    umount "${SQUASHFS_DIR}/run" 2>/dev/null || true
    umount "${SQUASHFS_DIR}/sys" 2>/dev/null || true
    umount "${SQUASHFS_DIR}/proc" 2>/dev/null || true
    umount "${SQUASHFS_DIR}/dev/pts" 2>/dev/null || true
    umount "${SQUASHFS_DIR}/dev" 2>/dev/null || true
}

# Main execution
main() {
    print_banner
    check_root
    
    trap cleanup EXIT
    
    check_deps
    download_pop_iso
    download_guardian_packages
    extract_iso
    inject_guardian
    apply_branding
    configure_live_session
    repack_squashfs
    create_iso
}

# Run
main "$@"
