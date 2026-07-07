use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use fundsp::hacker::*;

use crate::audio::plugin_host::PluginHost;

const DEFAULT_BLOCK_SIZE: usize = 512;

/// A fundsp audio node that wraps a WASM plugin instance.
///
/// Bridges fundsp's pull-based sample model with the WASM plugin's
/// push-based block processing model. Incoming samples are buffered
/// until a full block is accumulated, then dispatched to the plugin's
/// `process` function.
#[derive(Clone)]
pub struct PluginAdapterNode {
    /// Unique plugin ID in the host.
    plugin_id: usize,
    /// Shared plugin host reference.
    host: Arc<Mutex<PluginHost>>,
    /// Block size (number of frames per WASM call).
    block_size: usize,
    /// Input sample buffer (accumulates samples for the block).
    input_buffer: Vec<f32>,
    /// Output sample buffer (filled by WASM plugin).
    output_buffer: Vec<f32>,
    /// Cursor within the current output block.
    output_cursor: usize,
    /// Whether the node is bypassed.
    bypass: Arc<AtomicBool>,
    /// Current sample rate.
    sample_rate: f64,
}

impl PluginAdapterNode {
    pub fn new(plugin_id: usize, host: Arc<Mutex<PluginHost>>) -> Self {
        let block_size = DEFAULT_BLOCK_SIZE;
        Self {
            plugin_id,
            host,
            block_size,
            input_buffer: Vec::with_capacity(block_size * 2),
            output_buffer: Vec::with_capacity(block_size * 2),
            output_cursor: 0,
            bypass: Arc::new(AtomicBool::new(false)),
            sample_rate: 44100.0,
        }
    }

    /// Set whether the plugin is bypassed (passthrough when true).
    pub fn set_bypass(&mut self, bypass: bool) {
        self.bypass.store(bypass, Ordering::SeqCst);
    }

    /// Set the block size for WASM calls.
    pub fn set_block_size(&mut self, block_size: usize) {
        self.block_size = block_size;
        self.input_buffer = Vec::with_capacity(block_size * 2);
        self.output_buffer = Vec::with_capacity(block_size * 2);
        self.output_cursor = 0;
    }
}

impl AudioNode for PluginAdapterNode {
    const ID: u64 = 43;
    type Inputs = U2;
    type Outputs = U2;

    fn reset(&mut self) {
        self.input_buffer.clear();
        self.output_buffer.clear();
        self.output_cursor = 0;
        // Notify the plugin to reset
        if let Ok(mut host) = self.host.lock() {
            if let Some(plugin) = host.get_mut(self.plugin_id) {
                let _ = plugin.reset();
            }
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        // If bypassed, pass through
        if self.bypass.load(Ordering::SeqCst) {
            return *input;
        }

        // If we have output samples buffered, serve from cache
        if self.output_cursor < self.output_buffer.len() {
            let left = self.output_buffer[self.output_cursor];
            let right = self.output_buffer.get(self.output_cursor + 1).copied().unwrap_or(0.0);
            self.output_cursor += 2;
            return [left, right].into();
        }

        // Accumulate input samples into the block buffer
        self.input_buffer.push(input[0]);
        self.input_buffer.push(input[1]);

        // When we have a full block, dispatch to WASM
        if self.input_buffer.len() >= self.block_size * 2 {
            let block_len = if self.input_buffer.len() > self.block_size * 2 {
                // If somehow overfilled, trim to exact block size
                self.input_buffer.truncate(self.block_size * 2);
                self.block_size * 2
            } else {
                self.input_buffer.len()
            };

            let input_block = std::mem::take(&mut self.input_buffer);
            let output_block = vec![0.0f32; block_len];

            if let Ok(mut host) = self.host.lock() {
                if let Some(plugin) = host.get_mut(self.plugin_id) {
                    match plugin.process(input_block, output_block) {
                        Ok(processed) => {
                            self.output_buffer = processed;
                        }
                        Err(e) => {
                            eprintln!("Plugin {} process error: {}", self.plugin_id, e);
                            self.output_buffer = vec![0.0; block_len];
                        }
                    }
                } else {
                    self.output_buffer = vec![0.0; block_len];
                }
            } else {
                self.output_buffer = vec![0.0; block_len];
            }

            self.output_cursor = 2;
            let left = self.output_buffer[0];
            let right = self.output_buffer.get(1).copied().unwrap_or(0.0);
            return [left, right].into();
        }

        // Not a full block yet, output silence
        [0.0, 0.0].into()
    }
}
