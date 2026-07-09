## Why

Log filter settings (message names filter and regex filter) are session-only and lost when the application is closed and reopened. Users must re-enter their filter preferences every session, making the feature significantly less useful. Settings like volume, mute, and mix configuration persist correctly via `AppConfig` — log filters should behave the same way.

## What Changes

- Add `log_filter_names` and `log_filter_regex` fields to the `AppConfig` struct in Rust
- Update `save_app_config` to persist current filter values from the frontend
- Update `load_app_config` to return saved filter values to the frontend
- Update `saveAppConfig()` TypeScript function to accept and send filter values
- Update `loadAppConfig()` TypeScript function to restore filter inputs and recompile the regex
- Call `saveAppConfig()` when log filter inputs change
- Existing `AppConfig` TOML files without the new fields remain valid (backward compatible)

## Capabilities

### New Capabilities
- `settings-persistence`: System-wide settings (including log filters, mix defaults, volume, mute) SHALL persist across application restarts via the AppConfig TOML system

### Modified Capabilities

None.

## Impact

- **`src-tauri/src/audio/config.rs`**: `AppConfig` struct — add two `String` fields with sensible defaults (empty strings)
- **`src-tauri/src/lib.rs`**: `save_app_config` — accept filter values as command arguments; `load_app_config` — return filter values in response JSON
- **`src/main.ts`**: `saveAppConfig()` — pass filter values; `loadAppConfig()` — restore filter state and call `updateLogFilters()`; filter input listeners — trigger save on change
- **Backward compatibility**: New fields default to empty strings; previously saved `app.toml` files without these fields deserialize correctly via Serde's `default`
