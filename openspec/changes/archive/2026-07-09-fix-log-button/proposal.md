## Why

The Log button in the header toolbar is broken. Clicking it produces no visible effect — the log panel does not appear, and there is no visual feedback on the button itself. Users cannot inspect debug logs, making it impossible to diagnose IPC traffic and audio backend events.

## What Changes

- Fix `toggleLogPanel()` so the log panel actually renders and becomes visible when the Log button is clicked
- Ensure the Log button shows an active/selected visual state when the log panel is open, and returns to normal when closed
- Fix state desync between `logPanelVisible` and the actual DOM content when plugin UI or settings panel overwrites the shared container
- Remove unnecessary inline `container.style.display = "block"` that overrides the flex layout and can cause the container to render incorrectly
- Ensure the hint placeholder text is correct and consistent across toggle states

## Capabilities

### New Capabilities
- `log-toggle`: Reliable toggling of the debug log panel with correct visibility state management and visual feedback on the toggle button

### Modified Capabilities
- `debug-log`: Fix the log panel visibility toggle requirement — the Log button must reliably show/hide the panel with proper visual active state

## Impact

- `src/main.ts` — `toggleLogPanel()`, `renderLogPanel()`, `openSettingsPanel()`, `loadPluginUi()`, and related state management
- `src/styles.css` — Add active/selected button state for `.header-toolbar button.active`
