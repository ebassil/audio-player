## MODIFIED Requirements

### Requirement: Transition duration from mix settings

The system SHALL use the mix duration (default 3.0 seconds, maximum 15.0 seconds) to determine when to start preparing the next track and how long the gain envelope lasts. The duration SHALL be resolved by checking per-track overrides first, then falling back to the engine's default config.

Transition trigger timing SHALL use the actual playback position (frames consumed from the ring buffer) rather than the decode-side position, ensuring the transition starts at the correct point in the audio output stream.

When the remaining track duration at transition start is shorter than the resolved mix duration, the envelope SHALL remain at the full mix duration length and the incoming track's decoder SHALL be seeked forward to compensate.

The mix duration SHALL be capped at 15.0 seconds, except when a `mix_out` point is defined for the current track. When `mix_out` is defined, the effective mix duration SHALL be `track_duration - mix_out_point` (not subject to the 15s cap), and the gain envelope SHALL be extended to match this duration.

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

#### Scenario: Mix duration capped at 15 seconds unless mix_out defined
- **WHEN** a mix duration value above 15.0 seconds is provided (via config default, per-track override, or slider input) and no `mix_out` point is defined for the current track
- **THEN** the value SHALL be clamped to 15.0 seconds
- **AND** the backend SHALL enforce the clamp at the point of resolution in `MixEngine::resolve()` or the pipeline transition start

#### Scenario: Mix_out point overrides the max cap
- **WHEN** a current track has a `mix_out` point defined at 120.0 seconds in a 140.0-second song
- **THEN** the effective mix duration SHALL be 20.0 seconds (`track_duration - mix_out_point`)
- **AND** the 15.0-second max cap SHALL NOT apply
- **AND** the gain envelope SHALL span 20.0 seconds
