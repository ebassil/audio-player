use crate::audio::decoder::{decode_file, DecodedAudio};
use crate::audio::graph::AudioGraph;
use crate::audio::output::{AudioCallback, AudioOutput};
use crate::audio::player::PlaybackState;
use crate::audio::volume::VolumeState;

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

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
    /// The current decoded audio (None when no track is loaded).
    current_track: Arc<Mutex<Option<DecodedAudio>>>,
    /// Current read position in samples (frame index).
    read_position: Arc<AtomicU32>,
    /// Whether the playback thread should keep running.
    running: Arc<AtomicBool>,
    /// Background playback thread handle.
    thread_handle: Option<thread::JoinHandle<()>>,
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
            current_track: Arc::new(Mutex::new(None)),
            read_position: Arc::new(AtomicU32::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    /// Load a track from a file path. Does not start playback.
    pub fn load_track(
        &self,
        path: &Path,
    ) -> Result<DecodedAudio, crate::audio::decoder::DecodeError> {
        let decoded = decode_file(path)?;
        let mut track = self.current_track.lock().unwrap();
        *track = Some(decoded.clone());
        self.read_position.store(0, Ordering::SeqCst);
        *self.playback_state.lock().unwrap() = PlaybackState::Stopped;
        Ok(decoded)
    }

    /// Start playback of the currently loaded track.
    pub fn play(&mut self) -> Result<(), String> {
        let track = self.current_track.lock().unwrap().clone();
        let _track = track.ok_or_else(|| "No track loaded".to_string())?;

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
        let current_track = Arc::clone(&self.current_track);
        let read_position = Arc::clone(&self.read_position);
        let playback_state = Arc::clone(&self.playback_state);
        let running = Arc::clone(&self.running);

        let callback: AudioCallback = Arc::new(Mutex::new(move |num_frames: u32| -> Vec<f32> {
            let track_guard = current_track.lock().unwrap();
            let track = match track_guard.as_ref() {
                Some(t) => t,
                None => return vec![0.0; num_frames as usize * 2],
            };

            if !running.load(Ordering::SeqCst) {
                return vec![0.0; num_frames as usize * 2];
            }

            let state = *playback_state.lock().unwrap();
            match state {
                PlaybackState::Playing => {
                    let pos = read_position.load(Ordering::SeqCst) as usize;
                    let total_samples = track.samples.len();
                    let channels = track.channels;
                    let samples_needed = num_frames as usize * channels;

                    if pos >= total_samples {
                        // End of track
                        *playback_state.lock().unwrap() = PlaybackState::Stopped;
                        return vec![0.0; samples_needed];
                    }

                    let end = (pos + samples_needed).min(total_samples);
                    let mut buffer = track.samples[pos..end].to_vec();
                    buffer.resize(samples_needed, 0.0);

                    read_position.store(end as u32, Ordering::SeqCst);
                    buffer
                }
                PlaybackState::Paused | PlaybackState::Stopped => {
                    vec![0.0; num_frames as usize * track.channels]
                }
                PlaybackState::Seeking => {
                    vec![0.0; num_frames as usize * track.channels]
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
        self.read_position.store(0, Ordering::SeqCst);
        let mut output = self.output.lock().unwrap();
        output.stop();
    }

    /// Seek to a position in seconds.
    pub fn seek(&mut self, position_secs: f64) {
        let track = self.current_track.lock().unwrap().clone();
        if let Some(track) = track {
            let sample_pos = (position_secs * track.sample_rate as f64) as u32;
            let max_pos = (track.samples.len() / track.channels) as u32;
            self.read_position
                .store(sample_pos.min(max_pos.saturating_sub(1)), Ordering::SeqCst);
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
        let track = self.current_track.lock().unwrap();
        match track.as_ref() {
            Some(t) => {
                let total_frames = t.samples.len() / t.channels;
                if total_frames == 0 {
                    return 0.0;
                }
                let pos = self.read_position.load(Ordering::SeqCst) as usize / t.channels;
                (pos as f64 / total_frames as f64).clamp(0.0, 1.0)
            }
            None => 0.0,
        }
    }

    /// Get the current position in seconds.
    pub fn position_secs(&self) -> f64 {
        let track = self.current_track.lock().unwrap();
        match track.as_ref() {
            Some(t) => {
                let pos = self.read_position.load(Ordering::SeqCst) as usize / t.channels;
                pos as f64 / t.sample_rate as f64
            }
            None => 0.0,
        }
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
