#!/usr/bin/env bash

set -e

# Get workspace directory (parent of scripts directory)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
MAPS_DIR="${PROJECT_DIR}/starcraft/maps/BroodWar"
TEMP_DIR="/tmp/sc_standard_maps"

echo "========================================"
echo "Step 3: Download Standard Maps"
echo "========================================"

# Create directories
mkdir -p "${MAPS_DIR}"
mkdir -p "${TEMP_DIR}"

# Check if maps already exist
EXISTING_MAPS=$(find "${MAPS_DIR}" -maxdepth 1 -type f \( -name "*.scm" -o -name "*.scx" -o -name "*.SCM" -o -name "*.SCX" \) ! -name "ICCup*" 2>/dev/null | wc -l)

if [ "$EXISTING_MAPS" -gt 10 ]; then
    echo "✓ Found $EXISTING_MAPS standard maps already installed."
    echo "Skipping download. Delete maps to re-download."
    exit 0
fi

echo "Downloading standard StarCraft Broodwar maps..."

echo ""
echo "Downloading SSCAIT map pack..."
ALT_MAPS_URL="https://sscaitournament.com/files/sscai_map_pack.zip"
TEMP_ZIP="${TEMP_DIR}/maps.zip"

if curl -L -f "${ALT_MAPS_URL}" -o "${TEMP_ZIP}" 2>/dev/null; then
    echo "Extracting maps..."
    unzip -q -o "${TEMP_ZIP}" -d "${TEMP_DIR}" 2>/dev/null || true
    
    # Move extracted maps to the maps directory
    find "${TEMP_DIR}" -type f \( -name "*.sc?" -o -name "*.SC?" \) | while read -r map; do
        filename=$(basename "$map")
        if [ ! -f "${MAPS_DIR}/${filename}" ]; then
            cp "$map" "${MAPS_DIR}/"
            echo "  ✓ Extracted ${filename}"
        fi
    done
else
    echo "SSCAIT map pack not available"
fi

# Clean up
rm -rf "${TEMP_DIR}"
