#!/bin/bash
# Guardian OS - Post-Install Hook
# Final configuration after all packages installed

set -e

echo "🛡️  Guardian OS Post-Install Hook"

# Enable guardian-daemon systemd service
echo "  Enabling guardian-daemon service..."
systemctl enable guardian-daemon.service || true

# Configure guardian-wizard to run on first login
echo "  Configuring first-boot wizard..."
mkdir -p /etc/skel/.config/autostart
cat > /etc/skel/.config/autostart/guardian-wizard.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Guardian Setup
Comment=First-time Guardian OS setup wizard
Exec=/usr/bin/guardian-wizard
Icon=guardian-shield
Terminal=false
Categories=System;Settings;
X-GNOME-Autostart-enabled=true
OnlyShowIn=COSMIC;GNOME;
EOF

# Create guardian-wizard completion flag check script
cat > /etc/profile.d/guardian-check.sh << 'EOF'
# Check if Guardian OS setup is complete
if [ ! -f "$HOME/.config/guardian/setup-complete" ]; then
    # First login - wizard will run via autostart
    :
fi
EOF

# Configure Flatpak for guardian-store
echo "  Configuring Flatpak..."
flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo || true

# Set Flathub as system-wide
flatpak remote-modify --system flathub || true

# OS Release branding
echo "  Updating OS release info..."
cat > /etc/os-release << 'EOF'
NAME="Guardian OS"
VERSION="1.0"
ID=guardian
ID_LIKE=ubuntu pop
PRETTY_NAME="Guardian OS 1.0"
VERSION_ID="1.0"
HOME_URL="https://gameguardian.ai/"
SUPPORT_URL="https://gameguardian.ai/support"
BUG_REPORT_URL="https://github.com/guardian-os/guardian-os/issues"
PRIVACY_POLICY_URL="https://gameguardian.ai/privacy"
VERSION_CODENAME=shield
UBUNTU_CODENAME=noble
LOGO=guardian-shield
EOF

# LSB release
cat > /etc/lsb-release << 'EOF'
DISTRIB_ID=GuardianOS
DISTRIB_RELEASE=1.0
DISTRIB_CODENAME=shield
DISTRIB_DESCRIPTION="Guardian OS 1.0 Shield"
EOF

# Update initramfs with new Plymouth theme
echo "  Updating initramfs..."
update-initramfs -u

echo "  ✓ Post-install complete"
echo ""
echo "🛡️  Guardian OS customization complete!"
