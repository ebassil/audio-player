## ADDED Requirements

### Requirement: Debug log panel

The system SHALL filter log entries from display based on user-configured message names and/or regex patterns, while continuing to store all entries in the buffer.

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
