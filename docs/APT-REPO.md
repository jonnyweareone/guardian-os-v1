# Guardian OS APT Repository

## Overview

Guardian OS uses a custom APT repository for distributing system packages. The repository is built as part of the ISO creation process and can optionally be hosted for network updates.

## Repository Structure

```
repo/
├── conf/
│   ├── distributions    # Repository configuration
│   └── options         # Build options
├── db/                 # Repository database
├── dists/
│   └── noble/
│       ├── Release
│       ├── Release.gpg
│       └── main/
│           └── binary-amd64/
│               └── Packages
├── pool/
│   └── main/
│       └── g/
│           └── guardian-*/
│               └── *.deb
└── GPG-KEY-GUARDIAN.asc
```

## Packages Included

### Core System
- `guardian-gnome-desktop` - GNOME desktop environment meta-package
- `guardian-gnome-theme` - Branding, wallpapers, and theme
- `guardian-gnome-layouts` - Parent/Child desktop layouts

### Authentication & Device Management
- `guardian-auth-client` - CLI tools for authentication
- `guardian-device-agent` - Device management daemon
- `guardian-activate` - Offline activation service

### Services
- `guardian-heartbeat` - Device heartbeat service (10-minute interval)
- `guardian-parental` - Parental control enforcement

### Applications
- `guardian-apps-base` - Core applications (Chrome, LibreOffice, etc.)
- `guardian-gaming-meta` - Gaming support (Steam, MangoHud, Gamescope)

### AI Features
- `guardian-reflex` - AI safety monitoring service
- `guardian-reflex-models` - AI model management

## Adding the Repository

### During Installation

The repository is automatically configured during OS installation.

### Manual Configuration

To add the repository to an existing Ubuntu 24.04 system:

```bash
# Download and add GPG key
wget -qO - https://apt.gameguardian.ai/GPG-KEY-GUARDIAN.asc | sudo apt-key add -

# Add repository
echo "deb https://apt.gameguardian.ai/ noble main" | \
    sudo tee /etc/apt/sources.list.d/guardian.list

# Update package list
sudo apt update

# Install Guardian packages
sudo apt install guardian-heartbeat guardian-device-agent
```

## Building the Repository

### Local Build

```bash
# From the guardian-os-v1 directory
make repo

# Or manually
./scripts/build-repo.sh
```

### GPG Signing

For production, sign packages with a GPG key:

```bash
# Generate GPG key
gpg --gen-key

# Export for repository
export GPG_KEYID="your-key@example.com"

# Build with signing
make repo
```

## Hosting the Repository

### S3 Hosting

```bash
# Configure AWS credentials
aws configure

# Set environment variables
export REPO_S3_BUCKET="apt.gameguardian.ai"
export CF_DISTRIBUTION_ID="your-cloudfront-id"

# Sync to S3
make sync
```

### Local HTTP Server (Testing)

```bash
cd repo
python3 -m http.server 8080

# On client, add:
echo "deb http://server-ip:8080/ noble main" | \
    sudo tee /etc/apt/sources.list.d/guardian-local.list
```

## Package Management

### Installing Packages

```bash
# Core system
sudo apt install guardian-gnome-desktop guardian-gnome-theme

# Services
sudo apt install guardian-heartbeat guardian-device-agent

# Applications
sudo apt install guardian-apps-base
```

### Updating Packages

```bash
sudo apt update
sudo apt upgrade
```

### Removing Packages

```bash
sudo apt remove guardian-heartbeat
sudo apt autoremove
```

## Troubleshooting

### GPG Errors

```bash
# Remove old key
sudo apt-key del guardian@gameguardian.ai

# Re-add key
wget -qO - https://apt.gameguardian.ai/GPG-KEY-GUARDIAN.asc | sudo apt-key add -
```

### Package Conflicts

```bash
# Check for issues
sudo apt-get check

# Fix broken packages
sudo apt-get -f install
```

### Repository Not Found

```bash
# Verify repository file
cat /etc/apt/sources.list.d/guardian.list

# Test connectivity
curl -I https://apt.gameguardian.ai/
```
