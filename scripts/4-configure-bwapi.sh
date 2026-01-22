#!/usr/bin/env bash

set -e

echo "========================================"
echo "Step 4: Configure BWAPI"
echo "========================================"

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PREFERENCES_FILE="${PREFERENCES_FILE:-$SCRIPT_DIR/bwapi-preferences.yml}"
BWAPI_INI_PATH="${BWAPI_INI_PATH:-$PROJECT_DIR/starcraft/bwapi-data/bwapi.ini}"

# Check if already configured (if bwapi.ini exists and preferences haven't changed)
if [ -f "$BWAPI_INI_PATH" ]; then
    echo "✓ BWAPI configuration file already exists."
    echo "Updating configuration from preferences..."
fi

# Check if preferences file exists
if [ ! -f "$PREFERENCES_FILE" ]; then
    echo "Error: Preferences file not found: $PREFERENCES_FILE"
    exit 1
fi

# Function to read YAML value (simple parser for our use case)
read_yaml_value() {
    local key="$1"
    local default="${2:-}"
    # Read value, ignoring comments and empty lines
    local value=$(grep "^${key}:" "$PREFERENCES_FILE" | head -1 | sed 's/^[^:]*:[[:space:]]*//' | sed 's/^"\(.*\)"$/\1/')
    echo "${value:-$default}"
}

# Function to read YAML array values
read_yaml_array() {
    local key="$1"
    grep -A 20 "^${key}:" "$PREFERENCES_FILE" | grep '^[[:space:]]*-[[:space:]]' | sed 's/^[[:space:]]*-[[:space:]]*//' | grep -v '^#'
}

echo "Reading preferences from: $PREFERENCES_FILE"

# Read configuration values
AUTO_MENU=$(read_yaml_value "auto_menu" "SINGLE_PLAYER")
MAP=$(read_yaml_value "map" "maps/BroodWar/(2)Benzene.scm")
PLAYER_RACE=$(read_yaml_value "player_race" "Zerg")
ENEMY_COUNT=$(read_yaml_value "enemy_count" "1")
GAME_TYPE=$(read_yaml_value "game_type" "MELEE")
AUTO_RESTART=$(read_yaml_value "auto_restart" "OFF")
SAVE_REPLAY=$(read_yaml_value "save_replay" "maps/replays/%BOTNAME6%/\$Y \$b \$d/%MAP%_%BOTRACE%%ALLYRACES%vs%ENEMYRACES%_\$H\$M\$S.rep")
WINDOWED=$(read_yaml_value "windowed" "OFF")
WINDOW_LEFT=$(read_yaml_value "window_left" "1556")
WINDOW_TOP=$(read_yaml_value "window_top" "704")
WINDOW_WIDTH=$(read_yaml_value "window_width" "640")
WINDOW_HEIGHT=$(read_yaml_value "window_height" "480")
SOUND=$(read_yaml_value "sound" "ON")
SCREENSHOTS=$(read_yaml_value "screenshots" "gif")
DROP_PLAYERS=$(read_yaml_value "drop_players" "ON")
SHARED_MEMORY=$(read_yaml_value "shared_memory" "ON")
HOLIDAY=$(read_yaml_value "holiday" "ON")
AI_MODULE=$(read_yaml_value "ai_module" "bwapi-data/AI/ExampleAIModule.dll")
AI_MODULE_DEBUG=$(read_yaml_value "ai_module_debug" "bwapi-data/AI/ExampleAIModuled.dll")
TOURNAMENT_MODULE=$(read_yaml_value "tournament_module" "")

# Optional settings
SPEED_OVERRIDE=$(read_yaml_value "speed_override" "")
SEED_OVERRIDE=$(read_yaml_value "seed_override" "")
CONSOLE_ATTACH_STARTUP=$(read_yaml_value "console_attach_on_startup" "FALSE")
CONSOLE_ALLOC_STARTUP=$(read_yaml_value "console_alloc_on_startup" "FALSE")
CONSOLE_ATTACH_AUTO=$(read_yaml_value "console_attach_auto" "TRUE")
CONSOLE_ALLOC_AUTO=$(read_yaml_value "console_alloc_auto" "TRUE")
WAIT_MIN_PLAYERS=$(read_yaml_value "wait_for_min_players" "2")
WAIT_MAX_PLAYERS=$(read_yaml_value "wait_for_max_players" "8")
WAIT_TIME=$(read_yaml_value "wait_for_time" "60000")
DOUBLE_SIZE=$(read_yaml_value "double_size" "ON")

