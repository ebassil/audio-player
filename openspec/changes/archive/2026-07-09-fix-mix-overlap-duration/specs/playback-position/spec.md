## ADDED Requirements

### Requirement: Playback position tracking

The system SHALL track the actual playback position (frames consumed from the ring buffer by the audio callback) separately from the decode-side position (frames pushed by the decode thread). The playback position SHALL be used for transition trigger and effective-duration calculations.

#### Scenario: Playback position starts at zero
- **WHEN** a new track begins playback
- **THEN** `playback_position_secs()` SHALL return approximately 0.0

#### Scenario: Playback position advances with audio output
- **WHEN** 30 seconds of audio have been output to the system audio device
- **THEN** `playback_position_secs()` SHALL return approximately 30.0, regardless of how many frames the decode thread has buffered

#### Scenario: Playback position unaffected by decode ahead
- **WHEN** the decode thread has buffered 6 seconds of audio in the ring buffer while the callback has consumed 10 seconds
- **THEN** `playback_position_secs()` SHALL return approximately 10.0 (not 16.0)

### Requirement: Ring buffer consumed frame counter

The `AudioRingBuf` SHALL expose the number of samples consumed (popped) via a `consumed()` method, returning the head index.

#### Scenario: Consumed counter matches pops
- **WHEN** 44100 samples have been popped from the ring buffer
- **THEN** `consumed()` SHALL return 44100

#### Scenario: Consumed counter wraps with buffer index
- **WHEN** the head index has wrapped around the ring buffer capacity multiple times
- **THEN** `consumed()` SHALL be monotonic and SHALL always reflect the total popped count (capacity * wraps + current_head)
