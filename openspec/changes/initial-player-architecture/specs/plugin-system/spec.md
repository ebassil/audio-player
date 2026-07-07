## ADDED Requirements

### Requirement: Plugin Manifest Format
Each plugin SHALL have a `plugin.json` manifest file with the following fields: name, author, version, path to the compiled WASM binary, plugin type (pre-fx or post-fx), and a list of configurable parameters with name, type, default value, min, and max.
The manifest MAY optionally specify a `gui` field pointing to an HTML/JS UI directory.

#### Scenario: Load valid plugin manifest
- **WHEN** the system scans the plugins directory
- **THEN** it SHALL parse each plugin.json and register the plugin if valid

#### Scenario: Invalid manifest
- **WHEN** a plugin.json is missing required fields
- **THEN** the system SHALL skip the plugin and log an error

### Requirement: WASM Plugin Interface (WIT)
The system SHALL define a WIT interface for audio plugins with at minimum: `init(sample-rate: u32, channels: u32) -> result`, `process(input: list<float32>, output: list<float32>) -> result`, and `reset() -> result`.
The system SHALL load plugin WASM binaries using wasmtime and instantiate them with the WIT bindings.
The system SHALL call `init` when a plugin is first loaded into the audio graph, passing the current sample rate and channel count.

#### Scenario: Load and initialize WASM plugin
- **WHEN** a WASM plugin is loaded into the audio graph
- **THEN** the system SHALL call init with the current sample rate and channel count

#### Scenario: Process audio through plugin
- **WHEN** audio is routed through a WASM plugin during playback
- **THEN** the system SHALL call process(input_buffer, output_buffer) and use the filled output buffer for the next graph stage

### Requirement: Plugin Host in fundsp (Adapter Node)
The system SHALL provide a fundsp adapter node that bridges the pull-based graph with the push-based WASM plugin interface.
The adapter node SHALL buffer incoming pull requests and dispatch them as block-based process calls (configurable block size, default 512 samples).
The adapter node SHALL handle latency introduced by block-based processing.

#### Scenario: Plugin in audio graph
- **WHEN** a WASM plugin is inserted into the audio graph
- **THEN** audio SHALL flow through the adapter node into the plugin and return processed audio

### Requirement: Plugin UI in Webview
Plugins MAY ship an HTML/JS UI directory (`ui/` folder) that the webview loads when the plugin is selected.
Plugin UI components SHALL communicate parameter changes to the Rust backend via `@tauri-apps/api/core`'s `invoke` (Tauri v2 pattern, per ADR-006).
The webview SHALL sandbox plugin UIs to prevent DOM access to the main application.

#### Scenario: Load plugin UI
- **WHEN** the user selects a plugin with a UI component
- **THEN** the webview SHALL load the plugin's UI into a sandboxed frame

#### Scenario: Plugin parameter change from UI
- **WHEN** the user adjusts a plugin parameter via its UI
- **THEN** the system SHALL update the parameter in the Rust plugin host and apply it in the audio graph

### Requirement: Plugin Discovery
The system SHALL scan a `plugins/` directory at the application root for plugin.json manifests on startup.
The system SHALL present discovered plugins in the UI for the user to enable/disable and reorder in the audio graph.
Plugin state (which plugins are enabled, their parameter values, their order) SHALL persist between sessions via config file.

#### Scenario: Discover plugins on startup
- **WHEN** the application starts
- **THEN** it SHALL scan plugins/ and register all valid plugins

#### Scenario: Enable/disable plugin
- **WHEN** the user toggles a plugin off in the UI
- **THEN** the plugin SHALL be bypassed in the audio graph
