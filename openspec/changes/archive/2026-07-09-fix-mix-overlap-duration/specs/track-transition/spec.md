## MODIFIED Requirements

### Requirement: Transition duration from mix settings

The system SHALL use the mix duration (default 3.0 seconds) to determine when to start preparing the next track and how long the gain envelope lasts. The duration SHALL be resolved by checking per-track overrides first, then falling back to the engine's default config.

Transition trigger timing SHALL use the actual playback position (frames consumed from the ring buffer) rather than the decode-side position, ensuring the transition starts at the correct point in the audio output stream.

The effective transition envelope duration SHALL be recalculated when the next decoder finishes loading (asynchronously) — not at trigger time — to account for the brief advance of playback position during the load window.

#### Scenario: Transition starts before track end
- **WHEN** the actual playback position (not decode-side position) reaches `(track duration - mix duration)` seconds
- **THEN** the system SHALL begin preparing the next track for the transition

#### Scenario: Short track with duration shorter than mix duration
- **WHEN** a track's remaining duration at transition trigger time is shorter than the configured mix duration
- **THEN** the transition SHALL prepare the next track and the transition envelope duration SHALL be recalculated at transition start to match the remaining track duration at that time

#### Scenario: Per-track duration override used
- **WHEN** a track has `mix_duration_override` set to `8.0`
- **THEN** the transition SHALL use 8.0 seconds for the gain ramp duration
- **AND** SHALL trigger transition preparation at `(track duration - 8.0)` seconds of actual playback (or at the mix-out point, if set)

#### Scenario: Default duration used when no override
- **WHEN** a track has no `mix_duration_override`
- **THEN** the transition SHALL use the default mix duration from the engine config

#### Scenario: Short track with override longer than track
- **WHEN** a track's remaining duration at transition start (after async load) is shorter than the resolved mix duration
- **THEN** the transition envelope duration SHALL be clamped to the remaining track duration at transition start

#### Scenario: Envelope duration recalculated at transition start
- **WHEN** the async decoder load completes and the transition is about to begin
- **THEN** the system SHALL recalculate the remaining track duration using current playback position
- **AND** SHALL size the gain envelope to `min(mix_duration, remaining_at_transition_start)` instead of using the trigger-time value

### Requirement: Transition with configured mix pattern

The system SHALL apply the mix pattern (crossfade, fade, or hard fade) during each track transition. The pattern SHALL be resolved by checking per-track overrides first, then falling back to the engine's default config.

#### Scenario: Cross-fade transition with correct duration
- **WHEN** the mix pattern is set to CrossFade and mix duration is 15.0s
- **THEN** the outgoing track SHALL fade out while the incoming track fades in
- **AND** the audible overlap between the two tracks SHALL be approximately 15.0 seconds (within tolerance for the async decoder load time)
