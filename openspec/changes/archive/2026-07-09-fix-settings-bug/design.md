## Context

The settings button (⚙) is dynamically appended to `#player-controls` inside `.content`, which has `overflow: hidden`. When the log panel is displayed in `#plugin-ui-container`, it fills the remaining flex space in the content area. The existing `openSettingsPanel()` replaces the innerHTML of `#plugin-ui-container` but does not update the `logPanelVisible` boolean, so the log panel's visibility state becomes stale.

The view management in `src/main.ts` is a simple content-slot pattern: each view function (log, settings, plugin UI) replaces the `#plugin-ui-container` innerHTML independently, with no shared state management. Only the log view tracks its visibility state via `logPanelVisible`.

## Goals / Non-Goals

**Goals:**
- Ensure the settings button is always visible in the player controls area regardless of which view is active in the content area
- When clicking the settings button, properly deactivate the log panel (set `logPanelVisible = false`) before rendering the settings panel
- Provide consistent view deactivation: opening any full-content view should properly deactivate the log view

**Non-Goals:**
- No changes to the Rust backend
- No new view management framework or router
- No changes to the plugin UI view or popup window
- No new UI elements

## Decisions

1. **Fix visibility in CSS** — The settings button may be hidden by `overflow: hidden` on `.content` if the button is at the edge or if there's a flex constraint. Add `flex-shrink: 0` to `#btn-settings` to prevent it from being compressed, and ensure the button is not clipped. This is the simplest, most targeted fix.
2. **Set `logPanelVisible = false` in `openSettingsPanel()`** — Before rendering the settings panel, set the `logPanelVisible` flag to `false`. This ensures that `addLogEntry()` no longer tries to append to the (now non-existent) `#log-entries` DOM, and the next "Log" button click correctly toggles the panel back on.
3. **No architectural change** — The existing content-slot pattern is sufficient. No need for a view stack, history, or routing layer. Adding a generalized "close any active view" helper function is optional and low-risk.

## Risks / Trade-offs

- **[Low] Stale DOM references** — If `logPanelVisible` is `false` but code elsewhere still holds references to log DOM elements, errors could occur. Mitigation: only `addLogEntry()` and `renderLogPanel()` touch log DOM, and both check `logPanelVisible`.
- **[Low] Missing deactivation for other views** — If a future view is added and doesn't deactivate the log panel, the same bug could reoccur. Mitigation: document the pattern in code comments, or add a generic `closeAllViews()` helper.
