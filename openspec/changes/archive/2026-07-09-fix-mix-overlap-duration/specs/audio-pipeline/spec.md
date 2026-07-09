## ADDED Requirements

### Requirement: Transition trigger uses playback position

The pipeline SHALL use the actual playback position (frames consumed from the ring buffer) for transition trigger and effective-duration calculations, rather than the decode-side frame count. The public-facing position reported to the frontend via IPC SHALL remain unchanged.

#### Scenario: Transition trigger fires at correct output position
- **WHEN** the audio output has reached `(track duration - mix duration)` seconds of playback
- **THEN** the pipeline SHALL begin preparing the next track, regardless of the decode thread's buffered frame count

### Requirement: Transition envelope recalculated on load completion

The pipeline SHALL defer the gain envelope frame calculation until the async next-decoder load completes, using the playback position at that moment rather than the trigger-time position.

#### Scenario: Envelope accounts for async load delay
- **WHEN** the async next-decoder load takes 0.3 seconds to complete
- **THEN** the gain envelope SHALL be sized using the playback position at load-complete time, not the trigger time
- **AND** the envelope SHALL be `min(mix_duration, duration - current_playback_position)` frames long
