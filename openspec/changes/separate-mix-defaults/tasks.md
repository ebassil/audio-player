## 1. Rust: Data Model Extensions

- [ ] 1.1 Add `mix_duration_override: Option<f64>` to `PlaylistTrack` in `playlist.rs` with `#[serde(skip_serializing_if = "Option::is_none")]`
- [ ] 1.2 Add `mix_pattern_override: Option<String>` and `mix_duration_override: Option<f64>` to `PlaylistContextEntry` in `pipeline.rs`

## 2. Rust: Mix Engine Override Support

- [ ] 2.1 Update `MixEngine::resolve()` signature to accept `pattern_override: Option<MixPattern>` and `duration_override: Option<f64>` parameters
- [ ] 2.2 Implement fallback logic: use override if `Some`, otherwise use `self.config`
- [ ] 2.3 Update existing callers of `resolve()` to pass `None` for the new parameters (pipeline, tests)
- [ ] 2.4 Update `MixEngine::resolve()` unit tests to cover override paths

## 3. Rust: Pipeline Transition Logic

- [ ] 3.1 In the audio callback's transition trigger logic, extract `mix_pattern_override` and `mix_duration_override` from the current track's `PlaylistContextEntry`
- [ ] 3.2 Pass the overrides to `MixEngine::resolve()` when computing the transition
- [ ] 3.3 Update `set_playlist_context` in `lib.rs` to parse `mix_pattern_override` and `mix_duration_override` from the JSON payload

## 4. Rust: New Tauri Commands

- [ ] 4.1 Add `get_current_track_mix_overrides` command: reads the current track from `state.playlist`, returns `{ pattern_override, duration_override }`
- [ ] 4.2 Add `set_current_track_mix_overrides` command: updates the current track in the playlist, triggers `save_playlist_state()`
- [ ] 4.3 Register both commands in the `invoke_handler` in `run()`

## 5. Rust: Settings Persistence Fix

- [ ] 5.1 Update settings panel mix onChange handlers in `lib.rs` to call `save_app_config` when defaults change (backend-side: add `save_app_config` call to a new or existing command that settings panel triggers on mix change)
- [ ] 5.2 Ensure `load_app_config` correctly applies defaults to the engine on startup and does not overwrite per-track overrides that are already loaded from playlist state

## 6. Frontend: Main Screen Mix Controls

- [ ] 6.1 Replace `loadMixConfig()` call on track selection with a new function that fetches per-track overrides via `get_current_track_mix_overrides` and falls back to app config defaults
- [ ] 6.2 Wire main screen mix-pattern-select onChange to `set_current_track_mix_overrides` instead of `set_mix_config` + `saveAppConfig`
- [ ] 6.3 Wire main screen mix-duration-slider onInput to `set_current_track_mix_overrides` instead of `set_mix_config` + `saveAppConfig`
- [ ] 6.4 Remove `saveAppConfig` call from main screen mix controls

## 7. Frontend: Settings Panel Mix Controls

- [ ] 7.1 Wire settings-mix-pattern onChange to `set_mix_config` + `save_app_config` (ensure both are called)
- [ ] 7.2 Wire settings-mix-duration onInput to `set_mix_config` + `save_app_config` (ensure both are called)
- [ ] 7.3 Ensure settings panel initial values load from `load_app_config` defaults (not from per-track overrides)

## 8. Testing

- [ ] 8.1 Run existing Rust unit tests and fix any failures from changed signatures
- [ ] 8.2 Add unit tests for `MixEngine::resolve()` with overrides
- [ ] 8.3 Add unit tests for `PlaylistTrack` serialization/deserialization with `mix_duration_override`
- [ ] 8.4 Verify manual: change defaults in settings → restart → defaults load; change per-track on main screen → restart → overrides persist
