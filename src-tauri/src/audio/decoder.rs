use std::fs::File;
use std::path::Path;

use symphonia::core::audio::{Channels, Signal};
use symphonia::core::codecs::CODEC_TYPE_NULL;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Decoded audio data in f32 PCM format.
#[derive(Clone, Debug)]
pub struct DecodedAudio {
    /// Interleaved PCM f32 samples (L, R, L, R, ... for stereo).
    pub samples: Vec<f32>,
    /// Sample rate in Hz (e.g. 44100).
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: usize,
    /// Duration in seconds.
    pub duration_secs: f64,
}

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

/// Decode an audio file at `path` into PCM f32 samples.
///
/// Supports MP3, WAV, FLAC, OGG, and AAC via symphonia's "all" feature set.
pub fn decode_file(path: &Path) -> Result<DecodedAudio, DecodeError> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    // Validate supported formats early
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

    let mut format = probed.format;

    // Select the first audio track
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or(DecodeError::NoAudioTrack)?;

    let codec_params = &track.codec_params;
    let sample_rate = codec_params.sample_rate.unwrap_or(44100);
    let channels = codec_params
        .channels
        .unwrap_or(Channels::FRONT_LEFT | Channels::FRONT_RIGHT)
        .count();

    let codec_registry = symphonia::default::get_codecs();
    let mut decoder = codec_registry
        .make(codec_params, &Default::default())
        .map_err(|e| DecodeError::Symphonia(e.into()))?;

    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(pkt) => pkt,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(symphonia::core::errors::Error::IoError(_)) => break,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(_) => break,
        };

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(_) => break,
        };

        // Convert to interleaved f32 samples
        let spec = *decoded.spec();
        let num_frames = decoded.frames();
        let num_channels = spec.channels.count();

        match decoded {
            symphonia::core::audio::AudioBufferRef::F32(buf) => {
                for frame in 0..num_frames {
                    for ch in 0..num_channels {
                        all_samples.push(buf.chan(ch)[frame]);
                    }
                }
            }
            other => {
                let mut scratch =
                    symphonia::core::audio::AudioBuffer::<f32>::new(num_frames as u64, spec);
                other.convert(&mut scratch);
                for frame in 0..num_frames {
                    for ch in 0..num_channels {
                        all_samples.push(scratch.chan(ch)[frame]);
                    }
                }
            }
        }
    }

    let total_frames = all_samples.len() / channels;
    let duration_secs = if sample_rate > 0 {
        total_frames as f64 / sample_rate as f64
    } else {
        0.0
    };

    Ok(DecodedAudio {
        samples: all_samples,
        sample_rate,
        channels,
        duration_secs,
    })
}
