## Purpose

This spec defines the track transition component that manages automatic track advancement, mix pattern application, and gain envelope processing during song transitions.

## Requirements

### Requirement: Automatic track advancement on track end

The system SHALL automatically advance to the next track in the playlist when the current track's playback reaches its end, using the configured mix pattern and duration for the transition.

The system SHALL continue advancing through the playlist until all tracks have been played or the user manually stops playback.

#### Scenario: Next track plays after current track ends
- **WHEN** a track finishes playing and there is a next track in the playlist
- **THEN** the system SHALL automatically start playing the next track using the configured mix settings

#### Scenario: Playback stops at end of playlist
- **WHEN** the last track in the playlist finishes playing
- **THEN** the system SHALL stop playback and report the Stopped state

#### Scenario: User stops during transition
- **WHEN** the user triggers stop during a track transition
- **THEN** the system SHALL immediately stop all audio and reset to Stopped state

### Requirement: Transition with configured mix pattern

The system SHALL apply the configured mix pattern (crossfade, fade, or hard fade) during each track transition.

#### Scenario: Cross-fade transition
- **WHEN** the mix pattern is set to CrossFade
- **THEN** the outgoing track SHALL fade out while the incoming track fades in, overlapping for the configured mix duration

#### Scenario: Fade transition
- **WHEN** the mix pattern is set to Fade
- **THEN** the outgoing track SHALL fade out, followed by a silence gap, then the incoming track SHALL fade in

#### Scenario: Hard fade transition
- **WHEN** the mix pattern is set to HardFade
- **THEN** the outgoing track SHALL stop instantly, a silence gap SHALL occur, then the incoming track SHALL start instantly

### Requirement: Transition duration from mix settings

The system SHALL use the configured mix duration (default 3.0 seconds) to determine when to start preparing the next track and how long the gain envelope lasts.

#### Scenario: Transition starts before track end
- **WHEN** playback position reaches `(track duration - mix duration)` seconds
- **THEN** the system SHALL begin preparing the next track for the transition

#### Scenario: Short track with duration shorter than mix duration
- **WHEN** a track's remaining duration is shorter than the configured mix duration
- **THEN** the transition SHALL start immediately and the transition duration SHALL be clamped to the remaining track time

### Requirement: Per-song mix point overrides

The system SHALL respect per-song mix-out and mix-in point overrides when they are defined for the current or next track.

#### Scenario: Mix-out point overrides transition start
- **WHEN** the current track has a mix-out point defined
- **THEN** the transition SHALL begin at that mix-out time offset instead of `(duration - mix duration)`

#### Scenario: Mix-in point overrides fade-in timing
- **WHEN** the next track has a mix-in point defined
- **THEN** the next track SHALL reach full gain at that mix-in time offset instead of after `mix_duration` seconds

### Requirement: Gain envelope during transition

The system SHALL apply gain envelopes to both outgoing and incoming track audio during the transition period.

#### Scenario: Gain ramps are sample-accurate
- **WHEN** a cross-fade transition is in progress
- **THEN** the outgoing track's gain SHALL ramp from 1.0 to 0.0 and the incoming track's gain SHALL ramp from 0.0 to 1.0 over the transition duration

### Requirement: Frontend tracks current playback index

The system SHALL notify the frontend when the track advances so the playlist selection and display can update.

#### Scenario: Playlist selection updates on advance
- **WHEN** playback advances to the next track
- **THEN** the frontend SHALL update the playlist view to reflect the new current track
