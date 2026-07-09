## Context

The Log button in `src/main.ts` calls `toggleLogPanel()` to show/hide a debug log panel inside the `#plugin-ui-container` div — the same container shared with plugin UI and settings panel. State is managed via module-level variables (`logPanelVisible`, `logFilterNames`, etc.) in vanilla TypeScript with no framework. The button has no visual active state, and the toggle replaces `innerHTML` rather than showing/hiding the panel element.

## Goals / Non-Goals

**Goals:**
- Log button reliably shows the debug log panel on click
- Log button shows an active/selected state when the log panel is visible
- State desync is fixed: when plugin UI or settings overwrite the shared container, `logPanelVisible` is reset and the button state updates
- Remove unnecessary inline `display: block` that overrides flex layout

**Non-Goals:**
- No framework migration (keep vanilla TypeScript)
- No changes to log filtering behavior
- No changes to log entry rendering logic

## Decisions

1. **CSS `.active` class on button instead of inline style** — The Log button gets a `.active` class when the panel is visible. This provides visual feedback and is cleaner than inline styles.

2. **Reset `logPanelVisible` when plugin UI loads** — `loadPluginUi()` already replaces the container's content; it should also set `logPanelVisible = false` and remove the `.active` class from the Log button, preventing the state desync where clicking "Log" unexpectedly hides the panel.

3. **Remove unnecessary `container.style.display = "block"`** — The container is already rendered by CSS `display` rules (it's a flex child). The inline override is redundant and can cause layout issues.

## Risks / Trade-offs

- **[Risk]** Other code paths might also write to `#plugin-ui-container` — they would need the same fix. → Audit all code that sets `plugin-ui-container.innerHTML` and ensure they reset `logPanelVisible`.
- **[Risk]** The shared-container pattern is inherently fragile — a future feature adding another panel could reintroduce the bug. → Document the pattern constraint; consider a future refactor to separate containers.
