## ADDED Requirements

### Requirement: Settings persist across application restarts
The system SHALL persist user-configurable settings to disk so they survive application restarts.

#### Scenario: Log filter names persist after restart
- **WHEN** the user enters "player-status, audio-log" in the log filter names input and closes the application
- **THEN** after reopening the application, the log filter names input SHALL display "player-status, audio-log"
- **AND** log entries matching those names SHALL continue to be filtered

#### Scenario: Log filter regex persists after restart
- **WHEN** the user enters "state=Stopped" in the log filter regex input and closes the application
- **THEN** after reopening the application, the log filter regex input SHALL display "state=Stopped"
- **AND** the regex filter SHALL be active (log entries matching the regex are filtered)

#### Scenario: Empty filter defaults restore as empty
- **WHEN** the user has never set log filter values (or clears both inputs)
- **THEN** after reopening the application, both filter inputs SHALL be empty
- **AND** no log entries SHALL be filtered by name or regex

#### Scenario: Filter changes auto-save immediately
- **WHEN** the user types or modifies either log filter input
- **THEN** the current values SHALL be persisted to disk immediately (within the same event loop tick)

#### Scenario: Existing config without filter fields loads without error
- **WHEN** an `app.toml` file exists from a previous version without `log_filter_names` or `log_filter_regex` fields
- **THEN** the application SHALL load successfully
- **AND** both filter inputs SHALL default to empty strings

#### Scenario: Mix settings continue to persist alongside filters
- **WHEN** the user changes the mix pattern and log filter names, then restarts the application
- **THEN** both the mix pattern AND the log filter names SHALL be restored to their previous values
