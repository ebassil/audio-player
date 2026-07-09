## Context

The mix duration is capped at 15.0 seconds — enforced by the frontend slider (`max="15"`) and must be enforced in the backend as well. Per-track overrides, config defaults, or any other path that sets the mix duration SHALL be clamped to [1.0, 15.0]. **Exception**: when a `mix_out` point is defined for the current track, the cap is lifted — the effective mix duration becomes `track_duration - mix_out_point` and the gain envelope extends to match.

## Context

The `AudioPipeline` transition logic (pipeline.rs:326-330) currently clamps the envelope duration when the remaining track time is shorter than the configured mix duration:

```rust
let remaining = track_duration - playback_pos;
let effective_duration = r.duration_secs.min(remaining);
let total_frames = (effective_duration * output_sample_rate) as usize;
```

This produces two problems:

1. **Envelope truncation**: The transition overlap is shortened to `remaining` seconds instead of the full `mix_duration`. A 15s mix becomes a 10s mix when the user seeks to 10s before track end.

2. **No decoder offset**: The next decoder always starts at position 0. Even though the out-track ends early, the in-track plays from its beginning, producing a position discontinuity from what a full-length mix would deliver.

The `ResolvedMix` struct already carries `mix_in_point` but it is never applied — neither to the envelope timing nor to the decoder start position.

## Goals / Non-Goals

**Goals:**
- Full mix_duration envelope length is preserved even when remaining < mix_duration
- Next decoder is seeked forward by `excess = mix_duration - remaining` seconds
- When `mix_in_point` is defined, it adds to the offset: `offset = mix_in_point + excess`
- Out-track silence after EOF is handled gracefully (existing underrun behavior)

**Non-Goals:**
- Modifying the envelope shape based on mix_in_point (the fade-in always ramps over total_frames; mix_in_point only affects decoder start position)
- Changing the trigger timing, trigger position, or trigger calculation
- Frontend changes
- Modifying `MixEngine` or `ResolvedMix` structs

## Decisions

### D1: Keep envelope at full mix_duration, offset next decoder

At transition start (async load complete), compute:

```
excess = max(0, mix_duration - remaining)
next_offset = mix_in_point.map_or(excess, |m| m + excess)
```

If `next_offset > 0`, call `next_decoder.seek(next_offset)` to advance the incoming track's start position. The envelope is sized to `mix_duration` frames (unchanged).

- **Why**: The user explicitly wants the incoming track to start at `mix_duration - remaining` seconds in (or `mix_in_point + that`). This preserves the intended transition timing across the full mix_duration window.
- **Why not clamp**: Clamping the envelope to `remaining` shortens the overlap and defeats the purpose of the configured mix duration.
- **Note**: When the out-track reaches EOF during the transition, `current.read()` returns zeros. The envelope still applies — out_gain is applied to silence, in_gain continues ramping. This is already the existing underrun behavior (D4 from the overlap-fix design).

### D2: Use `playback_position_secs()` for remaining

The remaining calculation already uses `playback_position_secs()` (ground truth from consumed frames). No change needed — reuse the existing value computed at transition start.

### D3: Clamp offset to next track duration

If `next_offset >= next_duration_secs`, clamp to `(next_duration_secs - epsilon)` to avoid seeking past EOF. The decoder already returns silence when read beyond duration, but a valid seek position avoids edge-case behavior.

### D4: Interaction with mix patterns

The offset applies uniformly to all mix patterns (CrossFade, Fade, HardFade):

- **CrossFade**: Next decoder advances, both tracks overlap for full `mix_duration`. Out-track goes silent at EOF; in-track continues at whatever gain the envelope provides.
- **Fade**: Same offset logic. The fade-out / gap / fade-in segments maintain their 1/3 proportions over `mix_duration` frames. Next track's content is shifted forward by the offset.
- **HardFade**: Same offset logic. The gap occurs at the full mix_duration timing, but with next track content shifted forward.

- **Why uniform**: The underlying problem (remaining < mix_duration) and the compensation (offset the in-track) are pattern-independent. Each pattern's envelope structure is preserved.

### D5: Offset applied only when excess > 0

When `remaining >= mix_duration`, `excess = 0`, and no seeking occurs. This preserves existing behavior where the next decoder starts at position 0 (and the mix_in_point solely determines the fade-in resolution point for future use).

## Risks / Trade-offs

- **Seek during transition setup**: `next_decoder.seek()` is asynchronous (writes to a `Mutex<Option<f64>>` consumed by the decode thread). The seek completes before the transition phase starts reading from the next decoder, because the next decoder's decode thread processes the seek during the gap between `transition_load_requested` being cleared and the next audio callback invocation. This is safe.
- **Excess > mix_duration**: If the async load took so long that `remaining` becomes negative (track already finished), `excess > mix_duration`. The envelope stays at `mix_duration` frames and the out-track is already silent. The next decoder offset is clamped to the track duration. This is an edge case that produces a reasonable outcome (in-track plays at offset with silent out-track).
- **mix_in_point + excess exceeds track**: Clamp to `duration_secs - small_epsilon`. The fade-in completes during silence, which is acceptable for an edge case.
