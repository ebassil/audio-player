## Context

The codebase has a fully-defined `MixEngine` with envelope generators (fade, cross-fade, hard fade), `MixConfig` with persistent defaults (`crossfade` / `3.0s`), and per-song `MixPoint` overrides for mix-in/mix-out timing — but none of it is wired into the playback callback. The `AudioPipeline` callback reads from a single `StreamingDecoder` and sets `Stopped` on EOF. The frontend has an `initGlobalShortcuts` system but the `NextTrack` and `PreviousTrack` handlers are empty stubs.

The playlist data lives in the frontend (`currentTracks[]`) and is synced to the Rust side via IPC (`set_playlist_tracks`). The Rust `AppState` holds a `Playlist` struct but the pipeline has no reference to it.

## Goals / Non-Goals

**Goals:**
- Automatically advance to the next track when the current one finishes, using the configured mix pattern and duration
- Support all three mix patterns (crossfade, fade, hard fade) during transitions
- Respect per-song mix-out/mix-in point overrides for transition timing
- Apply gain envelopes from the MixEngine during transitions
- Update frontend playlist selection and status on track change
- Wire the NextTrack and PreviousTrack global shortcut stubs to call the existing playNextTrack/playPrevTrack functions
- Continue playing through the playlist until the user manually stops or the playlist ends
- Handle end-of-playlist gracefully (stop, don't loop)

**Non-Goals:**
- Gapless playback without mixing (all transitions use a mix pattern)
- Loop/wrap-around playback from last track to first
- Timeline waveform view for mix-point editing (already has simple percent-based buttons)
- Beat-grid sync or BPM detection
- WASM plugin involvement in transitions
- Changes to the MixEngine API itself

## Decisions

### D1: Trigger transitions from the playback callback via time-position monitoring
Monitor `position_secs` vs `duration_secs` inside the callback. When `position >= duration - min(mix_duration, effective_mix_out)`, start the transition phase.
- **Why**: The callback is called at audio rate (~44.1kHz buffer cycles), giving precise timing. No separate timer thread, no drift.
- **Alternative considered**: Emit an event from the callback to an external thread. Rejected because of added latency and complexity — the callback already has all the state needed.

### D2: Dual-decoder pipeline with a transition state machine embedded in the callback
Add a `next_decoder: Arc<Mutex<Option<StreamingDecoder>>>` field. During `Normal` playback, read from `current_decoder`. When transition is triggered, load the next track into `next_decoder`, enter `Transitioning` phase, read from both, apply gain envelopes, sum. When transition completes, swap `current_decoder = next_decoder`, `next_decoder = None`.
- **Why**: Enables true cross-fade overlap (both tracks producing samples simultaneously). The MixEngine's `cross_fade_envelope` returns two gain curves — this maps naturally to two decoder outputs.
- **Alternative: Pre-encode the transition mix into a buffer**. Rejected — would require decoding both tracks fully before transition, increasing memory and latency.

### D3: Pass playlist context to the pipeline via a new IPC command
New command `set_playlist_context(file_paths: Vec<String>, mix_points: Vec<Option<MixPoint>>)` stores the playlist data on the pipeline. When a transition is needed, the callback looks up `current_index + 1` to find the next track path and its mix overrides.
- **Why**: The callback runs on the audio thread — it cannot make IPC calls or lock the `AppState` playlist mutex (which may be held by the frontend). Pre-loading the context avoids cross-thread contention.
- **Alternative: Frontend sends `load_next` command when it detects EOF**. Rejected because EOF detection in the callback at audio rate doesn't easily propagate to the async frontend in time for seamless transition. Also, the frontend would need sub-250ms resolution (the status event interval).

### D4: Transition state as an enum, not a PlaybackState variant
Keep the transition state as an internal enum in the callback closure rather than adding `Transitioning` to the `PlaybackState` enum.
- **Why**: The transition is an internal pipeline concern. External consumers (frontend, IPC) should see `Playing` throughout the transition. Exposing `Transitioning` would require all IPC commands and the state machine to handle it, adding complexity without benefit.
- **Alternative: Add PlaybackState::Transitioning**. Rejected — would require changes to the state machine, IPC handlers, and frontend status parsing, all of which would need to treat it identically to `Playing`.

### D5: Apply gain envelopes by multiplying decoder output samples before returning from the callback
During transition, read from both decoders, apply the per-frame gain from the chosen envelope, sum them, return the combined buffer.
- **Why**: The existing envelope generators (`cross_fade_envelope`, `fade_envelope`, `hard_fade_envelope`) return per-frame gain vectors. Applying them as scalar multipliers is zero-copy for the envelope (pre-computed) and O(n) for the audio data.
- **Alternative: Use a fundsp node for mixing**. While the ADR-005 envisions a MixEngine fundsp node, the current AudioGraph is designed for plugin pre/post chains, not dual-stream mixing. Adding a fundsp mixer node would require fundsp graph restructuring. The simple gain approach is equivalent in audio quality and much simpler to implement.

### D6: Emit a Tauri event on track advance so the frontend can update
When the transition completes and the pipeline swaps to the next track, emit a `track-changed` event with the new track index.
- **Why**: The frontend needs to update playlist selection highlight, metadata display, and reset the timeline. The existing `player-status` event doesn't carry track index info.
- **Alternative: Poll `get_status` + track index**. Less responsive and requires polling. Events are the standard pattern in this codebase.

## Risks / Trade-offs

- **CPU spike during transition**: Two simultaneous decoders doubles decode work during the overlap. Mitigation: the pre-fetch buffer for `next_decoder` is small (8192 frames as per streaming decode spec), and transition durations are typically short (1-15s).
- **Sample clock drift between decoders**: Two independent symphonia decoders may drift relative to each other during a cross-fade. Mitigation: cross-fade durations are short (seconds), and both decoders are driven from the same cpal callback clock, so drift is negligible.
- **Thread safety of decoder swap**: The callback holds `Arc<Mutex<Option<StreamingDecoder>>>` for both decoders. Swapping requires locking both mutexes. Mitigation: always lock in the same order (current then next) to prevent deadlock, and minimize the critical section.
- **Position tracking during transition**: The pipeline's `position_secs()` should ideally reflect the current track, not the next track. Mitigation: during transition, continue reporting the outgoing track's position. On completion, report the new track's position from time 0.
- **Mix point boundary conditions**: A mix-out point beyond the track's duration should be clamped. A mix-in point beyond the next track's duration should be clamped. MixEngine.resolve already handles this by returning the computed values as-is — the pipeline must clamp at use time.
