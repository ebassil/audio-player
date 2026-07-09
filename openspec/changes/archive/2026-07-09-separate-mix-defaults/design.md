## Context

The mix controls on the main screen and settings panel both read/write the same engine config (`set_mix_config` / `get_mix_config`). The settings panel is meant to manage application-wide **defaults** (persisted to `app.toml`), while the main screen controls should manage **per-track overrides** (persisted to `playlist_state.json`). Currently:

- Both UIs call `get_mix_config` / `set_mix_config` on the engine
- Only the main screen calls `saveAppConfig` after changes
- The settings panel does not persist its changes at all
- `PlaylistTrack` already has `mix_pattern_override: Option<String>` but no duration override
- `PlaylistContextEntry` does not carry pattern or duration overrides to the pipeline

## Goals / Non-Goals

**Goals:**
- Decouple main screen mix controls from settings panel mix controls
- Settings panel saves defaults to `app.toml` (app config), loaded on startup and used as fallback
- Main screen per-track overrides (pattern + duration) saved to `playlist_state.json`
- Pipeline resolves mix by checking per-track override → fallback to defaults
- Mix-in/mix-out points remain per-track only (unchanged)

**Non-Goals:**
- Adding new UI widgets beyond existing controls (no new buttons/selectors)
- Changing mix point UX beyond separating the data persistence
- Migration of existing `playlist_state.json` — old format without `mix_duration_override` loads gracefully (field is `Option`, defaults to `None`)

## Decisions

### Decision 1: Extend data model with mix_duration_override

- `PlaylistTrack` in `playlist.rs` gets `mix_duration_override: Option<f64>`
- `PlaylistContextEntry` in `pipeline.rs` gets `mix_pattern_override: Option<String>` and `mix_duration_override: Option<f64>`
- These fields are `#[serde(skip_serializing_if = "Option::is_none")]` to reduce noise when unset
- *Why this approach?*: Minimal intrusion into existing types; serde handles absent fields gracefully. No migration needed — old JSON files without the field will deserialize as `None`.

### Decision 2: Allow MixEngine::resolve() to accept optional overrides

Change `resolve()` signature from:
```rust
pub fn resolve(&self, current_track_mix: &MixPoint, next_track_mix: &MixPoint) -> ResolvedMix
```
to:
```rust
pub fn resolve(
    &self,
    current_track_mix: &MixPoint,
    next_track_mix: &MixPoint,
    pattern_override: Option<MixPattern>,
    duration_override: Option<f64>,
) -> ResolvedMix
```
When `pattern_override` or `duration_override` is `Some`, use it instead of `self.config.pattern` / `self.config.duration_secs`.

*Alternative considered*: Keeping resolve() unchanged and doing the override merge at the call site. Rejected because it would require every caller to reimplement the fallback logic.

### Decision 3: Pipeline transition logic passes overrides

The `PlaylistContextEntry` carries `mix_pattern_override` and `mix_duration_override`. When the audio callback's transition trigger fires, it reads these from the current track's context entry and passes them to `MixEngine::resolve()`.

`set_playlist_context` in `lib.rs` is updated to parse the new fields from frontend JSON and populate them into `PlaylistContextEntry`.

*Why not store overrides in pipeline directly?* The context entry is already the per-track data source for transitions. Adding the fields there keeps all per-track data in one place.

### Decision 4: New Tauri commands for per-track overrides

Two new IPC commands:
- `get_current_track_mix_overrides` → returns `{ pattern_override: Option<String>, duration_override: Option<f64> }` for the currently playing/selected track
- `set_current_track_mix_overrides` → accepts `{ pattern_override: Option<String>, duration_override: Option<f64> }`, updates the track in the playlist state, and saves

The commands operate on `state.playlist` (the in-memory playlist). After mutating the track entry, they call the existing `save_playlist_state()` helper.

*Why not reuse `set_mix_config`?* `set_mix_config` targets the engine's defaults and has different semantics. Mixing concerns would re-introduce the bug being fixed.

### Decision 5: Frontend load/save separation

**Main screen `loadMixConfig()` replacement**: On track selection / app startup, the frontend calls a new function that:
1. Fetches per-track overrides from the playlist state via `get_current_track_mix_overrides`
2. Fetches app config defaults via `loadAppConfig`
3. For each control: use override if `Some`, else use config default, else use engine fallback (`crossfade` / `3.0s`)
4. Sets the UI controls accordingly

**Main screen on-change**: Calls `set_current_track_mix_overrides` with the new values. Does NOT call `saveAppConfig`.

**Settings panel on-change**: Calls `set_mix_config` (updates engine defaults) AND `save_app_config` (persists to `app.toml`). Does NOT modify per-track overrides.

*Fallback chain:* per-track override (playlist state) → app config default (app.toml) → hardcoded engine default (crossfade, 3.0s).

### Decision 6: Labels for clarity

- Settings panel: "Default Mix Pattern" / "Default Mix Duration (s)" — already has these labels, keep unchanged
- Main screen: "Mix Pattern" / "Mix Duration" — optionally add "(per-track)" tooltip to distinguish

## Risks / Trade-offs

- **[Risk] Stale defaults in settings if user only uses main screen**: Main screen changes never touch app config. If a user sets per-track overrides and then removes them, the fallback defaults are whatever was last saved in settings. → Mitigation: This is correct behaviour; settings defaults are the authoritative fallback.
- **[Risk] Race condition on startup**: `loadAppConfig` and `playlist_state.json` restore happen asynchronously. → Mitigation: Frontend already loads config after DOM ready; the per-track query can be satisfied from the existing in-memory playlist. If no track is selected yet, the main screen mix controls are disabled/hidden (existing behaviour).
- **[Trade-off] Two sources of truth**: Mix resolution now reads from three places (track override → app config → engine built-in defaults). → Acceptable because each layer has a clear priority and purpose.
