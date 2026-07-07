use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixPointOverride {
    pub mix_out: Option<f64>,
    pub mix_in: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_secs: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mix_points: Option<MixPointOverride>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mix_pattern_override: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TrackMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<PlaylistTrack>,
}

impl Playlist {
    pub fn new(name: String) -> Self {
        Playlist {
            name,
            tracks: Vec::new(),
        }
    }

    pub fn save_json(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize playlist: {}", e))?;
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write playlist file: {}", e))?;
        Ok(())
    }

    pub fn load_json(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read playlist: {}", e))?;
        let playlist: Playlist =
            serde_json::from_str(&content).map_err(|e| format!("Invalid playlist JSON: {}", e))?;
        Ok(playlist)
    }

    pub fn export_m3u8(&self, path: &Path) -> Result<(), String> {
        let mut content = String::from("#EXTM3U\n");
        for track in &self.tracks {
            if let Some(ref meta) = track.metadata {
                if let (Some(title), Some(duration)) = (&meta.title, meta.duration_secs) {
                    content.push_str(&format!(
                        "#EXTINF:{},{}\n",
                        duration.round() as u64,
                        title
                    ));
                }
            }
            content.push_str(&track.file_path);
            content.push('\n');
        }
        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write M3U8 file: {}", e))?;
        Ok(())
    }

    pub fn import_m3u8(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read M3U8: {}", e))?;
        let mut tracks = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with("http")
            {
                continue;
            }
            tracks.push(PlaylistTrack {
                file_path: trimmed.to_string(),
                mix_points: None,
                mix_pattern_override: None,
                metadata: None,
            });
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();
        Ok(Playlist { name, tracks })
    }
}
