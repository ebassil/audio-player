use crate::audio::decoder::{BufferedDecoder, DecodeError};
use crate::audio::graph::AudioGraph;
use crate::audio::mixing::{MixConfig, MixEngine, MixPattern, MixPoint, ResolvedMix};
use crate::audio::output::{AudioCallback, AudioOutput};
use crate::audio::player::PlaybackState;
use crate::audio::volume::VolumeState;

use std::path::Path;
use cpal::traits::DeviceTrait;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// Resample interleaved f32 audio from `src_frames` to `dst_frames` using linear interpolation.
fn resample(src: &[f32], src_frames: usize, dst_frames: usize, channels: usize) -> Vec<f32> {
    if src_frames == dst_frames || src_frames == 0 || dst_frames == 0 {
        return src.to_vec();
    }
    let mut out = Vec::with_capacity(dst_frames * channels);
    let ratio = src_frames as f64 / dst_frames as f64;
    for i in 0..dst_frames {
        let pos = i as f64 * ratio;
        let idx = pos as usize;
        let frac = (pos - idx as f64) as f32;
        for ch in 0..channels {
            let a = src[idx * channels + ch];
            let b = if idx + 1 < src_frames {
                src[(idx + 1) * channels + ch]
            } else {
                a
            };
            out.push(a + (b - a) * frac);
        }
    }
    out
}

/// Track metadata returned by `load_track`.
#[derive(Clone, Debug)]
pub struct TrackMetadata {
    pub sample_rate: u32,
    pub channels: usize,
    pub duration_secs: f64,
}

/// An entry in the playlist context passed to the pipeline for auto-advance.
#[derive(Clone, Debug)]
pub struct PlaylistContextEntry {
    pub file_path: String,
    pub mix_out: Option<f64>,
    pub mix_in: Option<f64>,
}

/// Internal phase of a track-to-track transition.
#[derive(Clone)]
enum TransitionPhase {
    Normal,
    Transitioning {
        out_gain: Vec<f32>,
        in_gain: Vec<f32>,
        cursor: usize,
    },
}

/// The audio pipeline: decode -> graph -> output.
///
/// This struct owns the decoder, the fundsp processing graph, and the cpal output,
/// and runs the continuous playback loop on a background thread.
pub struct AudioPipeline {
    /// Audio graph with pre/post processing chains.
    graph: Arc<Mutex<AudioGraph>>,
    /// Audio output to the system default device.
    output: Arc<Mutex<AudioOutput>>,
    /// Shared volume state.
    volume: VolumeState,
    /// Current playback state.
    playback_state: Arc<Mutex<PlaybackState>>,
    /// The current buffered decoder (None when no track is loaded).
    current_decoder: Arc<Mutex<Option<BufferedDecoder>>>,
    /// The next buffered decoder pre-loaded for transition (None when not transitioning).
    next_decoder: Arc<Mutex<Option<BufferedDecoder>>>,
    /// Whether a transition decoder load has been requested (prevents duplicate spawns).
    transition_load_requested: Arc<AtomicBool>,
    /// Resolved transition info stored while the next decoder is loading asynchronously.
    pending_transition_info: Arc<Mutex<Option<(usize, ResolvedMix, f64)>>>,
    /// Whether the playback thread should keep running.
    running: Arc<AtomicBool>,
    /// Sample rate of the audio output device.
    output_sample_rate: Arc<AtomicU32>,
    /// Background playback thread handle.
    thread_handle: Option<thread::JoinHandle<()>>,
    /// Mixing engine for track transitions.
    mix_engine: Arc<Mutex<MixEngine>>,
    /// Mix points for the current track (interior mutability via Mutex).
    mix_points: Arc<Mutex<(Option<f64>, Option<f64>)>>,
    /// The playlist context for auto-advance (ordered track paths + mix overrides).
    playlist_context: Arc<Mutex<Vec<PlaylistContextEntry>>>,
    /// Index of the currently playing track within the playlist context.
    current_track_index: Arc<Mutex<Option<usize>>>,
    /// Pending track change index for event emission (consumed by status thread).
    pending_track_change: Arc<Mutex<Option<usize>>>,
    /// Current transition phase (normal playback, or actively transitioning).
    transition_phase: Arc<Mutex<TransitionPhase>>,
}

