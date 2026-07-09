## 1. Ring Buffer — Expose consumed frame count

- [x] 1.1 Add `consumed()` method to `AudioRingBuf` returning the head index (total frames popped)
- [x] 1.2 Test that `consumed()` is monotonic and wraps correctly with the ring buffer capacity

## 2. BufferedDecoder — Track playback position

- [x] 2.1 Add `consumed_frames: AtomicU64` field to `BufferedDecoderShared`
- [x] 2.2 Increment `consumed_frames` by `read / channels` inside `BufferedDecoder::read()` after each pop
- [x] 2.3 Add `playback_position_secs()` method returning `consumed_frames / sample_rate`
- [x] 2.4 Reset `consumed_frames` to 0 on seek (alongside the existing `ring_buf.clear()`)

## 3. Pipeline — Use playback position for trigger

- [x] 3.1 Trigger uses decode-side position (`d.position_secs()`) to fire at the correct wall-clock time; effective duration recalculated at transition start using `d.playback_position_secs()` (task 4.3)
- [x] 3.2 Verify trigger fires and transition selects correct next track

## 4. Pipeline — Recalculate effective duration at transition start

- [x] 4.1 Store `trigger_position` (playback position at trigger time) instead of `effective_duration` in `pending_transition_info`
- [x] 4.2 Change `pending_transition_info` from `(usize, ResolvedMix, f64)` to carry the stored track duration and trigger position: `(usize, ResolvedMix, f64, f64)` or `(usize, ResolvedMix, track_duration, trigger_position)`
- [x] 4.3 At transition start (when async load completes), compute `remaining = track_duration - current_playback_position` and `effective_duration = mix_duration.min(remaining)`
- [x] 4.4 Use this recalculated `effective_duration` for the gain envelope frame count

## 5. Verify and build

- [x] 5.1 Build the project with `cargo build` and fix any compilation errors
- [ ] 5.2 Test with a short track (e.g., 30s) and 15s mix duration — verify overlap duration matches configured value
- [ ] 5.3 Test with a mix-out point — verify trigger still fires at the correct output position
- [ ] 5.4 Test seek during transition — verify consumed_frames reset doesn't cause position discontinuity