# Read computer races array
COMPUTER_RACES=()
while IFS= read -r race; do
    [ -n "$race" ] && COMPUTER_RACES+=("$race")
done < <(read_yaml_array "computer_races")

# Set enemy races with defaults
ENEMY_RACE="${COMPUTER_RACES[0]:-Terran}"
ENEMY_RACE_1="${COMPUTER_RACES[0]:-Default}"
ENEMY_RACE_2="${COMPUTER_RACES[1]:-Default}"
ENEMY_RACE_3="${COMPUTER_RACES[2]:-Default}"
ENEMY_RACE_4="${COMPUTER_RACES[3]:-Default}"
ENEMY_RACE_5="${COMPUTER_RACES[4]:-Default}"
ENEMY_RACE_6="${COMPUTER_RACES[5]:-Default}"
ENEMY_RACE_7="${COMPUTER_RACES[6]:-Default}"

echo "Configuring BWAPI at: $BWAPI_INI_PATH"

# Create directory if it doesn't exist
mkdir -p "$(dirname "$BWAPI_INI_PATH")"

# Prepare conditional config lines
if [ -n "$SEED_OVERRIDE" ]; then
    SEED_LINE="seed_override = $SEED_OVERRIDE"
else
    SEED_LINE=";seed_override = 123456789"
fi

if [ -n "$SPEED_OVERRIDE" ]; then
    SPEED_LINE="speed_override = $SPEED_OVERRIDE"
else
    SPEED_LINE=";speed_override = -1"
fi

# Generate bwapi.ini content in one go
cat > "$BWAPI_INI_PATH" << EOF
[ai]
; Paths and revisions for AI
;   - Use commas to specify AI for multiple instances.
;   - If there are more instances than the amount of 
;         DLLs specified, then the last entry is used.
;   - Example: SomeAI.dll, SecondInstance.dll, ThirdInstance.dll
;   - Absolute paths are acceptable.
ai     = $AI_MODULE
ai_dbg = $AI_MODULE_DEBUG

; Used only for tournaments
; Tournaments can only be run in RELEASE mode
tournament = $TOURNAMENT_MODULE

[auto_menu]
; auto_menu = OFF | SINGLE_PLAYER | LAN | BATTLE_NET
; for replays, just set the map to the path of the replay file
auto_menu = $AUTO_MENU

; character_name = FIRST | WAIT | <other>
; if FIRST (default), use the first character in the list
; if WAIT, stop at this screen
; else the character with the given value is used/created
character_name = ProtossPest

; pause_dbg = ON | OFF
; This specifies if auto_menu will pause until a debugger is attached to the process.
; Only works in DEBUG mode.
pause_dbg = OFF

; lan_mode = Same as the text that appears in the multiplayer connection list
;            Examples: Local Area Network (UDP), Local PC, Direct IP
lan_mode = Local Area Network (UDP)

; auto_restart = ON | OFF
; if ON, BWAPI will automate through the end of match screen and start the next match
; if OFF, BWAPI will pause at the end of match screen until you manually click OK,
; and then BWAPI resume menu automation and start the next match
auto_restart = $AUTO_RESTART

; map = path to map to host relative to Starcraft folder, i.e. map = maps/(2)Boxer.scm
; leaving this field blank will join a game instead of creating it
; The filename(NOT the path) can also contain wildcards, example: maps/(?)*.sc?
; A ? is a wildcard for a single character and * is a wildcard for a string of characters
map = $MAP

; game = name of the game to join | JOIN_FIRST
;	i.e. game = BWAPI will join the game called "BWAPI"
;   and game = JOIN_FIRST will join the first game in the list.
;	If the game does not exist and the "map" entry is not blank, then the game will be created instead
;	If this entry is blank, then it will follow the rules of the "map" entry
game = 

; mapiteration =  RANDOM | SEQUENCE
; type of iteration that will be done on a map name with a wildcard
mapiteration = RANDOM

; race = Terran | Protoss | Zerg | Random
;   - Use commas to specify race for each AI module when running multiple instances.
;   - If there are more instances than the amount of 
;         races specified, then the last entry is used.
;	- To be used in conjunction with multiple AI modules
;   - Example: Terran, Protoss, Terran, Zerg
race = $PLAYER_RACE

; enemy_count = 1-7, for 1v1 games, set enemy_count = 1
; only used in single player games
enemy_count = $ENEMY_COUNT

