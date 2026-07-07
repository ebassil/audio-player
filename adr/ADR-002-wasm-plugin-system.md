## ADR-002: WASM Plugin System via wasmtime

* **Status:** Accepted
* **Date:** 2026-07-07
* **Author:** Architect

### Context

The audio player must support third-party DSP plugins (limiters, equalizers, amplifiers, etc.) that can be loaded at runtime without recompiling the application. The plugins need to process decoded PCM audio buffers in real time and expose configurable parameters to the user via custom UI components.

Three approaches were considered:

**Option A — Native Dynamic Libraries (.dylib/.so):** Plugins compiled as shared libraries loaded at runtime via `libloading`. Fastest performance but platform-specific, no sandboxing, and security risk (full process access).

**Option B — Lua Scripting:** Embed a Lua runtime, define a DSP API. Simple to write but poor performance for sample-by-sample DSP — LuaJIT helps but still slower than native for heavy processing.

**Option C — WASM via wasmtime:** Plugins compiled to WebAssembly components with a WIT interface. Portable across platforms, sandboxed by default (no filesystem or network access), and wasmtime provides near-native execution speed for numerical workloads.

Option C was chosen. The combination of portability, security sandboxing, and performance meets all requirements. The WIT interface provides a versioned contract that plugin authors can target from Rust, C, or Zig.

### Decision

We will define a WIT interface for audio plugins with `init(sample_rate, channels)`, `process(input, output)`, and `reset()` functions. Plugins will be compiled to WASM components and discovered from a `plugins/` directory via `plugin.json` manifests. The Rust backend will load them using `wasmtime`, wrap each in a fundsp adapter node that bridges the pull-model graph with push-model WASM calls, and host plugin UIs as sandboxed HTML/JS loaded in the webview.

### Visual Architecture

```mermaid
graph TD
    subgraph Plugin Package
        A[plugin.json] --> B[Manifest: name, author,<br/>version, type, params]
        C[plugin.wasm] --> D[Compiled WASM<br/>component with WIT]
        E[ui/index.html] --> F[Plugin UI<br/>HTML/JS/CSS]
    end

    subgraph Rust Backend
        G[Plugin Scanner] -->|Scan plugins/| H[Manifest Parser]
        H --> I[wasmtime Host]
        I --> J[Instantiate WASM]
        J -->|init()| K[Plugin Instance]
        K -->|process()| L[fundsp Adapter Node]
        L --> M[Audio Graph]
        N[IPC Bridge] <-->|param_changed| K
    end

    subgraph Webview
        O[Plugin Rack UI] -->|Select plugin| P[Sandboxed Iframe]
        P -->|Load| F
        P -->|param_change| N
    end

    M --> Q[Audio Output]
```

### Consequences

**Positive (Benefits):**
- Plugins are fully sandboxed — no filesystem, network, or process access via wasmtime.
- Plugin authors can use Rust, C, or Zig — any language that compiles to WASM.
- No recompilation needed to add new plugins.
- WASM provides deterministic, portable binaries across macOS, Windows, and Linux.

**Negative (Risks/Trade-offs):**
- WASM-to-native call overhead for every audio block — measurable but acceptable for DSP workloads with block sizes of 256–1024 samples.
- The fundsp pull-model to WASM push-model adapter adds complexity and must handle buffering correctly.
- Plugin debugging is harder — no native debugger attachment to WASM modules.
- Plugin binary size and load time must be considered for complex DSP (e.g., convolution reverbs with large IRs).

**Neutral/Mitigations:**
- Define block size as a configurable parameter (default 512 samples) to balance latency and throughput.
- Provide a plugin SDK and example plugin with full build toolchain documentation.
- Consider an "allowlist" of native plugin paths for performance-critical DSP in a future iteration.
