## Why

Clicking a song takes >5 seconds to load and start playing because the entire audio file is decoded into memory before playback begins. For long tracks (hours-long mixes, audiobooks, podcasts), this delay grows proportionally with file size, and memory consumption spikes to hold the full decoded PCM buffer.

## What Changes

- Replace `decode_file()` full-decode with a streaming decoder that yields decoded packets on-demand
- Change `AudioPipeline::load_track()` to open the file and begin streaming decode instead of blocking on full decode
- Keep a ring buffer / sliding window of decoded samples in memory instead of the full track
- The cpal playback callback reads from the streaming decoder's current buffer, triggering decode of subsequent packets as needed
- Seeking requires re-seeking the symphonia format reader instead of indexing into a pre-decoded array
- The `DecodedAudio` struct is replaced or augmented with a `StreamingDecoder` that symphonia internals

## Capabilities

### New Capabilities
- `streaming-decode`: Decode audio files incrementally — decode only enough to fill the output buffer on each callback invocation, keeping a small lookahead of decoded samples in memory

### Modified Capabilities
- `audio-pipeline`: The pipeline's load/play lifecycle changes — `load_track` opens and probes the file then immediately reports readiness; decode happens during playback via the streaming decoder. The `current_track` field changes from `Arc<Mutex<Option<DecodedAudio>>>` to a streaming decoder handle. Seeking requires re-seeking within the compressed stream.

## Impact

- `src-tauri/src/audio/decoder.rs`: Rewrite — replace `decode_file()` with `StreamingDecoder` struct and methods (`new`, `next_samples`, `seek`, `duration`, `sample_rate`, `channels`)
- `src-tauri/src/audio/pipeline.rs`: `load_track` no longer returns fully-decoded audio; `play` callback reads from streaming decoder; `seek` uses decoder's seek; `progress`/`position_secs` compute from decoder state
- `src-tauri/src/audio/output.rs`: Likely no changes — callback interface stays the same
- `src-tauri/src/audio/mod.rs`: Add `StreamingDecoder` to public API if needed
- `src-tauri/src/lib.rs`: `load_track` IPC handler may need adjustment since it currently returns `DecodedAudio` metadata; metadata extraction should still work from the probed stream header without full decode
- Frontend (`src/main.ts`): No changes expected — `loadTrack` already awaits and the delay will shrink
