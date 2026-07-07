## 1. Project Scaffolding

> **ADR-006 Compliance:** All scaffolding must follow the v2 standards — capability permissions in `src-tauri/capabilities/default.json`, v2 config schema (`app.windows[]`), split `main.rs`/`lib.rs` entry-point, and plugin imports from `@tauri-apps/plugin-*`.

- [x] 1.1 Initialize Tauri v2 project with TypeScript frontend template (per ADR-006 schema)
- [x] 1.2 Configure Rust dependencies: symphonia, fundsp, cpal, wasmtime, serde, serde_json, toml
- [x] 1.3 Configure Tauri plugins and declare capability permissions: global-shortcut, dialog, fs in `src-tauri/capabilities/default.json`
- [x] 1.4 Set up project directory structure (src-tauri/src/, plugins/, src/ for webview)
- [x] 1.5 Create configurations directory for shortcuts and app settings

## 2. Audio Pipeline

- [x] 2.1 Implement audio decoder module using symphonia supporting MP3, WAV, FLAC
- [x] 2.2 Build fundsp audio graph with configurable pre/post processing chains
- [x] 2.3 Implement cpal audio output with device enumeration and change handling
- [x] 2.4 Implement volume control node in fundsp graph (gain, mute)
- [x] 2.5 Wire decode → graph → output into a continuous playback pipeline
- [x] 2.6 Handle playback states: playing, paused, stopped, seeking

## 3. Plugin System

- [ ] 3.1 Define WIT interface for audio plugins (init, process, reset)
- [ ] 3.2 Build wasmtime host that loads, instantiates, and calls WASM plugins
- [ ] 3.3 Implement fundsp adapter node that bridges pull graph → push WASM with buffering
- [ ] 3.4 Build plugin manifest parser (plugin.json discovery from plugins/ directory)
- [ ] 3.5 Implement plugin scanning on startup and register discovered plugins
- [ ] 3.6 Build IPC bridge for plugin UI ↔ Rust parameter communication
- [ ] 3.7 Implement plugin sandboxed UI loader in webview (iframe/frame)
- [ ] 3.8 Implement plugin enable/disable and reordering in UI and graph

## 4. Mixing Engine

- [ ] 4.1 Implement fade mix pattern (gain ramp out, pause, gain ramp in)
- [ ] 4.2 Implement cross-fade mix pattern (gain ramp out + in with overlap)
- [ ] 4.3 Implement hard fade mix pattern (silence gap between tracks)
- [ ] 4.4 Implement configurable default mix duration
- [ ] 4.5 Implement per-song mix-in/mix-out point resolution and override logic
- [ ] 4.6 Build UI for setting mix points on a waveform/timeline
- [ ] 4.7 Wire mix engine into playback pipeline between pre-fx and post-fx chains

## 5. Playlist Management

- [ ] 5.1 Define JSON playlist schema and implement save/load
- [ ] 5.2 Implement M3U8 playlist export and import
- [ ] 5.3 Implement drag-drop handler for files and folders
- [ ] 5.4 Implement directory loading dialog (recursive scan with filtering)
- [ ] 5.5 Implement Delete (playlist only, no confirmation)
- [ ] 5.6 Implement DeletePlus (playlist + disk, confirmation dialog with session toggle)
- [ ] 5.7 Build playlist view in UI (track listing, selection, context menu)

## 6. Shortcut System

- [ ] 6.1 Define default shortcut bindings config file
- [ ] 6.2 Implement shortcut engine that reads config and registers global hotkeys
- [ ] 6.3 Integrate tauri-plugin-global-shortcut for background operation (requires `global-shortcut:default` capability in `src-tauri/capabilities/default.json`)
- [ ] 6.4 Build shortcut configuration UI (rebind, conflict detection)
- [ ] 6.5 Implement session-level toggle system (confirm-delete flag with reset on restart)
- [ ] 6.6 Implement extensible action registry for future shortcut actions

## 7. Tauri IPC Layer

> **ADR-006 Compliance:** Commands must be registered via `tauri::generate_handler![]` in `lib.rs::run()` (split entry-point). Frontend calls use `invoke` from `@tauri-apps/api/core`. Event system uses `@tauri-apps/api/event`.

- [ ] 7.1 Define all Tauri IPC commands (play, pause, next, prev, volume, load_playlist, etc.)
- [ ] 7.2 Implement command handlers in Rust backend (return `Result` for robust error handling)
- [ ] 7.3 Build event system for Rust → webview state updates (current track, time, status)

## 8. Webview UI

- [ ] 8.1 Build main layout (playlist panel, player controls, plugin rack, timeline)
- [ ] 8.2 Implement playback controls (play/pause, next/prev, seek, volume)
- [ ] 8.3 Build playlist view with track list, selection, drag-reorder
- [ ] 8.4 Build plugin rack UI (list, enable/disable, reorder, parameter controls)
- [ ] 8.5 Build settings panel (shortcuts, mix defaults, audio device)
- [ ] 8.6 Build timeline/waveform view with mix point editing

## 9. Configuration & Persistence

- [ ] 9.1 Implement app config file (TOML) for settings, shortcuts, plugin state, mix defaults
- [ ] 9.2 Implement config UI binding (read on load, write on save)
- [ ] 9.3 Ensure config survives app restarts

## 10. Testing & Polish

- [ ] 10.1 Write integration tests for audio pipeline (decode → process → output)
- [ ] 10.2 Write integration tests for playlist save/load round-trip (JSON + M3U8)
- [ ] 10.3 Write integration tests for shortcut registration and dispatch
- [ ] 10.4 Write plugin SDK documentation and provide example plugin
- [ ] 10.5 Test on macOS with various audio devices and formats
- [ ] 10.6 Implement graceful error handling for missing files, failed plugins, device changes
