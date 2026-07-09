## 1. Pipeline — Compute excess and offset next decoder at transition start

- [x] 1.1 At transition start block (pipeline.rs ~314), after computing `remaining` and `playback_pos`, calculate `excess = (r.duration_secs - remaining).max(0.0)`
- [x] 1.2 Calculate `next_offset = r.mix_in_point.map_or(excess, |m| m + excess)`
- [x] 1.3 If `next_offset > 0.0`, lock `next_decoder`, clamp to track duration, and call `next.seek(clamped_offset)`
- [x] 1.4 Release the next_decoder lock before setting transition phase (avoid double-lock when transitioning phase reads)

## 2. Pipeline — Keep envelope at full mix_duration

- [x] 2.1 Remove the `r.duration_secs.min(remaining)` clamp — set `total_frames` from `mix_duration` directly (not `effective_duration`)
- [x] 2.2 Verify `total_frames = (mix_duration * output_sample_rate) as usize` is used for envelope generation

## 3. Enforce max mix duration (15.0s) in backend with mix_out exemption

- [x] 3.1 When a `mix_out` point is defined for the current track, compute effective mix duration as `track_duration - mix_out_point` — do NOT apply the 15.0s cap
- [x] 3.2 When no `mix_out` is defined, clamp `mix_duration_override` to `[1.0, 15.0]` in `MixEngine::resolve()` or at the point of use in the pipeline transition start
- [x] 3.3 Clamp the default `mix_duration_secs` from config to `[1.0, 15.0]` in `AppConfig::to_mix_config()`
- [x] 3.4 Ensure any IPC path that sets mix duration (`set_mix_config`, `set_current_track_mix_overrides`) respects the clamp
- [x] 3.5 In the trigger calculation (pipeline.rs ~398), when `mix_out` is defined, use `mix_out` as the trigger point and `track_duration - mix_out` as the effective mix duration for envelope sizing

## 4. Edge case handling

- [x] 4.1 Clamp `next_offset` to `(next.duration_secs() - 0.001).max(0.0)` (small epsilon to avoid EOF-edge behavior)
- [x] 4.2 Handle the case where `next_offset < 0.0` (should not happen with `max(0.0)` guard, but defend against floating-point edge cases)
- [x] 4.3 Handle case where `excess` is NaN or infinite (float guard)

## 5. Verify and build

- [x] 5.1 Build with `cargo build` and fix compilation errors
- [ ] 5.2 Test with 15s mix on a track seeked to 10s from end — verify next track starts at 5s offset
- [ ] 5.3 Test with mix_in_point set to 8s, same seek scenario — verify next track starts at 8+5=13s offset
- [ ] 5.4 Test with remaining >= mix_duration — verify no offset applied and existing behavior preserved
- [x] 5.5 Run existing unit tests (`cargo test`) — verify no regressions
