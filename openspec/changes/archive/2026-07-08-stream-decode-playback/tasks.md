## 1. StreamingDecoder Implementation (decoder.rs)

- [x] 1.1 Define `StreamingDecoder` struct with fields: `format` (symphonia FormatReader), `decoder` (symphonia Decoder), `track_id`, `sample_rate`, `channels`, `duration_secs`, `buffer` (ring buffer Vec<f32>), `buffer_read_pos`, `total_frames_read`, `prefill_frames`, `refill_threshold`, `eof_reached`
- [x] 1.2 Implement `StreamingDecoder::new(path)` — open file, probe format, select first audio track, create decoder, store metadata, return without decoding any packets
- [x] 1.3 Implement `StreamingDecoder::read(num_frames)` — if buffer has enough frames, copy and return; otherwise decode packets in a loop until buffer is filled or EOF; handle all symphonia error types
- [x] 1.4 Implement `StreamingDecoder::fill_buffer()` — inner method: decode packets via `format.next_packet()` + `decoder.decode()`, convert to interleaved f32, push to buffer; stop when buffer >= prefill_frames
- [x] 1.5 Implement `StreamingDecoder::seek(position_secs)` — call `format.seek()`, reset decoder, clear buffer, reset `total_frames_read`, set `eof_reached = false`
- [x] 1.6 Add public accessors: `sample_rate()`, `channels()`, `duration_secs()`, `position_secs()` (from `total_frames_read`), `progress()`
- [x] 1.7 Remove the old `DecodedAudio` struct and `decode_file()` function if no longer used anywhere

## 2. Pipeline Integration (pipeline.rs)

- [x] 2.1 Replace `current_track: Arc<Mutex<Option<DecodedAudio>>>` with `current_decoder: Arc<Mutex<Option<StreamingDecoder>>>`
- [x] 2.2 Rewrite `load_track()` — create `StreamingDecoder::new(path)`, store in `current_decoder`, return metadata without full decode
- [x] 2.3 Rewrite `play()` callback — read from `StreamingDecoder::read(num_frames)` instead of pre-decoded buffer slice; handle end-of-track (decoder returns silence)
- [x] 2.4 Rewrite `seek(position_secs)` — call `current_decoder.lock().unwrap().as_mut().map(|d| d.seek(position_secs))`
- [x] 2.5 Rewrite `progress()` and `position_secs()` — delegate to `StreamingDecoder` accessors
- [x] 2.6 Remove `read_position` atomic field (replaced by decoder's internal position tracking)

## 3. IPC Layer Updates (lib.rs)

- [x] 3.1 Update `load_track` handler to extract metadata from `StreamingDecoder` and return it to frontend (same response shape)
- [x] 3.2 Verify all existing IPC commands (`play`, `pause`, `resume`, `stop`, `seek`, `get_position`) still work with streaming decoder
- [x] 3.3 Verify `player-status` event emission still reports correct position and progress

## 4. Cleanup

- [x] 4.1 Remove unused `DecodedAudio` struct and `decode_file()` function from `decoder.rs`
- [x] 4.2 Remove unused `std::sync::atomic::AtomicU32` import from `pipeline.rs`
- [x] 4.3 Remove unused `read_position` field from `AudioPipeline`
- [x] 4.4 Run `cargo build` and fix any compilation errors
- [x] 4.5 Run `cargo clippy` and address warnings
- [x] 4.6 Test end-to-end: load track, play, seek, pause, resume, stop with various formats (MP3, FLAC, WAV, OGG, M4A)

## 5. Verification

- [x] 5.1 Confirm load-to-play latency is under 500ms for a 30-minute FLAC file
- [x] 5.2 Confirm seeking is accurate (within 0.5s of target for CBR formats, within 2s for VBR)
- [x] 5.3 Confirm memory usage during playback of a large file stays bounded (no growth beyond the ring buffer + symphonia internal state)
