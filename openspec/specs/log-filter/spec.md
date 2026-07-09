## Purpose

This spec defines the log filter settings that allow users to suppress debug log entries by message name and/or regex pattern.

## Requirements

### Requirement: Log filter configuration UI

The system SHALL provide a "Log Filters" section in the settings panel where users can configure which log entries are hidden from the debug log.

#### Scenario: Settings panel shows log filter section
- **WHEN** the user opens the settings panel
- **THEN** a "Log Filters" section is visible with message names input, regex input, and an info icon

#### Scenario: Message names input
- **WHEN** the user enters comma-separated message names (e.g., `player-status, audio-log`) in the message names field
- **THEN** log entries with matching names are hidden from the log panel

#### Scenario: Regex input
- **WHEN** the user enters a regex pattern (e.g., `state=Stopped`) in the regex field
- **THEN** log entries whose combined text matches the pattern are hidden from the log panel

#### Scenario: Combined filters
- **WHEN** both message names and a regex are configured
- **THEN** log entries matching either the message names OR the regex are hidden (OR logic)

#### Scenario: Invalid regex handled gracefully
- **WHEN** the user enters an invalid regex pattern
- **THEN** no entries are filtered (the invalid regex is silently ignored)
- **AND** the UI does not break or show an error

### Requirement: Filter info tooltip

The system SHALL display an info icon next to the log filter settings header with a tooltip showing usage examples.

#### Scenario: Info icon tooltip on hover
- **WHEN** the user hovers over the info icon
- **THEN** a tooltip is displayed showing examples: message name only (`player-status`), regex only (`state=Stopped`), and combination (`player-status, state=Stopped`)

#### Scenario: Info icon tooltip on blur
- **WHEN** the user moves the cursor away from the info icon
- **THEN** the tooltip disappears

### Requirement: Real-time filter application

The system SHALL apply filters in real-time as the user types filter criteria.

#### Scenario: Filters apply immediately
- **WHEN** the user types in the message names or regex field
- **THEN** the log panel updates immediately to show/hide matching entries without requiring a button click

#### Scenario: Filter changes affect existing entries
- **WHEN** the user modifies a filter
- **THEN** all existing log entries are re-evaluated and visibility updates accordingly

#### Scenario: New entries respect active filters
- **WHEN** a new log entry arrives while filters are active
- **THEN** the entry is hidden if it matches any active filter
