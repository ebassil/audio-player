## Context

The app currently initializes a new, empty `Playlist::new("Default")` on every launch in `run()`. Existing `save_app_config` / `load_app_config` already demonstrates the pattern of persisting state to `app_config_dir()` using TOML serialization. The `Playlist` struct implements `Serialize`/`Deserialize`, so JSON round-tripping is trivial. There is currently no mechanism to persist or restore the active playlist (tracks, current track index, playback position) across restarts.

## Goals / Non-Goals

**Goals:**
- Automatically save the active playlist state (tracks) to disk whenever it changes
- Automatically save the current track index and playback position when they change
- On startup, restore the last active playlist automatically into the `AppState.playlist` and `AudioPipeline` state
- Use the existing `app_config_dir()` pattern for storage — no new config directories or save dialogs
- Keep persistence transparent to the user (no UI changes required)

**Non-Goals:**
- Persisting the entire decoder/buffer state (we save file paths, not decoded audio)
- Saving named/saved playlist files — this is purely about the "working" playlist
- Frontend-side persistence — all logic lives in the Rust backend
- Migration of existing data formats

## Decisions

1. **Storage format: JSON over TOML**
   - `Playlist` already derives `Serialize`/`Deserialize` for JSON via serde. Reusing JSON avoids an extra TOML schema and lets us serialize the exact same `Playlist` struct as is used for file save/load.
   - A separate small struct (`PlaylistState`) wraps the playlist alongside `current_track_index` and `position_secs` so we save one file instead of two.

2. **File location: `app_config_dir() / playlist_state.json`**
   - Matches the existing pattern (`app.toml`, `shortcuts.toml`).
   - No need for user-visible path selection.

3. **Save trigger: explicit calls on mutation**
   - Rather than a background watcher, call `save_playlist_state()` at the end of every Tauri command that mutates playlist tracks, the current track index, or position.
   - This is simpler, deterministic, and avoids threading complexity with file watchers.

4. **Restore timing: during `run()` after `AppState` construction, before `tauri::Builder` setup**
   - Load the persisted file, parse it, and replace the default `Playlist` in `AppState` and the `current_track_index` in `AudioPipeline`.
   - If the file is missing or corrupt, fall back to the current behavior (empty "Default" playlist).

5. **Separate module: `playlist_persist.rs`**
   - Encapsulates save/load logic cleanly rather than bloating `lib.rs` or `playlist.rs`.

## Risks / Trade-offs

- [Risk] Saving on every mutation adds minor latency to playlist commands.
  → Mitigation: File writes are tiny (< 100 KB for a large playlist) and synchronous on a short-lived Mutex lock. Acceptable for this use case.
- [Risk] Corrupt save file could prevent startup.
  → Mitigation: Load gracefully returns `None` on any error (missing file, parse error, I/O error), falling back to the default empty playlist.
- [Risk] Position persistence may be slightly stale (last saved position, not real-time).
  → Mitigation: Position is saved alongside track index when the track changes, and can be saved periodically or on pause/stop. Exact timing is a frontend concern — the backend provides the mechanism.
