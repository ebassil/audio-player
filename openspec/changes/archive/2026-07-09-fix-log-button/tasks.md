## 1. Add CSS active state for Log button

- [x] 1.1 Add `.header-toolbar button.active` CSS rule to `src/styles.css` with a highlighted background/border style
- [x] 1.2 Toggle `.active` class on `#btn-log` in `toggleLogPanel()` when showing/hiding the log panel

## 2. Fix toggleLogPanel function

- [x] 2.1 Remove unnecessary `container.style.display = "block"` from both branches of `toggleLogPanel()`
- [x] 2.2 Verify `renderLogPanel()` uses proper visibility (the container already has `flex: 1` from CSS)

## 3. Fix state desync in loadPluginUi

- [x] 3.1 In `loadPluginUi()`, set `logPanelVisible = false` before overwriting the container
- [x] 3.2 Remove `.active` class from `#btn-log` when plugin UI loads

## 4. Fix state desync in openSettingsPanel

- [x] 4.1 Verify `openSettingsPanel()` already sets `logPanelVisible = false` (line 1223)
- [x] 4.2 Add removal of `.active` class from `#btn-log` in `openSettingsPanel()`

## 5. Verify all scenarios

- [x] 5.1 Run `npm run build` (or `cargo tauri dev`) and verify the Log button toggles the panel correctly
- [ ] 5.2 Verify Log button active state shows/hides on toggle (requires running the app with `cargo tauri dev`)
- [ ] 5.3 Verify loading a plugin UI resets log state and the Log button can toggle again (requires running the app)
- [ ] 5.4 Verify opening settings resets log state and the Log button can toggle again (requires running the app)
