## Context

Current layout shares the sidebar between the playlist (top) and plugin rack (bottom), limiting space for both. Plugin UIs and settings share the main content area below the timeline. There is no logging mechanism to inspect frontend↔backend IPC communication.

The frontend is a single vanilla TypeScript file (`src/main.ts`, ~1124 lines) with no framework. Styling is a single `src/styles.css` (~440 lines). The Rust backend defines all Tauri commands in `src-tauri/src/lib.rs` (~682 lines). Plugins are loaded via wasmtime and managed by `PluginManager`.

## Goals / Non-Goals

**Goals:**
- Move the plugin list (with drag-reorder, enable/disable, UI iframe) to a resizeable popup window opened from a header toolbar button
- Expand the playlist to fill the full sidebar height
- Add a toggleable timestamped log panel in the main content area (replacing the plugin UI container)
- Log all `invoke()` calls and Tauri events with timestamps in the log panel
- Keep the settings panel accessible (from the popup or as a separate button)

**Non-Goals:**
- No changes to the Rust audio pipeline, plugin loading, or playback logic
- No changes to the plugin WASM interface or manifest format
- No persistent log storage or log export
- No changes to the existing settings panel functionality

## Decisions

**1. Plugin popup: Tauri secondary window (WebviewWindow)**

Use `@tauri-apps/api/webviewWindow` (`WebviewWindow.create()`) to open a secondary window for the plugin rack. The new window loads the same `index.html` with a URL query parameter `?view=plugins`. The frontend checks for this param and renders only the plugin rack UI. This approach:
- Gives native resize behavior for free
- Keeps the plugin rack accessible while interacting with the main window
- Avoids building a custom resizeable overlay
- Reuses existing plugin rack rendering code

**2. Log system: Frontend-side invoke wrapper + Rust events**

- Wrap `invoke()` calls in a logging function that records timestamp, command name, args (truncated), result/error, and direction (`→ invoke`)
- Add a listener for Tauri events (`listen`) that records timestamp, event name, payload, direction (`← event`)
- Store log entries in a circular buffer (max 1000 entries) in global state
- Log panel renders the buffer with auto-scroll, filterable by direction

**3. Layout: Expanded sidebar + header toolbar buttons**

- Remove the "Plugin Rack" section from the sidebar; playlist section gets `flex: 1` to fill full height
- Add a header toolbar row above (or within) the player controls with two buttons: "Plugins" and "Log"
- The plugin-ui-container in the main content area becomes the log panel container
- Settings panel: either accessible from the plugin popup, or the ⚙ button remains and opens settings in the log panel container

**4. Log data flow from Rust backend**

Add new Tauri events: `audio-log` emitted from Rust for audio processing milestones (decode start/complete, seek, transition start/end, plugin process errors). These appear in the frontend log alongside invoke wrappers.

## Risks / Trade-offs

- **Secondary window focus**: If the user closes the plugin window, plugin state persists but the UI is gone. Mitigation: the "Plugins" button reopens the window (or refocuses it if already open).
- **Performance**: Logging every invoke call adds overhead. Mitigation: limit buffer size, avoid logging rapid events at full rate (throttle player-status to one entry per second max).
- **CSS complexity**: The sidebar layout change from split sections to full-height scrollable list needs careful CSS adjustments.
- **Plugin iframe communication**: Iframe `postMessage` listeners are set up in the main window; the plugin popup's iframe will need its own message listener.
- **Settings panel access**: Settings currently opens in the plugin-ui-container. With that area becoming the log panel, settings needs a new home — either in the plugin popup or in a separate panel.
