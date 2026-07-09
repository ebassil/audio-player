## MODIFIED Requirements

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

## ADDED Requirements

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
