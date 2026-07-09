## 1. Persistence Module

- [x] 1.1 Create `src-tauri/src/audio/playlist_persist.rs` with `PlaylistState` struct (wraps `Playlist` + `current_track_index`), `save()` and `load()` functions
- [x] 1.2 Add `mod playlist_persist` to `src-tauri/src/audio/mod.rs`

## 2. Core Integration

- [x] 2.1 In `src-tauri/src/lib.rs` `run()`: after initializing `AppState`, attempt to load persisted playlist state; if found, replace the default playlist and set `current_track_index` on the pipeline
- [x] 2.2 Add `save_playlist_state` helper in `lib.rs` (or call into `playlist_persist`) that serializes the current `AppState.playlist` + `pipeline.current_track_index` to `app_config_dir()/playlist_state.json`

## 3. Auto-save Hooks

- [x] 3.1 Hook `set_playlist_tracks` command to call `save_playlist_state` after updating tracks
- [x] 3.2 Hook `load_playlist` command to call `save_playlist_state` after replacing playlist
- [x] 3.3 Hook `import_m3u8` command to call `save_playlist_state` after importing
- [x] 3.4 Hook `remove_tracks_from_playlist` command to call `save_playlist_state` after removal
- [x] 3.5 Hook `set_current_track_index` command to call `save_playlist_state` after updating index

## 4. Build & Test

- [x] 4.1 Run `cargo build` and fix any compilation errors
- [x] 4.2 Run existing tests (`cargo test`) to confirm no regressions
- [x] 4.3 Manual smoke test: add tracks, close app, reopen — verify playlist is restored
