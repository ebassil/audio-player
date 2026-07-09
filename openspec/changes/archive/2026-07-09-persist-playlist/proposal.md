## Why

Every time the app reloads, the playlist resets to an empty "Default" state. Users lose their current track lineup even when they had an active unsaved playlist. This breaks flow for anyone who spends time curating a listening session.

## What Changes

- Persist the active playlist (tracks, current track index, and playback position) automatically to the app config directory whenever tracks or position changes
- On startup, automatically restore the last persisted playlist into the active playlist state
- Add a new Tauri command to fetch the persisted playlist path for the frontend (if needed)
- The persisted playlist is transparent to the user — no save dialog, no file management

## Capabilities

### New Capabilities
- `playlist-persist`: Automatic persistence and restoration of the active playlist state across application restarts

### Modified Capabilities
- *(none — no existing spec requirements are changing)*

## Impact

- **New file**: `src-tauri/src/audio/playlist_persist.rs` — module for auto-save/restore logic
- **Modified**: `src-tauri/src/audio/mod.rs` — add module declaration
- **Modified**: `src-tauri/src/lib.rs` — hook auto-save on playlist mutations, load persisted playlist in `run()` / `setup()`
- **Modified**: `src-tauri/src/audio/playlist.rs` — may need a helper to produce a "snapshot" that includes current-track metadata
- Uses existing `app_config_dir()` for storage (already established pattern)
- No new external dependencies
