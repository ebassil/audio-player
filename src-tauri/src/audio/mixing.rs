use serde::{Deserialize, Serialize};

/// The mix pattern between consecutive tracks.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MixPattern {
    /// Fade out → pause → fade in
    Fade,
    /// Fade out + fade in overlapped
    CrossFade,
    /// Silence gap between tracks
    HardFade,
}

/// Per-song mix-in/mix-out point overrides.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MixPoint {
    /// Time offset (seconds) from the start of the song where mix-out begins.
    pub mix_out: Option<f64>,
    /// Time offset (seconds) from the start of the song where mix-in resolves.
    pub mix_in: Option<f64>,
}

/// Mix configuration for the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixConfig {
    /// Default mix pattern.
    pub pattern: MixPattern,
    /// Default mix duration in seconds.
    pub duration_secs: f64,
}

impl Default for MixConfig {
    fn default() -> Self {
        Self {
            pattern: MixPattern::CrossFade,
            duration_secs: 3.0,
        }
    }
}

/// Resolved mix parameters for a specific transition.
#[derive(Debug, Clone)]
pub struct ResolvedMix {
    pub pattern: MixPattern,
    pub duration_secs: f64,
    /// Time offset into the current track where transition begins.
    pub mix_out_point: Option<f64>,
    /// Time offset into the next track where transition ends.
    pub mix_in_point: Option<f64>,
}

impl MixPoint {
    pub fn new() -> Self {
        Self {
            mix_out: None,
            mix_in: None,
        }
    }
}

impl Default for MixPoint {
    fn default() -> Self {
        Self::new()
    }
}

/// The mixing engine that computes gain ramps for transitions.
pub struct MixEngine {
    config: MixConfig,
}

impl MixEngine {
    pub fn new(config: MixConfig) -> Self {
        Self { config }
    }

    /// Resolve the mix parameters for a transition between two tracks.
    pub fn resolve(
        &self,
        current_track_mix: &MixPoint,
        next_track_mix: &MixPoint,
    ) -> ResolvedMix {
        let pattern = self.config.pattern;
        let duration = self.config.duration_secs;

        // Per-song mix points override the default duration for that transition
        let mix_out_point = current_track_mix.mix_out;
        let mix_in_point = next_track_mix.mix_in;

        ResolvedMix {
            pattern,
            duration_secs: duration,
            mix_out_point,
            mix_in_point,
        }
    }

    /// Generate gain envelope for a fade transition.
    ///
    /// Returns a gain scalar for each frame of the transition.
    /// Phase 0 = fade out, phase 1 = gap, phase 2 = fade in.
    pub fn fade_envelope(total_frames: usize, duration_frames: usize) -> Vec<f32> {
        let mut envelope = Vec::with_capacity(total_frames);
        let gap_frames = (total_frames.saturating_sub(duration_frames * 2)) / 2;

        // Fade out (0..duration_frames)
        for i in 0..duration_frames.min(total_frames) {
            let gain = 1.0 - (i as f32 / duration_frames as f32);
            envelope.push(gain);
        }

        // Gap (silence)
        for _ in 0..gap_frames {
            envelope.push(0.0);
        }

        // Fade in
        let fade_in_start = envelope.len();
        for i in 0..duration_frames {
            if fade_in_start + i >= total_frames {
                break;
            }
            let gain = i as f32 / duration_frames as f32;
            envelope.push(gain);
        }

        // Fill remaining with 1.0
        while envelope.len() < total_frames {
            envelope.push(1.0);
        }

        envelope
    }

    /// Generate gain envelope for a cross-fade transition.
    ///
    /// Both tracks overlap during the transition.
    pub fn cross_fade_envelope(
        total_frames: usize,
        duration_frames: usize,
    ) -> (Vec<f32>, Vec<f32>) {
        let mut out_gain = Vec::with_capacity(total_frames);
        let mut in_gain = Vec::with_capacity(total_frames);

        for i in 0..total_frames {
            if i < duration_frames && duration_frames > 0 {
                let t = i as f32 / duration_frames as f32;
                out_gain.push(1.0 - t);
                in_gain.push(t);
            } else {
                out_gain.push(0.0);
                in_gain.push(1.0);
            }
        }

        (out_gain, in_gain)
    }

    /// Generate gain envelope for a hard fade (silence gap).
    ///
    /// Current track stops, silence for gap, then next track starts.
    pub fn hard_fade_envelope(
        total_frames: usize,
        gap_frames: usize,
    ) -> (Vec<f32>, Vec<f32>) {
        let mut out_gain = vec![1.0; total_frames];
        let mut in_gain = vec![0.0; total_frames];

        // Out track: sudden stop after gap
        // In track: starts after gap
        if gap_frames < total_frames {
            for i in gap_frames..total_frames {
                out_gain[i] = 0.0;
                in_gain[i] = 1.0;
            }
        }

        (out_gain, in_gain)
    }

    /// Update the default config.
    pub fn set_config(&mut self, config: MixConfig) {
        self.config = config;
    }

    /// Get the current config.
    pub fn config(&self) -> &MixConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fade_envelope_length() {
        let env = MixEngine::fade_envelope(100, 20);
        assert_eq!(env.len(), 100);
        assert!((env[0] - 1.0).abs() < 0.001);
        assert!((env[99] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cross_fade_envelope_length() {
        let (out_g, in_g) = MixEngine::cross_fade_envelope(100, 20);
        assert_eq!(out_g.len(), 100);
        assert_eq!(in_g.len(), 100);
        assert!((out_g[0] - 1.0).abs() < 0.001);
        assert!((in_g[0] - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_hard_fade_envelope() {
        let (out_g, in_g) = MixEngine::hard_fade_envelope(100, 30);
        assert_eq!(out_g.len(), 100);
        assert_eq!(in_g.len(), 100);
        assert_eq!(out_g[0], 1.0);
        assert_eq!(in_g[0], 0.0);
        assert_eq!(out_g[30], 0.0);
        assert_eq!(in_g[30], 1.0);
    }

    #[test]
    fn test_resolve_uses_defaults() {
        let engine = MixEngine::new(MixConfig::default());
        let resolved = engine.resolve(&MixPoint::new(), &MixPoint::new());
        assert_eq!(resolved.pattern, MixPattern::CrossFade);
        assert_eq!(resolved.duration_secs, 3.0);
    }

    #[test]
    fn test_resolve_uses_overrides() {
        let engine = MixEngine::new(MixConfig::default());
        let current = MixPoint {
            mix_out: Some(120.0),
            mix_in: None,
        };
        let next = MixPoint {
            mix_out: None,
            mix_in: Some(30.0),
        };
        let resolved = engine.resolve(&current, &next);
        assert_eq!(resolved.mix_out_point, Some(120.0));
        assert_eq!(resolved.mix_in_point, Some(30.0));
    }
}
