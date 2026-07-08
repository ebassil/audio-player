## ADDED Requirements

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

#### Scenario: Playback starts immediately after load
- **WHEN** `play()` is called after `load_track()`
- **THEN** audio SHALL start within 200ms (the time to decode the first buffer of samples)

### Requirement: Seek via Streaming Decoder

The pipeline's `seek(position_secs)` method SHALL delegate to `StreamingDecoder::seek()` instead of computing a sample index into a pre-decoded buffer.

#### Scenario: Seek during streaming playback
- **WHEN** the user seeks to a new position during playback
- **THEN** the audio SHALL continue from approximately that position

### Requirement: Pipeline Progress and Position

The pipeline's `progress()` and `position_secs()` methods SHALL derive their values from the streaming decoder's current read position and the track's total duration.

#### Scenario: Progress reports correct position
- **WHEN** playback has advanced 30 seconds into a 120-second track
- **THEN** `position_secs()` SHALL return approximately 30.0 and `progress()` SHALL return approximately 0.25

### Requirement: Backward Compatibility

The pipeline SHALL maintain the same IPC command interface (`load_track`, `play`, `pause`, `resume`, `stop`, `seek`) and event emission format so the frontend requires no changes.

The `load_track` command SHALL still return the track metadata (sample rate, channels, duration) to the frontend as before.

#### Scenario: Frontend unchanged
- **WHEN** the existing frontend calls `invoke("load_track", { path })` and `invoke("play")`
- **THEN** the behavior SHALL be identical except that audio starts faster
