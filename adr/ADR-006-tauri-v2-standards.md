## ADR-006: Tauri v2 Standards

- **Status:** Accepted
- **Date:** 2026-07-07
- **Author:** Architect

### Context

The project uses Tauri v2 (the first v2 stable release). Tauri v2 introduces significant breaking changes from v1 — a new capability-based permission system, a restructured configuration schema, and a split Rust entry-point pattern. To ensure consistency, avoid regressions, and reduce cognitive overhead, we codify the Tauri v2 conventions that all implementation tasks must follow.

### Decision

We adopt the following standards for all Tauri v2 code in this project:

#### 1. Tooling & Core Commands

- Use the Tauri v2 CLI via `npm run tauri [command]` or `npx tauri [command]`.
- To add plugins, prefer the CLI tool to manage both Rust and JavaScript dependencies simultaneously: `npx tauri plugin add <plugin-name>`.

#### 2. Configuration Standards (tauri.conf.json)

- **Strict Schema compliance:** Do not place window properties (like `title`, `width`, `height`, `resizable`) directly under the `app` block — they must be inside `app.windows[]`.
- **Window Definitions:** Every window object must include a `label` field (unique identifier), `title`, `url`, `width`, and `height`.
- **Plugin Configurations:** All plugin configurations belong under the top-level `"plugins": {}` object, not under `"tauri"`.
- **Identifier:** Ensure a unique bundle identifier is set under `identifier` (e.g., `"com.audioplayer.app"`).

```json
{
  "productName": "Audio Player",
  "version": "0.1.0",
  "identifier": "com.audioplayer.app",
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Audio Player",
        "url": "/",
        "width": 1024,
        "height": 700
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all"
  },
  "plugins": {}
}
```

#### 3. V2 Permission & Capability System (Critical)

Tauri v2 uses a strict capability system instead of the old v1 allowlist.
- Permissions must be defined explicitly inside `src-tauri/capabilities/` (typically `default.json`).
- Every core feature or plugin API used by the frontend must be explicitly allowed here.
- Include the `$schema` reference for IDE support.

```json
{
  "$schema": "../gen/schemas/desktop-capability.json",
  "identifier": "default",
  "description": "Capabilities for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:event:default",
    "core:window:default",
    "global-shortcut:default",
    "dialog:default",
    "fs:default"
  ]
}
```

#### 4. Frontend API Imports (JavaScript / TypeScript)

- **Do not** import from `@tauri-apps/api/window`, `@tauri-apps/api/fs`, or `@tauri-apps/api/path` directly — these are legacy v1 paths.
- Core APIs are split:
  - Submodules are accessed via `@tauri-apps/api/core` (e.g., `invoke`).
  - Webview/window management uses `@tauri-apps/api/webviewWindow`.
- **Plugins:** Official plugins use scoped npm packages named `@tauri-apps/plugin-<name>`.
  - *Example:* Use `import { readTextFile } from '@tauri-apps/plugin-fs'`, not `@tauri-apps/api/fs`.

```typescript
import { invoke } from "@tauri-apps/api/core";
import { readTextFile } from "@tauri-apps/plugin-fs";
```

#### 5. Rust Backend (main.rs / lib.rs)

- Tauri v2 splits initialization out of `main.rs` into `lib.rs` using a `run()` function.
- Use `tauri::Builder::default().plugin(tauri_plugin_XXXX::init())` to register plugins.
- Commands must be standard Rust functions returning `Result` for robust error handling, registered via `.invoke_handler(tauri::generate_handler![...])`.
- The `main.rs` entry point calls into `lib.rs`:

```rust
// main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    app_lib::run()
}

// lib.rs
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // commands registered here
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Consequences

**Positive:**
- Consistent configuration reduces the chance of v1/v2 confusion during development.
- The capability system provides granular, auditable permission control.
- Separating `main.rs` / `lib.rs` aligns with Rust best practices and makes testing easier.

**Negative:**
- Requires all developers to be familiar with the v2 capability model (no v1-style allowlist).
- Plugin management requires the Tauri CLI tool rather than manual Cargo/npm edits.

### Compliance Checklist

When implementing any feature that touches Tauri:
- [ ] Does it need a new capability permission in `src-tauri/capabilities/default.json`?
- [ ] If adding a plugin, was `npx tauri plugin add <name>` used to manage both sides?
- [ ] Does `tauri.conf.json` follow the v2 schema (windows inside `app.windows`, plugins at top-level)?
- [ ] Are frontend imports using `@tauri-apps/plugin-*` (not `@tauri-apps/api/*`)?
