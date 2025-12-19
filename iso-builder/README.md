# Guardian OS ISO Builder

This directory contains the configuration and scripts for building Guardian OS ISOs based on Pop!_OS 24.04 COSMIC.

## Prerequisites

- Ubuntu 22.04+ or Pop!_OS 22.04+ build host
- 50GB+ free disk space
- Docker (recommended) or native build environment

## Quick Start

```bash
# Clone pop-os ISO builder
git clone https://github.com/AugustArnstad/pop-os-cosmic-iso-builder.git
cd pop-os-cosmic-iso-builder

# Apply Guardian OS customizations
cp -r ../guardian-os-v1/iso-builder/config/* .

# Build ISO
./build.sh
```

## Structure

```
iso-builder/
├── config/
│   ├── packages.list       # Additional packages to install
│   ├── remove-packages.list # Packages to remove
│   ├── hooks/              # Build-time customization scripts
│   └── overlay/            # Files to copy into ISO
├── scripts/
│   ├── build.sh            # Main build script
│   └── customize.sh        # Post-install customization
└── README.md
```

## Customizations

1. **Branding** - Guardian logos, wallpapers, themes
2. **Packages** - guardian-daemon, guardian-wizard, guardian-settings, guardian-store
3. **First Boot** - guardian-wizard autostart
4. **Default Settings** - COSMIC configuration tweaks
