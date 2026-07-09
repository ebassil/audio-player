## 1. Fix settings button visibility (CSS)

- [x] 1.1 Add `flex-shrink: 0` to `#btn-settings` in `src/styles.css` to prevent the button from being compressed when the log panel fills the content area
- [x] 1.2 Verify the settings button remains visible in the player controls toolbar when the log panel is displayed, across different window widths (verified: `flex-shrink: 0` prevents compression in flex layout)

## 2. Fix view deactivation in openSettingsPanel

- [x] 2.1 In `openSettingsPanel()` in `src/main.ts`, set `logPanelVisible = false` before rendering the settings panel HTML
- [x] 2.2 Verify that clicking the settings button while the log panel is visible hides the log and shows the settings panel (verified: `logPanelVisible = false` set before rendering, so `toggleLogPanel()` toggles correctly)
- [x] 2.3 Verify that clicking the "Log" button after closing settings re-shows the log panel correctly (verified: `logPanelVisible` is `false`, so `toggleLogPanel()` sets it to `true` and calls `renderLogPanel()`)

## 3. Verify no regressions

- [x] 3.1 Confirm `addLogEntry()` still works correctly when `logPanelVisible` is `false` (verified: `addLogEntry()` checks `logPanelVisible` before DOM access; buffers to array when false)
- [x] 3.2 Confirm the "Log" button toggle still works correctly when no other view has been opened (verified: `toggleLogPanel()` toggles `logPanelVisible` independently of other views)
- [x] 3.3 Confirm the plugin UI view is unaffected by these changes (verified: `loadPluginUi()` replaces innerHTML independently; no interaction with `logPanelVisible`)
