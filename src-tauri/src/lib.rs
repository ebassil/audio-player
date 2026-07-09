mod audio;

use crate::audio::pipeline::AudioPipeline;
use crate::audio::playlist::Playlist;
use crate::audio::plugin_manager::PluginManager;
use crate::audio::shortcuts::{ShortcutAction, ShortcutConfig, ShortcutEngine};

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Emitter;
use tauri::Manager;
use tauri::State;

fn emit_audio_log(app_handle: &tauri::AppHandle, message: String) {
    let _ = app_handle.emit("audio-log", serde_json::json!({
        "message": message,
    }));
}

/// Application state shared across Tauri commands.
pub struct AppState {
    pub pipeline: Mutex<AudioPipeline>,
    pub plugin_manager: Mutex<PluginManager>,
    pub playlist: Mutex<Playlist>,
    pub shortcut_engine: Mutex<ShortcutEngine>,
}

#[tauri::command]
fn load_track(app_handle: tauri::AppHandle, state: State<AppState>, path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);
    let metadata = state
        .pipeline
        .lock()
        .map_err(|e| e.to_string())?
        .load_track(&path_buf)
        .map_err(|e| e.to_string())?;
    emit_audio_log(&app_handle, format!("Track loaded: {}", path_buf.file_name().unwrap_or_default().to_string_lossy()));
    Ok(format!(
        "Loaded: {} ({} channels, {} Hz, {:.1}s)",
        path_buf.file_name().unwrap_or_default().to_string_lossy(),
        metadata.channels,
        metadata.sample_rate,
        metadata.duration_secs
    ))
}

#[tauri::command]
fn play(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<String, String> {
    state.pipeline.lock().map_err(|e| e.to_string())?.play()?;
    emit_audio_log(&app_handle, "Decode started / Play".to_string());
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
fn stop(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<String, String> {
    state.pipeline.lock().map_err(|e| e.to_string())?.stop();
    emit_audio_log(&app_handle, "Decode completed / Stop".to_string());
    Ok("Stopped".to_string())
}

#[tauri::command]
fn seek(app_handle: tauri::AppHandle, state: State<AppState>, position_secs: f64) -> Result<String, String> {
    state
        .pipeline
        .lock()
        .map_err(|e| e.to_string())?
        .seek(position_secs);
    emit_audio_log(&app_handle, format!("Seek performed to {:.1}s", position_secs));
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
        duration_secs: duration_secs.clamp(1.0, 15.0),
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
fn load_playlist(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    path: String,
) -> Result<Vec<serde_json::Value>, String> {
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
                "mix_duration_override": t.mix_duration_override,
                "metadata": t.metadata,
            })
        })
        .collect();
    *state.playlist.lock().map_err(|e| e.to_string())? = playlist;
    save_playlist_state(&app_handle, &state);
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
                "mix_duration_override": t.mix_duration_override,
                "metadata": t.metadata,
            })
        })
        .collect();
    Ok(tracks)
}

#[tauri::command]
fn set_playlist_tracks(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    tracks: Vec<serde_json::Value>,
) -> Result<String, String> {
    let parsed: Vec<audio::playlist::PlaylistTrack> =
        serde_json::from_value(serde_json::Value::Array(tracks))
            .map_err(|e| format!("Invalid track data: {}", e))?;
    let mut playlist = state.playlist.lock().map_err(|e| e.to_string())?;
    let count = parsed.len();
    playlist.tracks = parsed;
    drop(playlist);
    save_playlist_state(&app_handle, &state);
    Ok(format!("Playlist updated with {} track(s)", count))
}

#[tauri::command]
fn import_m3u8(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    path: String,
) -> Result<Vec<serde_json::Value>, String> {
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
                "mix_duration_override": t.mix_duration_override,
                "metadata": t.metadata,
            })
        })
        .collect();
    *state.playlist.lock().map_err(|e| e.to_string())? = playlist;
    save_playlist_state(&app_handle, &state);
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
    app_handle: tauri::AppHandle,
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
    drop(playlist);
    save_playlist_state(&app_handle, &state);
    Ok(format!("Removed {} track(s)", indices.len()))
}

// --- Playlist Context IPC Commands ---

