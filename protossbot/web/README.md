# Protoss Bot Web Control Panel

A Leptos web interface to control the Protoss bot in real-time.

## Setup

1. Install cargo-leptos:
```bash
cargo install cargo-leptos
```

2. Run the development server:
```bash
cd web
cargo leptos watch
```

3. Open your browser to `http://localhost:3000`

## Features

- **Game Speed Control**: Set the desired game speed with convenient buttons
- Real-time updates via server functions
- Dark theme matching StarCraft aesthetics

## Integration with Bot

The web server exposes a global `GAME_SPEED` variable that can be read by the bot. To integrate:

1. Add the web crate as a dependency in your bot's `Cargo.toml`
2. Read the speed value: `protoss_bot_web::GAME_SPEED.read().unwrap()`
3. Apply it to the game using BWAPI's setLocalSpeed() or setFrameSkip()
