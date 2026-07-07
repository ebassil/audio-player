use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Configuration for the audio output stream.
#[derive(Clone, Debug)]
pub struct OutputConfig {
    /// Desired sample rate in Hz (e.g. 44100).
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Buffer size in sample frames.
    pub buffer_size: u32,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            buffer_size: 512,
        }
    }
}

/// Audio output device information.
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub supported_sample_rates: Vec<(u32, u32)>,
    pub default_sample_rate: u32,
    pub max_channels: u16,
}

/// Callback type for filling audio buffers.
///
/// Receives the number of frames requested and returns interleaved f32 samples.
pub type AudioCallback = Arc<Mutex<dyn FnMut(u32) -> Vec<f32> + Send + 'static>>;

/// A wrapper around `cpal::Stream` that implements `Send`.
///
/// On macOS (CoreAudio), stream handles are actually safe to move between threads,
/// but cpal conservatively marks them as `!Send`. This wrapper asserts safety.
struct StreamHandle(Option<Stream>);

unsafe impl Send for StreamHandle {}
unsafe impl Sync for StreamHandle {}

/// Manages audio output via cpal.
pub struct AudioOutput {
    /// The cpal host.
    host: Host,
    /// Currently active output device.
    current_device: Option<Device>,
    /// Active output stream (wrapped for Send safety).
    stream: StreamHandle,
    /// Current output configuration.
    config: OutputConfig,
    /// Callback that fills audio buffers for playback.
    callback: Option<AudioCallback>,
    /// Whether the output is currently running.
    is_playing: Arc<AtomicBool>,
    /// Whether a device change is pending notification.
    device_changed: Arc<AtomicBool>,
}

// AudioOutput is safe to send across threads on macOS (CoreAudio is thread-safe).
unsafe impl Send for AudioOutput {}
unsafe impl Sync for AudioOutput {}

impl AudioOutput {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
            current_device: None,
            stream: StreamHandle(None),
            config: OutputConfig::default(),
            callback: None,
            is_playing: Arc::new(AtomicBool::new(false)),
            device_changed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Enumerate all available audio output devices.
    pub fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>, cpal::DevicesError> {
        let mut devices = Vec::new();
        for device in self.host.output_devices()? {
            let name = device.name().unwrap_or_else(|_| "Unknown".into());
            let default_cfg = device.default_output_config().ok();
            let supported_sample_rates = device
                .supported_output_configs()
                .ok()
                .map(|configs| {
                    configs
                        .flat_map(|cfg| {
                            let min = cfg.min_sample_rate().0;
                            let max = cfg.max_sample_rate().0;
                            vec![(min, max)]
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let default_sample_rate = default_cfg
                .as_ref()
                .map(|c| c.sample_rate().0)
                .unwrap_or(44100);
            let max_channels = default_cfg.as_ref().map(|c| c.channels()).unwrap_or(2);
            devices.push(DeviceInfo {
                name,
                supported_sample_rates,
                default_sample_rate,
                max_channels,
            });
        }
        Ok(devices)
    }

    /// Get the default output device.
    pub fn default_device(&self) -> Result<Device, cpal::DefaultStreamConfigError> {
        self.host.default_output_device().ok_or(
            cpal::DefaultStreamConfigError::DeviceNotAvailable,
        )
    }

    /// Select an output device by name. Falls back to default if not found or name is empty.
    pub fn select_device(&mut self, name: Option<&str>) -> Result<DeviceInfo, String> {
        let device = match name {
            Some(n) if !n.is_empty() => self
                .host
                .output_devices()
                .map_err(|e| e.to_string())?
                .find(|d| d.name().map(|dn| dn == n).unwrap_or(false))
                .ok_or_else(|| format!("Device '{}' not found", n))?,
            _ => self
                .host
                .default_output_device()
                .ok_or_else(|| "No default output device available".to_string())?,
        };

        let info = self.device_info(&device)?;
        self.current_device = Some(device);
        Ok(info)
    }

    /// Build device info for a given device.
    fn device_info(&self, device: &Device) -> Result<DeviceInfo, String> {
        let name = device.name().map_err(|e| e.to_string())?;
        let default_cfg = device
            .default_output_config()
            .map_err(|e| e.to_string())?;
        Ok(DeviceInfo {
            name,
            supported_sample_rates: Vec::new(),
            default_sample_rate: default_cfg.sample_rate().0,
            max_channels: default_cfg.channels(),
        })
    }

    /// Set the audio callback that fills buffers.
    pub fn set_callback(&mut self, callback: AudioCallback) {
        self.callback = Some(callback);
    }

    /// Start audio playback on the currently selected device.
    pub fn start(&mut self) -> Result<(), String> {
        let device = self
            .current_device
            .as_ref()
            .ok_or_else(|| "No output device selected".to_string())?;

        let config: StreamConfig = device
            .default_output_config()
            .map_err(|e| e.to_string())?
            .into();

        let callback = self
            .callback
            .as_ref()
            .ok_or_else(|| "No audio callback set".to_string())?;
        let callback_clone = Arc::clone(callback);
        let device_changed = Arc::clone(&self.device_changed);

        let err_fn = move |err: cpal::StreamError| {
            eprintln!("Audio stream error: {}", err);
            device_changed.store(true, Ordering::SeqCst);
        };

        let channels = config.channels as usize;

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                    let num_frames = data.len() / channels;
                    let samples = {
                        let mut cb = callback_clone.lock().unwrap();
                        cb(num_frames as u32)
                    };
                    for (i, sample) in data.iter_mut().enumerate() {
                        *sample = samples.get(i).copied().unwrap_or(0.0);
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| e.to_string())?;

        stream.play().map_err(|e| e.to_string())?;
        self.stream = StreamHandle(Some(stream));
        self.is_playing.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Pause audio playback.
    pub fn pause(&mut self) -> Result<(), String> {
        if let Some(ref stream) = self.stream.0 {
            stream.pause().map_err(|e| e.to_string())?;
        }
        self.is_playing.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Resume audio playback.
    pub fn resume(&mut self) -> Result<(), String> {
        if let Some(ref stream) = self.stream.0 {
            stream.play().map_err(|e| e.to_string())?;
        }
        self.is_playing.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Stop audio playback and drop the stream.
    pub fn stop(&mut self) {
        self.stream = StreamHandle(None);
        self.is_playing.store(false, Ordering::SeqCst);
    }

    /// Check if the device has been disconnected/changed.
    pub fn device_changed(&self) -> bool {
        self.device_changed.swap(false, Ordering::SeqCst)
    }

    /// Whether the output is currently playing.
    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    /// Get the current output config.
    pub fn config(&self) -> &OutputConfig {
        &self.config
    }

    /// Update output config (will take effect on next stream start).
    pub fn set_config(&mut self, config: OutputConfig) {
        self.config = config;
    }
}

impl Default for AudioOutput {
    fn default() -> Self {
        Self::new()
    }
}
