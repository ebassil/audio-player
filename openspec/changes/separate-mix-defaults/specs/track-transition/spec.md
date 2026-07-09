## MODIFIED Requirements

### Requirement: Transition with configured mix pattern

The system SHALL apply the mix pattern (crossfade, fade, or hard fade) during each track transition. The pattern SHALL be resolved by checking per-track overrides first, then falling back to the engine's default config.

#### Scenario: Per-track override used for pattern
- **WHEN** a track has `mix_pattern_override` set to `"fade"`
- **THEN** the transition SHALL use the Fade pattern regardless of the default mix config
- **AND** SHALL generate the corresponding gain envelopes (fade out → gap → fade in)

#### Scenario: Default pattern used when no override
- **WHEN** a track has no `mix_pattern_override`
- **THEN** the transition SHALL use the default mix pattern from the engine config

### Requirement: Transition duration from mix settings

The system SHALL use the mix duration (default 3.0 seconds) to determine when to start preparing the next track and how long the gain envelope lasts. The duration SHALL be resolved by checking per-track overrides first, then falling back to the engine's default config.

#### Scenario: Per-track duration override used
- **WHEN** a track has `mix_duration_override` set to `8.0`
- **THEN** the transition SHALL use 8.0 seconds for the gain ramp duration
- **AND** SHALL trigger transition preparation at `(track duration - 8.0)` seconds (or at the mix-out point, if set)

#### Scenario: Default duration used when no override
- **WHEN** a track has no `mix_duration_override`
- **THEN** the transition SHALL use the default mix duration from the engine config

#### Scenario: Short track with override longer than track
- **WHEN** a track's remaining duration is shorter than the resolved mix duration (whether from override or default)
- **THEN** the transition SHALL start immediately and the transition duration SHALL be clamped to the remaining track time

### Requirement: Per-song mix point overrides

The system SHALL respect per-song mix-out and mix-in point overrides when they are defined for the current or next track. Mix points SHALL function independently from pattern and duration overrides — they can be combined arbitrarily.

#### Scenario: Mix-out point and duration override combine
- **WHEN** the current track has both a mix-out point of `120.0` and a mix duration override of `5.0`
- **THEN** the transition SHALL begin at 120.0 seconds
- **AND** the gain envelope SHALL span 5.0 seconds
- **AND** if no mix-in point is set, the fade-in SHALL resolve at `120.0 + 5.0` seconds into the next track

#### Scenario: Mix-out point overrides default trigger calculation
- **WHEN** the current track has a mix-out point defined but no duration override
- **THEN** the transition SHALL begin at that mix-out time offset instead of `(duration - default_mix_duration)`
