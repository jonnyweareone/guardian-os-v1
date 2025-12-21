# Guardian OS Post-Install Wizard

A Rust/COSMIC-based first-boot wizard for Guardian OS.

## Purpose

This is the **POST-INSTALL** wizard that runs on first boot after Guardian OS is installed.
It handles simple desktop customization - the main Guardian authentication happens during 
the actual installation (in pop-installer).

## Modes

### Normal Mode (PostInstall)
When device was properly registered during installation:
```
Welcome → WiFi → Appearance → Layout → Done
```

### Fallback Mode (UnregisteredDevice)  
If device wasn't registered during installation (manual install, recovery, etc.):
```
Welcome → WiFi → Guardian Auth → Child Selection → Sync → Appearance → Layout → Done
```

## Architecture

| Component | Language | When | Purpose |
|-----------|----------|------|---------|
| **pop-installer** | Vala/GTK | Live ISO | Install OS + Guardian auth |
| **guardian-installer** (this) | Rust/COSMIC | First boot | Desktop customization |

## The Installation Flow

```
LIVE ISO SESSION:
┌─────────────────────────────┐
│   pop-installer (Vala)      │
│                             │
│   1. Language               │
│   2. Keyboard               │
│   3. Try/Install            │
│   4. GUARDIAN AUTH ★        │  ← Added to pop-installer
│   5. CHILD SELECTION ★      │  ← Added to pop-installer
│   6. User Creation          │
│   7. Disk Selection         │
│   8. Encryption             │
│   9. Install Progress       │
│   10. Success/Reboot        │
└─────────────────────────────┘
              ↓
         [REBOOT]
              ↓
FIRST BOOT:
┌─────────────────────────────┐
│   guardian-installer (Rust) │
│                             │
│   1. Welcome                │
│   2. WiFi                   │
│   3. Appearance             │
│   4. Layout                 │
└─────────────────────────────┘
```

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

## Files Created

The wizard checks `/etc/guardian/device.conf` to determine if the device
was registered during installation. If present, runs simple customization.
If absent, runs full Guardian auth flow.

## License

GPL-3.0 - See [LICENSE](LICENSE)

## Related

- [Guardian OS](https://github.com/guardian-network/guardian-os)
- [pop-installer fork](../pop-installer/) - Live ISO installer with Guardian auth
- [Guardian Daemon](../guardian-components/guardian-daemon/)
