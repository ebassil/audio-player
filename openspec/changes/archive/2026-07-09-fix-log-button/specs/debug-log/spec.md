## MODIFIED Requirements

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
