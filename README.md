# Game Library

A Tauri desktop application for managing and launching your personal game collection from a single place.

<img width="3792" height="2091" alt="image" src="https://github.com/user-attachments/assets/0493f452-0a80-4c6c-88c5-d97d4e69d4e5" />

## Features

- **Steam Integration** — auto-detects your local Steam installation and lists all installed games
- **Custom Games** — add any non-Steam executable with a title, cover image, and tags
- **Visual Library** — grid/card layout with cover art, platform badges, search, and filters
- **One-click Launch** — launch Steam games via `steam://run/<appid>` or run any custom binary
- **Controller Support** — full gamepad navigation using the browser Gamepad API

## Tech Stack

| Layer | Technology |
|---|---|
| Frontend | Vue 3 + TypeScript + Vite |
| Styling | Tailwind CSS |
| Desktop shell | Tauri 2 |
| Backend | Rust |

## Project Structure

```
game-library/
├── src/                        # Vue frontend
│   ├── components/
│   │   ├── GameCard.vue        # Individual game card (cover, title, badge)
│   │   ├── GameGrid.vue        # Responsive grid of GameCards
│   │   └── Sidebar.vue         # Search, filter, sort controls
│   └── App.vue                 # Root component
├── src-tauri/                  # Rust/Tauri backend
│   └── src/
│       ├── main.rs             # Entry point
│       ├── lib.rs              # Tauri command registration
│       ├── steam.rs            # Steam VDF parsing & game discovery
│       ├── library.rs          # Custom game CRUD (JSON persistence)
│       └── launcher.rs         # Game process spawning
└── CLAUDE.md                   # Project plan
```

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+
- [Tauri CLI prerequisites](https://tauri.app/start/prerequisites/) for your OS
- Steam installed (optional — required for Steam game discovery)

## Getting Started

```bash
# Install frontend dependencies
yarn install

# Run in development mode (hot-reload frontend + Rust backend)
yarn tauri dev

# Build a production release
yarn tauri build
```

## Tauri Commands

| Command | Description |
|---|---|
| `get_steam_games` | Returns all installed Steam games detected on the system |
| `get_custom_games` | Returns all manually added custom games |
| `add_game` | Adds a new custom game entry |
| `remove_game` | Removes a custom game by ID |
| `launch_game` | Launches a game (Steam URI or direct executable) |

## Development

### Running Tests

```bash
# Rust unit tests
cd src-tauri && cargo test
```

### Adding a Custom Game

Custom game entries are stored in the Tauri app data directory as `custom_games.json`.
