#!/usr/bin/env bash

echo "========================================"
echo "Step 5: Run StarCraft"
echo "========================================"

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

export WINEPREFIX="$PROJECT_DIR/.wine"
export WINEARCH=win64
export DISPLAY=:0
export WINEDLLOVERRIDES="mscoree,mshtml="
export WINEDEBUG=-all


INSTALL_DIR="$PROJECT_DIR/starcraft"

SC_WIN_PATH="$(winepath -w "$INSTALL_DIR" 2>/dev/null || echo "Z:${INSTALL_DIR}" | sed 's/\//\\\\/g')\\\\"

# if wine REG QUERY "HKEY_LOCAL_MACHINE\\SOFTWARE\\Blizzard Entertainment\\Starcraft" /v InstallPath >/dev/null 2>&1; then
#     echo "Registry already configured, skipping..."
#     exit 0
# fi

echo "Configuring registry..."
# Core registry entries
wine REG ADD "HKEY_LOCAL_MACHINE\\SOFTWARE\\Blizzard Entertainment\\Starcraft" \
    /v InstallPath /t REG_EXPAND_SZ /d "${SC_WIN_PATH}" /f 2>/dev/null || true

wine REG ADD "HKEY_LOCAL_MACHINE\\SOFTWARE\\Blizzard Entertainment\\Starcraft" \
    /v Program /t REG_EXPAND_SZ /d "${SC_WIN_PATH}StarCraft.exe" /f 2>/dev/null || true

# Disable Intro
wine REG ADD "HKEY_CURRENT_USER\\Software\\Blizzard Entertainment\\Starcraft" /v "Intro" /t REG_SZ /d "0" /f 2>/dev/null || true
wine REG ADD "HKEY_CURRENT_USER\\Software\\Blizzard Entertainment\\Starcraft" /v "IntroX" /t REG_SZ /d "0" /f 2>/dev/null || true
wine REG ADD "HKEY_CURRENT_USER\\Software\\Blizzard Entertainment\\Starcraft" /v "Tip" /t REG_SZ /d "0" /f 2>/dev/null || true

# Chaoslauncher configuration
SC_CHAOS_PATH=$(winepath -w "$INSTALL_DIR" | sed 's/\\/\\\\/g')
wine REG ADD "HKEY_CURRENT_USER\\Software\\Chaoslauncher\\Launcher" /v "ScPath" /t REG_SZ /d "${SC_CHAOS_PATH}" /f 2>/dev/null || true
wine REG ADD "HKEY_CURRENT_USER\\Software\\Chaoslauncher\\Launcher" /v "GameVersion" /t REG_SZ /d "Starcraft 1.16.1" /f 2>/dev/null || true
wine REG ADD "HKEY_CURRENT_USER\\Software\\Chaoslauncher\\Launcher" /v "RunScOnStartup" /t REG_SZ /d "1" /f 2>/dev/null || true

# Enable BWAPI 4.4.0 Injector plugin by default
wine REG ADD "HKEY_CURRENT_USER\\Software\\Chaoslauncher\\PluginsEnabled" /v "BWAPI 4.4.0 Injector [RELEASE]" /t REG_SZ /d "1" /f 2>/dev/null || true
wine REG ADD "HKEY_CURRENT_USER\\Software\\Chaoslauncher\\PluginsEnabled" /v "W-MODE 1.02" /t REG_SZ /d "1" /f 2>/dev/null || true
