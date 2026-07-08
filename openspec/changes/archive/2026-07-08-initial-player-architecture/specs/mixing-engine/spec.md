## ADDED Requirements

### Requirement: Mix Patterns
The system SHALL support the following mix patterns between consecutive tracks: fade, cross-fade, and hard fade.
The user SHALL select a default mix pattern and default mix duration (in seconds) via the UI or config.
The default mix duration SHALL apply to all transitions unless a per-song mix point overrides it.

#### Scenario: Fade out / fade in
- **WHEN** the current track ends with fade pattern
- **THEN** the system SHALL fade out the current track over the mix duration, then fade in the next track

#### Scenario: Cross-fade
- **WHEN** the current track ends with cross-fade pattern
- **THEN** the system SHALL fade out the current track while simultaneously fading in the next track over the mix duration, overlapping both

#### Scenario: Hard fade (gap)
- **WHEN** the current track ends with hard fade pattern
- **THEN** the system SHALL stop the current track, insert silence for the mix duration, then start the next track

### Requirement: Per-Song Mix Points
The system SHALL allow defining mix-in and mix-out points for each song stored in the playlist JSON.
A mix-out point SHALL define a time offset from the start of the song where the mix transition begins.
A mix-in point SHALL define a time offset from the start of the song where the incoming track becomes fully audible.
When a song has defined mix points, they SHALL override the default mix duration for that transition.

#### Scenario: Mix-out point defined
- **WHEN** a song has a mix-out point at 3:00
- **THEN** the mix engine SHALL begin the transition at 3:00, regardless of the song's total duration

#### Scenario: Mix-in point defined
- **WHEN** a song has a mix-in point at 0:30
- **THEN** the mix engine SHALL complete the transition by 0:30 into the incoming song

### Requirement: Mix Point Configuration
Mix-in and mix-out points SHALL be configurable from the UI while a song is playing or selected.
Mix points SHALL be saved as part of the playlist JSON.
The UI SHALL provide a waveform or timeline visualization for setting mix points visually.

#### Scenario: Set mix point from UI
- **WHEN** the user sets a mix-out point on a song via the UI
- **THEN** the point SHALL be stored in the current playlist data

#### Scenario: Mix point persists in playlist
- **WHEN** the user saves a playlist with mix points and reloads it
- **THEN** the mix points SHALL be restored
