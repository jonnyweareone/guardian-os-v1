#!/bin/bash
# Generate GRUB theme assets
# Run this on a Linux system with ImageMagick and grub-mkfont

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

mkdir -p png fonts

echo "🔧 Generating GRUB assets..."

# ============================================
# Generate PNG assets using ImageMagick
# ============================================

# Background (if rsvg-convert available, use it, otherwise create solid)
if command -v rsvg-convert &> /dev/null && [ -f background.svg ]; then
    rsvg-convert -w 1920 -h 1080 background.svg > png/background.png
else
    convert -size 1920x1080 'gradient:#010409-#000000' png/background.png
fi
echo "  ✓ background.png"

# Logo
if command -v rsvg-convert &> /dev/null && [ -f ../logo/guardian-shield.svg ]; then
    rsvg-convert -w 100 -h 120 ../logo/guardian-shield.svg > png/guardian-logo.png
else
    convert -size 100x120 xc:transparent \
        -fill '#00E5FF' -draw "polygon 50,0 100,30 100,90 50,120 0,90 0,30" \
        png/guardian-logo.png
fi
echo "  ✓ guardian-logo.png"

# Menu selection highlight (left, center, right for 9-slice)
convert -size 10x40 xc:'#00E5FF20' -fill '#00E5FF30' -draw "roundrectangle 0,0 9,39 3,3" png/select_w.png
convert -size 480x40 xc:'#00E5FF20' png/select_c.png
convert -size 10x40 xc:'#00E5FF20' -fill '#00E5FF30' -draw "roundrectangle 0,0 9,39 3,3" png/select_e.png
echo "  ✓ select_*.png (menu selection)"

# Terminal box (for recovery/CLI mode)
convert -size 10x10 xc:'#1a1a2eCC' png/terminal_box_c.png
convert -size 10x10 xc:'#1a1a2eCC' png/terminal_box_n.png
convert -size 10x10 xc:'#1a1a2eCC' png/terminal_box_s.png
convert -size 10x10 xc:'#1a1a2eCC' png/terminal_box_e.png
convert -size 10x10 xc:'#1a1a2eCC' png/terminal_box_w.png
convert -size 10x10 xc:'#1a1a2eCC' png/terminal_box_ne.png
convert -size 10x10 xc:'#1a1a2eCC' png/terminal_box_nw.png
convert -size 10x10 xc:'#1a1a2eCC' png/terminal_box_se.png
convert -size 10x10 xc:'#1a1a2eCC' png/terminal_box_sw.png
echo "  ✓ terminal_box_*.png"

# Scrollbar
convert -size 6x20 xc:'#00E5FF80' -fill '#00E5FF' -draw "roundrectangle 0,0 5,19 3,3" png/scrollbar_thumb_c.png
convert -size 6x5 xc:'#00E5FF80' png/scrollbar_thumb_n.png
convert -size 6x5 xc:'#00E5FF80' png/scrollbar_thumb_s.png
convert -size 6x100 xc:'#1a1a2e80' png/scrollbar_frame_c.png
convert -size 6x5 xc:'#1a1a2e80' png/scrollbar_frame_n.png
convert -size 6x5 xc:'#1a1a2e80' png/scrollbar_frame_s.png
echo "  ✓ scrollbar_*.png"

# Progress bar highlight
convert -size 10x8 xc:'#00E5FF' png/highlight_c.png
convert -size 5x8 xc:'#00E5FF' -fill '#00B8D4' -draw "roundrectangle 0,0 4,7 2,2" png/highlight_w.png
convert -size 5x8 xc:'#00E5FF' -fill '#00B8D4' -draw "roundrectangle 0,0 4,7 2,2" png/highlight_e.png
echo "  ✓ highlight_*.png"

# ============================================
# Generate fonts (requires grub-mkfont on Linux)
# ============================================

if command -v grub-mkfont &> /dev/null; then
    echo ""
    echo "📝 Generating GRUB fonts..."
    
    # Try to find suitable fonts
    for font in /usr/share/fonts/truetype/dejavu/DejaVuSans.ttf \
                /usr/share/fonts/TTF/DejaVuSans.ttf \
                /usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf; do
        if [ -f "$font" ]; then
            grub-mkfont -s 16 -o fonts/regular_16.pf2 "$font"
            grub-mkfont -s 12 -o fonts/regular_12.pf2 "$font"
            grub-mkfont -s 11 -o fonts/regular_11.pf2 "$font"
            echo "  ✓ Generated regular fonts from DejaVuSans"
            break
        fi
    done
    
    for font in /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf \
                /usr/share/fonts/TTF/DejaVuSans-Bold.ttf \
                /usr/share/fonts/dejavu-sans-fonts/DejaVuSans-Bold.ttf; do
        if [ -f "$font" ]; then
            grub-mkfont -s 16 -o fonts/bold_16.pf2 "$font"
            echo "  ✓ Generated bold font from DejaVuSans-Bold"
            break
        fi
    done
    
    for font in /usr/share/fonts/truetype/terminus/TerminusTTF.ttf \
                /usr/share/fonts/terminus/terminus.ttf \
                /usr/share/fonts/misc/ter-u14n.pcf.gz; do
        if [ -f "$font" ]; then
            grub-mkfont -s 14 -o fonts/terminus_14.pf2 "$font"
            echo "  ✓ Generated terminal font"
            break
        fi
    done
else
    echo ""
    echo "⚠️  grub-mkfont not found - fonts will use GRUB defaults"
    echo "   Run this script on Linux with: apt install grub-common"
fi

echo ""
echo "✅ GRUB assets generated in:"
echo "   • png/   - Image assets"
echo "   • fonts/ - PF2 font files (if grub-mkfont available)"
