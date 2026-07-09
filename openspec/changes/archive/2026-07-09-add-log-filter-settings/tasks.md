## 1. Filter State and Logic

- [x] 1.1 Add filter state variables: `logFilterNames: string`, `logFilterRegex: string`, `logFilterRegexObj: RegExp | null`
- [x] 1.2 Implement `shouldFilterEntry(entry: LogEntry): boolean` — checks name match (comma-split, trimmed) and regex test against rendered text; returns true if entry should be hidden
- [x] 1.3 Update `createLogEntryElement()` to accept an optional filter flag and return null for filtered entries
- [x] 1.4 Update `renderLogPanel()` to skip filtered entries when building the HTML
- [x] 1.5 Update `addLogEntry()` to check filter before appending DOM element

## 2. Settings UI for Log Filters

- [x] 2.1 Add "Log Filters" section to `openSettingsPanel()` with message names text input and regex text input
- [x] 2.2 Add info icon (ⓘ) next to the section header with a CSS tooltip showing examples
- [x] 2.3 Wire `input` event listeners on both fields to update filter state and re-render the log panel in real-time

## 3. Info Icon Tooltip

- [x] 3.1 Add CSS for `.info-icon` and `.info-icon:hover::after` tooltip with the example text
- [x] 3.2 Tooltip content shows: message name only (`player-status`), regex only (`state=Stopped`), and combination (`player-status, state=Stopped`)
