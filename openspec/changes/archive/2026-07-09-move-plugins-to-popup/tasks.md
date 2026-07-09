## 1. Layout: Expand sidebar & add header toolbar

- [x] 1.1 Remove the "Plugin Rack" sidebar section from the HTML template (`#plugin-rack` div and its `<h3>`) in `src/main.ts`
- [x] 1.2 Make the playlist section fill the full sidebar height — change `.sidebar-section` to use `flex: 1` and adjust `.sidebar` for full-height layout
- [x] 1.3 Add a header toolbar row to the HTML template with "Plugins" and "Log" buttons, placed above the player controls
- [x] 1.4 Add CSS styles for the header toolbar (`.header-toolbar`, button styles) in `src/styles.css`
- [x] 1.5 Update sidebar styles: remove border-bottom from the last section, adjust `.playlist` max-height to fill available space

## 2. Plugin popup: Secondary Tauri window

- [x] 2.1 Add URL query parameter detection in `src/main.ts` — check `window.location.search` for `?view=plugins` at startup
- [x] 2.2 Create a `renderPluginPopup()` function that renders only the plugin rack content (plugin list, drag-reorder, enable/disable, UI buttons) when `?view=plugins` is detected
- [x] 2.3 Wire up the "Plugins" button in the header toolbar to call `WebviewWindow.create('plugins', { url: 'index.html?view=plugins', title: 'Plugins', ... })` with resizeable settings
- [x] 2.4 Handle window focus: if the plugin window already exists, call `getWebviewWindow('plugins').show()` / `.setFocus()` instead of creating a new window
- [x] 2.5 Handle window close: clean up the reference so "Plugins" button can reopen
- [x] 2.6 Add iframe `postMessage` listener in the popup window for `param_change` events, calling `set_plugin_parameter`

## 3. Debug log panel: Frontend

- [x] 3.1 Create a `LogEntry` interface and `logEntries: LogEntry[]` circular buffer (max 1000 entries) in global state
- [x] 3.2 Create a `loggedInvoke()` wrapper function that calls `invoke()`, records timestamp, command name, args (truncated to 200 chars), result/error, and pushes to the buffer
- [x] 3.3 Replace all direct `invoke()` calls in `src/main.ts` with `loggedInvoke()`
- [x] 3.4 Add event listeners (`listen`) for `player-status`, `track-changed`, and `audio-log` events that push entries to the buffer with direction `←`
- [x] 3.5 Implement throttle for `player-status` events: max one log entry per second for this event type
- [x] 3.6 Add the log panel HTML to the main content area (replacing `#plugin-ui-container` content) with a scrollable list
- [x] 3.7 Implement log panel rendering: each entry shows timestamp, direction arrow, command/event name, status, color-coded (green=success, red=error, blue=event)
- [x] 3.8 Implement auto-scroll: if scrolled to bottom, auto-scroll on new entry; pause if scrolled up, resume when back at bottom
- [x] 3.9 Wire up the "Log" button to toggle the log panel visibility
- [x] 3.10 Add CSS styles for the log panel (`.log-panel`, `.log-entry`, timestamp, direction, status colors) in `src/styles.css`

## 4. Debug log panel: Rust backend events

- [x] 4.1 Add `emit_audio_log()` helper in `src-tauri/src/lib.rs` that emits an `audio-log` event with a timestamp and message
- [x] 4.2 Emit `audio-log` events at key audio milestones: track loaded, decode started, decode completed, seek performed, transition started, transition completed, plugin error
- [x] 4.3 Add event emission in the background status loop for state changes (Playing→Paused→Stopped)

## 5. Settings panel relocation

- [x] 5.1 Add a "Settings" button to the plugin popup window or keep the ⚙ button in main window and open settings in a modal/overlay
- [x] 5.2 Ensure `openSettingsPanel()` works when rendered outside the main content area (update container selector if needed)
- [x] 5.3 Wire up settings access so it's always reachable (either via popup or a dedicated button)
