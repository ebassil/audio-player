use std::path::Path;

use wasmtime::component::{Component, Func, Instance, Linker, ResourceTable, Val};
use wasmtime::{Config, Engine, Store};

/// A loaded WASM audio plugin instance.
pub struct PluginInstance {
    /// Unique plugin ID.
    pub id: usize,
    /// Human-readable name.
    pub name: String,
    /// The wasmtime store (holds plugin state).
    store: Store<PluginContext>,
    /// The component instance.
    instance: Instance,
    /// The component's exported interface.
    iface: PluginInterface,
}

/// Context stored per-instance in the wasmtime Store.
struct PluginContext {
    table: ResourceTable,
}

/// Wraps the WIT-exported functions from the plugin component.
struct PluginInterface {
    init: Func,
    process: Func,
    reset: Func,
}

impl PluginInstance {
    /// Load a WASM plugin from a `.wasm` file path.
    pub fn load(id: usize, name: String, wasm_path: &Path) -> Result<Self, String> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config).map_err(|e| e.to_string())?;

        let component = Component::from_file(&engine, wasm_path).map_err(|e| e.to_string())?;

        let mut linker = Linker::new(&engine);
        linker.allow_shadowing(true);

        let ctx = PluginContext {
            table: ResourceTable::new(),
        };
        let mut store = Store::new(&engine, ctx);

        let instance = linker
            .instantiate(&mut store, &component)
            .map_err(|e| e.to_string())?;

        let init = instance
            .get_func(&mut store, "init")
            .ok_or_else(|| "Failed to resolve init".to_string())?;

        let process = instance
            .get_func(&mut store, "process")
            .ok_or_else(|| "Failed to resolve process".to_string())?;

        let reset = instance
            .get_func(&mut store, "reset")
            .ok_or_else(|| "Failed to resolve reset".to_string())?;

        let iface = PluginInterface {
            init,
            process,
            reset,
        };

        Ok(Self {
            id,
            name,
            store,
            instance,
            iface,
        })
    }

    /// Initialize the plugin with the given sample rate and channel count.
    pub fn init(&mut self, sample_rate: u32, channels: u32) -> Result<(), String> {
        let params = [Val::U32(sample_rate), Val::U32(channels)];
        let mut results = vec![Val::Bool(false)];
        self.iface
            .init
            .call(&mut self.store, &params, &mut results)
            .map_err(|e| format!("init failed: {}", e))?;

        match results.first() {
            Some(Val::Result(Ok(_))) => Ok(()),
            Some(Val::Result(Err(Some(msg)))) => {
                if let Val::String(s) = msg.as_ref() {
                    Err(format!("init returned error: {}", s))
                } else {
                    Err("init returned error".to_string())
                }
            }
            _ => Ok(()),
        }
    }

    /// Process a block of audio samples (interleaved f32).
    /// Returns the processed output buffer of the same length.
    pub fn process(&mut self, input: Vec<f32>, output: Vec<f32>) -> Result<Vec<f32>, String> {
        let input_vals: Vec<Val> = input.into_iter().map(Val::Float32).collect();
        let output_vals: Vec<Val> = output.into_iter().map(Val::Float32).collect();
        let params = [Val::List(input_vals), Val::List(output_vals)];
        let mut results = vec![Val::Bool(false)];
        self.iface
            .process
            .call(&mut self.store, &params, &mut results)
            .map_err(|e| format!("process failed: {}", e))?;

        match results.first() {
            Some(Val::Result(Ok(Some(val)))) => {
                if let Val::List(list) = val.as_ref() {
                    Ok(list
                        .iter()
                        .map(|v| match v {
                            Val::Float32(f) => *f,
                            _ => 0.0,
                        })
                        .collect())
                } else {
                    Err("process returned unexpected success value".to_string())
                }
            }
            Some(Val::Result(Err(Some(msg)))) => {
                if let Val::String(s) = msg.as_ref() {
                    Err(format!("process returned error: {}", s))
                } else {
                    Err("process returned error".to_string())
                }
            }
            _ => Err("process failed".to_string()),
        }
    }

    /// Reset the plugin's internal state.
    pub fn reset(&mut self) -> Result<(), String> {
        let params: [Val; 0] = [];
        let mut results = vec![Val::Bool(false)];
        self.iface
            .reset
            .call(&mut self.store, &params, &mut results)
            .map_err(|e| format!("reset failed: {}", e))?;

        match results.first() {
            Some(Val::Result(Ok(_))) => Ok(()),
            Some(Val::Result(Err(Some(msg)))) => {
                if let Val::String(s) = msg.as_ref() {
                    Err(format!("reset returned error: {}", s))
                } else {
                    Err("reset returned error".to_string())
                }
            }
            _ => Ok(()),
        }
    }
}

/// Manages all loaded WASM plugin instances.
pub struct PluginHost {
    plugins: Vec<PluginInstance>,
    next_id: usize,
}

impl PluginHost {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            next_id: 0,
        }
    }

    /// Load a plugin from a wasm file.
    pub fn load_plugin(&mut self, name: String, wasm_path: &Path) -> Result<usize, String> {
        let id = self.next_id;
        self.next_id += 1;
        let instance = PluginInstance::load(id, name, wasm_path)?;
        let pid = instance.id;
        self.plugins.push(instance);
        Ok(pid)
    }

    /// Unload a plugin by ID.
    pub fn unload_plugin(&mut self, id: usize) {
        self.plugins.retain(|p| p.id != id);
    }

    /// Initialize all plugins with the given audio parameters.
    pub fn init_all(&mut self, sample_rate: u32, channels: u32) -> Vec<(usize, String)> {
        let mut errors = Vec::new();
        for plugin in &mut self.plugins {
            if let Err(e) = plugin.init(sample_rate, channels) {
                errors.push((plugin.id, e));
            }
        }
        errors
    }

    /// Get a mutable reference to a plugin by ID.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut PluginInstance> {
        self.plugins.iter_mut().find(|p| p.id == id)
    }

    /// Get all loaded plugins.
    pub fn plugins(&self) -> &[PluginInstance] {
        &self.plugins
    }

    /// Number of loaded plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for PluginInstance {}
unsafe impl Send for PluginHost {}
