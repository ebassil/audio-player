## Context

The `BufferedDecoder` tracks `total_frames` as the count of frames pushed into the ring buffer by the background decode thread. `position_secs()` returns `total_frames / sample_rate`, which can be up to ~6s ahead of actual playback (the ring buffer capacity). The pipeline uses this value for two critical calculations:

1. **Trigger decision**: `position >= trigger` fires when decode-side position reaches the trigger point, which can happen when actual playback is still 6s away.
2. **Effective duration**: `remaining = duration - position` is ~6s less than actual remaining audio.

As a result, `effective_duration = min(mix_duration, remaining)` is clipped below the configured mix duration. In worst case (decode thread finishes before trigger point), remaining ≈ 0, giving zero overlap.

Additionally, the effective duration is computed at trigger time but applied when the async decoder load completes. Any time spent loading (typically 100-500ms for file I/O + symphonia probing) is lost from the overlap.

## Goals / Non-Goals

**Goals:**
- Actual audible overlap matches the configured mix duration (within tolerance for async load time)
- Transition trigger uses playback-ground-truth position, not decode-side position
- Effective duration is recalculated at transition start to account for async load delay
- Maintain lock-free read path in the audio callback

**Non-Goals:**
- Eliminating async load latency entirely (inherent in background decoder loading)
- Changing the frontend position/progress display (continues to use decode-side position)
- Rewriting the ring buffer or decoder architecture

## Decisions

### D1: Track consumed frames in the ring buffer consumer
Track frames popped from the ring buffer by exposing a `consumed()` method on `AudioRingBuf` (`head` value), and have `BufferedDecoder.read()` increment a `consumed_frames` atomic after each pop. Expose `playback_position_secs()` = `consumed_frames / sample_rate`.

- **Why**: The head index is already maintained atomically by the ring buffer. The consumer thread (audio callback) is the only thread that calls `pop()`, so there is no contention. The atomic increment cost is negligible on the audio callback path.
- **Alternative**: Estimate from `total_frames - readable/channels`. Rejected because readable() requires an Acquire load of tail and Relaxed load of head — the subtraction only gives a snapshot, not a position counter.
- **Alternative**: Feed a separate position counter from the callback. Rejected — the consumed_frames approach piggybacks on existing ring buffer state with zero additional synchronization.

### D2: Use playback position for trigger and remaining calculation
Replace `d.position_secs()` with `d.playback_position_secs()` in the pipeline's trigger logic (line 302) and remaining calculation (line 419).

- **Why**: Playback position is the ground truth for when audio actually reaches the listener's ears. Triggering based on it guarantees the mix starts at the correct point in the outgoing audio stream.
- **Why not both**: Keeping decode-side position for trigger would perpetuate the bug. The trigger must fire based on what the listener actually hears.
- **Note**: The frontend-facing `position_secs()` in the pipeline's `progress()` method can remain unchanged (exposing decode-side position to the status polling loop) to avoid changing the UI behavior.

### D3: Recalculate effective duration at transition start, not trigger time
When the async-loaded `next_decoder` becomes ready (line 314), recalculate `effective_duration` using current playback position before creating the gain envelope.

- **Why**: Between trigger time and load-complete time, the playback position advances by T_load seconds. Using the stale trigger-time remaining causes the envelope to extend beyond the out-track's actual audio, wasting gain computation and creating a silent tail.
- **How**: Store `trigger_position` at trigger time instead of `effective_duration`. At transition start, compute `remaining = duration - current_playback_position`, then `effective_duration = min(mix_duration, remaining)`.

### D4: Graceful out-track underrun during transition
If the out-track decoder reaches EOF during a transition (should not happen with correct position tracking, but protects against edge cases), its `read()` returns silence. The envelope continues as normal — the in-track continues fading in while the out-track contributes nothing. Transition completes when the envelope cursor reaches the end.

- **Why**: This is already the behavior from the existing code (decoder.read() returns zeros on underrun). No change needed.

## Risks / Trade-offs

- **Atomic read overhead in audio callback**: Adding a `Relaxed` load of `consumed_frames` per callback. This is a single atomic load — negligible compared to the existing ring buffer `pop()` operations and resampling.
- **Backward compatibility**: `playback_position_secs()` is additive — existing `position_secs()` callers (frontend polling, progress display) continue to work unchanged.
- **Async load delay remains**: The fix recalculates duration but doesn't eliminate the T_load delay. If T_load > mix_duration, the transition starts after the out-track has already ended. This is an edge case (loading a very slow medium like a network file) that should be handled separately with a timeout or fallback.
