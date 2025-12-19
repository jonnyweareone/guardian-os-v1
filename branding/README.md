# Guardian OS Branding

Brand assets for Guardian OS matching the GameGuardian.ai website design.

## Color Palette

| Color | Hex | Usage |
|-------|-----|-------|
| Background Dark | `#010409` | Primary background |
| Background Mid | `#0d1117` | Secondary background |
| Cyan Primary | `#00E5FF` | Accent, highlights |
| Cyan Secondary | `#00B8D4` | Gradient end |
| Text Primary | `#FFFFFF` | Main text |
| Text Secondary | `#a0a0a0` | Muted text |
| Text Tertiary | `#666666` | Disabled text |

## Typography

- **Primary Font:** Inter (fallback: SF Pro Display, system sans-serif)
- **Headings:** Inter Bold/Semi-bold
- **Body:** Inter Regular

## Assets

### Logo (`logo/`)
- `guardian-shield.svg` - Shield icon only
- `guardian-os-wordmark.svg` - Full "Guardian OS" wordmark with mini shield

### Wallpapers (`wallpapers/`)
- `guardian-dark.svg` - Minimal dark background with subtle grid
- `guardian-shield.svg` - Large centered shield watermark

### Plymouth Boot Splash (`plymouth/`)
- `guardian.plymouth` - Theme metadata
- `guardian.script` - Animation script
- `guardian-logo.svg` - Animated shield logo
- `guardian-wordmark.svg` - "Guardian OS" text
- `progress-bg.svg` - Progress bar background
- `progress-fill.svg` - Cyan progress fill

### GRUB Theme (`grub/`)
- `theme.txt` - GRUB2 theme configuration
- `background.svg` - Boot menu background
- Selection highlight and scrollbar assets

## Converting to PNG

SVGs must be converted to PNG for Plymouth and GRUB:

```bash
cd branding
chmod +x convert-assets.sh
./convert-assets.sh
```

Requires:
- ImageMagick (`brew install imagemagick`)
- librsvg (`brew install librsvg`)

## Installation Paths (Ubuntu/Pop!_OS)

### Plymouth
```
/usr/share/plymouth/themes/guardian/
├── guardian.plymouth
├── guardian.script
├── guardian-logo.png
├── guardian-wordmark.png
├── progress-bg.png
└── progress-fill.png
```

Enable with:
```bash
sudo update-alternatives --install /usr/share/plymouth/themes/default.plymouth default.plymouth /usr/share/plymouth/themes/guardian/guardian.plymouth 100
sudo update-alternatives --set default.plymouth /usr/share/plymouth/themes/guardian/guardian.plymouth
sudo update-initramfs -u
```

### GRUB
```
/boot/grub/themes/guardian/
├── theme.txt
├── background.png
├── guardian-logo.png
├── select_c.png
└── fonts/
```

Enable in `/etc/default/grub`:
```
GRUB_THEME="/boot/grub/themes/guardian/theme.txt"
```

Then: `sudo update-grub`

### Wallpapers
```
/usr/share/backgrounds/guardian/
├── guardian-dark-1080p.png
├── guardian-dark-4k.png
├── guardian-shield-1080p.png
└── guardian-shield-4k.png
```

## Preview

### Boot Sequence

1. **GRUB** - Dark background, cyan-highlighted menu, shield logo
2. **Plymouth** - Centered shield with pulse animation, progress bar
3. **Login** - guardian-wizard if first boot, else COSMIC login
4. **Desktop** - guardian-dark wallpaper by default
