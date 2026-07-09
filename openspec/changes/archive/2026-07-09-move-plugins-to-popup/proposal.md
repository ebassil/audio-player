## Why

The sidebar is shared between the playlist and the plugin rack, limiting space for both. The plugin rack should be a separate resizeable popup window, freeing the sidebar for the playlist. Additionally, there is no way to inspect the low-level communication between the frontend and the Rust audio backend — a live log viewer is needed for debugging and development.

## What Changes

- Remove the plugin rack from the sidebar and move it to a new resizeable popup window, opened via a button in the header toolbar.
- The playlist expands to fill the entire sidebar (no longer sharing space with plugins).
- Add a "Log" button in the header toolbar that toggles a timestamped log panel in the main content area (where the plugin UI container currently lives).
- The log panel displays all IPC invocations (commands sent from frontend to backend) and events emitted from backend to frontend, with timestamps.
- Plugin configuration UI (iframe-based plugin UIs) is moved into the popup alongside the plugin list.
- The settings panel remains accessible — adjust its placement or make it accessible from the popup.

## Capabilities

### New Capabilities
- `plugin-popup`: A resizeable popup window containing the plugin list (with drag-reorder, enable/disable) and plugin configuration UI (iframe), opened from the header toolbar.
- `debug-log`: A timestamped log panel in the main content area that records frontend ↔ backend IPC communication (commands and events).

### Modified Capabilities
- (none — no spec-level requirement changes to existing capabilities)

## Impact

- **Frontend (`src/main.ts`)**: Major rework of the sidebar layout — remove plugin rack, expand playlist. Add header toolbar buttons for plugin popup and log toggle. Add log panel component with auto-scroll. Add popup window management.
- **Rust backend (`src-tauri/src/`)**: Add a new `emit_event` / logging infrastructure to broadcast IPC traffic to the frontend log. Possibly add Tauri commands to retrieve buffered logs. May need a new Tauri window for the popup.
- **Plugin system**: Plugin management UI moves to a separate window — may require Tauri multi-window support or a custom popup element within the same window.
- **Styles (`src/styles.css`)**: Significant layout changes — sidebar full-height playlist, header toolbar additions, log panel styles, popup styles.