impl AudioPipeline {
    /// Create a new audio pipeline.
    pub fn new() -> Self {
        let sample_rate = 44100.0;
        let volume = VolumeState::new();

        Self {
            graph: Arc::new(Mutex::new(AudioGraph::new(sample_rate))),
            output: Arc::new(Mutex::new(AudioOutput::new())),
            volume,
            playback_state: Arc::new(Mutex::new(PlaybackState::Stopped)),
            current_decoder: Arc::new(Mutex::new(None)),
            next_decoder: Arc::new(Mutex::new(None)),
            transition_load_requested: Arc::new(AtomicBool::new(false)),
            pending_transition_info: Arc::new(Mutex::new(None)),
            running: Arc::new(AtomicBool::new(false)),
            output_sample_rate: Arc::new(AtomicU32::new(44100)),
            thread_handle: None,
            mix_engine: Arc::new(Mutex::new(MixEngine::new(MixConfig::default()))),
            mix_points: Arc::new(Mutex::new((None, None))),
            playlist_context: Arc::new(Mutex::new(Vec::new())),
            current_track_index: Arc::new(Mutex::new(None)),
            pending_track_change: Arc::new(Mutex::new(None)),
            transition_phase: Arc::new(Mutex::new(TransitionPhase::Normal)),
        }
    }

    /// Load a track from a file path. Does not start playback.
    /// Returns the track metadata (sample rate, channels, duration).
    pub fn load_track(
        &self,
        path: &Path,
    ) -> Result<TrackMetadata, DecodeError> {
        let decoder = BufferedDecoder::new(path)?;
        let metadata = TrackMetadata {
            sample_rate: decoder.sample_rate(),
            channels: decoder.channels(),
            duration_secs: decoder.duration_secs(),
        };
        self.transition_load_requested.store(false, Ordering::Relaxed);
        *self.pending_transition_info.lock().unwrap() = None;
        *self.next_decoder.lock().unwrap() = None;
        *self.current_decoder.lock().unwrap() = Some(decoder);
        *self.playback_state.lock().unwrap() = PlaybackState::Stopped;
        Ok(metadata)
    }

