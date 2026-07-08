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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_playlist() -> Playlist {
        Playlist {
            name: "Test".to_string(),
            tracks: vec![
                PlaylistTrack {
                    file_path: "/music/track1.mp3".to_string(),
                    mix_points: Some(MixPointOverride {
                        mix_out: Some(120.0),
                        mix_in: None,
                    }),
                    mix_pattern_override: None,
                    metadata: Some(TrackMetadata {
                        title: Some("Track 1".to_string()),
                        artist: Some("Artist".to_string()),
                        album: Some("Album".to_string()),
                        duration_secs: Some(240.0),
                    }),
                },
                PlaylistTrack {
                    file_path: "/music/track2.flac".to_string(),
                    mix_points: None,
                    mix_pattern_override: Some("crossfade".to_string()),
                    metadata: None,
                },
            ],
        }
    }

    #[test]
    fn test_playlist_json_roundtrip() {
        let playlist = create_test_playlist();
        let path = std::env::temp_dir().join("test_playlist.json");
        playlist.save_json(&path).unwrap();
        let loaded = Playlist::load_json(&path).unwrap();
        assert_eq!(loaded.name, "Test");
        assert_eq!(loaded.tracks.len(), 2);
        assert_eq!(loaded.tracks[0].file_path, "/music/track1.mp3");
        assert_eq!(loaded.tracks[0].mix_points.as_ref().unwrap().mix_out, Some(120.0));
        assert_eq!(
            loaded.tracks[1].mix_pattern_override,
            Some("crossfade".to_string())
        );
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_playlist_m3u8_roundtrip() {
        let playlist = create_test_playlist();
        let path = std::env::temp_dir().join("test_playlist.m3u8");
        playlist.export_m3u8(&path).unwrap();
        let loaded = Playlist::import_m3u8(&path).unwrap();
        assert_eq!(loaded.tracks.len(), 2);
        assert_eq!(loaded.tracks[0].file_path, "/music/track1.mp3");
        // M3U8 doesn't store mix points
        assert!(loaded.tracks[0].mix_points.is_none());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_playlist_new_is_empty() {
        let playlist = Playlist::new("Empty".to_string());
        assert!(playlist.tracks.is_empty());
        assert_eq!(playlist.name, "Empty");
    }

    #[test]
    fn test_m3u8_skips_headers_and_urls() {
        let path = std::env::temp_dir().join("test_import.m3u8");
        let content = "#EXTM3U\n#EXTINF:240,Track 1\nhttp://example.com/stream\n/local/file.mp3\n";
        std::fs::write(&path, content).unwrap();
        let playlist = Playlist::import_m3u8(&path).unwrap();
        assert_eq!(playlist.tracks.len(), 1);
        assert_eq!(playlist.tracks[0].file_path, "/local/file.mp3");
        std::fs::remove_file(path).ok();
    }
}
