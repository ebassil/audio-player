## Why

The debug log panel currently displays all entries with no filtering, making it noisy and hard to find relevant messages — especially `player-status` events which arrive every second. Users need a way to control what appears in the log panel.

## What Changes

- Add a "Log Filters" section to the existing settings panel
- Filters accept: message names (exact match, comma-separated) and a regex pattern
- Filtered entries are hidden from the log panel in real-time
- Info icon next to the filter setting with a tooltip showing usage examples
- The existing hard-coded `player-status` throttle (1/sec) remains, but the new filter can additionally suppress it entirely or target other messages

## Capabilities

### New Capabilities

- `log-filter`: Configurable filtering of log entries by message name and/or regex pattern, with settings UI and informational tooltip

### Modified Capabilities

- `debug-log`: Add the ability to filter out log entries from display. The existing throttle behavior remains unchanged; filtering is an additional layer that can hide entries entirely or selectively.

## Impact

- **src/main.ts**: Add filter state, filtering logic, settings UI section for log filters, info tooltip, update `addLogEntry()` / `renderLogPanel()` to apply filters
- **openspec/specs/debug-log/spec.md**: Add requirements for filtering capability
- **openspec/config.yaml** (conceptual): No persistence changes initially — filters will be session-only (unless persistence is added later)
