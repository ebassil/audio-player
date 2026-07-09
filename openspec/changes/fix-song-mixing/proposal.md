## Why

When a song finishes playing, playback stops instead of automatically advancing to the next song in the playlist. The MixEngine, MixConfig, and mix controls already exist in the codebase but are never triggered — the pipeline callback only reads from a single decoder and sets `Stopped` on EOF. This makes the player unusable for continuous playlist listening.

## What Changes

- Wire the existing `MixEngine` into the playback callback to detect when a track is near its end and prepare the next track according to the configured mix pattern and duration
- Add a `next_track_decoder` slot to the pipeline for pre-loading the upcoming track during the transition window
- Apply gain envelopes (fade, cross-fade, or hard fade) during transitions using the MixEngine API
- Automatically advance to the next track on transition completion
- Loop through the playlist, advancing track-by-track until the user manually stops playback
- Respect per-song mix-out/mix-in point overrides when computing transition parameters
- Keep playback stopped or at end-of-playlist when the last track finishes (no loop wrap)
- Update frontend to reflect current track advancement and playlist selection

## Capabilities

### New Capabilities
- `track-transition`: Automatic track-to-track transitions using the configured mix pattern and duration, with per-song mix point overrides

### Modified Capabilities
- `audio-pipeline`: Playback callback must support dual-decoder transitions and emit EOF/advance events; pipeline must own a playlist index for auto-advance

## Impact

- **Pipeline** (`pipeline.rs`): Core callback rewrite to support two decoders and mixing envelopes; new fields for next-track decoder, transition state machine, and playlist index
- **Frontend** (`main.ts`): Listen for track-advance events and update playlist selection + progress display; wire the stub shortcut handlers for NextTrack/PreviousTrack; optionally advance playlist when status events indicate a new track
- **IPC** (`lib.rs`): New commands or events for playlist context passing (tell pipeline about the playlist order and mix overrides); or extend existing status events with track index info
- **Player state** (`player.rs`): May need a `Transitioning` state or handle the transition within the callback without exposing it to the state machine
- **Mixing** (`mixing.rs`): No changes needed — the envelope generators and resolution API are complete