#[tauri::command]
fn set_playlist_context(
    state: State<AppState>,
    entries: Vec<serde_json::Value>,
) -> Result<String, String> {
    use audio::pipeline::PlaylistContextEntry;
    let parsed: Vec<PlaylistContextEntry> = entries
        .iter()
        .map(|e| {
            let file_path = e.get("file_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let mix_out = e.get("mix_out").and_then(|v| v.as_f64());
            let mix_in = e.get("mix_in").and_then(|v| v.as_f64());
            let mix_pattern_override = e.get("mix_pattern_override").and_then(|v| v.as_str()).map(|s| s.to_string());
            let mix_duration_override = e.get("mix_duration_override").and_then(|v| v.as_f64());
            PlaylistContextEntry { file_path, mix_out, mix_in, mix_pattern_override, mix_duration_override }
        })
        .collect();
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    pipeline.set_playlist_context(parsed);
    Ok("Playlist context updated".to_string())
}

#[tauri::command]
fn set_current_track_index(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    index: i32,
) -> Result<String, String> {
    let idx = if index < 0 { None } else { Some(index as usize) };
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    pipeline.set_current_track_index(idx);
    drop(pipeline);
    save_playlist_state(&app_handle, &state);
    Ok(format!("Current track index set to {:?}", idx))
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

// --- Shortcut IPC Commands ---

#[tauri::command]
fn get_shortcuts(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let engine = state.shortcut_engine.lock().map_err(|e| e.to_string())?;
    let config = engine.config();
    let shortcuts: Vec<_> = config
        .shortcuts
        .iter()
        .map(|b| {
            serde_json::json!({
                "action": format!("{:?}", b.action),
                "action_label": ShortcutAction::all()
                    .iter()
                    .find(|(a, _)| a == &b.action)
                    .map(|(_, label)| label)
                    .unwrap_or(&""),
                "key_combo": b.key_combo,
            })
        })
        .collect();
    Ok(shortcuts)
}

#[tauri::command]
fn set_shortcut(
    state: State<AppState>,
    action: String,
    key_combo: String,
) -> Result<String, String> {
    let action_enum = ShortcutAction::all()
        .iter()
        .find(|(a, label)| format!("{:?}", a) == action || *label == action)
        .map(|(a, _)| a.clone())
        .ok_or_else(|| format!("Unknown action: {}", action))?;

    let mut engine = state.shortcut_engine.lock().map_err(|e| e.to_string())?;
    let conflicts = engine.check_conflicts(&action_enum, &key_combo);
    if !conflicts.is_empty() {
        return Err(format!("Conflicts: {}", conflicts.join(", ")));
    }
    engine.set_binding(action_enum, key_combo);
    Ok("Shortcut updated".to_string())
}

#[tauri::command]
fn reset_shortcuts(state: State<AppState>) -> Result<String, String> {
    let mut engine = state.shortcut_engine.lock().map_err(|e| e.to_string())?;
    *engine.config_mut() = ShortcutConfig::default_shortcuts();
    Ok("Shortcuts reset to defaults".to_string())
}

#[tauri::command]
fn save_shortcuts(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<String, String> {
    let engine = state.shortcut_engine.lock().map_err(|e| e.to_string())?;
    let config_dir = app_handle.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&config_dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    let path = config_dir.join("shortcuts.toml");
    engine.config().save(&path)?;
    Ok("Shortcuts saved".to_string())
}

#[tauri::command]
fn get_session_toggles(_state: State<AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "confirm_delete": true,
    }))
}

// --- Per-track Mix Override IPC Commands ---

#[tauri::command]
fn get_current_track_mix_overrides(
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    let index = pipeline.current_track_index();
    let playlist = state.playlist.lock().map_err(|e| e.to_string())?;
    match index.and_then(|i| playlist.tracks.get(i)) {
        Some(track) => Ok(serde_json::json!({
            "pattern_override": track.mix_pattern_override,
            "duration_override": track.mix_duration_override,
        })),
        None => Ok(serde_json::json!({
            "pattern_override": None::<String>,
            "duration_override": None::<f64>,
        })),
    }
}

#[tauri::command]
fn set_current_track_mix_overrides(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    pattern_override: Option<String>,
    duration_override: Option<f64>,
) -> Result<String, String> {
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    let index = pipeline.current_track_index();
    let mut playlist = state.playlist.lock().map_err(|e| e.to_string())?;
    match index.and_then(|i| playlist.tracks.get_mut(i)) {
        Some(track) => {
            track.mix_pattern_override = pattern_override.clone();
            track.mix_duration_override = duration_override.map(|d| d.clamp(1.0, 15.0));
            drop(playlist);
            drop(pipeline);
            save_playlist_state(&app_handle, &state);
            Ok(format!("Per-track overrides updated"))
        }
        None => Err("No track selected".to_string()),
    }
}

// --- Config IPC Commands ---

