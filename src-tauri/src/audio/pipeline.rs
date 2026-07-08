use crate::audio::decoder::{DecodeError, StreamingDecoder};
use crate::audio::graph::AudioGraph;
use crate::audio::mixing::{MixConfig, MixEngine};
use crate::audio::output::{AudioCallback, AudioOutput};
use crate::audio::player::PlaybackState;
use crate::audio::volume::VolumeState;

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// Track metadata returned by `load_track`.
#[derive(Clone, Debug)]
pub struct TrackMetadata {
    pub sample_rate: u32,
    pub channels: usize,
    pub duration_secs: f64,
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
    /// The current streaming decoder (None when no track is loaded).
    current_decoder: Arc<Mutex<Option<StreamingDecoder>>>,
    /// Whether the playback thread should keep running.
    running: Arc<AtomicBool>,
    /// Background playback thread handle.
    thread_handle: Option<thread::JoinHandle<()>>,
    /// Mixing engine for track transitions.
    mix_engine: Arc<Mutex<MixEngine>>,
    /// Mix points for the current track (interior mutability via Mutex).
    mix_points: Arc<Mutex<(Option<f64>, Option<f64>)>>,
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
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            mix_engine: Arc::new(Mutex::new(MixEngine::new(MixConfig::default()))),
            mix_points: Arc::new(Mutex::new((None, None))),
        }
    }

    /// Load a track from a file path. Does not start playback.
    /// Returns the track metadata (sample rate, channels, duration).
    pub fn load_track(
        &self,
        path: &Path,
    ) -> Result<TrackMetadata, DecodeError> {
        let decoder = StreamingDecoder::new(path)?;
        let metadata = TrackMetadata {
            sample_rate: decoder.sample_rate(),
            channels: decoder.channels(),
            duration_secs: decoder.duration_secs(),
        };
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

        // Prepare the audio callback
        let current_decoder = Arc::clone(&self.current_decoder);
        let playback_state = Arc::clone(&self.playback_state);
        let running = Arc::clone(&self.running);

        let callback: AudioCallback = Arc::new(Mutex::new(move |num_frames: u32| -> Vec<f32> {
            if !running.load(Ordering::SeqCst) {
                return vec![0.0; num_frames as usize * 2];
            }

            let state = *playback_state.lock().unwrap();
            let mut decoder_guard = current_decoder.lock().unwrap();
            let decoder = match decoder_guard.as_mut() {
                Some(d) => d,
                None => return vec![0.0; num_frames as usize * 2],
            };
            let channels = decoder.channels();
            let silence = vec![0.0; num_frames as usize * channels];

            match state {
                PlaybackState::Playing => {
                    let samples = decoder.read(num_frames as usize);
                    if decoder.is_eof() {
                        *playback_state.lock().unwrap() = PlaybackState::Stopped;
                    }
                    samples
                }
                PlaybackState::Paused | PlaybackState::Stopped | PlaybackState::Seeking => silence,
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
        if let Some(ref mut d) = *self.current_decoder.lock().unwrap() {
            d.seek(0.0);
        }
        let mut output = self.output.lock().unwrap();
        output.stop();
    }

    /// Seek to a position in seconds.
    pub fn seek(&mut self, position_secs: f64) {
        let mut decoder = self.current_decoder.lock().unwrap();
        if let Some(ref mut d) = *decoder {
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
        match decoder.as_ref() {
            Some(d) => d.progress(),
            None => 0.0,
        }
    }

    /// Get the current position in seconds.
    pub fn position_secs(&self) -> f64 {
        let decoder = self.current_decoder.lock().unwrap();
        match decoder.as_ref() {
            Some(d) => d.position_secs(),
            None => 0.0,
        }
    }

    /// Get the duration of the current track in seconds.
    pub fn duration_secs(&self) -> f64 {
        let decoder = self.current_decoder.lock().unwrap();
        match decoder.as_ref() {
            Some(d) => d.duration_secs(),
            None => 0.0,
        }
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