; enemy_race = Terran | Protoss | Zerg | Random | RandomTP | RandomTZ | RandomPZ | RandomTPZ
; only used in single player games
enemy_race = $ENEMY_RACE

; enemy_race_# = Default
; Values for enemy_race are acceptable, Default will use the value specified in enemy_race
enemy_race_1 = $ENEMY_RACE_1
enemy_race_2 = $ENEMY_RACE_2
enemy_race_3 = $ENEMY_RACE_3
enemy_race_4 = $ENEMY_RACE_4
enemy_race_5 = $ENEMY_RACE_5
enemy_race_6 = $ENEMY_RACE_6
enemy_race_7 = $ENEMY_RACE_7

;game_type = TOP_VS_BOTTOM | MELEE | FREE_FOR_ALL | ONE_ON_ONE | USE_MAP_SETTINGS | CAPTURE_THE_FLAG
;           | GREED | SLAUGHTER | SUDDEN_DEATH | TEAM_MELEE | TEAM_FREE_FOR_ALL | TEAM_CAPTURE_THE_FLAG
game_type = $GAME_TYPE

; game_type_extra = Text that appears in the drop-down list below the Game Type drop-down list.
; If empty, the Starcraft default will be used.
; The following are the game types that use this setting, and corresponding example values
;   TOP_VS_BOTTOM          3 vs 1 | 2 vs 2 | 1 vs 3 | # vs #
;   GREED                  2500 | 5000 | 7500 | 10000
;   SLAUGHTER              15 | 30 | 45 | 60
;   TEAM_MELEE             2 | 3 | 4 | 5 | 6 | 7 | 8
;   TEAM_FREE_FOR_ALL      2 | 3 | 4 | 5 | 6 | 7 | 8
;   TEAM_CAPTURE_THE_FLAG  2 | 3 | 4 | 5 | 6 | 7 | 8
game_type_extra =

; save_replay = path to save replay to
; Accepts all environment variables including custom variables. See wiki for more info.
save_replay = $SAVE_REPLAY

; wait_for_min_players = #
; # of players to wait for in a network game before starting.
; This includes the BWAPI player. The game will start immediately when it is full.
wait_for_min_players = $WAIT_MIN_PLAYERS

; wait_for_max_players = #
; Start immediately when the game has reached # players.
; This includes the BWAPI player. The game will start immediately when it is full.
wait_for_max_players = $WAIT_MAX_PLAYERS

; wait_for_time = #
; The time in milliseconds (ms) to wait after the game has met the min_players requirement.
; The game will start immediately when it is full.
wait_for_time = $WAIT_TIME

[config]
; holiday = ON | OFF
; This will apply special easter eggs to the game when it comes time for a holiday.
holiday = $HOLIDAY

; shared_memory = ON | OFF
; This is specifically used to disable shared memory (BWAPI Server) in the Windows Emulator "WINE"
; Setting this to OFF will disable the BWAPI Server, default is ON
; MUST be ON for client bots to connect
shared_memory = $SHARED_MEMORY

; console_* = TRUE | FALSE
; Used for getting a console for displaying text written to stdout and stderr, and read from stdin.
; console_attach_*
;   Allows BWAPI to attach to the parent process' console. i.e. if the parent
;   has a console, output will be displayed on that console, and that console
;   also kept open even if the parent dies.
; console_alloc_*
;   Allows BWAPI to allocate it's own system console window. Not executed if
;   corresponding console_attach_* is enabled and succeeds.
; console_*_on_startup
;   Executes when BWAPI.dll is first attached to Starcraft.
; console_*_auto
;   Executes when something is written to std::cout or std::cerr,
;   and no console was successfully attached/allocated on startup.
console_attach_on_startup = $CONSOLE_ATTACH_STARTUP
console_alloc_on_startup = $CONSOLE_ALLOC_STARTUP
console_attach_auto = $CONSOLE_ATTACH_AUTO
console_alloc_auto = $CONSOLE_ALLOC_AUTO

[window]
; These values are saved automatically when you move, resize, or toggle windowed mode

; windowed = ON | OFF
; This causes BWAPI to enter windowed mode when it is injected.
windowed = $WINDOWED

; left, top
; Determines the position of the window
left = $WINDOW_LEFT
top  = $WINDOW_TOP

; width, height
; Determines the width and height of the client area and not the window itself
width  = $WINDOW_WIDTH
height = $WINDOW_HEIGHT