    /// Start playback of the currently loaded track.
    pub fn play(&mut self) -> Result<(), String> {
        let has_decoder = {
            let decoder = self.current_decoder.lock().unwrap();
            decoder.is_some()
        };
        if !has_decoder {
            return Err("No track loaded".to_string());
        }

        // Update playback state
        *self.playback_state.lock().unwrap() = PlaybackState::Playing;

        // If the output is already running, just update state
        {
            let output = self.output.lock().unwrap();
            if output.is_playing() {
                return Ok(());
            }
        }

        // Query device sample rate before creating the callback so the
        // captured AtomicU32 has the correct value from the start.
        {
            let output = self.output.lock().unwrap();
            if let Ok(device) = output.default_device() {
                if let Ok(default_cfg) = device.default_output_config() {
                    self.output_sample_rate
                        .store(default_cfg.sample_rate().0, Ordering::Relaxed);
                }
            }
        }

        // Prepare the audio callback
        let current_decoder = Arc::clone(&self.current_decoder);
        let next_decoder = Arc::clone(&self.next_decoder);
        let playback_state = Arc::clone(&self.playback_state);
        let running = Arc::clone(&self.running);
        let mix_engine = Arc::clone(&self.mix_engine);
        let playlist_context = Arc::clone(&self.playlist_context);
        let current_track_index = Arc::clone(&self.current_track_index);
        let transition_phase = Arc::clone(&self.transition_phase);
        let mix_points = Arc::clone(&self.mix_points);
        let pending_track_change = Arc::clone(&self.pending_track_change);
        let transition_load_requested = Arc::clone(&self.transition_load_requested);
        let pending_transition_info = Arc::clone(&self.pending_transition_info);

        let output_sample_rate = Arc::clone(&self.output_sample_rate);

        let callback: AudioCallback = Arc::new(Mutex::new(move |num_frames: u32| -> Vec<f32> {
            if !running.load(Ordering::SeqCst) {
                return vec![0.0; num_frames as usize * 2];
            }

            let state = *playback_state.lock().unwrap();

            match state {
                PlaybackState::Playing => {
                    // Check if we're currently in a transition
                    let is_transitioning = {
                        let phase = transition_phase.lock().unwrap();
                        matches!(&*phase, TransitionPhase::Transitioning { .. })
                    };

                    if is_transitioning {
                        // --- Transitioning phase: read from both decoders ---
                        let mut cur_guard = current_decoder.lock().unwrap();
                        let mut nxt_guard = next_decoder.lock().unwrap();

                        let current = match cur_guard.as_ref() {
                            Some(d) => d,
                            None => return vec![0.0; num_frames as usize * 2],
                        };
                        let next = match nxt_guard.as_ref() {
                            Some(d) => d,
                            None => return vec![0.0; num_frames as usize * 2],
                        };

                        let channels = current.channels();
                        // read() is lock-free (ring buffer pop) — safe in audio callback
                        let dst_rate = output_sample_rate.load(Ordering::Relaxed);
                        let cur_src_rate = current.sample_rate();
                        let nxt_src_rate = next.sample_rate();
                        let cur_samples = if cur_src_rate != dst_rate {
                            let src_frames = (num_frames as f64 * cur_src_rate as f64 / dst_rate as f64).max(1.0) as usize;
                            let raw = current.read(src_frames);
                            resample(&raw, src_frames, num_frames as usize, channels)
                        } else {
                            current.read(num_frames as usize)
                        };
                        let nxt_samples = if nxt_src_rate != dst_rate {
                            let src_frames = (num_frames as f64 * nxt_src_rate as f64 / dst_rate as f64).max(1.0) as usize;
                            let raw = next.read(src_frames);
                            resample(&raw, src_frames, num_frames as usize, channels)
                        } else {
                            next.read(num_frames as usize)
                        };

                        let mut phase = transition_phase.lock().unwrap();
                        if let TransitionPhase::Transitioning { out_gain, in_gain, cursor } = &mut *phase {
                            let mut output = Vec::with_capacity(num_frames as usize * channels);
                            for frame in 0..(num_frames as usize) {
                                let og = out_gain.get(*cursor + frame).copied().unwrap_or(0.0);
                                let ig = in_gain.get(*cursor + frame).copied().unwrap_or(1.0);
                                for ch in 0..channels {
                                    let idx = frame * channels + ch;
                                    let cs = cur_samples.get(idx).copied().unwrap_or(0.0);
                                    let ns = nxt_samples.get(idx).copied().unwrap_or(0.0);
                                    output.push(cs * og + ns * ig);
                                }
                            }
                            *cursor += num_frames as usize;

                            if *cursor >= out_gain.len() {
                                // Transition complete — promote next decoder asynchronously.
                                // Dropping the old BufferedDecoder joins its decode thread,
                                // which can block; spawn to avoid stalling the audio callback.
                                let old = cur_guard.take();
                                *cur_guard = nxt_guard.take();
                                if let Some(dec) = old {
                                    thread::spawn(move || drop(dec));
                                }
                                if let Some(ref mut idx) = *current_track_index.lock().unwrap() {
                                    *idx += 1;
                                    *pending_track_change.lock().unwrap() = Some(*idx);
                                }
                                *phase = TransitionPhase::Normal;
                            }
                            output
                        } else {
                            vec![0.0; num_frames as usize * channels]
                        }
                    } else {
                        // --- Normal phase: read from current decoder (lock-free) ---
                            let (samples, position, duration) = {
                            let guard = current_decoder.lock().unwrap();
                            match guard.as_ref() {
                                Some(d) => {
                                    let src_rate = d.sample_rate();
                                    let dst_rate = output_sample_rate.load(Ordering::Relaxed);
                                    let s = if src_rate != dst_rate {
                                        let src_frames = (num_frames as f64 * src_rate as f64 / dst_rate as f64).max(1.0) as usize;
                                        let raw = d.read(src_frames);
                                        resample(&raw, src_frames, num_frames as usize, d.channels())
                                    } else {
                                        d.read(num_frames as usize)
                                    };
                                    let p = d.position_secs();
                                    let dur = d.duration_secs();
                                    (s, p, dur)
                                }
                                None => return vec![0.0; num_frames as usize * 2],
                            }
                        };

                        // Check if a previously-requested async load has completed.
                        let load_was_requested = transition_load_requested.load(Ordering::Relaxed);
                        let next_is_ready = next_decoder.lock().unwrap().is_some();

                        if load_was_requested && next_is_ready {
                            // Next decoder loaded asynchronously — start transition now.
                            transition_load_requested.store(false, Ordering::Relaxed);
                            if let Some((_, ref r, effective_duration)) =
                                *pending_transition_info.lock().unwrap()
                            {
                                let total_frames = (effective_duration
                                    * output_sample_rate.load(Ordering::Relaxed) as f64)
                                    as usize;
                                let (out_gain, in_gain) = match r.pattern {
                                    MixPattern::CrossFade => {
                                        MixEngine::cross_fade_envelope(total_frames, total_frames)
                                    }
                                    MixPattern::Fade => {
                                        let seg = total_frames / 3;
                                        let mut out_g = Vec::with_capacity(total_frames);
                                        let mut in_g = Vec::with_capacity(total_frames);
                                        for i in 0..seg {
                                            let t = if seg > 0 {
                                                i as f32 / seg as f32
                                            } else {
                                                1.0
                                            };
                                            out_g.push(1.0 - t);
                                            in_g.push(0.0);
                                        }
                                        for _ in seg..(seg * 2) {
                                            if out_g.len() < total_frames {
                                                out_g.push(0.0);
                                                in_g.push(0.0);
                                            }
                                        }
                                        while out_g.len() < total_frames {
                                            let i = out_g.len() - seg * 2;
                                            let t = if seg > 0 {
                                                i as f32 / seg as f32
                                            } else {
                                                1.0
                                            };
                                            out_g.push(0.0);
                                            in_g.push(t);
                                        }
                                        (out_g, in_g)
                                    }
                                    MixPattern::HardFade => MixEngine::hard_fade_envelope(
                                        total_frames,
                                        total_frames / 2,
                                    ),
                                };

                                let mut phase = transition_phase.lock().unwrap();
                                *phase = TransitionPhase::Transitioning {
                                    out_gain,
                                    in_gain,
                                    cursor: 0,
                                };
                                // Return current-track samples; transition takes effect next frame
                                return samples;
                            }
                        }

                        // Check if we should trigger a transition (before EOF).
                        // If trigger fires, spawn a background thread to load the next decoder
                        // so the audio callback never blocks on I/O.
                        let mut should_load = false;
                        let mut resolved: Option<(usize, ResolvedMix, f64)> = None;
                        {
                            let ctx = playlist_context.lock().unwrap();
                            let idx = *current_track_index.lock().unwrap();
                            if !ctx.is_empty()
                                && idx.is_some_and(|i| i + 1 < ctx.len())
                                && duration > 0.0
                                && !load_was_requested
                            {
                                let mix_eng = mix_engine.lock().unwrap();
                                let mix_duration = mix_eng.config().duration_secs;
                                let trigger = mix_points
                                    .lock()
                                    .unwrap()
                                    .0
                                    .map(|mo| mo.min(duration))
                                    .unwrap_or_else(|| (duration - mix_duration).max(0.0));
                                let should_start = position >= trigger;

                                if should_start {
                                    if let Some(index) = idx {
                                        if index + 1 < ctx.len() {
                                            let next_entry = &ctx[index + 1];
                                            let current_mix = MixPoint {
                                                mix_out: mix_points.lock().unwrap().0,
                                                mix_in: None,
                                            };
                                            let next_mix = MixPoint {
                                                mix_out: None,
                                                mix_in: next_entry.mix_in,
                                            };
                                            let r = mix_eng.resolve(&current_mix, &next_mix);
                                            let remaining = duration - position;
                                            let effective_duration =
                                                r.duration_secs.min(remaining);
                                            resolved =
                                                Some((index, r, effective_duration));
                                            should_load = true;
                                        }
                                    }
                                }
                            }
                        }

                        if should_load {
                            if let Some((index, ref r, effective_duration)) = resolved {
                                transition_load_requested
                                    .store(true, Ordering::Relaxed);
                                *pending_transition_info.lock().unwrap() =
                                    Some((index, r.clone(), effective_duration));

                                let ctx = playlist_context.lock().unwrap();
                                if index + 1 < ctx.len() {
                                    let next_path =
                                        ctx[index + 1].file_path.clone();
                                    // Release playlist context before spawn
                                    drop(ctx);

                                    let next_dec = Arc::clone(&next_decoder);
                                    thread::spawn(move || {
                                        if let Ok(dec) = BufferedDecoder::new(
                                            Path::new(&next_path),
                                        ) {
                                            *next_dec.lock().unwrap() = Some(dec);
                                        }
                                    });
                                }
                            }
                        }

                        if !should_load
                            && !load_was_requested
                            && {
                                // Check EOF without locking the decoder
                                let guard = current_decoder.lock().unwrap();
                                guard
                                    .as_ref()
                                    .map(|d| d.is_eof())
                                    .unwrap_or(false)
                            }
                        {
                            *playback_state.lock().unwrap() = PlaybackState::Stopped;
                        }

                        samples
                    }
                }
                PlaybackState::Paused | PlaybackState::Stopped | PlaybackState::Seeking => {
                    let channels = current_decoder.lock().unwrap().as_ref().map_or(2, |d| d.channels());
                    vec![0.0; num_frames as usize * channels]
                }
            }
        }));

        // Set up and start the output
        {
            let mut output = self.output.lock().unwrap();
            output.set_callback(callback);

            if output.select_device(None).is_err() {
                return Err("No audio output device available".to_string());
            }

            output.start()?;
        }

        self.output_sample_rate.store({
            let output = self.output.lock().unwrap();
            output.config().sample_rate
        }, Ordering::Relaxed);
        self.graph.lock().unwrap().set_sample_rate(
            self.output_sample_rate.load(Ordering::Relaxed) as f64
        );
        self.running.store(true, Ordering::SeqCst);

        // Start background monitor thread for device changes
        let output_clone = Arc::clone(&self.output);
        let running_clone = Arc::clone(&self.running);
        let playback_state_clone = Arc::clone(&self.playback_state);

        self.thread_handle = Some(thread::spawn(move || {
            while running_clone.load(Ordering::SeqCst) {
                {
                    let mut output = output_clone.lock().unwrap();
                    if output.device_changed() {
                        eprintln!("Audio device changed, pausing playback");
                        let _ = output.pause();
                        *playback_state_clone.lock().unwrap() = PlaybackState::Paused;
                    }
                }
                thread::sleep(std::time::Duration::from_millis(500));
            }
        }));

        Ok(())
    }

