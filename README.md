# openCode Monitor

A desktop dashboard for monitoring and analyzing your [opencode](https://opencode.ai) AI assistant usage. Built with Tauri v2, React, Rust, and SQLite.

## Features

- **Dashboard** — Overview KPIs: total sessions, tokens, cost, active sessions
- **Sessions** — Browse session history with filters by model, project, and date range
- **Analytics** — Token trends, model usage breakdown, cost breakdown, daily activity heatmap
- **Projects** — Per-project token and cost statistics
- **Settings** — Database path configuration and auto-refresh interval

## Tech Stack

| Layer   | Technology                                   |
| ------- | -------------------------------------------- |
| Frontend | React 19, TypeScript, Vite, Tailwind CSS v4 |
| Backend  | Tauri v2, Rust                               |
| Database | SQLite (via rusqlite)                        |
| Charts   | Recharts                                     |
| State    | Zustand                                      |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) v22+
- [Rust](https://rustup.rs/) with `cargo`
- [Tauri CLI](https://v2.tauri.app/start/cli/)

```bash
cargo install tauri-cli --version "^2.0"
```

### Development

```bash
# Install dependencies
npm install

# Start dev server with hot-reload
npm run tauri dev
```

### Production Build

```bash
# Full build (lint + typecheck + test + MSI)
.\build.ps1 -Configuration release

# Or step by step:
npm run build
cargo tauri build --bundles msi
```

### Publishing a Release

```bash
.\publish-github.ps1 -Version 0.2.0
```

## Database

openCode Monitor reads from the `opencode.db` SQLite database created by the openCode CLI. By default it looks in:

- **Windows:** `%APPDATA%/opencode/opencode.db`
- **Linux/macOS:** `~/.local/share/opencode/opencode.db`

You can override the path in Settings.

## License

MIT
