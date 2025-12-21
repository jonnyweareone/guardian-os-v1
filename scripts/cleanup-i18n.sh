#!/bin/bash
# Clean up i18n - keep only en-GB

cd /Users/davidsmith/Documents/GitHub/guardian-os-v1/guardian-installer/i18n

# Remove all language folders except en-GB
for dir in */; do
    if [ "$dir" != "en-GB/" ]; then
        echo "Removing $dir"
        rm -rf "$dir"
    fi
done

echo "Cleanup complete. Only en-GB remains."
ls -la
