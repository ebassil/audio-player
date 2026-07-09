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

The system SHALL apply the mix pattern (crossfade, fade, or hard fade) during each track transition. The pattern SHALL be resolved by checking per-track overrides first, then falling back to the engine's default config.

#### Scenario: Cross-fade transition
- **WHEN** the mix pattern is set to CrossFade
- **THEN** the outgoing track SHALL fade out while the incoming track fades in, overlapping for the configured mix duration

#### Scenario: Cross-fade transition with correct duration
- **WHEN** the mix pattern is set to CrossFade and mix duration is 15.0s
- **THEN** the outgoing track SHALL fade out while the incoming track fades in
- **AND** the audible overlap between the two tracks SHALL be approximately 15.0 seconds (within tolerance for the async decoder load time)

#### Scenario: Fade transition
- **WHEN** the mix pattern is set to Fade
- **THEN** the outgoing track SHALL fade out, followed by a silence gap, then the incoming track SHALL fade in

#### Scenario: Hard fade transition
- **WHEN** the mix pattern is set to HardFade
- **THEN** the outgoing track SHALL stop instantly, a silence gap SHALL occur, then the incoming track SHALL start instantly

#### Scenario: Per-track override used for pattern
- **WHEN** a track has `mix_pattern_override` set to `"fade"`
- **THEN** the transition SHALL use the Fade pattern regardless of the default mix config
- **AND** SHALL generate the corresponding gain envelopes (fade out → gap → fade in)

#### Scenario: Default pattern used when no override
- **WHEN** a track has no `mix_pattern_override`
- **THEN** the transition SHALL use the default mix pattern from the engine config

### Requirement: Transition duration from mix settings

The system SHALL use the mix duration (default 3.0 seconds, maximum 15.0 seconds) to determine when to start preparing the next track and how long the gain envelope lasts. The duration SHALL be resolved by checking per-track overrides first, then falling back to the engine's default config.

Transition trigger timing SHALL use the actual playback position (frames consumed from the ring buffer) rather than the decode-side position, ensuring the transition starts at the correct point in the audio output stream.

When the remaining track duration at transition start is shorter than the resolved mix duration, the envelope SHALL remain at the full mix duration length and the incoming track's decoder SHALL be seeked forward to compensate.

The mix duration SHALL be capped at 15.0 seconds, except when a `mix_out` point is defined for the current track. When `mix_out` is defined, the effective mix duration SHALL be `track_duration - mix_out_point` (not subject to the 15s cap), and the gain envelope SHALL be extended to match this duration.

#### Scenario: Transition starts before track end
- **WHEN** the actual playback position (not decode-side position) reaches `(track duration - mix duration)` seconds
- **THEN** the system SHALL begin preparing the next track for the transition

#### Scenario: Full mix duration preserved when remaining < mix_duration
- **WHEN** the remaining track duration at transition start is 10.0 seconds and the resolved mix duration is 15.0 seconds
- **THEN** the transition envelope SHALL span 15.0 seconds (not clamped to 10.0)
- **AND** the incoming track decoder SHALL be seeked forward by 5.0 seconds (`mix_duration - remaining`)

#### Scenario: Incoming track offset interacts with mix_in_point
- **WHEN** the next track has a mix_in_point of 8.0 seconds AND `mix_duration - remaining = 5.0` seconds
- **THEN** the incoming track decoder SHALL be seeked forward by 13.0 seconds (`mix_in_point + excess`)

#### Scenario: No offset when remaining >= mix_duration
- **WHEN** the remaining track duration at transition start is 20.0 seconds and the resolved mix duration is 15.0 seconds
- **THEN** the transition envelope SHALL span 15.0 seconds
- **AND** the incoming track decoder SHALL NOT be seeked (starts at position 0)

#### Scenario: Offset clamped to track duration
- **WHEN** the calculated offset exceeds the incoming track's total duration
- **THEN** the decoder seek SHALL be clamped to `track_duration - epsilon`

#### Scenario: Out-track underrun during transition
- **WHEN** the outgoing track reaches EOF before the transition envelope completes
- **THEN** the outgoing decoder SHALL return silence for the remaining frames
- **AND** the transition SHALL continue normally with the incoming track's gain progressing as scheduled

#### Scenario: Per-track duration override used
- **WHEN** a track has `mix_duration_override` set to `8.0`
- **THEN** the transition SHALL use 8.0 seconds for the gain ramp duration
- **AND** SHALL trigger transition preparation at `(track duration - 8.0)` seconds of actual playback (or at the mix-out point, if set)

#### Scenario: Default duration used when no override
- **WHEN** a track has no `mix_duration_override`
- **THEN** the transition SHALL use the default mix duration from the engine config

#### Scenario: Mix duration capped at 15 seconds unless mix_out defined
- **WHEN** a mix duration value above 15.0 seconds is provided (via config default, per-track override, or slider input) and no `mix_out` point is defined for the current track
- **THEN** the value SHALL be clamped to 15.0 seconds
- **AND** the backend SHALL enforce the clamp at the point of resolution in `MixEngine::resolve()` or the pipeline transition start

#### Scenario: Mix_out point overrides the max cap
- **WHEN** a current track has a `mix_out` point defined at 120.0 seconds in a 140.0-second song
- **THEN** the effective mix duration SHALL be 20.0 seconds (`track_duration - mix_out_point`)
- **AND** the 15.0-second max cap SHALL NOT apply
- **AND** the gain envelope SHALL span 20.0 seconds

### Requirement: Per-song mix point overrides

The system SHALL respect per-song mix-out and mix-in point overrides when they are defined for the current or next track. Mix points SHALL function independently from pattern and duration overrides — they can be combined arbitrarily.

#### Scenario: Mix-out point overrides transition start
- **WHEN** the current track has a mix-out point defined
- **THEN** the transition SHALL begin at that mix-out time offset instead of `(duration - mix duration)`

#### Scenario: Mix-in point overrides fade-in timing
- **WHEN** the next track has a mix-in point defined
- **THEN** the next track SHALL reach full gain at that mix-in time offset instead of after `mix_duration` seconds

#### Scenario: Mix-out point and duration override combine
- **WHEN** the current track has both a mix-out point of `120.0` and a mix duration override of `5.0`
- **THEN** the transition SHALL begin at 120.0 seconds
- **AND** the gain envelope SHALL span 5.0 seconds
- **AND** if no mix-in point is set, the fade-in SHALL resolve at `120.0 + 5.0` seconds into the next track

#### Scenario: Mix-out point overrides default trigger calculation
- **WHEN** the current track has a mix-out point defined but no duration override
- **THEN** the transition SHALL begin at that mix-out time offset instead of `(duration - default_mix_duration)`

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
