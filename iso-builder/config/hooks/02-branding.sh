#!/bin/bash
# Guardian OS - Branding Hook
# Installs Guardian branding (Plymouth, GRUB, wallpapers)

set -e

echo "🛡️  Guardian OS Branding Hook"

# Plymouth theme
echo "  Installing Plymouth theme..."
mkdir -p /usr/share/plymouth/themes/guardian
cp /tmp/guardian-branding/plymouth/* /usr/share/plymouth/themes/guardian/

# Set as default Plymouth theme
update-alternatives --install /usr/share/plymouth/themes/default.plymouth default.plymouth \
    /usr/share/plymouth/themes/guardian/guardian.plymouth 200
update-alternatives --set default.plymouth /usr/share/plymouth/themes/guardian/guardian.plymouth

# GRUB theme
echo "  Installing GRUB theme..."
mkdir -p /boot/grub/themes/guardian
cp -r /tmp/guardian-branding/grub/* /boot/grub/themes/guardian/

# Update GRUB config
if ! grep -q "GRUB_THEME=" /etc/default/grub; then
    echo 'GRUB_THEME="/boot/grub/themes/guardian/theme.txt"' >> /etc/default/grub
else
    sed -i 's|GRUB_THEME=.*|GRUB_THEME="/boot/grub/themes/guardian/theme.txt"|' /etc/default/grub
fi

# Wallpapers
echo "  Installing wallpapers..."
mkdir -p /usr/share/backgrounds/guardian
cp /tmp/guardian-branding/wallpapers/*.png /usr/share/backgrounds/guardian/

# Set default wallpaper for COSMIC
mkdir -p /etc/skel/.config/cosmic/com.system76.CosmicBackground
cat > /etc/skel/.config/cosmic/com.system76.CosmicBackground/v1/all << 'EOF'
("/usr/share/backgrounds/guardian/guardian-dark-1080p.png", "zoom")
EOF

echo "  ✓ Branding installed"
