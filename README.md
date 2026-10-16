# OpenCode Monitor

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/chandupahashen/opencode-monitor?label=release)](https://github.com/chandupahashen/opencode-monitor/releases)
[![Build](https://github.com/chandupahashen/opencode-monitor/actions/workflows/build.yml/badge.svg)](https://github.com/chandupahashen/opencode-monitor/actions/workflows/build.yml)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?logo=tauri)](https://v2.tauri.app)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)](https://react.dev)

A desktop dashboard for monitoring and analyzing your [opencode](https://opencode.ai) AI assistant usage. Built with Tauri v2, React, Rust, and SQLite.

> **New in 1.6.0:** Full **opencode v2** support — reads `session_v2` / `session_message` (with `session` / `message` / `part` fallback for v1), `OPENCODE_DB` override, `provider/model#variant` model refs, and fork `slug` / `parent_id` display.

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
pnpm install

# Start dev server with hot-reload
pnpm tauri dev
```

## Database

OpenCode Monitor reads from the `opencode.db` SQLite database created by the OpenCode CLI. Detection order:

1. `OPENCODE_DB` env var (v2 documented override)
2. `~/.local/share/opencode/opencode.db` (`$XDG_DATA_HOME` respected)
3. `%USERPROFILE%/.local/share/opencode/opencode.db` (Windows v2 location)
4. `%APPDATA%/opencode/opencode.db` and `%LOCALAPPDATA%/opencode/opencode.db` (legacy)

You can override the path in Settings.

Schema support:

- **opencode v2** — `session_v2` (with `slug`, `parent_id`/`fork_session_id`, `directory`, token/cost columns) + `session_message` (`type`/`seq`/`data` JSON, ordered by `seq`). Message `parts_count` is derived from `json_array_length(data, '$.content')`.
- **opencode v1 fallback** — legacy `session` + `message`/`part` tables still work when `session_v2` is absent.
- **Models** — JSON `{"id":"...","providerID":"opencode","variant":"xhigh"}` plus string refs `provider/model#variant` and `model#variant` (variant stripped for grouping, shown in the UI).
- **Projects** — `project.worktree` + `name` (nullable in v2, falls back to worktree basename); date filters are `s.time_created`-qualified to avoid ambiguity with `project.time_created`.
- The app survives a locked DB via `busy_timeout(3s)` and creates its helper indexes best-effort.

## Contributing

We welcome contributions — see [CONTRIBUTING.md](CONTRIBUTING.md) for how to get started and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for our community standards.

Quick start for contributors:

```bash
git clone https://github.com/chandupahashen/opencode-monitor.git
cd opencode-monitor
pnpm install
pnpm tauri dev
```

Please read `CONTRIBUTING.md` for project structure, DB notes, signing/updater details, and our Conventional Commits + PR checklist.

## Security

If you find a security issue, please open a private security advisory on GitHub or contact the maintainer directly rather than filing a public issue.

## Acknowledgments

Built on [Tauri](https://v2.tauri.app), [React](https://react.dev), [Rust](https://www.rust-lang.org), and [opencode](https://opencode.ai). Thanks to all contributors.

## License

[MIT](LICENSE) © 2025-2026 chandupahashen — see [LICENSE](LICENSE) for details.
