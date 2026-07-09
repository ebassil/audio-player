## Context

The debug log panel (`src/main.ts`) records all IPC calls and backend events with no filtering. `player-status` events every 250ms (throttled to 1/sec) dominate the log. Settings already exist in `openSettingsPanel()` with sections for shortcuts, mix defaults, and audio device — a log filter section slots in there naturally.

## Goals / Non-Goals

**Goals:**
- Allow users to suppress log entries by message name (exact match, comma-separated)
- Allow users to suppress log entries by regex (matched against entry text)
- Info icon with tooltip showing examples: message name only, regex only, both combined
- Filters apply in real-time as entries are added and when filter config changes
- Minimal changes to the existing log rendering

**Non-Goals:**
- Persisting filter settings across sessions (session-only initially)
- Server-side or backend filtering (frontend-only)
- Saved filter presets or multiple filter profiles
- Filtering of Rust stderr/stdout diagnostics

## Decisions

**Decision 1: Single text input for message names instead of per-name toggles.**  
A comma-separated text field is simpler to implement and more flexible than fixed checkboxes. It handles any event/command name without UI changes when new message types are added.

**Decision 2: Filter applied on render, not on ingestion.**  
Log entries are still stored in `logEntries[]` (needed for buffer management). Filtering is applied in `renderLogPanel()` and `addLogEntry()` when creating DOM elements. This keeps the data model clean and allows toggling filters off without losing history.

**Decision 3: Session-only state initially.**  
The `AppConfig` TOML struct in Rust would need a new field for log filters. Keeping it as a frontend-only `string` variable avoids cross-language changes and keeps the change small. Persistence can be added later if needed.

**Decision 4: OR logic for combined filters.**  
If both message names and regex are set, an entry matching either criterion is hidden. This matches the mental model of "I want to hide these things."

**Decision 5: Tooltip via CSS `::after` pseudo-element or inline HTML with a hover listener.**  
The existing codebase uses vanilla JS + direct innerHTML manipulation. A simple `<span class="info-icon">ⓘ</span>` with a CSS `:hover::after` tooltip fits the existing pattern without adding dependencies.

## Risks / Trade-offs

- **Risk: Regex entered by user could be slow or crash.** → Mitigation: wrap `RegExp` construction and `.test()` in try/catch; invalid regex shows nothing (implicitly no match) to avoid breaking the UI.
- **Risk: Session-only means filters are lost on refresh.** → Acceptable for this iteration; Tauri apps rarely refresh mid-session.
- **Risk: OR logic might surprise users who think both must match.** → The tooltip examples demonstrate combination behavior. If feedback suggests AND is preferred, it's a one-line change.
