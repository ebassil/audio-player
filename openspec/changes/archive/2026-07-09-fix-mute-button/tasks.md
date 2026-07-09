## 1. Pipeline Changes

- [x] 1.1 Add `volume_state: Arc<VolumeState>` field to `AudioPipeline` struct
- [x] 1.2 In `AudioPipeline::new()`, wrap the created `VolumeState` in `Arc` and store it in both the existing `volume` field and the new `volume_state` field
- [x] 1.3 In the `play()` callback closure, capture `Arc<VolumeState>` and apply `effective_gain()` to output samples before returning them

## 2. Verification

- [x] 2.1 Build the project with `cargo build`
- [x] 2.2 Run the app and confirm mute button silences audio
- [x] 2.3 Confirm unmute restores previous volume level
- [x] 2.4 Confirm volume slider (if present) continues to work