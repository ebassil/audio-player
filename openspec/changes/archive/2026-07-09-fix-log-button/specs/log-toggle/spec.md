## ADDED Requirements

### Requirement: Log button visual active state

The Log button SHALL show a visual active state when the log panel is visible, and return to normal state when the log panel is hidden.

#### Scenario: Button shows active state
- **WHEN** the user clicks the Log button
- **THEN** the log panel is displayed
- **AND** the Log button receives a visual active state (e.g., highlighted background, border)

#### Scenario: Button returns to normal state
- **WHEN** the user clicks the Log button while the log panel is visible
- **THEN** the log panel is hidden
- **AND** the Log button returns to its normal visual state

### Requirement: State consistency across panel switches

The log panel visibility state SHALL remain consistent when other panels (plugin UI, settings) modify the shared content area.

#### Scenario: Plugin UI resets log state
- **WHEN** the user loads a plugin UI while the log panel is visible
- **THEN** the log panel is replaced by the plugin UI
- **AND** `logPanelVisible` is set to `false`
- **AND** the Log button's active state is removed

#### Scenario: Settings panel resets log state
- **WHEN** the user opens the settings panel while the log panel is visible
- **THEN** the log panel is replaced by the settings panel
- **AND** `logPanelVisible` is set to `false`
- **AND** the Log button's active state is removed

### Requirement: Reliable toggling after state reset

The Log button SHALL reliably toggle the log panel on and off even after the log state has been reset by another panel.

#### Scenario: Toggle after plugin UI closes
- **WHEN** the user clicks the Log button after a plugin UI has replaced the log panel
- **THEN** the log panel is displayed as expected

#### Scenario: Toggle after settings closes
- **WHEN** the user clicks the Log button after the settings panel has replaced the log panel
- **THEN** the log panel is displayed as expected
