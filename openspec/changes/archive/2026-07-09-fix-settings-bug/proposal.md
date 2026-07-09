## Why

The settings button (⚙) in the player controls toolbar is not visible when the debug log panel is displayed. Additionally, clicking the settings button should hide the log panel and show the settings view, but currently the log panel state is not properly managed when switching views.

## What Changes

- Fix the settings button visibility so it remains visible regardless of which panel is displayed in the content area
- When the settings button is clicked, hide the log panel (set `logPanelVisible = false`) before rendering the settings panel
- Ensure consistent view interaction: opening one view should properly deactivate any other active view
- Add a CSS fix to prevent the settings button from being clipped/hidden by overflow or layout issues

## Capabilities

### New Capabilities

*(None — this is a bug fix, not a new capability.)*

### Modified Capabilities

- `debug-log`: The log panel's visibility state must be properly managed when other views (specifically the settings panel) are opened. Opening the settings panel should deactivate the log view and reset `logPanelVisible` to `false`.

## Impact

- `src/main.ts`: Update `openSettingsPanel()` to set `logPanelVisible = false` and manage view state consistently. Ensure the settings button click handler properly toggles views.
- `src/styles.css`: Fix any CSS issues that cause the settings button to be hidden when the log panel is displayed (e.g., overflow, z-index, or layout issues in the content area).
- No changes to the Rust backend.
- No API changes.
