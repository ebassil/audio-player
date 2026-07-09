## Purpose

Automatically persist the active playlist state (tracks, current track index, playback position) to disk and restore it on application startup.

## Requirements

### Requirement: Auto-save on track mutations

When tracks are added, removed, reordered, or replaced via any Tauri command (`set_playlist_tracks`, `load_playlist`, `import_m3u8`, `remove_tracks_from_playlist`), the backend must write the current `Playlist` state to a persistent file. This SHALL include per-track mix overrides (pattern, duration, and mix points).

#### Scenario: Per-track mix overrides included in save
- **WHEN** a track is added with `mix_pattern_override`, `mix_duration_override`, and `mix_points`
- **THEN** all three override fields SHALL be persisted in the playlist state JSON
- **AND** restored on next application startup

### Requirement: Save on per-track mix override change

When the user modifies per-track mix pattern, mix duration, or mix points for any track in the playlist, the system SHALL auto-save the playlist state to persist the new overrides.

#### Scenario: Per-track mix override triggers auto-save
- **WHEN** the user changes the mix pattern override for a track via the main screen
- **THEN** the playlist state SHALL be saved to disk immediately
- **AND** the persisted value SHALL match the user's selection

### Requirement: Auto-save on track index changes

When the current track index changes (via `set_current_track_index` or auto-advance in the pipeline), the backend must persist the updated index alongside the playlist tracks.

### Requirement: Startup restore

On application startup, the backend must:
- Check for a persisted playlist state file
- If found and valid: restore it as the active playlist and set the current track index
- If missing or invalid: start with an empty "Default" playlist (current behavior)

#### Scenario: Valid persisted state found
- **WHEN** the application starts and `{app_config_dir}/playlist_state.json` exists with valid JSON
- **THEN** the backend restores it as the active playlist
- **AND** sets the current track index from the persisted state

#### Scenario: Missing or invalid persisted state
- **WHEN** the application starts and `{app_config_dir}/playlist_state.json` is missing, corrupt, or unreadable
- **THEN** the backend starts with an empty "Default" playlist
- **AND** a warning is logged (not shown to the user)

### Requirement: Storage location

The persisted state file must be stored in the application's config directory (`app_config_dir()`) as `playlist_state.json`.

### Requirement: Graceful degradation

If the persisted state file cannot be read (missing, corrupt, I/O error), the application must start normally with an empty playlist. The error must be silently handled — no crash or user-visible error.

### Requirement: Transparent persistence

The auto-save and restore must be transparent to the user. No new UI elements, dialogs, or settings are required.

### Requirement: Idempotent saves

Repeated saves with the same state must not cause errors or side effects.

### Requirement: Per-track mix duration in data model

The `PlaylistTrack` struct SHALL include an optional `mix_duration_override` field (type `Option<f64>`) that stores a per-track mix duration in seconds. This field SHALL be serialised and deserialised alongside existing fields.

#### Scenario: Mix duration override round-trips
- **WHEN** a `PlaylistTrack` is serialised with `mix_duration_override: Some(7.5)` and deserialised
- **THEN** the deserialised value SHALL be `Some(7.5)`
- **AND** no data loss SHALL occur

### Requirement: Playlist context includes mix overrides

The `PlaylistContextEntry` (used to pass track information to the pipeline) SHALL include optional `mix_pattern_override` and `mix_duration_override` fields so the pipeline can resolve the correct mix settings for each transition.

#### Scenario: Context entry carries overrides
- **WHEN** a track has per-track mix overrides set
- **THEN** the `PlaylistContextEntry` sent to the pipeline SHALL include those overrides
- **AND** the pipeline SHALL use them when resolving the transition mix

## Data Model

### PlaylistState

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
