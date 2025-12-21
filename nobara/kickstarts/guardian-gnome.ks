# Guardian OS Kickstart - GNOME Edition
# Based on Nobara Linux kickstart
# SPDX-License-Identifier: GPL-3.0-or-later

# Basic setup
lang en_US.UTF-8
keyboard us
timezone UTC --utc
selinux --enforcing
firewall --enabled --service=mdns
services --enabled=NetworkManager,sshd,guardian-daemon

# Root password (will be set during install)
rootpw --lock

# Bootloader
bootloader --location=mbr --timeout=5

# Partitioning (let installer handle it)
zerombr
clearpart --all --initlabel
autopart --type=plain

# Repository configuration
# Use Nobara repos as base
repo --name=fedora --mirrorlist=https://mirrors.fedoraproject.org/mirrorlist?repo=fedora-$releasever&arch=$basearch
repo --name=fedora-updates --mirrorlist=https://mirrors.fedoraproject.org/mirrorlist?repo=updates-released-f$releasever&arch=$basearch
repo --name=nobara --baseurl=https://download.copr.fedorainfracloud.org/results/gloriouseggroll/nobara/fedora-$releasever-$basearch/
repo --name=rpmfusion-free --mirrorlist=https://mirrors.rpmfusion.org/mirrorlist?repo=free-fedora-$releasever&arch=$basearch
repo --name=rpmfusion-nonfree --mirrorlist=https://mirrors.rpmfusion.org/mirrorlist?repo=nonfree-fedora-$releasever&arch=$basearch
# Guardian OS repo (will be set up)
# repo --name=guardian --baseurl=https://repo.gameguardian.ai/fedora/$releasever/$basearch/

%packages
# Core system
@core
@base-x
@fonts
@hardware-support
@multimedia
@networkmanager-submodules

# GNOME Desktop
@gnome-desktop
gnome-tweaks
gnome-extensions-app

# Gaming (from Nobara)
steam
lutris
wine
winetricks
gamemode
mangohud

# Nobara packages
nobara-welcome
nobara-driver-manager
nobara-package-manager

# Guardian OS packages
guardian-daemon
guardian-wizard
guardian-branding

# Useful apps
firefox
libreoffice
vlc
flatpak

# Development (optional, for power users)
# @development-tools

%end

%post
# Guardian OS branding
cat > /etc/os-release << 'EOF'
NAME="Guardian OS"
VERSION="1.1.0 (Nobara Edition)"
ID=guardian
ID_LIKE=fedora nobara
VERSION_ID="1.1.0"
VERSION_CODENAME="Safe Gaming"
PRETTY_NAME="Guardian OS 1.1.0"
ANSI_COLOR="0;38;2;99;102;241"
LOGO=guardian-logo
CPE_NAME="cpe:/o:gameguardian:guardian_os:1.1.0"
HOME_URL="https://gameguardian.ai"
DOCUMENTATION_URL="https://gameguardian.ai/docs"
SUPPORT_URL="https://gameguardian.ai/support"
BUG_REPORT_URL="https://github.com/jonnyweareone/guardian-os-v1/issues"
PRIVACY_POLICY_URL="https://gameguardian.ai/privacy"
VARIANT="GNOME"
VARIANT_ID=gnome
EOF

# Create Guardian config directory
mkdir -p /etc/guardian
chmod 700 /etc/guardian

# Enable Guardian daemon
systemctl enable guardian-daemon

# Set up Flatpak
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# Configure GNOME defaults for family-friendly use
mkdir -p /etc/dconf/db/local.d
cat > /etc/dconf/db/local.d/01-guardian << 'DCONF'
# Guardian OS GNOME defaults

[org/gnome/desktop/session]
idle-delay=uint32 600

[org/gnome/desktop/screensaver]
lock-enabled=true
lock-delay=uint32 0

[org/gnome/desktop/privacy]
remember-recent-files=false

[org/gnome/desktop/interface]
color-scheme='prefer-dark'

[org/gnome/shell]
favorite-apps=['firefox.desktop', 'org.gnome.Nautilus.desktop', 'steam.desktop', 'guardian-settings.desktop']
DCONF

dconf update

# Welcome message
cat > /etc/motd << 'EOF'

  ╔═══════════════════════════════════════════════════════════╗
  ║              Welcome to Guardian OS                       ║
  ║           AI-Powered Protection for Families              ║
  ╚═══════════════════════════════════════════════════════════╝

  This device is protected by Guardian OS.
  
  For support: https://gameguardian.ai/support

EOF

%end

%post --nochroot
# Copy Guardian branding
cp -r /run/install/repo/guardian-branding/* /mnt/sysroot/usr/share/

%end
