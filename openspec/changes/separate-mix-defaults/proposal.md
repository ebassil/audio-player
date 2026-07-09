## Why

The mix duration and pattern controls on the main screen and in the settings panel currently modify the same underlying value. This conflates two distinct concepts: global default mix settings (belonging in the app config) and per-track mix overrides (belonging in the playlist JSON). Users cannot set per-song mix behavior different from the default, and mix-in/mix-out points are saved per-track but mix pattern/duration are not.

## What Changes

- **Settings panel mix controls** become "Default Mix" controls that read/write the application config file (`app.toml`) — these are the fallback values used when no per-track override exists.
- **Main screen mix controls** become per-track overrides that are saved to the playlist state JSON alongside each track's mix-in/mix-out points.
- When a track is loaded, the main screen loads mix pattern and duration from the per-track override (if set), falling back to the app config defaults.
- `mix_pattern_override` and a new `mix_duration_override` field are persisted per-track in the playlist state JSON.
- The backend mix resolution combines: per-track overrides (highest priority) → app config defaults (fallback).
- Mix-in and mix-out points remain per-track only, read solely from playlist JSON.

## Capabilities

### New Capabilities
- `per-track-mix`: Per-track mix pattern and duration overrides stored in playlist state JSON, editable from the main screen UI.

### Modified Capabilities
- `settings-persistence`: The settings panel mix controls explicitly manage DEFAULT mix values in `app.toml`; these serve as fallback when no per-track override is present.
- `playlist-persist`: Playlist state JSON now persists `mix_pattern_override` and `mix_duration_override` per track, loaded and saved alongside existing mix point data.
- `track-transition`: Mix resolution logic prioritises per-track overrides over app config defaults; the `MixEngine` or a new resolver combines both sources to produce `ResolvedMix`.

## Impact

- **Rust backend** (`config.rs`, `playlist.rs`, `playlist_persist.rs`, `mixing.rs`, `lib.rs`): Add `mix_duration_override` to `PlaylistTrack`; update `AppConfig` mix fields to serve as defaults only; add new Tauri commands for per-track mix get/set on playlist state; update `MixEngine::resolve()` or add a resolver that merges defaults with overrides.
- **Frontend** (`src/main.ts`): Disconnect main screen mix controls from the engine's config commands; wire them to per-track get/set on playlist state; wire settings panel mix controls to app config commands only; load values with fallback logic (per-track → config → engine defaults).
- **CSS** (`styles.css`): Minor label/text changes to distinguish "Default Mix" in settings from "Track Mix" on main screen (if necessary).
