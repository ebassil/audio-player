use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::audio::graph::{AudioGraph, NodeType};
use crate::audio::plugin_host::PluginHost;
use crate::audio::plugin_manifest::{scan_plugins_dir, DiscoveredPlugin};

/// Manages plugin lifecycle: discovery, loading, and graph integration.
pub struct PluginManager {
    /// The shared WASM plugin host.
    host: Arc<Mutex<PluginHost>>,
    /// Discovered plugin metadata (persists across loads).
    discovered: Vec<DiscoveredPlugin>,
    /// Plugins/ directory path.
    plugins_dir: PathBuf,
}

impl PluginManager {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            host: Arc::new(Mutex::new(PluginHost::new())),
            discovered: Vec::new(),
            plugins_dir,
        }
    }

    /// Scan the plugins directory and load all valid plugins.
    ///
    /// Returns the count of successfully loaded plugins.
    pub fn scan_and_load(&mut self) -> Result<usize, String> {
        let discovered = scan_plugins_dir(&self.plugins_dir).map_err(|e| e.to_string())?;
        let count = discovered.len();
        self.discovered = discovered;
        Ok(count)
    }

    /// Load a specific discovered plugin by index into the WASM host.
    pub fn load_plugin(&mut self, index: usize) -> Result<usize, String> {
        let plugin = self
            .discovered
            .get(index)
            .ok_or_else(|| format!("Plugin index {} out of range", index))?;

        let name = plugin.manifest.name.clone();
        let wasm_path = &plugin.wasm_path;

        let mut host = self.host.lock().map_err(|e| e.to_string())?;
        host.load_plugin(name, wasm_path)
    }

    /// Load all discovered plugins into the WASM host.
    ///
    /// Returns (successful_ids, errors).
    pub fn load_all_plugins(&mut self) -> (Vec<usize>, Vec<String>) {
        let mut ids = Vec::new();
        let mut errors = Vec::new();

        let indices: Vec<usize> = (0..self.discovered.len()).collect();
        for i in indices {
            match self.load_plugin(i) {
                Ok(id) => ids.push(id),
                Err(e) => errors.push(e),
            }
        }

        (ids, errors)
    }

    /// Initialize all loaded plugins with the given audio parameters.
    pub fn init_all_plugins(&mut self, sample_rate: u32, channels: u32) -> Vec<(usize, String)> {
        let mut host = self.host.lock().unwrap();
        host.init_all(sample_rate, channels)
    }

    /// Get a reference to the shared plugin host.
    pub fn host(&self) -> Arc<Mutex<PluginHost>> {
        Arc::clone(&self.host)
    }

    /// Get the discovered plugins metadata.
    pub fn discovered_plugins(&self) -> &[DiscoveredPlugin] {
        &self.discovered
    }
}

/// Perform startup plugin scan and load.
///
/// Called once during application initialization.
pub fn startup_plugin_scan(
    manager: &mut PluginManager,
    graph: &Arc<Mutex<AudioGraph>>,
) -> (usize, Vec<String>) {
    let count = match manager.scan_and_load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Plugin scan error: {}", e);
            return (0, vec![e]);
        }
    };

    if count == 0 {
        println!("No plugins found");
        return (0, vec![]);
    }

    println!("Discovered {} plugin(s), loading...", count);

    let (ids, errors) = manager.load_all_plugins();

    if ids.is_empty() && errors.is_empty() {
        return (0, errors);
    }

    // Register each loaded plugin as a node in the audio graph
    let configured = ids.len();
    for &id in &ids {
        let plugin_name = {
            let host = manager.host();
            let host_guard = host.lock().unwrap();
            host_guard
                .plugins()
                .iter()
                .find(|p| p.id == id)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| format!("plugin-{}", id))
        };

        let mut graph = graph.lock().unwrap();
        graph.add_node(plugin_name, NodeType::PreFx);
    }

    println!("Loaded {}/{} plugins into graph", configured, count);
    (configured, errors)
}
