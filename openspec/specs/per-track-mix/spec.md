## Purpose

Support per-track mix pattern and duration overrides stored in the playlist state, allowing each track to use mix settings different from the application defaults.

## Requirements

### Requirement: Per-track mix pattern override

Each track in the playlist SHALL support an optional mix pattern override stored in the playlist state JSON. When set, this override SHALL be used instead of the default mix pattern from the application config when transitioning from this track to the next.

#### Scenario: Track has mix pattern override
- **WHEN** a track in the playlist has `mix_pattern_override` set to `"fade"`
- **THEN** the system SHALL use the Fade pattern for the transition from this track to the next
- **AND** SHALL ignore the default mix pattern from the application config for this transition

#### Scenario: Track has no mix pattern override
- **WHEN** a track in the playlist has `mix_pattern_override` set to `null` or is absent
- **THEN** the system SHALL use the default mix pattern from the application config for the transition

### Requirement: Per-track mix duration override

Each track in the playlist SHALL support an optional mix duration override stored in the playlist state JSON. When set, this override SHALL be used instead of the default mix duration from the application config when transitioning from this track to the next.

#### Scenario: Track has mix duration override
- **WHEN** a track in the playlist has `mix_duration_override` set to `5.0`
- **THEN** the system SHALL use a 5-second mix duration for the transition from this track to the next
- **AND** SHALL ignore the default mix duration from the application config for this transition

#### Scenario: Track has no mix duration override
- **WHEN** a track in the playlist has `mix_duration_override` set to `null` or is absent
- **THEN** the system SHALL use the default mix duration from the application config for the transition

### Requirement: Main screen mix controls edit per-track overrides

The mix pattern selector and mix duration slider on the main screen SHALL read and write per-track overrides in the playlist state JSON, not the application config.

#### Scenario: Main screen mix controls load per-track override
- **WHEN** the user selects a track in the playlist that has `mix_pattern_override` and `mix_duration_override` set
- **THEN** the main screen mix pattern selector SHALL display the overridden pattern
- **AND** the mix duration slider SHALL display the overridden duration

#### Scenario: Main screen mix controls fall back to defaults
- **WHEN** the user selects a track in the playlist that has no mix overrides
- **THEN** the main screen mix pattern selector SHALL display the default pattern from the application config
- **AND** the mix duration slider SHALL display the default duration from the application config

#### Scenario: Main screen mix controls save per-track
- **WHEN** the user changes the mix pattern or duration on the main screen
- **THEN** the system SHALL save the new values as per-track overrides in the playlist state JSON
- **AND** SHALL NOT modify the application config defaults

### Requirement: Per-track mix overrides persist in playlist state

The per-track mix pattern override and mix duration override SHALL be serialised to and deserialised from the playlist state JSON file alongside the existing mix point overrides.

#### Scenario: Per-track overrides survive restart
- **WHEN** the user sets a mix pattern override for a track, then restarts the application
- **THEN** after restart, the mix pattern override SHALL still be associated with that track
- **AND** the main screen mix controls SHALL display the restored overrides when that track is selected
