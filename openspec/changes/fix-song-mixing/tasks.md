## 1. Pipeline: Next-track decoder and transition state

- [x] 1.1 Add `next_decoder: Arc<Mutex<Option<StreamingDecoder>>>` field to `AudioPipeline`
- [x] 1.2 Add `playlist_context: Arc<Mutex<Vec<PlaylistContextEntry>>>` field to `AudioPipeline` (file paths + mix point overrides)
- [x] 1.3 Add `current_track_index: Arc<Mutex<Option<usize>>>` field to `AudioPipeline`
- [x] 1.4 Define internal `TransitionPhase` enum (`Normal`, `Transitioning { out_gain, in_gain, cursor }`) 
- [x] 1.5 Initialise new fields in `AudioPipeline::new()`

## 2. Pipeline: Transition trigger and next-track loading

- [x] 2.1 In the playback callback, after reading from current decoder, compare `position_secs` vs `duration_secs - mix_duration` to detect when transition should start
- [x] 2.2 When transition triggers, lock `playlist_context`, look up next track path and mix overrides, create a new `StreamingDecoder` into `next_decoder`
- [x] 2.3 Call `MixEngine::resolve(&current_mix, &next_mix)` to get the `ResolvedMix` with pattern, duration, and mix points
- [x] 2.4 Pre-compute the gain envelope based on the resolved mix pattern and duration

## 3. Pipeline: Gain envelope application during transition

- [x] 3.1 During `Transitioning` phase, read `num_frames` from both decoders
- [x] 3.2 Apply gain envelope frames to each decoder's output (outgoing: decreasing gain, incoming: increasing gain)
- [x] 3.3 Sum the two gain-scaled buffers
- [x] 3.4 Return the summed buffer as the callback output
- [x] 3.5 Track envelope frame cursor across callback invocations

## 4. Pipeline: Decoder swap and event emission

- [x] 4.1 When envelope frames are exhausted (transition complete), swap `next_decoder` into `current_decoder`, set `next_decoder` to `None`
- [x] 4.2 Increment `current_track_index`
- [x] 4.3 Emit `track-changed` Tauri event with new track index and metadata
- [x] 4.4 If no next track exists (end of playlist), set state to Stopped instead of advancing
- [x] 4.5 Handle stop command during transition: clear both decoders, reset transition state

## 5. Pipeline: Position tracking during transition

- [x] 5.1 Ensure `position_secs()` and `progress()` report outgoing track position during transition
- [x] 5.2 After swap, `position_secs()` reports new track position from time 0
- [x] 5.3 Clamp mix-out point to track duration, clamp mix-in point to next track duration

## 6. IPC: New commands for playlist context

- [x] 6.1 Add `set_playlist_context` Tauri command in `lib.rs` that accepts file paths + mix point overrides
- [x] 6.2 Add `set_current_track_index` Tauri command to sync which track is currently selected
- [x] 6.3 Register new commands in the `invoke_handler` macro

## 7. Frontend: Playlist context sync on load

- [x] 7.1 After loading/setting tracks in the playlist, call `invoke("set_playlist_context", { ... })` to push track file paths and mix points to the pipeline
- [x] 7.2 After selecting a track, call `invoke("set_current_track_index", { index })` to sync the starting position
- [x] 7.3 Update `loadTrack`, `playNextTrack`, `playPrevTrack`, `loadDirectory`, and `loadPlaylistJson` to sync context

## 8. Frontend: Track advance event handling

- [x] 8.1 Listen for `track-changed` event in `initPlayerEvents()`
- [x] 8.2 On event, update `selectedTrackIndex` and re-render the playlist to highlight the new track
- [x] 8.3 Update status text and timeline on track change

## 9. Frontend: Wire global shortcut stubs

- [x] 9.1 In `handleShortcutAction`, fill `NextTrack` case to call `playNextTrack()`
- [x] 9.2 Fill `PreviousTrack` case to call `playPrevTrack()`
