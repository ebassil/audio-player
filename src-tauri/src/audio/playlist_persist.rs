use crate::audio::playlist::Playlist;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistState {
    pub playlist: Playlist,
    pub current_track_index: Option<usize>,
}

impl PlaylistState {
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize playlist state: {}", e))?;
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write playlist state: {}", e))?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read playlist state: {}", e))?;
        let state: PlaylistState = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid playlist state JSON: {}", e))?;
        Ok(state)
    }
}
