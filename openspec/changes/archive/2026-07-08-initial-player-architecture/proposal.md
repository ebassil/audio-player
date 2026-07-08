## Why

Modern music players fall into two camps: lightweight players with no processing pipeline (Winamp, Audacious) or heavyweight library-managed players (Spotify, Music.app). There's a gap for a local-file player with a programmable mixing pipeline, per-song mix points, and an open plugin ecosystem — a Winamp for the modern era that treats audio processing as a first-class citizen.

## What Changes

- **Audio Pipeline in Rust**: symphonia for decoding → fundsp audio graph for mixing/effects → cpal for output. Webview is pure UI.
- **Plugin System**: WASM-based DSP plugins loaded at runtime via wasmtime. Plugins define `init()` and `process()` via WIT interface. Discovered from a `plugins/` directory with manifest files. Plugin UIs load as HTML/JS in the webview.
- **Mixing Engine**: Per-song mix-in/mix-out points. Mix patterns: fade, cross-fade, hard fade. Configurable default mix duration.
- **Playlist Management**: Dual-format playlists — `.json` (extended, includes mix points and metadata) and `.m3u8` (compatibility). No library database. Drag-drop and directory-scan produce playlists.
- **Shortcut System**: Configurable keyboard shortcuts with session-level toggles (e.g., delete confirmation). Global shortcut registration via Tauri, avoiding OS conflicts.
- **Delete Actions**: Delete (playlist only, no confirmation). DeletePlus (playlist + disk removal, confirmation with session toggle).
- **Tauri v2** cross-platform shell following [ADR-006](../adr/ADR-006-tauri-v2-standards.md) — capability-based permissions in `src-tauri/capabilities/default.json`, v2 configuration schema (`app.windows[]`), split `main.rs`/`lib.rs` entry-point, and frontend imports from `@tauri-apps/plugin-*` packages. Initial target: macOS.

## Capabilities

### New Capabilities
- `audio-pipeline`: Audio decoding, processing graph, and output. The core engine running in Rust.
- `plugin-system`: WASM-based plugin loading, WIT interface, manifest discovery, and plugin UI hosting in webview.
- `mixing-engine`: Per-song mix points, mix patterns, configurable durations. The DSP for transitions.
- `playlist-management`: JSON/M3U8 playlist format, drag-drop and directory import, save/load without library DB.
- `shortcut-system`: Config file + UI shortcut binding, global hotkeys, session-level toggle flags.

### Modified Capabilities
<!-- No existing specs to modify — this is the initial architecture. -->

## Impact

- New Rust dependencies: `tauri` 2.0, `symphonia`, `fundsp`, `cpal`, `wasmtime`
- New Tauri plugins: `tauri-plugin-global-shortcut`, `tauri-plugin-dialog`, `tauri-plugin-fs` (added via `npx tauri plugin add`, permissions declared in `src-tauri/capabilities/default.json`)
- Webview becomes a pure UI layer — no audio processing in JavaScript/TypeScript
- Plugin development requires WIT interface compliance and WASM compilation toolchain
- No SQLite / library database — state lives in playlists and config files
