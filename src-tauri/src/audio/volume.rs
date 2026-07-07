use fundsp::hacker::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Shared volume state accessible from both UI and audio threads.
#[derive(Clone)]
pub struct VolumeState {
    /// Linear gain (0.0 = silence, 1.0 = unity).
    gain: Arc<Mutex<f64>>,
    /// Whether audio is muted.
    mute: Arc<AtomicBool>,
    /// Gain before mute was activated (to restore on unmute).
    pre_mute_gain: Arc<Mutex<f64>>,
}

impl VolumeState {
    pub fn new() -> Self {
        Self {
            gain: Arc::new(Mutex::new(1.0)),
            mute: Arc::new(AtomicBool::new(false)),
            pre_mute_gain: Arc::new(Mutex::new(1.0)),
        }
    }

    /// Set the linear gain (clamped to 0.0..=1.0).
    pub fn set_gain(&self, gain: f64) {
        let gain = gain.clamp(0.0, 1.0);
        *self.gain.lock().unwrap() = gain;
        if gain > 0.0 && self.mute.load(Ordering::SeqCst) {
            self.mute.store(false, Ordering::SeqCst);
        }
    }

    /// Get the current effective gain (respects mute).
    pub fn effective_gain(&self) -> f64 {
        if self.mute.load(Ordering::SeqCst) {
            0.0
        } else {
            *self.gain.lock().unwrap()
        }
    }

    /// Get the raw gain setting (ignoring mute).
    pub fn raw_gain(&self) -> f64 {
        *self.gain.lock().unwrap()
    }

    /// Mute or unmute audio.
    pub fn set_mute(&self, mute: bool) {
        if mute {
            let current_gain = *self.gain.lock().unwrap();
            *self.pre_mute_gain.lock().unwrap() = current_gain;
            self.mute.store(true, Ordering::SeqCst);
        } else {
            self.mute.store(false, Ordering::SeqCst);
            let restored = *self.pre_mute_gain.lock().unwrap();
            *self.gain.lock().unwrap() = restored;
        }
    }

    pub fn is_muted(&self) -> bool {
        self.mute.load(Ordering::SeqCst)
    }
}

impl Default for VolumeState {
    fn default() -> Self {
        Self::new()
    }
}

/// A fundsp audio node that applies volume gain.
///
/// Shares state with `VolumeState` so the UI can adjust gain
/// without directly accessing the audio graph.
#[derive(Clone)]
pub struct VolumeNode {
    state: VolumeState,
}

impl VolumeNode {
    pub fn new(state: VolumeState) -> Self {
        Self { state }
    }
}

impl AudioNode for VolumeNode {
    const ID: u64 = 42;
    type Inputs = U2;
    type Outputs = U2;

    fn reset(&mut self) {}

    fn set_sample_rate(&mut self, _sample_rate: f64) {}

    fn tick(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
    ) -> Frame<f32, Self::Outputs> {
        let gain = self.state.effective_gain() as f32;
        let left = input[0] * gain;
        let right = input[1] * gain;
        [left, right].into()
    }
}
