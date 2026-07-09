## MODIFIED Requirements

### Requirement: Transition envelope uses full mix duration with decoder offset

The pipeline SHALL size the transition gain envelope to the resolved mix duration regardless of the remaining track time. When the remaining track time is less than the mix duration, the pipeline SHALL seek the next decoder's start position forward to compensate.

#### Scenario: Envelope sized at mix_duration, decoder offset computed
- **WHEN** the async next-decoder load completes and the transition is about to begin
- **THEN** the pipeline SHALL compute `excess = max(0, mix_duration - remaining)`
- **AND** SHALL compute `next_offset = mix_in_point.map_or(excess, |m| m + excess)`
- **AND** SHALL seek the next decoder to `next_offset` (clamped to track duration)
- **AND** SHALL size the gain envelope to `mix_duration` frames (not `mix_duration.min(remaining)`)

#### Scenario: Transition uses existing next_decoder.seek()
- **WHEN** the pipeline seeks the next decoder during transition setup
- **THEN** it SHALL call `next_decoder.seek(offset)` which is asynchronous — the seek completes on the decode thread before the transition phase begins reading samples
- **AND** the pipeline SHALL NOT hold a lock on next_decoder across the transition phase boundary

### Requirement: Mix duration capped at 15 seconds, exempted by mix_out point

The pipeline SHALL enforce a maximum mix duration of 15.0 seconds, except when a `mix_out` point is defined for the current track. Any mix duration value — whether from the default config, a per-track override, or the `MixEngine::resolve()` output — that exceeds 15.0 seconds SHALL be clamped to 15.0 seconds before sizing the gain envelope, unless `mix_out` is set.

When `mix_out` is defined for the current track, the effective mix duration SHALL be `track_duration - mix_out_point`, and the 15.0-second cap SHALL NOT apply. The gain envelope SHALL be extended to match this duration.

#### Scenario: Pipeline clamps overridden mix duration
- **WHEN** a per-track `mix_duration_override` of `20.0` is resolved and no `mix_out` point is defined
- **THEN** the effective mix duration used for the envelope SHALL be `15.0` seconds
- **AND** the trigger timing SHALL use the clamped value for `(duration - min(override, 15.0))`

#### Scenario: Pipeline exempts cap when mix_out defined
- **WHEN** a track has `mix_out` at 120.0 seconds, track duration is 140.0 seconds, and the resolved mix duration is 15.0 seconds
- **THEN** the effective mix duration SHALL be 20.0 seconds (`140.0 - 120.0`)
- **AND** the 15.0-second cap SHALL NOT apply
- **AND** the gain envelope SHALL span 20.0 seconds
