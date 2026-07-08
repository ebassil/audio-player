## Context

The current `AudioPipeline` fully decodes every audio file into a `Vec<f32>` before playback starts. For a 3-minute FLAC at 44.1kHz stereo, this means decoding ~15M samples (~60 MB of f32 data) upfront. Users with large libraries, long-form content (podcasts, mixes), or slower storage experience a multi-second delay between clicking a track and hearing audio.

The `load_track` → `play` two-step IPC flow in `lib.rs` blocks the UI thread during the `invoke("load_track")` call, making the entire webview unresponsive for the duration of the full decode.

symphonia natively supports packet-by-packet decode — the current code already loops over `format.next_packet()` / `decoder.decode()` but accumulates every sample into a single buffer before returning. The building blocks for streaming are already in place.

## Goals / Non-Goals

**Goals:**
- Start playback within ~200ms of clicking a song (time to probe, open, and decode first buffer)
- Stream-decode audio in the cpal callback thread, decoding packets on-demand as the buffer advances
- Keep a bounded lookahead window (e.g. 2-3 cpal buffer periods) to prevent buffer underruns
- Support seek operations by re-seeking the symphonia format reader to the target packet
- Expose track metadata (duration, sample rate, channels, title, artist) from the stream header without full decode

**Non-Goals:**
- Changing the IPC interface or frontend code (backward-compatible)
- Adding new file format support beyond what symphonia already provides
- Rewriting the mixing engine or DSP graph integration
- Network streaming (HTTP/RTSP) — local file playback only

## Decisions

### Decision 1: StreamingDecoder struct replaces DecodedAudio

A new `StreamingDecoder` struct owns the symphonia `FormatReader`, `Decoder`, and a small internal ring buffer of decoded f32 samples. The cpal callback calls `StreamingDecoder::read(num_frames)` which decodes additional packets if the ring buffer is running low, then copies interleaved samples into the output.

- **Alternatives considered:** Keeping `DecodedAudio` and decoding in batches pre-playback (still blocks), using symphonia's `AudioBuffer` directly (tight coupling to symphonia types)
- **Rationale:** The callback needs to be fast and non-blocking; a dedicated struct with an internal decode buffer isolates symphonia complexity from the pipeline

### Decision 2: Probe on load, decode on play

`load_track` opens the file, probes the format, reads stream metadata (duration, codec params), and stores the `StreamingDecoder` — but does NOT decode any audio packets. `play()` then starts the cpal stream which calls the callback, which triggers decoding from the actual stream position.

- **Alternatives considered:** Decoding the first few packets during `load_track` to pre-warm the buffer
- **Rationale:** Clean separation; `load_track` returns in <50ms (just file open + probe). Pre-warming can be added later as a performance optimization if needed

### Decision 3: Seeking re-seeks symphonia then resets the decoder

`StreamingDecoder::seek(position_secs)` calls `format.seek()`, which seeks to the nearest packet before the target time, then resets the symphonia `Decoder` and clears the internal ring buffer. The next `read()` call will begin decoding from the new position.

- **Alternative considered:** Seeking by re-decoding from start to target (simple but slow)
- **Rationale:** symphonia's `FormatReader::seek` is designed for this and is efficient (seeks within the compressed stream)

### Decision 4: Ring buffer holds ~8192 frames (configurable)

The internal decode buffer prefills up to 8192 frames (≈186ms at 44.1kHz) and refills when the remaining frames drop below a threshold. This is enough to cover worst-case cpal buffer sizes (512-2048 samples) with headroom.

- **Rationale:** Small enough to not waste memory on silent pauses, large enough to survive scheduling jitter on the audio thread

## Risks / Trade-offs

- **Risk: seek accuracy** → symphonia may not seek to exact sample positions for all codecs (e.g. VBR MP3). Mitigation: after seeking, decode a few packets and use the new position as the authoritative timestamp
- **Risk: decode latency spike** → decoding a complex packet (e.g. large FLAC frame) could take longer than the buffer period, causing an underrun. Mitigation: the 8192-frame ring buffer provides ~186ms of headroom; if underruns occur, increase the buffer size
- **Risk: no full PCM in memory** → plugins (via fundsp graph) that need random access to the full track will not work without re-reading the file. Mitigation: document this limitation; the plugin system is not yet wired into the audio path anyway
- **Trade-off: seeking in compressed streams** → some codecs (MP3 with VBRI headers, WAV) have no efficient seek tables. symphonia may fall back to linear scan. For WAV files this is fine since they're uncompressed
- **Trade-off: memory vs CPU** → streaming decode uses less peak memory but slightly more CPU per sample (decode is interleaved with playback). Acceptable trade-off for near-instant start times
