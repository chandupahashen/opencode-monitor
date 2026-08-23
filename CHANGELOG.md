# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.5.5] - 2026-08-23

### Added
- **Auto-release via GitHub Actions** — push to `main`/`master` now lints, tests, builds signed MSI (`cargo tauri build --bundles msi`) and auto-creates GitHub Release `v<version>` with `*.msi`, `*.msi.sig`, `update.json`; `update.json` is also synced to `main` (`[skip ci]`) for old `1.5.2` clients using `raw.githubusercontent.com`
- **License & community docs** — `LICENSE` updated to `2025-2026 chandupahashen — OpenCode Monitor` (MIT), added `CONTRIBUTING.md` (fork/clone, project structure, DB notes, signing, Conventional Commits), `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1), `SECURITY.md` (supported `1.5.x`, private advisory flow), `package.json`/`Cargo.toml` `license`/`repository`/`homepage`/`bugs` metadata, and README badges (`License`, `Release`, `Build`, `Tauri`, `React`) + `Contributing`/`Security`/`Acknowledgments` sections
- **Updater process handling** — `tauri-plugin-process` (`2.3.x`), `capabilities/default.json` `updater:allow-download-and-install` + `process:allow-restart/exit`, Settings UI now shows download progress (`Started`/`Progress`/`Finished` events) and auto-`relaunch()` after install
- **Database: live message counts** — `get_session_detail` now queries `message` (45k rows) + `part` (212k rows) with `part` counts (was empty `Vec`), handling legacy `session_message`

### Changed
- **DB compatibility for opencode upgrade** — `session.model` is now JSON `{"id":"deepseek-v4-flash-free","providerID":"opencode","variant":"max"}` (was plain string); `src-tauri/src/db.rs:9-13` `model_key_expr()` uses `json_valid`/`json_extract($.id,$.modelID)` collapsing variants (e.g. `deepseek-v4-flash-free` 803 vs 4× split), `get_sessions` dual `model=? OR model_key=normalized`, `get_model_usage`/`get_models_list` `GROUP BY model_key` with fallback; `project` table is now authoritative — `get_project_stats` `LEFT JOIN project` (`worktree` → `project_name`/`directory: Option<String>`), `extract_project_name` prefers `worktree` basename; `open_db()` `busy_timeout(3s)` + best-effort `SCHEMA_VERSION=2` indexes (`message`, `part`)
- **Updater endpoints & CSP** — `tauri.conf.json:plugins.updater` dual endpoints `releases/latest/download/update.json` → `raw.githubusercontent.com/main/update.json`, `pubkey` unchanged, `windows.installMode: passive`, `dialog:false`, `bundle.createUpdaterArtifacts:true`, `security.csp` `connect-src 'self' https://raw.githubusercontent.com https://github.com https://objects.githubusercontent.com https://release-assets.githubusercontent.com`
- **Release versioning** — keep `package.json`/`src-tauri/Cargo.toml`/`src-tauri/tauri.conf.json` in sync (all `1.5.5`); `src/types/index.ts` `ProjectStat.directory` added, `src/pages/Projects.tsx:99` prefers `p.directory` else `projectDir()`

### Fixed
- **Updater not working** — fixed `update.json` empty `signature` (was `""` → verification failure), `ConvertTo-Json` default depth `2` truncating `platforms.windows-x86_64` → `-Depth 10` in `.github/workflows/build.yml:88` and `build.ps1:186`, missing `TAURI_SIGNING_PRIVATE_KEY` ↔ `TAURI_PRIVATE_KEY` sync (CI set `TAURI_PRIVATE_KEY` but CLI expected `TAURI_SIGNING_PRIVATE_KEY` and vice-versa), fallback `src-tauri/tauri-key.pem` loading, and Settings `handleInstallUpdate` double `check()` + missing `relaunch`
- **Build script parser crash (v1.5.4)** — `build.ps1:164` UTF-8 `—` mis-parsed as `�` → `The string is missing the terminator: "` + `Missing closing '}'`; replaced with `-`, fixed `[string]::IsNullOrWhiteSpace` null `Trim()` crash, and made unsigned builds warn-but-continue if MSI exists (`WARNING: Tauri build exited ... continuing`)
- **Frontend/CI** — added `@tauri-apps/plugin-process` (`2.3.0`) to `package.json`, `cargo check`/`npm run build` now pass; `Settings.tsx` imports `Update` type correctly

### Removed
- **PowerShell release scripts** — deleted `build.ps1` (209 lines) and `publish-github.ps1` (195 lines); all signing, `update.json` generation (`EscapeDataString` handling for `OpenCode-Monitor_*.msi`), and release creation now handled solely by `.github/workflows/build.yml` auto-release on `push`

## [1.5.4] - 2026-08-20

### Fixed
- `build.ps1` parser error (`—` → `-`) and `Cargo.toml` version drift (`1.5.3` vs `1.5.4`) — bumped `package.json`/`tauri.conf.json`/`Cargo.toml` to `1.5.4`

## [1.5.3] - 2026-08-19

### Added
- `tauri-plugin-process` (`2.3.1`) and updater `process:allow-restart` capability

### Fixed
- Updater `tauri.conf.json` now succeeds when `TAURI_PRIVATE_KEY` is set via CI secrets (synced to `TAURI_SIGNING_PRIVATE_KEY`) and `Cargo.lock` `tauri-plugin-process` added

## [1.5.2] - 2026-06-16

### Features
- App updater with automatic update detection (Settings → Updates)
- Theme selector (Dark/System/Light) now functional

### Bug Fixes
- Fixed UTF-8 panic when project names contain multi-byte characters
- Fixed mutex poisoning vulnerability across all Tauri commands
- Fixed incorrect LOCALAPPDATA database path detection
- Health check now verifies actual DB connectivity, not just file existence
- Added error handling for database path save in Settings

### Code Quality
- Extracted shared `formatCost`/`formatTokens` utilities (removed 16 duplicates)
- Cached `decodeModel()` calls in Sessions table (was called 4× per row)
- Consolidated store updates into single `setState` call
- Added `AppState` helper methods to eliminate repeated mutex boilerplate

### Infrastructure
- Updated GitHub Actions to Node 24 runtime
- Added updater signing and manifest generation to CI/CD
- Added signing keys to `.gitignore`

## [0.1.0] - Initial Release

- Dashboard with overview KPIs (sessions, tokens, cost, active sessions)
- Sessions page with filtering by model, project, date range
- Analytics with token trends, model usage, cost breakdown, daily activity
- Per-project statistics
- Settings with database path configuration and auto-refresh interval
- Dark theme with glass-morphism UI
- Custom titlebar with window controls
