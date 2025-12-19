#!/bin/bash
# Guardian OS ISO Builder
# Builds a custom Pop!_OS COSMIC-based ISO with Guardian components

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
BUILD_DIR="${PROJECT_ROOT}/iso-build"
OUTPUT_DIR="${PROJECT_ROOT}/iso-output"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║              🛡️  Guardian OS ISO Builder                      ║"
echo "║                                                               ║"
echo "║         AI Powered Protection For Families                    ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Check dependencies
check_deps() {
    echo -e "${CYAN}Checking dependencies...${NC}"
    
    local missing=()
    
    for cmd in docker git curl wget; do
        if ! command -v "$cmd" &> /dev/null; then
            missing+=("$cmd")
        fi
    done
    
    if [ ${#missing[@]} -ne 0 ]; then
        echo -e "${RED}Missing dependencies: ${missing[*]}${NC}"
        echo "Please install them and try again."
        exit 1
    fi
    
    echo -e "${GREEN}✓ All dependencies found${NC}"
}

# Clone or update Pop!_OS ISO builder
setup_builder() {
    echo -e "${CYAN}Setting up ISO builder...${NC}"
    
    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"
    
    if [ ! -d "pop-os-cosmic-iso-builder" ]; then
        echo "  Cloning Pop!_OS COSMIC ISO builder..."
        git clone https://github.com/AugustArnstad/pop-os-cosmic-iso-builder.git
    else
        echo "  Updating existing builder..."
        cd pop-os-cosmic-iso-builder
        git pull || true
        cd ..
    fi
    
    echo -e "${GREEN}✓ Builder ready${NC}"
}

# Build Guardian component .deb packages
build_packages() {
    echo -e "${CYAN}Building Guardian component packages...${NC}"
    
    local pkg_dir="${BUILD_DIR}/packages"
    mkdir -p "$pkg_dir"
    
    # Build each component
    for component in guardian-daemon guardian-wizard guardian-settings guardian-store; do
        echo "  Building ${component}..."
        
        local src="${PROJECT_ROOT}/guardian-components/${component}"
        
        if [ -d "$src" ]; then
            cd "$src"
            
            # Build release binary
            cargo build --release 2>/dev/null || {
                echo -e "${RED}  ✗ Failed to build ${component}${NC}"
                continue
            }
            
            # Create .deb package structure
            local deb_dir="${pkg_dir}/${component}_1.0.0_amd64"
            mkdir -p "${deb_dir}/DEBIAN"
            mkdir -p "${deb_dir}/usr/bin"
            mkdir -p "${deb_dir}/usr/share/applications"
            mkdir -p "${deb_dir}/usr/share/icons/hicolor/scalable/apps"
            
            # Copy binary
            cp "target/release/${component}" "${deb_dir}/usr/bin/" 2>/dev/null || \
            cp "target/release/${component//-/_}" "${deb_dir}/usr/bin/${component}" 2>/dev/null || true
            
            # Create control file
            cat > "${deb_dir}/DEBIAN/control" << EOF
Package: ${component}
Version: 1.0.0
Section: misc
Priority: optional
Architecture: amd64
Depends: libssl3, libsqlite3-0
Maintainer: Guardian OS Team <support@gameguardian.ai>
Description: ${component} - Guardian OS component
 Part of Guardian OS family safety system.
EOF
            
            # Create .desktop file for GUI apps
            if [[ "$component" =~ ^(guardian-wizard|guardian-settings|guardian-store)$ ]]; then
                local name="${component#guardian-}"
                name="${name^}" # Capitalize
                
                cat > "${deb_dir}/usr/share/applications/${component}.desktop" << EOF
[Desktop Entry]
Type=Application
Name=Guardian ${name}
Comment=Guardian OS ${name}
Exec=/usr/bin/${component}
Icon=guardian-shield
Terminal=false
Categories=System;Settings;
EOF
            fi
            
            # Build .deb
            dpkg-deb --build "$deb_dir" 2>/dev/null || {
                echo "  Note: dpkg-deb not available, skipping .deb creation"
            }
            
            echo -e "${GREEN}  ✓ ${component}${NC}"
        else
            echo -e "${RED}  ✗ ${component} not found${NC}"
        fi
    done
}

# Convert branding assets to PNG
convert_branding() {
    echo -e "${CYAN}Converting branding assets...${NC}"
    
    local branding="${PROJECT_ROOT}/branding"
    
    if [ -f "${branding}/convert-assets.sh" ]; then
        cd "$branding"
        chmod +x convert-assets.sh
        ./convert-assets.sh || echo "  Note: Asset conversion may require ImageMagick"
    fi
    
    echo -e "${GREEN}✓ Branding ready${NC}"
}

# Apply Guardian customizations to ISO builder
apply_customizations() {
    echo -e "${CYAN}Applying Guardian customizations...${NC}"
    
    local builder="${BUILD_DIR}/pop-os-cosmic-iso-builder"
    local config="${PROJECT_ROOT}/iso-builder/config"
    
    # Copy overlay files
    if [ -d "${config}/overlay" ]; then
        cp -r "${config}/overlay"/* "${builder}/" 2>/dev/null || true
    fi
    
    # Copy hooks
    mkdir -p "${builder}/hooks"
    cp "${config}/hooks"/*.sh "${builder}/hooks/" 2>/dev/null || true
    chmod +x "${builder}/hooks"/*.sh 2>/dev/null || true
    
    # Copy package list
    cp "${config}/packages.list" "${builder}/" 2>/dev/null || true
    
    # Copy branding
    mkdir -p "${builder}/branding"
    cp -r "${PROJECT_ROOT}/branding"/* "${builder}/branding/" 2>/dev/null || true
    
    # Copy .deb packages
    mkdir -p "${builder}/packages"
    cp "${BUILD_DIR}/packages"/*.deb "${builder}/packages/" 2>/dev/null || true
    
    echo -e "${GREEN}✓ Customizations applied${NC}"
}

# Build the ISO
build_iso() {
    echo -e "${CYAN}Building ISO...${NC}"
    echo ""
    echo "This will take 30-60 minutes depending on your system."
    echo ""
    
    local builder="${BUILD_DIR}/pop-os-cosmic-iso-builder"
    cd "$builder"
    
    # Run the build
    if [ -f "build.sh" ]; then
        chmod +x build.sh
        ./build.sh
    elif [ -f "Makefile" ]; then
        make
    else
        echo -e "${RED}No build script found in ISO builder${NC}"
        exit 1
    fi
    
    # Copy output
    mkdir -p "$OUTPUT_DIR"
    cp *.iso "$OUTPUT_DIR/" 2>/dev/null || true
    
    echo -e "${GREEN}✓ ISO built successfully${NC}"
    echo ""
    echo "Output: ${OUTPUT_DIR}/"
    ls -lh "$OUTPUT_DIR"/*.iso 2>/dev/null || true
}

# Main
main() {
    check_deps
    setup_builder
    build_packages
    convert_branding
    apply_customizations
    
    echo ""
    echo -e "${CYAN}Ready to build ISO.${NC}"
    echo ""
    read -p "Continue with ISO build? [y/N] " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        build_iso
    else
        echo "Build preparation complete. Run ./build-iso.sh again to build."
    fi
}

main "$@"
