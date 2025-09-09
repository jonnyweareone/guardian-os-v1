# Guardian OS

<p align="center">
  <img src="https://gameguardian.ai/lovable-uploads/guardian-logo2-transparent.png" alt="Guardian OS Logo" width="200">
</p>

<p align="center">
  <strong>A privacy-focused Ubuntu distribution with AI-powered parental controls</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Ubuntu-24.04%20LTS-orange" alt="Ubuntu 24.04">
  <img src="https://img.shields.io/badge/GNOME-46-blue" alt="GNOME 46">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="MIT License">
  <img src="https://img.shields.io/badge/Status-Beta-yellow" alt="Beta">
</p>

## 🚀 Features

- **Ubuntu 24.04 LTS** - Stable, long-term support base
- **GNOME Desktop** - Modern, user-friendly interface
- **Cloud Connected** - Supabase backend for device management
- **Parental Controls** - PIN-protected parent/child profiles
- **DNS Filtering** - NextDNS or Guardian DoH integration
- **AI Safety** - Guardian Reflex content monitoring (optional)
- **Gaming Support** - Steam, Proton, MangoHud included
- **Privacy First** - No telemetry without consent

## 📦 Quick Start

### Build from Source

```bash
# Clone the repository
git clone https://github.com/jonnyweare/guardian-os-v1.git
cd guardian-os-v1

# Install build dependencies
sudo apt update
sudo apt install -y live-build debootstrap reprepro dpkg-dev \
    debhelper devscripts equivs curl gnupg2 jq

# Build the ISO
make iso
```

### Download Pre-built ISO

Download the latest release from the [Releases](https://github.com/jonnyweare/guardian-os-v1/releases) page.

## 🏗️ Architecture

### System Components

- **Calamares Installer** - Custom modules for device registration
- **JWT Authentication** - Secure device-to-cloud communication
- **Systemd Services** - Heartbeat, parental controls, activation
- **APT Repository** - Signed packages for easy updates

### Package Structure

```
packages/
├── guardian-gnome-desktop    # GNOME meta-package
├── guardian-gnome-theme      # Branding and wallpapers
├── guardian-auth-client      # Authentication tools
├── guardian-device-agent     # Device management daemon
├── guardian-parental         # Parental control service
├── guardian-heartbeat        # Telemetry service
└── guardian-apps-base        # Core applications
```

## 🔐 Security

- **No API Keys on ISO** - Devices obtain JWT during installation
- **Hardware Fingerprinting** - Unique device identification
- **Encrypted Storage** - LUKS encryption by default
- **Secure Communication** - All API calls over HTTPS

## 🛠️ Development

### Building Packages

```bash
# Build individual package
cd packages/guardian-heartbeat
dpkg-buildpackage -b -uc -us

# Build all packages
make debs
```

### Testing

```bash
# Test in VM
qemu-system-x86_64 -m 4096 -cdrom guardian-os-*.iso -boot d

# Verify device registration
sudo cat /etc/guardian/supabase.env | grep JWT

# Check services
sudo systemctl status guardian-device-agent
sudo journalctl -u guardian-heartbeat
```

## 📡 API Integration

Guardian OS integrates with a Supabase backend for device management:

- **Authentication** - Parent login/registration
- **Device Claims** - Unique JWT per device
- **Heartbeats** - Regular status updates
- **Policy Sync** - Remote configuration updates

See [docs/API-INTEGRATION.md](docs/API-INTEGRATION.md) for details.

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📄 License

Guardian OS is released under the MIT License. See [LICENSE](LICENSE) for details.

## 🆘 Support

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/jonnyweare/guardian-os-v1/issues)
- **Discussions**: [GitHub Discussions](https://github.com/jonnyweare/guardian-os-v1/discussions)
- **Website**: [gameguardian.ai](https://gameguardian.ai)

## 🙏 Acknowledgments

- Ubuntu and Canonical for the excellent base system
- GNOME Project for the desktop environment
- Calamares team for the installer framework
- Supabase for the backend infrastructure

---

<p align="center">
  Made with ❤️ by the Guardian OS Team
</p>
