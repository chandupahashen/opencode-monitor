# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.5.x   | :white_check_mark: |
| < 1.5   | :x:                |

Only the latest `1.5.x` release line is actively supported. Please update to the latest release via **Settings → Updates** or from the [Releases](https://github.com/chandupahashen/opencode-monitor/releases) page.

## Reporting a Vulnerability

If you discover a security vulnerability, please **do not** open a public issue.

Instead:

1. Use GitHub's **Report a vulnerability** → **Security advisories** on this repository (private), or
2. Contact the maintainer directly via the email listed on https://github.com/chandupahashen

Include:

- Description of the vulnerability
- Steps to reproduce
- Impact assessment
- Any suggested fix (optional)

You will receive an acknowledgement within 72 hours and a follow-up with the remediation timeline. We will credit you in the release notes if you wish.

## Updater & Signing

Releases are signed with a minisign private key (`tauri-key.pem` — never committed). The public key is pinned in `src-tauri/tauri.conf.json:plugins.updater.pubkey` and `src-tauri/tauri-key.pem.pub`. If signature verification fails (`Settings → Check for Updates` shows a signature error), please ensure you are on an official GitHub release artifact and that `update.json` was fetched from `https://github.com/chandupahashen/opencode-monitor/releases/latest/download/update.json`.

## Database Access

OpenCode Monitor opens `opencode.db` **read-only** with `busy_timeout(3s)`. It never writes to the database except for best-effort index creation (`user_version` migrations). No credentials are stored.

Thank you for helping keep the project safe.
