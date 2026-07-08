## ADDED Requirements

### Requirement: Streaming Decoder

The system SHALL provide a `StreamingDecoder` that decodes audio files incrementally, yielding interleaved f32 PCM samples on-demand without decoding the entire file upfront.

The decoder SHALL support the same formats as the current decoder (MP3, WAV, FLAC, OGG, AAC, M4A, Opus) via symphonia.

The decoder SHALL expose the following metadata from the stream header without requiring full decode: sample rate, channel count, duration in seconds.

#### Scenario: Open and read metadata without decoding
- **WHEN** the `StreamingDecoder` is created for a valid audio file
- **THEN** it SHALL return the file's sample rate, channel count, and duration without decoding any audio packets

#### Scenario: Read samples incrementally
- **WHEN** `read(num_frames)` is called on a `StreamingDecoder`
- **THEN** it SHALL return up to `num_frames * channels` interleaved f32 samples, decoding only as many packets as needed to satisfy the request

#### Scenario: Back-to-back reads produce contiguous audio
- **WHEN** two consecutive `read()` calls are made
- **THEN** the second call SHALL return samples immediately following the first, with no gaps or overlaps

#### Scenario: Read past end of file
- **WHEN** `read()` is called after all audio packets have been consumed
- **THEN** it SHALL return the remaining partial buffer (if any) padded with silence, and subsequent calls SHALL return silence

#### Scenario: Unsupported format
- **WHEN** the `StreamingDecoder` is created for an unsupported audio format
- **THEN** it SHALL return an error

### Requirement: Internal Decode Buffer

The `StreamingDecoder` SHALL maintain an internal ring buffer of decoded f32 samples to decouple packet decode timing from the playback callback.

The buffer SHALL prefill to a configurable frame count (default 8192 frames) and refill when remaining frames fall below a threshold (default 2048 frames).

#### Scenario: Buffer prefill
- **WHEN** the `StreamingDecoder` is created and the first `read()` is called
- **THEN** it SHALL decode packets into the internal buffer until the buffer contains at least the prefill frame count

#### Scenario: Buffer refill during reading
- **WHEN** the remaining frames in the internal buffer drop below the refill threshold during a `read()` call
- **THEN** the decoder SHALL decode additional packets to refill the buffer

### Requirement: Seek in Streaming Decoder

The `StreamingDecoder` SHALL support seeking to a time position in seconds.

Seeking SHALL re-seek the underlying symphonia format reader to the nearest packet before the target time, reset the decoder state, and clear the internal ring buffer.

After seeking, the next `read()` call SHALL return samples from the seeked position.

#### Scenario: Seek forward
- **WHEN** `seek(30.0)` is called on a track longer than 30 seconds
- **THEN** the next `read()` call SHALL return samples starting from approximately 30 seconds into the track

#### Scenario: Seek backward
- **WHEN** `seek(10.0)` is called after the read position has advanced past 30 seconds
- **THEN** the next `read()` call SHALL return samples starting from approximately 10 seconds into the track

#### Scenario: Seek past end
- **WHEN** `seek(duration + 10)` is called
- **THEN** the next `read()` call SHALL return silence
