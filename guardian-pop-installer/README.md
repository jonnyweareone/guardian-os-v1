# Guardian Pop Installer

This directory contains the Guardian OS modifications to the pop-os/installer.

## Overview

The pop-os/installer is a Vala/GTK application that handles the OS installation from the live ISO.
Guardian OS modifies this to add parent authentication and child profile selection BEFORE the
user account creation step.

## Files

- `src/Views/GuardianAuthView.vala` - Parent authentication UI (Supabase)
- `src/Views/GuardianChildView.vala` - Child profile selection/creation
- `GUARDIAN-MODS.md` - Detailed instructions for integrating with pop-installer

## Building

1. Clone pop-os/installer:
   ```bash
   git clone https://github.com/pop-os/installer.git
   cd installer
   ```

2. Copy Guardian views:
   ```bash
   cp path/to/guardian-pop-installer/src/Views/*.vala src/Views/
   ```

3. Follow instructions in `GUARDIAN-MODS.md` to modify `MainWindow.vala`

4. Add files to `meson.build`

5. Build:
   ```bash
   meson setup build
   ninja -C build
   ```

## Flow

```
ORIGINAL:
Language → Keyboard → Try/Install → Disk → User → Encrypt → Install

GUARDIAN:
Language → Keyboard → Try/Install → Disk → AUTH → CHILD → User → Encrypt → Install
```

## Supabase Integration

The views connect to Guardian's Supabase backend:
- Project: `gkyspvcafyttfhyjryyk` (guardianos)
- Auth endpoint: `/auth/v1/token`
- REST endpoint: `/rest/v1/children`, `/rest/v1/devices`

## Created Files

On successful authentication, the installer creates:
- `/etc/guardian/credentials` - Auth tokens (restricted permissions)
- `/etc/guardian/device.conf` - Device registration
- `/etc/guardian/child.conf` - Child profile info

These are read by `guardian-daemon` on first boot.
