use std::fs::File;
use std::path::Path;

use symphonia::core::audio::{AudioBuffer, AudioBufferRef, Channels, Signal};
use symphonia::core::codecs::{Decoder, CODEC_TYPE_NULL};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

/// Errors that can occur during decoding.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Symphonia error: {0}")]
    Symphonia(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("No audio track found in file")]
    NoAudioTrack,

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Decoding failed: {0}")]
    DecodeFailed(String),
}

/// Streaming decoder that decodes audio incrementally on-demand.
///
/// Reads a compressed audio file via symphonia and maintains an internal
/// buffer of decoded f32 PCM samples. The buffer is refilled automatically
/// as samples are consumed by `read()`.
pub struct StreamingDecoder {
    /// The symphonia format reader (provides demuxed packets).
    format: Box<dyn FormatReader>,
    /// The symphonia audio decoder for the selected track.
    decoder: Box<dyn Decoder>,
    /// The selected track's ID.
    track_id: u32,
    /// Sample rate in Hz.
    sample_rate: u32,
    /// Number of audio channels.
    channels: usize,
    /// Total duration in seconds.
    duration_secs: f64,
    /// Internal buffer of decoded interleaved f32 samples.
    buffer: Vec<f32>,
    /// Number of frames (per-channel) that have been read out so far.
    total_frames_read: u64,
    /// Target number of frames to prefill the buffer to.
    prefill_frames: usize,
    /// Minimum remaining frames before triggering a refill.
    refill_threshold: usize,
    /// Whether the end of the audio stream has been reached.
    eof_reached: bool,
}

impl StreamingDecoder {
    /// Open an audio file and prepare for streaming decode without decoding any packets.
    pub fn new(path: &Path) -> Result<Self, DecodeError> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if !matches!(
            extension.as_str(),
            "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" | "opus"
        ) {
            return Err(DecodeError::UnsupportedFormat(extension));
        }

        let file = File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension(&extension);

        let format_opts = FormatOptions::default();
        let meta_opts = MetadataOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &meta_opts)
            .map_err(|e| DecodeError::Symphonia(e.into()))?;

        let format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or(DecodeError::NoAudioTrack)?
            .clone();

        let track_id = track.id;
        let codec_params = &track.codec_params;
        let sample_rate = codec_params.sample_rate.unwrap_or(44100);
        let channels = codec_params
            .channels
            .unwrap_or(Channels::FRONT_LEFT | Channels::FRONT_RIGHT)
            .count();

        let duration_secs = codec_params
            .n_frames
            .map(|n| n as f64 / sample_rate as f64)
            .unwrap_or(0.0);

        let codec_registry = symphonia::default::get_codecs();
        let decoder = codec_registry
            .make(codec_params, &Default::default())
            .map_err(|e| DecodeError::Symphonia(e.into()))?;

        let prefill_frames = 8192;
        let refill_threshold = 2048;

        Ok(Self {
            format,
            decoder,
            track_id,
            sample_rate,
            channels,
            duration_secs,
            buffer: Vec::with_capacity(prefill_frames * channels),
            total_frames_read: 0,
            prefill_frames,
            refill_threshold,
            eof_reached: false,
        })
    }

    /// Read up to `num_frames` of interleaved f32 samples. Returns exactly
    /// `num_frames * channels` samples, padding with silence at EOF.
    pub fn read(&mut self, num_frames: usize) -> Vec<f32> {
        if self.eof_reached {
            return vec![0.0; num_frames * self.channels];
        }

        let samples_needed = num_frames * self.channels;

        // Refill buffer if running low
        if self.buffer.len() < samples_needed {
            self.fill_buffer();
        }

        let available = self.buffer.len();
        let to_read = samples_needed.min(available);

        let mut result = Vec::with_capacity(samples_needed);
        result.extend_from_slice(&self.buffer[..to_read]);

        // Remove consumed samples from buffer
        self.buffer.drain(..to_read);

        self.total_frames_read += (to_read / self.channels) as u64;

        // Pad with silence if at EOF
        result.resize(samples_needed, 0.0);
        result
    }

    /// Sample rate in Hz.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Number of audio channels.
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Total track duration in seconds.
    pub fn duration_secs(&self) -> f64 {
        self.duration_secs
    }

    /// Current playback position in seconds.
    pub fn position_secs(&self) -> f64 {
        self.total_frames_read as f64 / self.sample_rate as f64
    }

    /// Playback progress as a fraction 0.0–1.0.
    pub fn progress(&self) -> f64 {
        if self.duration_secs <= 0.0 {
            return 0.0;
        }
        (self.position_secs() / self.duration_secs).clamp(0.0, 1.0)
    }

    /// Whether the end of the audio stream has been reached.
    pub fn is_eof(&self) -> bool {
        self.eof_reached
    }

    /// Seek to a time position in seconds. After seeking, the next `read()` call
    /// returns samples from the new position.
    pub fn seek(&mut self, position_secs: f64) {
        let seek_to = SeekTo::Time {
            time: Time::from(position_secs),
            track_id: None,
        };
        let _ = self.format.seek(SeekMode::Accurate, seek_to);

        // Re-create the decoder after seeking (decoder state is invalidated)
        if let Some(params) = self
            .format
            .tracks()
            .iter()
            .find(|t| t.id == self.track_id)
            .map(|t| &t.codec_params)
        {
            let codec_registry = symphonia::default::get_codecs();
            if let Ok(new_decoder) = codec_registry.make(params, &Default::default()) {
                self.decoder = new_decoder;
            }
        }

        self.buffer.clear();
        self.total_frames_read = (position_secs * self.sample_rate as f64) as u64;
        self.eof_reached = false;
    }

    /// Decode packets into the internal buffer until it reaches `prefill_frames` or EOF.
    fn fill_buffer(&mut self) {
        let target_samples = self.prefill_frames * self.channels;
        while self.buffer.len() < target_samples {
            if self.eof_reached {
                return;
            }

            let packet = match self.format.next_packet() {
                Ok(pkt) => pkt,
                Err(symphonia::core::errors::Error::IoError(ref e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    self.eof_reached = true;
                    return;
                }
                Err(symphonia::core::errors::Error::IoError(_)) => {
                    self.eof_reached = true;
                    return;
                }
                Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
                Err(_) => {
                    self.eof_reached = true;
                    return;
                }
            };

            let decoded = match self.decoder.decode(&packet) {
                Ok(d) => d,
                Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
                Err(_) => {
                    self.eof_reached = true;
                    return;
                }
            };

            let spec = *decoded.spec();
            let num_frames = decoded.frames();
            let num_channels = spec.channels.count();

            match decoded {
                AudioBufferRef::F32(buf) => {
                    for frame in 0..num_frames {
                        for ch in 0..num_channels {
                            self.buffer.push(buf.chan(ch)[frame]);
                        }
                    }
                }
                other => {
                    let mut scratch = AudioBuffer::<f32>::new(num_frames as u64, spec);
                    other.convert(&mut scratch);
                    for frame in 0..num_frames {
                        for ch in 0..num_channels {
                            self.buffer.push(scratch.chan(ch)[frame]);
                        }
                    }
                }
            }
        }
    }
}