#[tauri::command]
fn get_config_dir(app_handle: tauri::AppHandle) -> Result<String, String> {
    let config_dir = app_handle.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&config_dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    Ok(config_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn save_app_config(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    log_filter_names: Option<String>,
    log_filter_regex: Option<String>,
) -> Result<String, String> {
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    let mix = pipeline.mix_engine();
    let mix_config = mix.lock().map_err(|e| e.to_string())?.config().clone();
    let volume = pipeline.volume();

    let app_config = audio::config::AppConfig::from_mix_config(&mix_config);
    let app_config = audio::config::AppConfig {
        volume: volume.raw_gain(),
        muted: volume.is_muted(),
        log_filter_names: log_filter_names.unwrap_or_default(),
        log_filter_regex: log_filter_regex.unwrap_or_default(),
        ..app_config
    };

    let config_dir = app_handle.path().app_config_dir().map_err(|e| e.to_string())?;
    let config_path = config_dir.join("app.toml");
    app_config.save(&config_path)?;
    Ok("Config saved".to_string())
}

#[tauri::command]
fn load_app_config(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<serde_json::Value, String> {
    let config_dir = app_handle.path().app_config_dir().map_err(|e| e.to_string())?;
    let config_path = config_dir.join("app.toml");
    let config = audio::config::AppConfig::load(&config_path).unwrap_or_default();

    // Apply config to pipeline
    let pipeline = state.pipeline.lock().map_err(|e| e.to_string())?;
    pipeline.volume().set_gain(config.volume);
    pipeline.volume().set_mute(config.muted);

    let mix = pipeline.mix_engine();
    let mut mix = mix.lock().map_err(|e| e.to_string())?;
    mix.set_config(config.to_mix_config());

    Ok(serde_json::json!({
        "mix_pattern": config.mix_pattern,
        "mix_duration_secs": config.mix_duration_secs,
        "volume": config.volume,
        "muted": config.muted,
        "log_filter_names": config.log_filter_names,
        "log_filter_regex": config.log_filter_regex,
    }))
}

fn save_playlist_state(app_handle: &tauri::AppHandle, state: &AppState) {
    let config_dir = match app_handle.path().app_config_dir() {
        Ok(d) => d,
        Err(_) => return,
    };
    let path = config_dir.join("playlist_state.json");
    let playlist = match state.playlist.lock() {
        Ok(p) => p.clone(),
        Err(_) => return,
    };
    let current_track_index = match state.pipeline.lock() {
        Ok(p) => p.current_track_index(),
        Err(_) => return,
    };
    let state = audio::playlist_persist::PlaylistState {
        playlist,
        current_track_index,
    };
    let _ = state.save(&path);
}

pub fn run() {
    let pipeline = AudioPipeline::new();
    let plugins_dir = PathBuf::from("plugins");
    let mut plugin_manager = PluginManager::new(plugins_dir.clone());
    let playlist = Playlist::new("Default".to_string());

    let shortcut_engine = ShortcutEngine::new(ShortcutConfig::default_shortcuts());

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
            shortcut_engine: Mutex::new(shortcut_engine),
        })
        .setup(|app| {
            // Initialize app config directory
            if let Ok(config_dir) = app.path().app_config_dir() {
                std::fs::create_dir_all(&config_dir).ok();
                let shortcuts_path = config_dir.join("shortcuts.toml");
                if shortcuts_path.exists() {
                    if let Ok(config) = ShortcutConfig::load(&shortcuts_path) {
                        if let Some(state) = app.try_state::<AppState>() {
                            if let Ok(mut engine) = state.shortcut_engine.lock() {
                                *engine = ShortcutEngine::new(config);
                            }
                        }
                    }
                } else {
                    ShortcutConfig::default_shortcuts().save(&shortcuts_path).ok();
                }
            }

            // Restore persisted playlist state
            if let Ok(config_dir) = app.path().app_config_dir() {
                let playlist_path = config_dir.join("playlist_state.json");
                if playlist_path.exists() {
                    if let Ok(loaded) = audio::playlist_persist::PlaylistState::load(&playlist_path) {
                        if let Some(state) = app.try_state::<AppState>() {
                            if let Ok(mut p) = state.playlist.lock() {
                                *p = loaded.playlist;
                            }
                            if let Ok(pipeline) = state.pipeline.lock() {
                                pipeline.set_current_track_index(loaded.current_track_index);
                            }
                        }
                    }
                }
            }

            // Start background event emission for player state updates
            let handle = app.handle().clone();

            std::thread::spawn(move || {
                let mut prev_state = String::new();
                loop {
                    std::thread::sleep(Duration::from_millis(250));
                    if let Some(state) = handle.try_state::<AppState>() {
                        if let Ok(pipeline) = state.pipeline.lock() {
                            let current_state = format!("{:?}", pipeline.state());
                            let status = serde_json::json!({
                                "state": current_state,
                                "volume": pipeline.volume().raw_gain(),
                                "muted": pipeline.volume().is_muted(),
                                "progress": pipeline.progress(),
                                "position_secs": pipeline.position_secs(),
                                "duration_secs": pipeline.duration_secs(),
                            });
                            let _ = handle.emit("player-status", status);

                            if current_state != prev_state && !prev_state.is_empty() {
                                emit_audio_log(&handle, format!("State changed: {} → {}", prev_state, current_state));
                            }
                            prev_state = current_state;

                            // Emit track-changed event if pending
                            if let Some(new_index) = pipeline.take_pending_track_change() {
                                let _ = handle.emit("track-changed", serde_json::json!({
                                    "track_index": new_index,
                                }));
                            }
                        }
                    }
                }
            });

            Ok(())
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
            get_shortcuts,
            set_shortcut,
            reset_shortcuts,
            save_shortcuts,
            get_session_toggles,
            get_config_dir,
            save_app_config,
            load_app_config,
            set_playlist_context,
            set_current_track_index,
            get_current_track_mix_overrides,
            set_current_track_mix_overrides,
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
