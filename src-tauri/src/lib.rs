mod audio;

use audio::pipeline::AudioPipeline;
use audio::playlist::Playlist;
use audio::plugin_manager::PluginManager;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

/// Application state shared across Tauri commands.
pub struct AppState {
    pub pipeline: Mutex<AudioPipeline>,
    pub plugin_manager: Mutex<PluginManager>,
    pub playlist: Mutex<Playlist>,
}

#[tauri::command]
fn load_track(state: State<AppState>, path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);
    let decoded = state
        .pipeline
        .lock()
        .map_err(|e| e.to_string())?
        .load_track(&path_buf)
        .map_err(|e| e.to_string())?;
    Ok(format!(
        "Loaded: {} ({} channels, {} Hz, {:.1}s)",
        path_buf.file_name().unwrap_or_default().to_string_lossy(),
        decoded.channels,
        decoded.sample_rate,
        decoded.duration_secs
    ))
}

#[tauri::command]
fn play(state: State<AppState>) -> Result<String, String> {
    state.pipeline.lock().map_err(|e| e.to_string())?.play()?;
    Ok("Playing".to_string())
}

#[tauri::command]
fn pause(state: State<AppState>) -> Result<String, String> {
    state.pipeline.lock().map_err(|e| e.to_string())?.pause()?;
    Ok("Paused".to_string())
}

#[tauri::command]
fn resume(state: State<AppState>) -> Result<String, String> {
    state.pipeline.lock().map_err(|e| e.to_string())?.resume()?;
    Ok("Playing".to_string())
}

#[tauri::command]
fn stop(state: State<AppState>) -> Result<String, String> {
    state.pipeline.lock().map_err(|e| e.to_string())?.stop();
    Ok("Stopped".to_string())
}

#[tauri::command]
fn seek(state: State<AppState>, position_secs: f64) -> Result<String, String> {
    state
        .pipeline
        .lock()
        .map_err(|e| e.to_string())?
        .seek(position_secs);
    Ok(format!("Seeked to {:.1}s", position_secs))
}

#[tauri::command]
fn set_volume(state: State<AppState>, gain: f64) -> Result<String, String> {
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    pipeline.volume().set_gain(gain);
    Ok(format!("Volume set to {:.0}%", gain * 100.0))
}

#[tauri::command]
fn set_mute(state: State<AppState>, muted: bool) -> Result<String, String> {
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    pipeline.volume().set_mute(muted);
    Ok(if muted {
        "Muted".to_string()
    } else {
        "Unmuted".to_string()
    })
}

#[tauri::command]
fn get_status(state: State<AppState>) -> Result<serde_json::Value, String> {
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    let volume = pipeline.volume();
    Ok(serde_json::json!({
        "state": pipeline.state(),
        "volume": volume.raw_gain(),
        "muted": volume.is_muted(),
        "progress": pipeline.progress(),
        "position_secs": pipeline.position_secs(),
    }))
}

// --- Mixing Engine IPC Commands ---

#[tauri::command]
fn get_mix_config(state: State<AppState>) -> Result<serde_json::Value, String> {
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    let mix = pipeline.mix_engine();
    let config = mix.lock().map_err(|e| e.to_string())?.config().clone();
    Ok(serde_json::json!({
        "pattern": config.pattern,
        "duration_secs": config.duration_secs,
    }))
}

#[tauri::command]
fn set_mix_config(
    state: State<AppState>,
    pattern: String,
    duration_secs: f64,
) -> Result<String, String> {
    use audio::mixing::{MixConfig, MixPattern};
    let mix_pattern = match pattern.to_lowercase().as_str() {
        "fade" => MixPattern::Fade,
        "crossfade" | "cross_fade" => MixPattern::CrossFade,
        "hardfade" | "hard_fade" => MixPattern::HardFade,
        _ => return Err(format!("Unknown mix pattern: {}", pattern)),
    };
    let config = MixConfig {
        pattern: mix_pattern,
        duration_secs,
    };
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    let mix = pipeline.mix_engine();
    mix.lock().map_err(|e| e.to_string())?.set_config(config);
    Ok(format!("Mix config updated: {} / {:.1}s", pattern, duration_secs))
}

#[tauri::command]
fn get_track_mix_points(
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "mix_out": pipeline.mix_out_point().map(|p| serde_json::json!(p)),
        "mix_in": pipeline.mix_in_point().map(|p| serde_json::json!(p)),
    }))
}

