## 1. Rust Backend — Extend AppConfig

- [x] 1.1 Add `log_filter_names: String` and `log_filter_regex: String` fields to `AppConfig` struct (with `#[serde(default)]`), defaulting to `""`
- [x] 1.2 Update `save_app_config` Tauri command to accept optional `logFilterNames` and `logFilterRegex` string arguments and store them in the `AppConfig`
- [x] 1.3 Update `load_app_config` Tauri command to return `log_filter_names` and `log_filter_regex` in the JSON response
- [x] 1.4 Update `AppConfig::default()` and existing tests to handle the new fields

## 2. TypeScript Frontend — Wire Persistence

- [x] 2.1 Update `saveAppConfig()` in `src/main.ts` to read `logFilterNames` and `logFilterRegex` from the module-level variables and pass them as arguments to the `save_app_config` Tauri command
- [x] 2.2 Update `loadAppConfig()` in `src/main.ts` to accept `logFilterNames` and `logFilterRegex` in the config response, set the module-level variables, update the filter input elements, and call `updateLogFilters()` to recompile the regex
- [x] 2.3 Add `saveAppConfig()` calls to the `input` event listeners for both log filter inputs (lines 1372-1380 in `src/main.ts`)

## 3. Verify

- [x] 3.1 Run `cargo build` to verify Rust changes compile
- [x] 3.2 Run TypeScript compilation check (or app build) to verify frontend changes
- [x] 3.3 Manual test: set log filters, restart app, verify filters persist
- [x] 3.4 Manual test: launch with old `app.toml` (no filter fields), verify it loads without error and filters default to empty
