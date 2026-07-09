## Why

When the user seeks to a position near the end of the current track (e.g., 10s before the end) and the mix duration exceeds the remaining time (e.g., 15s mix on 10s remaining), the transition clamps the envelope to `min(mix_duration, remaining)` seconds. This produces a shorter overlap than configured and causes the next track to always start from position 0, losing the intended transition timing.

The result: a 15s mix that only lasts 10s, with the incoming track beginning at its natural start point rather than the position it would be at under a full-length mix. This breaks the expected timing for beatmatching, phrasing, and DJ-style transitions.

## What Changes

- Keep the transition envelope at the full `mix_duration` length even when `remaining < mix_duration`
- When `remaining < mix_duration`, seek the next decoder forward by `excess = mix_duration - remaining` seconds so the incoming track's playback position matches what it would be after a full-duration mix
- If a `mix_in_point` is defined for the next track, add it to the offset so the mix-in resolves at the intended position relative to the next track's content
- Use `playback_position_secs()` (ground-truth consumed frames) for the remaining-time calculation at transition start

## Capabilities

### Modified Capabilities
- `track-transition`: Change the short-track behavior from clamping envelope duration to keeping full envelope length and offsetting the next decoder; add mix-in-point interaction with the offset
- `audio-pipeline`: Update transition start logic to compute offset and seek the next decoder when remaining < mix_duration; pass mix_in_point from resolved mix into the offset calculation

## Impact

- `src-tauri/src/audio/pipeline.rs`: Modify transition start block to calculate `excess`, seek `next_decoder` by offset, and keep `effective_duration = mix_duration` (unclamped)
- `src-tauri/src/audio/mixing.rs`: No structural changes needed — `ResolvedMix` already carries `mix_in_point`
- No frontend changes required
- No spec or contract changes for IPC commands or events
