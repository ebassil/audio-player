## Context

The audio player is a Tauri v2 desktop application targeting macOS initially. Audio processing happens entirely in Rust — the webview is a pure UI layer. The user wants a Winamp-like local music player with a programmable mixing pipeline, WASM plugin ecosystem, and keyboard-driven workflow. No library database; playlists are the persistence mechanism.

**Tauri v2 Compliance:** All Tauri integration follows the standards defined in [ADR-006](../adr/ADR-006-tauri-v2-standards.md). This includes the capability-based permission system, split main.rs/lib.rs Rust entry-point pattern, v2 configuration schema (windows inside `app.windows`, plugins at top-level), and frontend imports from `@tauri-apps/plugin-*` packages rather than legacy `@tauri-apps/api/*` paths.

## Goals / Non-Goals

**Goals:**
- Audio decode (MP3, WAV, FLAC, OGG, AAC) via symphonia in Rust
- Configurable audio graph with pre/post processing via fundsp
- WASM-based plugin loading via wasmtime with WIT interface
- Per-song mix-in/mix-out points with multiple mix patterns (fade, cross-fade, hard fade)
- Dual-format playlists (.json extended, .m3u8 compat)
- Global keyboard shortcuts with session-level toggles
- Plugin discovery via manifest files from plugins/ directory
- Plugin UI hosted in webview (HTML/JS per plugin)

**Non-Goals:**
- Music library database (SQLite) — playlists are the persistence layer
- Streaming service integration
- Mobile/tablet support (desktop only, macOS first)
- Plugin marketplace or package manager
- Audio recording or capture

## Decisions

### Audio Pipeline Entirely in Rust
**Decision:** symphonia → fundsp → cpal, no Web Audio API involvement. Rust commands use `tauri::generate_handler![]` in `lib.rs::run()` following the v2 split entry-point pattern (main.rs calls `app_lib::run()`).
**Rationale:** Avoids heavy IPC of PCM buffers across Tauri boundaries. fundsp provides a pull-based audio graph that composes naturally. cpal handles OS-level audio output. Webview sends only lightweight control commands (play, pause, volume, etc.) via `@tauri-apps/api/core`'s `invoke`.
**Alternatives considered:** Web Audio in webview with PCM bridged over IPC — rejected due to latency and bandwidth concerns.

### WASM Plugins via wasmtime with WIT Interface
**Decision:** Define a WIT interface with `init(sample_rate, channels)` and `process(input, output)` functions. Plugins compile to `.wasm` components. Rust host loads via wasmtime, wraps as fundsp nodes.
**Rationale:** Allows arbitrary high-performance DSP without recompiling the app. WIT provides a strict, versioned contract. Rust/C/Zig authors can target WASM. fundsp impedance mismatch solved with an adapter node that buffers pull input into block-based WASM calls.
**Alternatives considered:** Lua plugins — simpler but lower performance for DSP. Native dylib plugins — faster but platform-specific and no sandboxing.

### Plugin Manifest Approach
**Decision:** Each plugin is a directory with `plugin.json` manifest, `.wasm` binary, and optional `ui/` folder with HTML/JS.
**Rationale:** Self-contained, discoverable via directory scan. HTML/JS UI loads in webview iframe and communicates parameter changes via Tauri IPC to Rust.
**Alternatives considered:** Single-file plugins — less flexible for UI. Registry-based — adds server dependency.

### Dual-Format Playlists (.json + .m3u8)
**Decision:** Primary format is `.json` containing file paths, mix points, mix patterns, metadata. Optional `.m3u8` for cross-player compatibility.
**Rationale:** `.json` stores extended features (mix-in/mix-out points, per-song plugin overrides). `.m3u8` is human-readable and importable by other players. No library DB keeps state management simple. Deleted files removed from both.
**Alternatives considered:** SQLite library — adds complexity of DB migrations and sync. Single `.json` — works but less compatible.

### Global Shortcuts via Tauri Plugin
**Decision:** Use `tauri-plugin-global-shortcut` (v2 npm/ Cargo package) with unique chords (e.g., Ctrl+Alt+Shift+<key>) to avoid macOS conflicts. Shortcuts persist in config file, editable from UI. The capability `global-shortcut:default` must be declared in `src-tauri/capabilities/default.json`.
**Rationale:** Global shortcuts work even when app is backgrounded. Unique chords prevent system conflicts. Session flags (like confirm-delete toggle) reset on app restart. The v2 capability system provides explicit, auditable permission control.
**Alternatives considered:** OS-level shortcut registration with NSApplication — more complex, no cross-platform benefit.

### Delete vs DeletePlus
**Decision:** Delete removes from playlist only (no confirmation). DeletePlus removes from playlist and disk (confirmation dialog with session-level "don't ask again" checkbox).
**Rationale:** Clear distinction between non-destructive and destructive actions. Session toggle for power users who want fast destructive workflow.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| WASM→native call overhead per audio block | Benchmark with representative plugins; consider block size tuning (256-1024 samples). For CPU-heavy FFT plugins, native compilation alternative could be added later. |
| fundsp pull model ↔ WASM push model adapter complexity | Build a buffered adapter node that collects pull requests and dispatches push calls in blocks. Test with silence/impulse to verify correctness. |
| Global shortcut conflicts on macOS | Document known-conflicting combos. Provide in-app feedback when registration fails. Use multi-modifier chords. |
| No library DB means no search/index across playlists | Search is scoped to current playlist. Cross-playlist search could be added as a future capability if needed. |
| Plugin security — WASM modules access PCM buffers | wasmtime provides sandboxing by default. No file system or network access for plugins. Consider adding memory limits. |
| Multiple audio formats require multiple symphonia decoders | symphonia handles many formats with feature flags. Start with MP3 + WAV + FLAC, extend as needed. |
