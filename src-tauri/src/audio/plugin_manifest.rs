use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Plugin type: where in the audio chain the plugin sits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginType {
    PreFx,
    PostFx,
}

/// A configurable parameter exposed by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub default: serde_json::Value,
    pub min: Option<serde_json::Value>,
    pub max: Option<serde_json::Value>,
}

/// A plugin manifest (`plugin.json`) describing a WASM audio plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub author: Option<String>,
    pub version: Option<String>,
    /// Path to the compiled WASM binary, relative to the manifest directory.
    pub wasm: String,
    /// Plugin type (pre-fx or post-fx).
    #[serde(rename = "type")]
    pub plugin_type: PluginType,
    /// Optional list of configurable parameters.
    #[serde(default)]
    pub parameters: Vec<PluginParameter>,
    /// Optional path to the HTML/JS UI directory, relative to the manifest dir.
    pub gui: Option<String>,
}

/// A discovered plugin with its manifest and resolved paths.
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    pub manifest: PluginManifest,
    /// Absolute path to the plugin directory.
    pub plugin_dir: PathBuf,
    /// Absolute path to the WASM binary.
    pub wasm_path: PathBuf,
    /// Absolute path to the UI directory (if any).
    pub ui_path: Option<PathBuf>,
}

/// Errors that can occur when parsing plugin manifests.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error in {path}: {detail}")]
    ParseError { path: PathBuf, detail: String },
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("WASM binary not found: {0}")]
    WasmNotFound(PathBuf),
}

/// Parse a plugin manifest from a `plugin.json` file at the given path.
///
/// Returns the parsed manifest and resolves relative paths against the
/// directory containing the manifest file.
pub fn parse_manifest(manifest_path: &Path) -> Result<DiscoveredPlugin, ManifestError> {
    let content = fs::read_to_string(manifest_path)?;
    let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
        ManifestError::ParseError {
            path: manifest_path.to_path_buf(),
            detail: e.to_string(),
        }
    })?;

    let plugin_dir = manifest_path
        .parent()
        .ok_or_else(|| ManifestError::MissingField("plugin directory".to_string()))?
        .to_path_buf();

    let wasm_path = plugin_dir.join(&manifest.wasm);
    if !wasm_path.exists() {
        return Err(ManifestError::WasmNotFound(wasm_path));
    }

    let ui_path = manifest
        .gui
        .as_ref()
        .map(|gui| plugin_dir.join(gui))
        .filter(|p| p.exists());

    Ok(DiscoveredPlugin {
        manifest,
        plugin_dir,
        wasm_path,
        ui_path,
    })
}

/// Scan a directory for plugin manifests (`plugin.json` files in subdirectories).
///
/// Each subdirectory of `plugins_dir` is expected to contain a `plugin.json`
/// manifest file defining a plugin.
pub fn scan_plugins_dir(plugins_dir: &Path) -> Result<Vec<DiscoveredPlugin>, ManifestError> {
    if !plugins_dir.exists() {
        return Ok(Vec::new());
    }

    let mut plugins = Vec::new();

    for entry in fs::read_dir(plugins_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("plugin.json");
        if !manifest_path.exists() {
            continue;
        }

        match parse_manifest(&manifest_path) {
            Ok(plugin) => plugins.push(plugin),
            Err(e) => {
                eprintln!("Skipping plugin at {}: {}", path.display(), e);
            }
        }
    }

    Ok(plugins)
}