[starcraft]
; Game sound engine = ON | OFF
sound = $SOUND
; Screenshot format = gif | pcx | tga | bmp
screenshots = $SCREENSHOTS

; Random seed override. This uses a fixed seed at the start of the game so that if played out the exact same way,
; the same occurrences will happen every time. This value must be a decimal integer.
;
; When this key is commented out, Starcraft will use the system time as a seed. This is the default behaviour.
;
; Note: This option affects both single AND multi-player modes (for game hosts only). This means that hosting a multi-player
; game with this option enabled will distribute this fixed seed to all other players in the game.
$SEED_LINE

; Speed override. This overrides the default game speed setting and prevents bots from changing the game speed.
; Enabling this option causes it to take effect. The value is the number of milliseconds per frame. A negative
; value uses the game's default speed value.
$SPEED_LINE

; drop_players = ON | OFF
; This specifies if BWAPI should drop other players from the game when the timeout dialog reaches 0. Players 
; usually time out when there are connection issues or their client is not responding. Setting this to OFF
; will cause BWAPI to wait an infinite amount of time until the player reconnects.
drop_players = $DROP_PLAYERS
EOF




# Configure W-MODE to enable/disable 2x view based on preferences
WMODE_INI="${PROJECT_DIR}/starcraft/wmode.ini"
if [ -f "$WMODE_INI" ]; then
    if [ "$DOUBLE_SIZE" = "ON" ]; then
        sed -i 's/^DblSizeMode=.*/DblSizeMode=1/' "$WMODE_INI"
    else
        sed -i 's/^DblSizeMode=.*/DblSizeMode=0/' "$WMODE_INI"
    fi
    echo "Configured W-MODE for 2x view"
else
    echo "Creating wmode.ini with 2x view enabled"
    cat > "$WMODE_INI" << 'EOF'
[W-MODE]

; What settings do you want to save when you exit StarCraft?

; Save WindowClientX?
; Default value: 1
SaveWindowClientX=1
; Save WindowClientY?
; Default value: 1
SaveWindowClientY=1
; Save WindowClientXDblSized?
; Default value: 1
SaveWindowClientXDblSized=1
; Save WindowClientYDblSized?
; Default value: 1
SaveWindowClientYDblSized=1
; Save ClipCursor?
; Default value: 0
SaveClipCursor=0
; Save doublesize mode?
; Default value: 1
SaveDblSizeMode=1
; Save EnableWindowMove?
; Default value: 1
SaveEnableWindowMove=1
; Save AlwaysOnTop?
; Default value: 1
SaveAlwaysOnTop=1
; Save DisableControls?
; Default value: 1
SaveDisableControls=1

; X and Y coordinates of the StarCraft game screen.
; (Upper left corner of client area.)
; Default values: center the game screen on the desktop.
; If you don't specify the WindowClientX value then
; the window will be centered horizontally.
; Omitting WindowClientY will cause the window to
; be centered vertically.
WindowClientX=30
WindowClientY=30
; X and Y coordinates of the StarCraft game screen in
; doublesize mode.
WindowClientXDblSized=30
WindowClientYDblSized=30

; Cursor clip (Toggle hotkey: ALT+F1)
; Default value: 0
ClipCursor=0
; Doublesize mode (Toggle hotkey: ALT+F9)
; Default value: 0
DblSizeMode=1
; Enable window move (Toggle hotkey: ALT+F10)
; Default value: 1
EnableWindowMove=1
; Enable always-on-top mode (Toggle hotkey: ALT+F11)
; Default value: 0
AlwaysOnTop=0
; Disable all controls in the caption of the StarCraft
; window. Disable the screensaver when the sc window is
; active. Window can not be closed with ALT+F4.
; (Toggle hotkey: ALT+F12)
; Default value: 0
DisableControls=0

; Limit the maximum frame/sec to reach better performance.
; This is extremely useful because during replays with
; fastest x 4 speed the frame rate raises to the skies and
; MaxFps limits to 100 the number of blits that require
; 8bit -> desktop resolution conversion of the StarCraft
; screen image.
; Default and recommended value: 100
; You can set it to higher value on faster machines.
; Minimum value: 30 (less than 100 isn't recommended)
MaxFps=100

; Enables StarCraft to mute all sound when the main window
; loses focus.
; Default value: 0
MuteNotFocused=0
EOF
    echo "✓ Created wmode.ini"
fi