    /// Pause playback.
    pub fn pause(&mut self) -> Result<(), String> {
        *self.playback_state.lock().unwrap() = PlaybackState::Paused;
        let mut output = self.output.lock().unwrap();
        output.pause()
    }

    /// Resume playback.
    pub fn resume(&mut self) -> Result<(), String> {
        *self.playback_state.lock().unwrap() = PlaybackState::Playing;
        let mut output = self.output.lock().unwrap();
        output.resume()
    }

    /// Stop playback and reset position.
    pub fn stop(&mut self) {
        *self.playback_state.lock().unwrap() = PlaybackState::Stopped;
        self.transition_load_requested.store(false, Ordering::Relaxed);
        *self.pending_transition_info.lock().unwrap() = None;
        *self.next_decoder.lock().unwrap() = None;
        *self.current_decoder.lock().unwrap() = None;
        *self.transition_phase.lock().unwrap() = TransitionPhase::Normal;
        let mut output = self.output.lock().unwrap();
        output.stop();
    }

    /// Seek to a position in seconds.
    pub fn seek(&mut self, position_secs: f64) {
        if let Some(ref d) = *self.current_decoder.lock().unwrap() {
            d.seek(position_secs);
        }
    }

    /// Get the current playback state.
    pub fn state(&self) -> PlaybackState {
        *self.playback_state.lock().unwrap()
    }

