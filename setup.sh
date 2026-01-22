#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_PATH="${SCRIPT_DIR}/scripts"

export WINEPREFIX="$SCRIPT_DIR/.wine"
export WINEARCH=win64
export DISPLAY=:0
export WINEDLLOVERRIDES="mscoree,mshtml="
export WINEDEBUG=-all

if [ ! -d "$WINEPREFIX" ]; then
    wine wineboot --init
fi

echo "Starting Xvfb virtual display..."
Xvfb :0 -auth ~/.Xauthority -screen 0 640x480x24 > /dev/null 2>&1 &
XVFB_PID=$!


for script in "${SCRIPTS_PATH}"/[0-9]-*.sh; do
    if [ -f "$script" ]; then
        script_name=$(basename "$script")
        echo ""
        echo "Running: $script_name"
        echo "-----------------------------------------"
        chmod +x "$script"
        "$script"
        exit_code=$?
        if [ $exit_code -ne 0 ]; then
            echo ""
            echo "❌ Error: $script_name failed with exit code $exit_code"
            exit $exit_code
        fi
        echo ""
    fi
done

echo "Setup Complete"
