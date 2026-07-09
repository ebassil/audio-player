## Context

Log filter settings (`logFilterNames`, `logFilterRegex`) are currently in-memory only — initialized as empty strings at `src/main.ts:938-940` and never saved to disk. The `AppConfig` system already persists volume, mute, and mix settings to `app.toml` via Tauri commands. The fix extends this existing mechanism to include log filter values, reusing the same save/load pathway.

## Goals / Non-Goals

**Goals:**
- Persist `log_filter_names` and `log_filter_regex` across application restarts
- Backward compatible: existing `app.toml` files without these fields load without error
- Filter values are saved automatically when the user types in the filter inputs (no separate "save" button)

**Non-Goals:**
- No UI changes — filter inputs remain in the Settings panel as-is
- No changes to filtering logic (`shouldFilterEntry`, `updateLogFilters`)
- No debouncing of save calls (acceptable for low-frequency typing)

## Decisions

**Decision 1: Add fields to existing `AppConfig` with `#[serde(default)]`.**
- Why: Reuses the proven config system (TOML serialize/deserialize). Using `#[serde(default)]` on each new field ensures old configs without the keys deserialize as empty strings — no migration needed.
- Alternative considered: A separate config file for log filters — adds complexity with no benefit since the data is tiny and shares the same lifecycle.

**Decision 2: Frontend passes filter values to `save_app_config` as command args.**
- Why: The Rust side has no access to the frontend's filter state. The command already exists and is called for other settings changes.
- Alternative considered: Rust reading filter values from a different source — no such source exists.

**Decision 3: Auto-save on input (no debounce).**
- Why: Typing in filter fields is infrequent and the TOML write is fast (<1ms). Debounce would add complexity and delay persistence unnecessarily.
- Alternative considered: Save only on settings panel close — adds risk of data loss if the app crashes.

## Risks / Trade-offs

- [Backward compat] New `app.toml` fields are invisible to older app versions — safe because older versions ignore unknown TOML keys (serde ignores extra fields by default for `#[derive(Deserialize)]`).
- [Race condition] No known write contention — `save_app_config` is only called from the frontend, never concurrently.
- [Edge case] Empty filter strings (default) serialize as `log_filter_names = ""` in TOML — acceptable and round-trips correctly.
