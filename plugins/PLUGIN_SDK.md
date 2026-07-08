# Plugin SDK

## Overview

Audio Player plugins are WebAssembly (WASM) components loaded at runtime via wasmtime. Plugins process audio in the DSP chain and can provide custom UIs rendered in the webview.

## Plugin Directory Structure

```
plugins/
  your-plugin/
    plugin.json       # Manifest file (required)
    plugin.wasm       # Compiled WASM binary (required)
    ui/
      index.html      # Plugin UI (optional)
      ...
```

## Manifest Format (`plugin.json`)

```json
{
  "name": "My Plugin",
  "author": "Your Name",
  "version": "0.1.0",
  "wasm": "plugin.wasm",
  "type": "pre-fx",
  "parameters": [
    {
      "name": "gain",
      "type": "f32",
      "default": 1.0,
      "min": 0.0,
      "max": 2.0
    }
  ],
  "gui": "ui"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Human-readable plugin name |
| `author` | String | No | Plugin author |
| `version` | String | No | Semantic version string |
| `wasm` | String | Yes | Path to WASM binary (relative to plugin dir) |
| `type` | String | Yes | `"pre-fx"` or `"post-fx"` |
| `parameters` | Array | No | Configurable parameters |
| `gui` | String | No | Path to UI directory (relative to plugin dir) |

## WIT Interface

Plugins must expose the following WIT interface:

```wit
interface audio-plugin {
    init(sample-rate: u32, channels: u32) -> result;
    process(input: list<float32>, output: list<float32>) -> result<list<float32>>;
    reset() -> result;
}
```

### `init(sample-rate, channels)`

Called when the plugin is first loaded into the audio graph. Receives the current sample rate and channel count. Perform any one-time initialization here.

### `process(input, output)`

Called with a block of interleaved f32 audio samples. The `input` buffer contains incoming samples, and `output` should be filled with processed samples. Return the processed output buffer.

Block size defaults to 512 samples (configurable).

### `reset()`

Called when the graph is reset or playback restarts. Clear any internal state (delays, filters, etc.).

## UI Development

Plugin UIs are HTML/JS files loaded in a sandboxed iframe. They communicate with the Rust backend via `postMessage`:

```javascript
// Send parameter change to host
window.parent.postMessage({
  type: "param_change",
  name: "gain",
  value: 0.75
}, "*");
```

The iframe has `allow-scripts allow-same-origin` sandbox attributes.

## Building Plugins

Plugins can be written in Rust, C, Zig, or any language that targets WASM. For Rust:

```rust
// Cargo.toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# No dependencies needed - expose functions directly

// src/lib.rs
#[no_mangle]
pub extern "C" fn init(sample_rate: u32, channels: u32) -> i32 {
    0 // return 0 on success
}

#[no_mangle]
pub extern "C" fn process(input_ptr: *const f32, input_len: usize, output_ptr: *mut f32, output_len: usize) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn reset() -> i32 {
    0
}
```

Build with: `cargo build --target wasm32-unknown-unknown --release`
