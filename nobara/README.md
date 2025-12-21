# Guardian OS - Nobara Edition

This directory contains the Nobara Linux-based build of Guardian OS.

## Why Nobara?

Guardian OS is pivoting from Pop!_OS to Nobara Linux for several key reasons:

### Gaming Focus
- **Pre-configured for gaming** - Steam, Lutris, Proton-GE, NVIDIA drivers out of box
- **Perfect fit for "Game Guardian"** brand positioning
- **Optimized performance** - Gaming-focused kernel patches and mesa-vulkan-drivers

### Easier Customization
- **Calamares installer** - Python/QML modules are much easier to customize than Vala/distinst
- **Kickstart-based ISOs** - Simple to modify and rebuild
- **nobara-welcome** - Easy to replace with Guardian first-boot wizard

### Better Hardware Support
- **BIOS + EFI** - Proper support for both boot modes
- **NVIDIA included** - No separate driver installation needed
- **Offline install** - Works without network connection

### Modern Base
- **Fedora 41/42** - Current packages, good security updates
- **Active development** - GloriousEggroll maintains it well
- **Large community** - Well-tested by gaming community

## Directory Structure

```
nobara/
├── README.md                 # This file
├── kickstarts/              # Fedora kickstart files for ISO building
│   ├── guardian-gnome.ks    # GNOME desktop variant
│   └── guardian-kde.ks      # KDE desktop variant
├── calamares-modules/       # Custom Calamares installer modules
│   ├── guardianauth/        # Parent authentication module
│   ├── guardianchild/       # Child selection module
│   └── guardianwelcome/     # Welcome/branding module
├── packages/                # RPM spec files and sources
│   ├── guardian-daemon/     # Core protection daemon
│   ├── guardian-wizard/     # First-boot setup wizard
│   └── guardian-branding/   # OS branding package
└── branding/                # Logos, wallpapers, themes
    ├── logos/
    ├── wallpapers/
    └── plymouth/
```

## Build Process

### Prerequisites
- Fedora/Nobara build system
- mock (RPM build tool)
- lorax/livemedia-creator (ISO building)

### Building Packages
```bash
cd packages/guardian-daemon
rpmbuild -ba guardian-daemon.spec
```

### Building ISO
```bash
# Using livemedia-creator with kickstart
sudo livemedia-creator \
    --ks kickstarts/guardian-gnome.ks \
    --no-virt \
    --resultdir /var/tmp/guardian-iso \
    --project "Guardian OS" \
    --releasever 41
```

## Attribution

Guardian OS Nobara Edition is based on:
- **Nobara Linux** by Thomas Crider (GloriousEggroll) - https://nobaraproject.org
- **Fedora Linux** by the Fedora Project - https://fedoraproject.org
- **Calamares** installer framework - https://calamares.io

We thank these projects for their excellent work enabling family-safe gaming on Linux.

## Pop!_OS Version

The original Pop!_OS-based Guardian OS remains in the repository root for reference and potential rollback. The Nobara edition is a parallel development track.

## License

Guardian OS components are licensed under GPL v3 unless otherwise specified.
Nobara and Fedora components retain their original licenses.
