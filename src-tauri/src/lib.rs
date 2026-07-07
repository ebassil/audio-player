mod audio;

use audio::pipeline::AudioPipeline;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

/// Application state shared across Tauri commands.
pub struct AppState {
    pub pipeline: Mutex<AudioPipeline>,
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

pub fn run() {
    let pipeline = AudioPipeline::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            pipeline: Mutex::new(pipeline),
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
