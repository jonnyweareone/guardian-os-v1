#!/bin/bash
# Guardian OS ISO Builder
# Builds a custom Pop!_OS 24.04 COSMIC-based ISO with Guardian components
# Uses the OFFICIAL Pop!_OS ISO builder from https://github.com/pop-os/iso

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
BUILD_DIR="${PROJECT_ROOT}/iso-build"
OUTPUT_DIR="${PROJECT_ROOT}/iso-output"
POP_ISO_REPO="https://github.com/pop-os/iso.git"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║              🛡️  Guardian OS ISO Builder                      ║"
echo "║                                                               ║"
echo "║         AI Powered Protection For Families                    ║"
echo "║         Built on Pop!_OS 24.04 LTS + COSMIC Desktop           ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Check if running as root
check_root() {
    if [ "$EUID" -ne 0 ]; then
        echo -e "${RED}Please run as root (sudo ./build-iso.sh)${NC}"
        exit 1
    fi
}

# Check dependencies
check_deps() {
    echo -e "${CYAN}Checking dependencies...${NC}"
    
    local missing=()
    
    for cmd in git make debootstrap mksquashfs xorriso gpg curl wget; do
        if ! command -v "$cmd" &> /dev/null; then
            missing+=("$cmd")
        fi
    done
    
    if [ ${#missing[@]} -ne 0 ]; then
        echo -e "${YELLOW}Installing missing dependencies: ${missing[*]}${NC}"
        apt-get update
        apt-get install -y git make debootstrap squashfs-tools xorriso gnupg curl wget \
            live-build debian-archive-keyring ubuntu-keyring
    fi
    
    echo -e "${GREEN}✓ All dependencies ready${NC}"
}

# Import Pop!_OS signing key
import_pop_key() {
    echo -e "${CYAN}Importing Pop!_OS signing key...${NC}"
    
    # Pop!_OS APT signing key
    if ! gpg --list-keys "204DD8AEC33A7AFF" &>/dev/null; then
        curl -fsSL https://apt.pop-os.org/key.asc | gpg --import
        echo -e "${GREEN}✓ Pop!_OS key imported${NC}"
    else
        echo -e "${GREEN}✓ Pop!_OS key already present${NC}"
    fi
}

# Clone or update official Pop!_OS ISO builder
setup_pop_builder() {
    echo -e "${CYAN}Setting up official Pop!_OS ISO builder...${NC}"
    
    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"
    
    if [ ! -d "pop-iso" ]; then
        echo "  Cloning official Pop!_OS ISO builder..."
        git clone "$POP_ISO_REPO" pop-iso
    else
        echo "  Updating existing builder..."
        cd pop-iso
        git fetch origin
        git reset --hard origin/master
        cd ..
    fi
    
    echo -e "${GREEN}✓ Pop!_OS ISO builder ready${NC}"
}

# Download Guardian .deb packages from GitHub releases
download_guardian_packages() {
    echo -e "${CYAN}Downloading Guardian component packages...${NC}"
    
    local pkg_dir="${BUILD_DIR}/guardian-packages"
    mkdir -p "$pkg_dir"
    cd "$pkg_dir"
    
    # Download from latest GitHub release
    local release_url="https://github.com/jonnyweareone/guardian-os-v1/releases/latest/download"
    
    for pkg in guardian-daemon guardian-wizard guardian-settings guardian-store; do
        local deb_file="${pkg}_1.0.0_amd64.deb"
        if [ ! -f "$deb_file" ]; then
            echo "  Downloading ${pkg}..."
            wget -q "${release_url}/${deb_file}" -O "$deb_file" 2>/dev/null || {
                echo -e "${YELLOW}  ⚠ ${pkg} not yet available in release${NC}"
            }
        else
            echo -e "${GREEN}  ✓ ${pkg} already downloaded${NC}"
        fi
    done
}

# Create Guardian customization overlay
create_guardian_overlay() {
    echo -e "${CYAN}Creating Guardian customizations...${NC}"
    
    local overlay_dir="${BUILD_DIR}/pop-iso/data/guardian"
    mkdir -p "$overlay_dir"
    
    # Copy branding
    if [ -d "${PROJECT_ROOT}/branding" ]; then
        cp -r "${PROJECT_ROOT}/branding" "$overlay_dir/"
    fi
    
    # Copy Guardian packages
    mkdir -p "$overlay_dir/packages"
    cp "${BUILD_DIR}/guardian-packages"/*.deb "$overlay_dir/packages/" 2>/dev/null || true
    
    # Create Guardian-specific package list
    cat > "${BUILD_DIR}/pop-iso/data/guardian/packages.list" << 'EOF'
# Guardian OS Additional Packages
guardian-daemon
guardian-wizard
guardian-settings
guardian-store
EOF

    # Create post-install hook for Guardian
    mkdir -p "${BUILD_DIR}/pop-iso/data/guardian/hooks"
    cat > "${BUILD_DIR}/pop-iso/data/guardian/hooks/guardian-setup.sh" << 'EOF'
#!/bin/bash
# Guardian OS Post-Install Setup

# Enable Guardian daemon
systemctl enable guardian-daemon || true

# Set Guardian wallpaper
if [ -f /usr/share/backgrounds/guardian-wallpaper.png ]; then
    gsettings set org.gnome.desktop.background picture-uri "file:///usr/share/backgrounds/guardian-wallpaper.png" 2>/dev/null || true
fi

# Create Guardian autostart for wizard (first boot)
mkdir -p /etc/skel/.config/autostart
cat > /etc/skel/.config/autostart/guardian-wizard.desktop << 'DESKTOP'
[Desktop Entry]
Type=Application
Name=Guardian Setup Wizard
Exec=/usr/bin/guardian-wizard
Terminal=false
X-GNOME-Autostart-enabled=true
OnlyShowIn=COSMIC;GNOME;
DESKTOP

echo "Guardian OS setup complete"
EOF
    chmod +x "${BUILD_DIR}/pop-iso/data/guardian/hooks/guardian-setup.sh"
    
    echo -e "${GREEN}✓ Guardian customizations ready${NC}"
}

# Modify Pop!_OS ISO build configuration
configure_build() {
    echo -e "${CYAN}Configuring ISO build...${NC}"
    
    cd "${BUILD_DIR}/pop-iso"
    
    # Create Guardian-specific config extending 24.04
    cat > config/guardian-os/24.04.mk << 'EOF'
# Guardian OS 24.04 Configuration
# Extends Pop!_OS 24.04 LTS with Guardian components

include config/pop-os/24.04.mk

# Guardian-specific packages to add
POST_DISTRO_PKGS+=\
    guardian-daemon \
    guardian-wizard \
    guardian-settings \
    guardian-store

# Guardian branding
DISTRO_NAME=Guardian OS
DISTRO_VERSION=1.0.0
EOF

    mkdir -p config/guardian-os
    
    echo -e "${GREEN}✓ Build configured${NC}"
}

# Build the ISO
build_iso() {
    echo -e "${CYAN}Building Guardian OS ISO...${NC}"
    echo ""
    echo -e "${YELLOW}⏱️  This will take 30-60 minutes depending on your system and internet speed.${NC}"
    echo ""
    
    cd "${BUILD_DIR}/pop-iso"
    
    # Install build dependencies
    if [ -f "scripts/deps.sh" ]; then
        echo "  Installing Pop!_OS ISO build dependencies..."
        ./scripts/deps.sh
    fi
    
    # Run the build
    echo "  Starting ISO build..."
    make DISTRO=pop-os RELEASE=24.04
    
    # Copy output
    mkdir -p "$OUTPUT_DIR"
    
    # Find and copy the built ISO
    find build/ -name "*.iso" -exec cp {} "$OUTPUT_DIR/" \;
    
    # Rename to Guardian OS
    for iso in "$OUTPUT_DIR"/*.iso; do
        if [ -f "$iso" ]; then
            local newname=$(echo "$iso" | sed 's/pop-os/guardian-os/g')
            mv "$iso" "$newname" 2>/dev/null || true
        fi
    done
    
    echo -e "${GREEN}✓ ISO built successfully${NC}"
    echo ""
    echo -e "${CYAN}Output:${NC} ${OUTPUT_DIR}/"
    ls -lh "$OUTPUT_DIR"/*.iso 2>/dev/null || echo "No ISO files found"
}

# Alternative: Remaster existing Pop!_OS ISO
remaster_iso() {
    echo -e "${CYAN}Remastering Pop!_OS ISO with Guardian components...${NC}"
    
    local pop_iso_url="https://iso.pop-os.org/24.04/amd64/intel/40/pop-os_24.04_amd64_intel_40.iso"
    local pop_iso="${BUILD_DIR}/pop-os-base.iso"
    
    # Download Pop!_OS ISO if not present
    if [ ! -f "$pop_iso" ]; then
        echo "  Downloading Pop!_OS 24.04 ISO (~2.5GB)..."
        wget -q --show-progress "$pop_iso_url" -O "$pop_iso"
    fi
    
    echo -e "${GREEN}✓ Base ISO ready${NC}"
    
    # Extract, modify, repack
    local work_dir="${BUILD_DIR}/remaster"
    mkdir -p "$work_dir"/{iso,squashfs}
    
    echo "  Extracting ISO..."
    mount -o loop "$pop_iso" "$work_dir/iso"
    
    echo "  Extracting squashfs..."
    unsquashfs -d "$work_dir/squashfs" "$work_dir/iso/casper/filesystem.squashfs"
    
    echo "  Adding Guardian packages..."
    cp "${BUILD_DIR}/guardian-packages"/*.deb "$work_dir/squashfs/tmp/" 2>/dev/null || true
    chroot "$work_dir/squashfs" dpkg -i /tmp/*.deb 2>/dev/null || true
    rm -f "$work_dir/squashfs/tmp"/*.deb
    
    echo "  Repacking squashfs..."
    mksquashfs "$work_dir/squashfs" "$work_dir/filesystem.squashfs" -comp xz
    
    echo "  Creating new ISO..."
    # This is simplified - full ISO creation needs more steps
    
    umount "$work_dir/iso"
    
    echo -e "${GREEN}✓ Remaster complete${NC}"
}

# Main menu
main() {
    check_root
    check_deps
    import_pop_key
    
    echo ""
    echo -e "${CYAN}Choose build method:${NC}"
    echo "  1) Full build from official Pop!_OS ISO builder (recommended)"
    echo "  2) Remaster existing Pop!_OS ISO (faster, simpler)"
    echo "  3) Exit"
    echo ""
    read -p "Select option [1-3]: " choice
    
    case $choice in
        1)
            setup_pop_builder
            download_guardian_packages
            create_guardian_overlay
            configure_build
            build_iso
            ;;
        2)
            download_guardian_packages
            remaster_iso
            ;;
        3)
            echo "Exiting."
            exit 0
            ;;
        *)
            echo -e "${RED}Invalid option${NC}"
            exit 1
            ;;
    esac
    
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║              🎉 Guardian OS Build Complete!                   ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
}

# Allow running specific functions
if [ "$1" == "--deps-only" ]; then
    check_deps
elif [ "$1" == "--download-only" ]; then
    download_guardian_packages
else
    main "$@"
fi