    /// Get the shared volume state.
    pub fn volume(&self) -> &VolumeState {
        &self.volume
    }

    /// Get the audio graph reference.
    pub fn graph(&self) -> Arc<Mutex<AudioGraph>> {
        Arc::clone(&self.graph)
    }

    /// Get current playback progress (0.0 to 1.0).
    pub fn progress(&self) -> f64 {
        let decoder = self.current_decoder.lock().unwrap();
        decoder.as_ref().map_or(0.0, |d| d.progress())
    }

    /// Get the current position in seconds.
    pub fn position_secs(&self) -> f64 {
        let decoder = self.current_decoder.lock().unwrap();
        decoder.as_ref().map_or(0.0, |d| d.position_secs())
    }

    /// Get the duration of the current track in seconds.
    pub fn duration_secs(&self) -> f64 {
        let decoder = self.current_decoder.lock().unwrap();
        decoder.as_ref().map_or(0.0, |d| d.duration_secs())
    }

    /// Get a reference to the mixing engine.
    pub fn mix_engine(&self) -> Arc<Mutex<MixEngine>> {
        Arc::clone(&self.mix_engine)
    }

    /// Get the current track's mix-out point.
    pub fn mix_out_point(&self) -> Option<f64> {
        self.mix_points.lock().unwrap().0
    }

