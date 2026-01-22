#!/usr/bin/env bash

set -euo pipefail

echo "========================================"
echo "Step 2: Installing BWAPI"
echo "========================================"

# Get workspace directory (parent of scripts directory)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
export SC_DIR="${PROJECT_DIR}/starcraft"
BWAPI_VERSION="4.4.0"
BWAPI_ARCHIVE="BWAPI.7z"
BWAPI_URL="https://github.com/bwapi/bwapi/releases/download/v${BWAPI_VERSION}/${BWAPI_ARCHIVE}"

# Check if BWAPI is already installed
if [ -d "${SC_DIR}/BWAPI" ] && [ -f "${SC_DIR}/BWAPI/Chaoslauncher/Chaoslauncher.exe" ]; then
    echo "✓ BWAPI is already installed!"
    echo "Skipping installation."
    exit 0
fi

if [ ! -d "${SC_DIR}" ]; then
    echo "ERROR: StarCraft directory not found at ${SC_DIR}"
    echo "Please install StarCraft first or set SC_DIR environment variable"
    exit 1
fi

if [ ! -f "${SC_DIR}/StarCraft.exe" ]; then
    echo "ERROR: StarCraft.exe not found in ${SC_DIR}"
    echo "Please ensure StarCraft is properly installed"
    exit 1
fi

TEMP_DIR=$(mktemp -d)
trap "rm -rf ${TEMP_DIR}" EXIT

cd "${TEMP_DIR}"

echo "Downloading BWAPI ${BWAPI_VERSION}..."
wget -q --show-progress "${BWAPI_URL}" || {
    echo "ERROR: Failed to download BWAPI archive"
    exit 1
}

7z x -y "${BWAPI_ARCHIVE}" -obwapi_extracted > /dev/null || {
    echo "ERROR: Failed to extract BWAPI archive"
    exit 1
}


cd bwapi_extracted

ls -alh Release_Binary

cp -r Release_Binary "${SC_DIR}/BWAPI"

cp -r Release_Binary/Starcraft/bwapi-data "${SC_DIR}/bwapi-data"

cd ..

# Create necessary subdirectories