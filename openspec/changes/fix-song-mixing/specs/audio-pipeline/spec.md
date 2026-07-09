## MODIFIED Requirements

### Requirement: Playback Callback with Stream Decode

The pipeline's `play()` method SHALL set up a cpal callback that reads from the `StreamingDecoder` instead of a pre-decoded buffer. The callback SHALL call `StreamingDecoder::read(num_frames)` to obtain interleaved f32 samples for each cpal buffer request.

During a track transition, the callback SHALL read from two decoders simultaneously and apply the MixEngine's gain envelopes before summing the outputs.

#### Scenario: Playback starts immediately after load
- **WHEN** `play()` is called after `load_track()`
- **THEN** audio SHALL start within 200ms (the time to decode the first buffer of samples)

#### Scenario: Normal playback reads from single decoder
- **WHEN** playback is active and no transition is in progress
- **THEN** the callback SHALL read from the current decoder only

#### Scenario: Transition reads from two decoders
- **WHEN** a transition between tracks is in progress
- **THEN** the callback SHALL read from both the current and next decoders, apply gain envelopes, and sum the outputs

### Requirement: Pipeline Progress and Position

The pipeline's `progress()` and `position_secs()` methods SHALL derive their values from the streaming decoder's current read position and the track's total duration.

During a transition, position SHALL continue reporting the outgoing track's position. After transition completion, position SHALL report the new track's position from time 0.

#### Scenario: Progress reports correct position
- **WHEN** playback has advanced 30 seconds into a 120-second track
- **THEN** `position_secs()` SHALL return approximately 30.0 and `progress()` SHALL return approximately 0.25

#### Scenario: Position during transition reports outgoing track
- **WHEN** a cross-fade transition from track A to track B is in progress
- **THEN** `position_secs()` SHALL return track A's position until the transition completes

#### Scenario: Position after transition reports incoming track
- **WHEN** a transition completes
- **THEN** `position_secs()` SHALL return the position within the new track

## ADDED Requirements

### Requirement: Playlist context for auto-advance

The pipeline SHALL accept a playlist context that provides the ordered list of track file paths and their per-song mix-point overrides. The context SHALL be used to determine the next track and apply mix overrides during transitions.

#### Scenario: Pipeline receives playlist context
- **WHEN** the playlist is loaded or modified
- **THEN** the pipeline SHALL receive the current playlist context via IPC

#### Scenario: No next track available
- **WHEN** playback reaches the end of the last track in the playlist context
- **THEN** the pipeline SHALL stop playback and emit a Stopped state

### Requirement: Track advance event emission

The pipeline SHALL emit a Tauri event when playback advances to a new track, carrying the new track index.

#### Scenario: Frontend receives track advance event
- **WHEN** the pipeline advances to the next track
- **THEN** a `track-changed` event SHALL be emitted with the new track index

### Requirement: Transition state without playback state change

The pipeline SHALL handle transitions internally without changing the externally visible playback state. Throughout a transition, the reported state SHALL remain `Playing`.

#### Scenario: State remains Playing during transition
- **WHEN** a transition between tracks is in progress
- **THEN** `get_status` SHALL report state as `Playing`
