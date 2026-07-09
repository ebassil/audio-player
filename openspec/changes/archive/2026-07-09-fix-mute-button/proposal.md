## Why

The mute button toggles an atomic boolean in `VolumeState` but produces no audible effect — audio continues playing at full volume. The `VolumeState` is never actually applied to the audio stream in the pipeline's output callback.

## What Changes

- Apply volume gain (including mute) to audio samples in the pipeline's playback callback before sending them to the audio output device.
- Verify that mute properly silences audio and unmute restores the previous volume level.

## Capabilities

### New Capabilities
- `volume-control`: Applies volume gain and mute state to the decoded audio stream before output.

### Modified Capabilities
- *(none)*

## Impact

- **`src-tauri/src/audio/pipeline.rs`**: Modify the audio callback to multiply sample values by `volume.effective_gain()` before returning them.
- **`src-tauri/src/audio/volume.rs`**: No changes needed — `effective_gain()` already returns `0.0` when muted and the stored gain when unmuted.
- **`src-tauri/src/audio/pipeline.rs`**: The callback needs access to `VolumeState`. The struct already holds `volume: VolumeState`, but the closure doesn't capture it. Must pass an `Arc<VolumeState>` or `Arc<AtomicBool>` + gain value into the callback closure.