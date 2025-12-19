#!/bin/bash
# update-version.sh - Sync VERSION file to all references
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
VERSION=$(cat "$ROOT_DIR/VERSION" | tr -d '\n')

echo "Updating all version references to: $VERSION"

# Update branding.desc
BRANDING_FILE="$ROOT_DIR/calamares/branding/guardian/branding.desc"
if [ -f "$BRANDING_FILE" ]; then
    sed -i.bak "s/version:.*/version:             $VERSION/" "$BRANDING_FILE"
    sed -i.bak "s/shortVersion:.*/shortVersion:        ${VERSION%.*}/" "$BRANDING_FILE"
    sed -i.bak "s/versionedName:.*/versionedName:       Guardian OS $VERSION/" "$BRANDING_FILE"
    sed -i.bak "s/shortVersionedName:.*/shortVersionedName:  Guardian ${VERSION%.*}/" "$BRANDING_FILE"
    rm -f "$BRANDING_FILE.bak"
    echo "  ✓ Updated calamares/branding/guardian/branding.desc"
fi

# Update guardian_claim.conf
CLAIM_CONF="$ROOT_DIR/calamares/modules/guardian_claim.conf"
if [ -f "$CLAIM_CONF" ]; then
    sed -i.bak "s/installerVersion:.*/installerVersion: \"$VERSION\"/" "$CLAIM_CONF"
    rm -f "$CLAIM_CONF.bak"
    echo "  ✓ Updated calamares/modules/guardian_claim.conf"
fi

# Update guardian_claim.py (installer_version in API call)
CLAIM_PY="$ROOT_DIR/calamares/modules-impl/guardian_claim.py"
if [ -f "$CLAIM_PY" ]; then
    sed -i.bak "s/\"installer_version\": \"[^\"]*\"/\"installer_version\": \"$VERSION\"/" "$CLAIM_PY"
    rm -f "$CLAIM_PY.bak"
    echo "  ✓ Updated calamares/modules-impl/guardian_claim.py"
fi

echo "Version sync complete: $VERSION"
