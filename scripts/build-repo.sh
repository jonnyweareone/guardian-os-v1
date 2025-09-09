#!/bin/bash
set -euo pipefail

GPG_KEYID="${GPG_KEYID:-guardian@gameguardian.ai}"

# Setup repo structure
mkdir -p repo/{conf,incoming}

# Create distributions file
cat > repo/conf/distributions << DIST
Origin: Guardian OS
Label: Guardian OS
Suite: stable
Codename: noble
Version: 24.04
Architectures: amd64
Components: main
Description: Guardian OS APT Repository
SignWith: ${GPG_KEYID}
DIST

# Create options file
cat > repo/conf/options << OPTIONS
verbose
ask-passphrase
OPTIONS

# Export public key (create placeholder if GPG not available)
gpg --armor --export "${GPG_KEYID}" > repo/GPG-KEY-GUARDIAN.asc 2>/dev/null || {
    echo "Warning: Could not export GPG key. Creating placeholder..."
    cat > repo/GPG-KEY-GUARDIAN.asc << 'GPGKEY'
-----BEGIN PGP PUBLIC KEY BLOCK-----
# Placeholder GPG key - replace with actual key for production
-----END PGP PUBLIC KEY BLOCK-----
GPGKEY
}

# Include all packages if reprepro is available
if command -v reprepro &> /dev/null; then
    cd repo
    for deb in incoming/*.deb; do
        [ -f "$deb" ] || continue
        reprepro includedeb noble "$deb" || {
            echo "Warning: Could not sign package. Using unsigned repo..."
            reprepro --ignore=missingfield includedeb noble "$deb" || true
        }
    done
    cd ..
else
    echo "Warning: reprepro not installed. Repository structure created but not populated."
fi

echo "APT repository structure created"