    /// Get the current track's mix-in point.
    pub fn mix_in_point(&self) -> Option<f64> {
        self.mix_points.lock().unwrap().1
    }

    /// Set mix points for the current track.
    pub fn set_mix_points(&self, mix_out: Option<f64>, mix_in: Option<f64>) {
        if let Some(v) = mix_out {
            if v < 0.0 {
                return;
            }
        }
        if let Some(v) = mix_in {
            if v < 0.0 {
                return;
            }
        }
        let mut points = self.mix_points.lock().unwrap();
        points.0 = mix_out;
        points.1 = mix_in;
    }

    /// Set the playlist context (ordered list of track paths + mix overrides).
    pub fn set_playlist_context(&self, entries: Vec<PlaylistContextEntry>) {
        *self.playlist_context.lock().unwrap() = entries;
    }

    /// Set the current track index within the playlist context.
    pub fn set_current_track_index(&self, index: Option<usize>) {
        *self.current_track_index.lock().unwrap() = index;
    }

    /// Get the current track index within the playlist context.
    pub fn current_track_index(&self) -> Option<usize> {
        *self.current_track_index.lock().unwrap()
    }

    /// Take any pending track change index (for event emission).
    pub fn take_pending_track_change(&self) -> Option<usize> {
        self.pending_track_change.lock().unwrap().take()
    }
}

impl Drop for AudioPipeline {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            handle.join().ok();
        }
        let mut output = self.output.lock().unwrap();
        output.stop();
    }
}

// AudioPipeline is safe to send across threads (inner components handle their own safety).
unsafe impl Send for AudioPipeline {}
