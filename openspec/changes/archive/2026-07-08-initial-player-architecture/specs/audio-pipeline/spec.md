## ADDED Requirements

### Requirement: Audio Decoding
The system SHALL decode audio files of supported formats into PCM float samples using the symphonia crate in Rust.
The system SHALL support MP3, WAV, and FLAC formats as a minimum.
The system SHALL report the sample rate, channel count, and bit depth of decoded audio.

#### Scenario: Decode MP3 file successfully
- **WHEN** the system loads an MP3 file
- **THEN** it SHALL return decoded PCM float samples with correct sample rate and channel count

#### Scenario: Decode unsupported format
- **WHEN** the system attempts to decode an unsupported audio format
- **THEN** it SHALL return an error indicating the format is not supported

### Requirement: Audio Processing Graph
The system SHALL process decoded audio through a fundsp audio graph before output.
The graph SHALL support a chain of pre-processing nodes (applied before mixing) and post-processing nodes (applied after mixing).
The graph SHALL be configurable at runtime — nodes can be added, removed, reordered, and bypassed.
The graph SHALL operate in a pull-based model where each node requests samples from its inputs on demand.

#### Scenario: Insert processing node into graph
- **WHEN** a plugin node is registered in the graph
- **THEN** audio SHALL pass through that node during playback

#### Scenario: Bypass processing node
- **WHEN** a processing node is bypassed
- **THEN** audio SHALL pass through unchanged as if the node were not present

### Requirement: Audio Output
The system SHALL output processed audio to the system's default audio device using the cpal crate.
The system SHALL handle audio device changes (e.g., plugging/unplugging headphones) gracefully without crashing.
The system SHALL support configurable output sample rate and buffer size.

#### Scenario: Playback to default device
- **WHEN** playback starts
- **THEN** audio SHALL be heard from the system's default audio output device

#### Scenario: Device disconnection during playback
- **WHEN** the audio output device is disconnected during playback
- **THEN** the system SHALL pause playback and notify the user

### Requirement: Volume Control
The system SHALL support volume control with gain applied in the audio graph.
Volume SHALL be controllable from the UI via `invoke` from `@tauri-apps/api/core` (Tauri v2 pattern, per ADR-006).
The system SHALL support mute/unmute without losing the current volume level.

#### Scenario: Set volume from UI
- **WHEN** the user moves the volume slider to 50%
- **THEN** the audio output gain SHALL be reduced accordingly

#### Scenario: Mute and unmute
- **WHEN** the user mutes playback
- **THEN** no audio SHALL be heard
- **WHEN** the user unmutes
- **THEN** audio SHALL resume at the previous volume level
