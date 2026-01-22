#!/usr/bin/env bash

set -e

# Get workspace directory (parent of scripts directory)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
INSTALL_DIR="${PROJECT_DIR}/starcraft"

STARCRAFT_URL="http://files.theabyss.ru/sc/starcraft.zip"
TEMP_ZIP="/tmp/starcraft.zip"

echo "========================================"
echo "Step 1: Download StarCraft"
echo "========================================"

if [ -f "${INSTALL_DIR}/StarCraft.exe" ]; then
    echo "✓ StarCraft is already installed in ${INSTALL_DIR}!"
    echo "Skipping download."
    echo ""
    exit 0
else
    if ! curl -L "${STARCRAFT_URL}" -o "${TEMP_ZIP}"; then
        echo "ERROR: Download failed!"
        echo "You may need to download manually from: ${STARCRAFT_URL}"
        exit 1
    fi

    mkdir -p "${INSTALL_DIR}"
    unzip -q "${TEMP_ZIP}" -d "${INSTALL_DIR}"

    rm "${TEMP_ZIP}"
fi
