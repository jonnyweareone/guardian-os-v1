# Guardian Installer v1.1.0 - Streamlined

## Changes Made

### Language Support
- Removed 60+ language directories from i18n/
- Kept only `en-GB` (English UK)
- Updated `i18n.toml` to use `en-GB` as fallback

### Package Renaming
- Renamed from `cosmic-initial-setup` to `guardian-installer`
- Updated `Cargo.toml` with new package name and metadata
- Updated `debian/control` with new package names:
  - `guardian-installer` (main package)
  - `guardian-installer-casper` (live session integration)
- Renamed all debian files to match

### Page Cleanup
Removed unused pages:
- `launcher.rs` - Not needed for child devices
- `location.rs` - Not needed (no timezone UI)
- `new_apps.rs` - COSMIC app showcase not relevant
- `new_shortcuts.rs` - Keyboard shortcuts not relevant
- `workflow.rs` - Workspace config not needed

### Two-Mode Architecture

#### Mode 1: Live Install (`--live-install` or LIVE_SESSION=true)
Runs during live ISO session BEFORE disk installation:
1. Welcome (accessibility)
2. WiFi (optional, for auth)
3. Language (EN-GB default)
4. Keyboard
5. **Guardian Auth** (parent sign-in/register)
6. **Child Selection** (select/create child profile)
7. User Creation (auto-filled from child)
8. Sync Enrollment (optional)

After this wizard, `pop-installer` handles disk partitioning.

#### Mode 2: Post-Install (`--first-boot` or default)
Runs on FIRST BOOT after installation:
1. Welcome to Guardian OS
2. Appearance (theme, colours)
3. Layout (panel position)

Simple customization - device already registered.

### Desktop Files
- `com.guardian.GuardianInstaller.desktop` - Main application
- `com.guardian.GuardianInstaller.Autostart.desktop` - First boot trigger

### ISO Build Config
Created `/upstream/pop-os-iso/config/guardian-os/24.04.mk`:
- Uses `guardian-installer` + `guardian-installer-casper`
- Includes `guardian-daemon` and `guardian-webapp`
- Removes `cosmic-initial-setup` from installed system

## File Structure

```
guardian-installer/
├── Cargo.toml                  # Package: guardian-installer v1.1.0
├── justfile                    # Build recipes
├── i18n.toml                   # Fallback: en-GB
├── i18n/
│   └── en-GB/
│       └── cosmic_initial_setup.ftl
├── debian/
│   ├── control                 # guardian-installer, guardian-installer-casper
│   ├── changelog
│   ├── 99guardian-installer-casper
│   ├── guardian-installer.install
│   ├── guardian-installer.postinst
│   ├── guardian-installer.postrm
│   ├── guardian-installer-casper.install
│   └── guardian-installer-casper.triggers
├── res/
│   ├── com.guardian.GuardianInstaller.desktop
│   ├── com.guardian.GuardianInstaller.Autostart.desktop
│   └── icon.svg
└── src/
    ├── main.rs                 # Mode detection, app entry
    ├── localize.rs             # i18n loading
    ├── accessibility.rs        # A11y support
    ├── greeter.rs
    └── page/
        ├── mod.rs              # Page routing for both modes
        ├── welcome.rs          # Accessibility options
        ├── wifi.rs             # Network connection
        ├── language.rs         # EN-GB default
        ├── keyboard.rs         # Layout selection
        ├── guardian_auth.rs    # Parent authentication
        ├── guardian_child.rs   # Child profile selection
        ├── user.rs             # Local account creation
        ├── guardian_sync.rs    # Settings sync enrollment
        ├── appearance.rs       # Theme customization
        └── layout.rs           # Panel/dock layout
```

## Next Steps

1. Build and test the installer package
2. Integrate with ISO build pipeline
3. Test live install flow end-to-end
4. Test first-boot wizard flow
5. Verify Guardian auth works with Supabase
