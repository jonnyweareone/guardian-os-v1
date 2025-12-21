# Guardian OS Installer

A streamlined Rust/COSMIC-based installer for Guardian OS, the family-safe Linux distribution.

## Overview

Guardian Installer handles the initial setup of Guardian OS devices, including:

1. **Parent Authentication** - Sign in with Guardian account (Supabase auth)
2. **Child Profile Selection** - Choose or create a child profile for the device
3. **User Account Creation** - Create local user account (auto-filled from child profile)
4. **Device Registration** - Link device to parent's Guardian dashboard
5. **Optional Sync Enrollment** - Enable settings synchronization

## Architecture

This is a fork of [cosmic-initial-setup](https://github.com/pop-os/cosmic-initial-setup) with Guardian-specific authentication flow integrated.

### Pages

| Page | Purpose | Required |
|------|---------|----------|
| `welcome` | Accessibility options, interface scaling | ✓ |
| `wifi` | Network connection | Optional |
| `language` | Language selection (EN-GB default) | ✓ |
| `keyboard` | Keyboard layout | ✓ |
| `guardian_auth` | Parent sign-in/registration | ✓ |
| `guardian_child` | Child profile selection | ✓ |
| `user` | Local account creation | ✓ |
| `guardian_sync` | Settings sync enrollment | Optional |
| `appearance` | Theme customization | ✓ |
| `layout` | Panel/dock layout | ✓ |

### Supabase Integration

Guardian Installer connects to the Guardian Network Supabase project:

- **Project**: `gkyspvcafyttfhyjryyk` (guardianos, eu-west-2)
- **Auth**: Email/password via Supabase Auth
- **API**: REST endpoints for device registration and child management

## Building

```bash
# Install dependencies (Pop!_OS/Ubuntu)
sudo apt install cargo cmake just libexpat1-dev libfontconfig-dev \
  libfreetype-dev libxkbcommon-dev pkgconf libssl-dev libflatpak-dev

# Build
just build

# Install
just install
```

## Integration with ISO Build

The installer is packaged as `guardian-installer` and included in the live ISO:

```makefile
# In iso-builder/config/guardian-os/24.04.mk
LIVE_PKGS=\
    casper \
    guardian-installer \
    guardian-installer-casper \
    distinst \
    ...
```

## Configuration Files Created

After setup, Guardian Installer creates:

```
/etc/guardian/
├── credentials      # Parent JWT token (encrypted)
├── device.conf      # Device ID and registration info
└── child.conf       # Active child profile ID

/home/<child>/
└── .config/guardian/
    └── profile.json  # Child profile details
```

## Differences from cosmic-initial-setup

| Feature | cosmic-initial-setup | Guardian Installer |
|---------|---------------------|-------------------|
| Purpose | Post-install wizard | Installation + setup |
| Auth | None | Supabase/Guardian |
| User creation | Standard admin | Limited child account |
| Languages | 60+ | English UK only |
| Pages | 12 | 10 (focused) |

## License

GPL-3.0 - See [LICENSE](LICENSE)

## Related

- [Guardian OS](https://github.com/guardian-network/guardian-os)
- [Guardian Daemon](../guardian-components/guardian-daemon/)
- [Guardian Webapp](../guardian-components/guardian-webapp/)
