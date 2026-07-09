## Purpose

This spec defines the audio pipeline component that manages audio playback, track loading, seeking, and progress reporting. It bridges the frontend IPC layer with the underlying decoder.

## Requirements

### Requirement: Pipeline Load with Streaming Decode

The pipeline SHALL accept a file path in `load_track()` and create a `StreamingDecoder` for it. The method SHALL NOT block on audio decode — it SHALL return as soon as the file is probed and stream metadata (sample rate, channels, duration) is available.

The pipeline SHALL replace the `current_track: Arc<Mutex<Option<DecodedAudio>>>` field with a `current_decoder: Arc<Mutex<Option<StreamingDecoder>>>` field.

The pipeline SHALL expose track metadata (sample rate, channels, duration) from the streaming decoder after load.

#### Scenario: Load track returns quickly
- **WHEN** `load_track()` is called with a path to a 2-hour audio file
- **THEN** the method SHALL return within 200ms without decoding any audio samples

#### Scenario: Load track exposes metadata
- **WHEN** `load_track()` succeeds
- **THEN** the pipeline SHALL provide the track's sample rate, channel count, and duration

### Requirement: Playback Callback with Stream Decode

The pipeline's `play()` method SHALL set up a cpal callback that reads from the `StreamingDecoder` instead of a pre-decoded buffer.

The callback SHALL call `StreamingDecoder::read(num_frames)` to obtain interleaved f32 samples for each cpal buffer request.

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

### Requirement: Seek via Streaming Decoder

The pipeline's `seek(position_secs)` method SHALL delegate to `StreamingDecoder::seek()` instead of computing a sample index into a pre-decoded buffer.

#### Scenario: Seek during streaming playback
- **WHEN** the user seeks to a new position during playback
- **THEN** the audio SHALL continue from approximately that position

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

### Requirement: Backward Compatibility

The pipeline SHALL maintain the same IPC command interface (`load_track`, `play`, `pause`, `resume`, `stop`, `seek`) and event emission format so the frontend requires no changes.

The `load_track` command SHALL still return the track metadata (sample rate, channels, duration) to the frontend as before.

#### Scenario: Frontend unchanged
- **WHEN** the existing frontend calls `invoke("load_track", { path })` and `invoke("play")`
- **THEN** the behavior SHALL be identical except that audio starts faster

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
