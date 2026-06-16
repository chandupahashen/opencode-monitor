# Changelog

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
