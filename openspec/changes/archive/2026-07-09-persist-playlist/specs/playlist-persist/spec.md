# Playlist Persist

Automatically persist the active playlist state (tracks, current track index, playback position) to disk and restore it on application startup.

## Requirements

### R1: Auto-save on track mutations
When tracks are added, removed, reordered, or replaced via any Tauri command (`set_playlist_tracks`, `load_playlist`, `import_m3u8`, `remove_tracks_from_playlist`), the backend must write the current `Playlist` state to a persistent file.

### R2: Auto-save on track index changes
When the current track index changes (via `set_current_track_index` or auto-advance in the pipeline), the backend must persist the updated index alongside the playlist tracks.

### R3: Startup restore
On application startup, the backend must:
- Check for a persisted playlist state file
- If found and valid: restore it as the active playlist and set the current track index
- If missing or invalid: start with an empty "Default" playlist (current behavior)

### R4: Storage location
The persisted state file must be stored in the application's config directory (`app_config_dir()`) as `playlist_state.json`.

### R5: Graceful degradation
If the persisted state file cannot be read (missing, corrupt, I/O error), the application must start normally with an empty playlist. The error must be silently handled — no crash or user-visible error.

### R6: No user UI for persistence
The auto-save and restore must be transparent to the user. No new UI elements, dialogs, or settings are required.

### R7: Idempotent saves
Repeated saves with the same state must not cause errors or side effects.

## Data Model

### PlaylistState (new struct)

```rust
struct PlaylistState {
    playlist: Playlist,
    current_track_index: Option<usize>,
}
```

Serialized as JSON to `{app_config_dir}/playlist_state.json`.

## Error Handling

| Scenario | Behavior |
|----------|----------|
| File not found on startup | Start with empty "Default" playlist |
| Corrupt JSON on startup | Log a warning, start with empty playlist |
| I/O error during save | Silently fail (log if possible) |
| I/O error during load | Log a warning, start with empty playlist |