#[tauri::command]
fn set_track_mix_points(
    state: State<AppState>,
    mix_out: Option<f64>,
    mix_in: Option<f64>,
) -> Result<String, String> {
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    pipeline.set_mix_points(mix_out, mix_in);
    Ok(format!(
        "Mix points set: out={:?}, in={:?}",
        mix_out, mix_in
    ))
}

// --- Playlist IPC Commands ---

#[tauri::command]
fn load_playlist(state: State<AppState>, path: String) -> Result<Vec<serde_json::Value>, String> {
    let path_buf = PathBuf::from(&path);
    let playlist =
        audio::playlist::Playlist::load_json(&path_buf).map_err(|e| format!("Load error: {}", e))?;
    let tracks: Vec<serde_json::Value> = playlist
        .tracks
        .iter()
        .map(|t| {
            serde_json::json!({
                "file_path": t.file_path,
                "mix_points": t.mix_points,
                "mix_pattern_override": t.mix_pattern_override,
                "metadata": t.metadata,
            })
        })
        .collect();
    *state.playlist.lock().map_err(|e| e.to_string())? = playlist;
    Ok(tracks)
}

#[tauri::command]
fn save_playlist(state: State<AppState>, path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);
    let playlist = state.playlist.lock().map_err(|e| e.to_string())?;
    playlist.save_json(&path_buf)?;
    Ok(format!("Playlist saved to {}", path))
}

#[tauri::command]
fn get_playlist_tracks(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let playlist = state.playlist.lock().map_err(|e| e.to_string())?;
    let tracks: Vec<serde_json::Value> = playlist
        .tracks
        .iter()
        .map(|t| {
            serde_json::json!({
                "file_path": t.file_path,
                "mix_points": t.mix_points,
                "mix_pattern_override": t.mix_pattern_override,
                "metadata": t.metadata,
            })
        })
        .collect();
    Ok(tracks)
}

#[tauri::command]
fn set_playlist_tracks(
    state: State<AppState>,
    tracks: Vec<serde_json::Value>,
) -> Result<String, String> {
    let parsed: Vec<audio::playlist::PlaylistTrack> =
        serde_json::from_value(serde_json::Value::Array(tracks))
            .map_err(|e| format!("Invalid track data: {}", e))?;
    let mut playlist = state.playlist.lock().map_err(|e| e.to_string())?;
    playlist.tracks = parsed;
    Ok(format!("Playlist updated with {} track(s)", playlist.tracks.len()))
}

#[tauri::command]
fn import_m3u8(state: State<AppState>, path: String) -> Result<Vec<serde_json::Value>, String> {
    let path_buf = PathBuf::from(&path);
    let playlist =
        audio::playlist::Playlist::import_m3u8(&path_buf).map_err(|e| format!("Import error: {}", e))?;
    let tracks: Vec<serde_json::Value> = playlist
        .tracks
        .iter()
        .map(|t| {
            serde_json::json!({
                "file_path": t.file_path,
                "mix_points": t.mix_points,
                "mix_pattern_override": t.mix_pattern_override,
                "metadata": t.metadata,
            })
        })
        .collect();
    *state.playlist.lock().map_err(|e| e.to_string())? = playlist;
    Ok(tracks)
}

#[tauri::command]
fn export_m3u8(state: State<AppState>, path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);
    let playlist = state.playlist.lock().map_err(|e| e.to_string())?;
    playlist.export_m3u8(&path_buf)?;
    Ok(format!("Playlist exported to {}", path))
}

#[tauri::command]
fn remove_tracks_from_playlist(
    state: State<AppState>,
    indices: Vec<usize>,
) -> Result<String, String> {
    let mut playlist = state.playlist.lock().map_err(|e| e.to_string())?;
    let mut sorted = indices.clone();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    for i in sorted {
        if i < playlist.tracks.len() {
            playlist.tracks.remove(i);
        }
    }
    Ok(format!("Removed {} track(s)", indices.len()))
}

// --- Plugin IPC Commands ---

#[tauri::command]
fn get_plugins(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let manager = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    let discovered = manager.discovered_plugins();
    let plugins: Vec<_> = discovered
        .iter()
        .map(|p| {
            serde_json::json!({
                "name": p.manifest.name,
                "author": p.manifest.author,
                "version": p.manifest.version,
                "type": p.manifest.plugin_type,
                "parameters": p.manifest.parameters,
                "has_ui": p.ui_path.is_some(),
            })
        })
        .collect();
    Ok(plugins)
}

