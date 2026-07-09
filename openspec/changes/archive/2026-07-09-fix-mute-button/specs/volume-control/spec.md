## ADDED Requirements

### Requirement: Audio output respects mute state
The system SHALL apply the current mute state to all audio output. When muted, the system SHALL output silence (all samples at 0.0). When unmuted, the system SHALL output audio at the current volume gain level.

#### Scenario: Mute silences playback
- **WHEN** audio is playing and user clicks the mute button
- **THEN** the audio output immediately produces silence

#### Scenario: Unmute restores volume
- **WHEN** audio is muted and user clicks the mute button
- **THEN** audio resumes at the volume level that was active before muting

#### Scenario: Mute state persists across tracks
- **WHEN** audio is muted and a new track starts
- **THEN** the new track remains muted (no audio output)