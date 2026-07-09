## MODIFIED Requirements

### Requirement: Settings persist across application restarts

The system SHALL persist user-configurable settings to disk so they survive application restarts.

#### Scenario: Default mix pattern persists after restart
- **WHEN** the user changes the default mix pattern in the settings panel and saves
- **THEN** after restarting the application, the default mix pattern SHALL be restored
- **AND** any track without a per-track mix pattern override SHALL use this default

#### Scenario: Default mix duration persists after restart
- **WHEN** the user changes the default mix duration in the settings panel and saves
- **THEN** after restarting the application, the default mix duration SHALL be restored
- **AND** any track without a per-track mix duration override SHALL use this default

#### Scenario: Main screen mix changes do not affect defaults
- **WHEN** the user changes the mix pattern on the main screen for a specific track
- **THEN** the default mix pattern in the settings panel SHALL remain unchanged
- **AND** the settings panel SHALL still display the previously saved default value

## ADDED Requirements

### Requirement: Settings panel mix controls save to app config

The mix pattern selector and mix duration slider in the settings panel SHALL save their values to the application config file (`app.toml`) immediately when changed. These values serve as the default mix for any track that does not have per-track overrides.

#### Scenario: Settings mix change saves to app config
- **WHEN** the user changes the default mix pattern or duration in the settings panel
- **THEN** the system SHALL call `save_app_config` to persist the new defaults to `app.toml`
- **AND** the engine's default mix config SHALL be updated to reflect the change

#### Scenario: Per-track override does not affect settings display
- **WHEN** the user sets a per-track mix override from the main screen
- **THEN** the settings panel mix controls SHALL continue to display the saved defaults from `app.toml`
- **AND** SHALL NOT change to reflect the per-track override
