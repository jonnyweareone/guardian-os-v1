# Guardian OS Branding Package

This package provides Guardian OS branding to replace COSMIC/Pop!_OS defaults.

## Contents

### 1. Installer Images (`installer-images/`)
Replace the Pop "Zoe" robot images with Guardian branding.

**To install:** Copy all PNG files to:
```
/path/to/pop-installer/data/images/
```

### 2. Desktop Wallpapers (`wallpapers/`)
Guardian-branded wallpapers in multiple resolutions:
- `guardian-wallpaper-1080p.png` - 1920x1080
- `guardian-wallpaper-1440p.png` - 2560x1440
- `guardian-wallpaper-4k.png` - 3840x2160
- `guardian-wallpaper-live-1080p.png` - For live environment

**To install:** Copy to:
```
/usr/share/backgrounds/guardian/
```

### 3. Desktop Entries (`desktop-entries/`)
Rename COSMIC apps to Guardian apps:

| Original | Guardian |
|----------|----------|
| COSMIC Files | Guardian Files |
| COSMIC Store | Guardian Store |
| COSMIC Settings | Guardian Settings |
| COSMIC Terminal | Guardian Terminal |
| COSMIC Edit | Guardian Editor |
| COSMIC Calculator | Guardian Calculator |

**To install:** Copy to:
```
/usr/share/applications/
```

## Building Debian Package

```bash
cd guardian-branding
dpkg-deb --build debian-package guardian-branding_1.1.0_all.deb
```

## Integration with ISO Build

Add to `guardian-os.mk`:
```makefile
# Install Guardian branding
cp -r guardian-branding/installer-images/* $(ISO_ROOT)/pop-installer/data/images/
cp -r guardian-branding/wallpapers/* $(ISO_ROOT)/usr/share/backgrounds/guardian/
cp guardian-branding/desktop-entries/*.desktop $(ISO_ROOT)/usr/share/applications/
```
