## Why

When mix duration is set to 15s, the actual audible overlap between songs is shorter than configured. The transition starts with a clipped envelope because the trigger position and remaining-time calculation use decode-side frame count (which can be 6s ahead of actual playback), and the envelope is sized at trigger time instead of transition start time.

## What Changes

- Fix the position/remaining calculation used for mix trigger and effective duration to use actual playback position (frames consumed from ring buffer) instead of decode-side frame count
- Recalculate effective transition duration at transition start time (when the async decoder load completes) rather than at trigger time
- Add a `consumed_frames` counter to `BufferedDecoder` to track actual playback position

## Capabilities

### New Capabilities
- `playback-position`: Track actual playback position (frames consumed from ring buffer) in `BufferedDecoder`, exposed as `playback_position_secs()` for accurate trigger and remaining-time calculations

### Modified Capabilities
- `track-transition`: Change transition trigger to use playback position instead of decode position; recalculate envelope duration when the async loaded decoder is ready, not at trigger time
- `audio-pipeline`: Wire playback position from decoder into transition trigger logic; defer envelope frame calculation until transition actually begins

## Impact

- `src-tauri/src/audio/decoder.rs`: Add `consumed_frames` counter, increment on `read()`, expose `playback_position_secs()`
- `src-tauri/src/audio/ringbuf.rs`: Expose `consumed()` or provide a way to query frames popped
- `src-tauri/src/audio/pipeline.rs`: Use playback position for trigger; recalculate effective_duration at load-complete time
- No frontend changes needed — position reported to frontend remains decode-side for consistency
- No spec or contract changes for IPC commands or events