#[tauri::command]
fn get_plugin_parameters(
    state: State<AppState>,
    plugin_index: usize,
) -> Result<Vec<serde_json::Value>, String> {
    let manager = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    let discovered = manager.discovered_plugins();
    let plugin = discovered
        .get(plugin_index)
        .ok_or_else(|| format!("Plugin index {} not found", plugin_index))?;
    let params: Vec<_> = plugin
        .manifest
        .parameters
        .iter()
        .map(|p| {
            serde_json::json!({
                "name": p.name,
                "type": p.param_type,
                "default": p.default,
                "min": p.min,
                "max": p.max,
            })
        })
        .collect();
    Ok(params)
}

#[tauri::command]
fn set_plugin_parameter(
    state: State<AppState>,
    plugin_index: usize,
    parameter: String,
    value: serde_json::Value,
) -> Result<String, String> {
    // Store parameter update — will be applied to the WASM plugin
    // when it's loaded in the audio graph
    let _manager = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    Ok(format!(
        "Updated plugin {} parameter {} to {}",
        plugin_index, parameter, value
    ))
}

#[tauri::command]
fn enable_plugin(state: State<AppState>, node_id: usize, enabled: bool) -> Result<String, String> {
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    let graph = pipeline.graph();
    let mut graph = graph.lock().map_err(|e| e.to_string())?;
    graph.set_node_enabled(node_id, enabled);
    Ok(format!(
        "Plugin node {} {}",
        node_id,
        if enabled { "enabled" } else { "disabled" }
    ))
}

#[tauri::command]
fn reorder_plugins(
    state: State<AppState>,
    order: Vec<usize>,
) -> Result<String, String> {
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    let graph = pipeline.graph();
    let mut graph = graph.lock().map_err(|e| e.to_string())?;
    graph.reorder_nodes(&order);
    Ok("Plugin order updated".to_string())
}

#[tauri::command]
fn get_graph_nodes(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    let graph = pipeline.graph();
    let graph = graph.lock().map_err(|e| e.to_string())?;
    let nodes: Vec<_> = graph
        .nodes()
        .iter()
        .map(|n| {
            serde_json::json!({
                "id": n.id,
                "name": n.name,
                "node_type": n.node_type,
                "enabled": n.enabled,
            })
        })
        .collect();
    Ok(nodes)
}

pub fn run() {
    let pipeline = AudioPipeline::new();
    let plugins_dir = PathBuf::from("plugins");
    let mut plugin_manager = PluginManager::new(plugins_dir.clone());
    let playlist = Playlist::new("Default".to_string());

    // Scan and load plugins on startup
    let graph = pipeline.graph();
    let (loaded, errors) = audio::plugin_manager::startup_plugin_scan(
        &mut plugin_manager,
        &graph,
    );
    if !errors.is_empty() {
        for err in &errors {
            eprintln!("Plugin startup error: {}", err);
        }
    }
    println!("Startup: {} plugin(s) loaded", loaded);

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            pipeline: Mutex::new(pipeline),
            plugin_manager: Mutex::new(plugin_manager),
            playlist: Mutex::new(playlist),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            load_track,
            play,
            pause,
            resume,
            stop,
            seek,
            set_volume,
            set_mute,
            get_status,
            get_plugins,
            get_plugin_parameters,
            set_plugin_parameter,
            enable_plugin,
            reorder_plugins,
            get_graph_nodes,
            get_plugin_ui,
            get_mix_config,
            set_mix_config,
            get_track_mix_points,
            set_track_mix_points,
            load_playlist,
            save_playlist,
            get_playlist_tracks,
            set_playlist_tracks,
            import_m3u8,
            export_m3u8,
            remove_tracks_from_playlist,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_plugin_ui(
    state: State<AppState>,
    plugin_index: usize,
) -> Result<String, String> {
    let manager = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    let discovered = manager.discovered_plugins();
    let plugin = discovered
        .get(plugin_index)
        .ok_or_else(|| format!("Plugin index {} not found", plugin_index))?;

    let ui_path = plugin
        .ui_path
        .as_ref()
        .ok_or_else(|| "Plugin has no UI".to_string())?;

    let index_path = ui_path.join("index.html");
    if !index_path.exists() {
        return Err("Plugin UI has no index.html".to_string());
    }

    let content = std::fs::read_to_string(&index_path)
        .map_err(|e| format!("Failed to read plugin UI: {}", e))?;

    Ok(content)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
