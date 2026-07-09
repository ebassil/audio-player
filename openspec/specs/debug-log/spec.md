## Purpose

This spec defines the debug log panel that records and displays IPC communication between the frontend and the Rust audio backend, along with audio processing milestone events from the backend.

## Requirements

### Requirement: Debug log panel

The system SHALL provide a toggleable timestamped log panel in the main content area that records IPC communication between the frontend and the Rust audio backend.

#### Scenario: Log panel visibility toggle
- **WHEN** user clicks the "Log" button in the header toolbar
- **THEN** the log panel is shown in the main content area (where the plugin UI container currently appears)
- **AND** the Log button displays a visual active state (highlighted background or border)
- **AND** clicking the button again hides the log panel and removes the active state

#### Scenario: Settings panel deactivates log panel
- **WHEN** user clicks the settings button while the log panel is visible
- **THEN** the log panel is hidden and its visibility state is reset
- **AND** the Log button's active state is removed
- **AND** the settings panel is displayed in the main content area
- **AND** clicking the "Log" button again re-shows the log panel

#### Scenario: Plugin UI replaces log panel
- **WHEN** the user loads a plugin UI while the log panel is visible
- **THEN** the log panel is hidden and its visibility state is reset
- **AND** the Log button's active state is removed
- **AND** clicking the "Log" button again re-shows the log panel

#### Scenario: Log records invoke calls
- **WHEN** any `invoke()` call is made from the frontend
- **THEN** a log entry is added with: ISO-8601 timestamp, command name, arguments (truncated to 200 chars), result or error status, and direction indicator `→`

#### Scenario: Log records Tauri events
- **WHEN** an event is received from the Rust backend via `listen()`
- **THEN** a log entry is added with: ISO-8601 timestamp, event name, payload summary, and direction indicator `←`

#### Scenario: Log display format
- **WHEN** the log panel is visible
- **THEN** entries are displayed newest-first or oldest-first (with a toggle)
- **AND** each entry shows timestamp, direction arrow, command/event name, and status
- **AND** entries are color-coded: green for success, red for errors, blue for events

#### Scenario: Auto-scroll behavior
- **WHEN** a new log entry is added and the panel is at the bottom
- **THEN** the panel auto-scrolls to show the new entry
- **AND** if the user has scrolled up, auto-scroll pauses until the user scrolls back to the bottom

#### Scenario: Log buffer limit
- **WHEN** the number of log entries exceeds 1000
- **THEN** the oldest entries are evicted to maintain the buffer limit

#### Scenario: Filter by message name
- **WHEN** the user configures a message name filter (e.g., `player-status`)
- **THEN** log entries whose name equals the configured value are hidden from the log display
- **AND** hidden entries remain in the buffer and reappear if the filter is cleared

#### Scenario: Filter by regex
- **WHEN** the user configures a regex filter
- **THEN** log entries whose rendered text matches the regex are hidden from the log display

#### Scenario: Filter hides entries from render
- **WHEN** `renderLogPanel()` or `createLogEntryElement()` is called
- **THEN** filtered entries are not rendered as DOM elements
- **AND** the log appears as if those entries never existed

#### Scenario: Throttled player-status events
- **WHEN** `player-status` events are received at 250ms intervals
- **THEN** only one log entry per second is recorded for this event type to avoid flooding
- **AND** other event types are recorded at full frequency

### Requirement: Audio backend event logging

The Rust backend SHALL emit `audio-log` Tauri events for audio processing milestones, which appear in the frontend log panel.

#### Scenario: Audio log events emitted
- **WHEN** a significant audio operation occurs (track loaded, decode started, decode completed, seek performed, transition started, transition completed, plugin error)
- **THEN** the backend emits an `audio-log` event with a message string and ISO-8601 timestamp

#### Scenario: Audio log displayed in log panel
- **WHEN** an `audio-log` event is received by the frontend
- **THEN** it appears in the log panel with the `←` direction indicator, event name `audio-log`, and the message
