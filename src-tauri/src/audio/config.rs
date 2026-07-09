use crate::audio::mixing::{MixConfig, MixPattern};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStateEntry {
    pub plugin_id: usize,
    pub name: String,
    pub enabled: bool,
    pub order: usize,
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mix_pattern: String,
    pub mix_duration_secs: f64,
    pub volume: f64,
    pub muted: bool,
    pub plugin_state: Vec<PluginStateEntry>,
    #[serde(default)]
    pub log_filter_names: String,
    #[serde(default)]
    pub log_filter_regex: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mix_pattern: "crossfade".to_string(),
            mix_duration_secs: 3.0,
            volume: 1.0,
            muted: false,
            plugin_state: Vec::new(),
            log_filter_names: String::new(),
            log_filter_regex: String::new(),
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("Invalid config: {}", e))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(path, content).map_err(|e| format!("Failed to write config: {}", e))
    }

    pub fn to_mix_config(&self) -> MixConfig {
        let pattern = match self.mix_pattern.to_lowercase().as_str() {
            "fade" => MixPattern::Fade,
            "crossfade" | "cross_fade" => MixPattern::CrossFade,
            "hardfade" | "hard_fade" => MixPattern::HardFade,
            _ => MixPattern::CrossFade,
        };
        MixConfig {
            pattern,
            duration_secs: self.mix_duration_secs.clamp(1.0, 15.0),
        }
    }

    pub fn from_mix_config(config: &MixConfig) -> Self {
        let mut cfg = Self::default();
        cfg.mix_pattern = match config.pattern {
            MixPattern::Fade => "fade".to_string(),
            MixPattern::CrossFade => "crossfade".to_string(),
            MixPattern::HardFade => "hardfade".to_string(),
        };
        cfg.mix_duration_secs = config.duration_secs;
        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::mixing::MixPattern;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.mix_pattern, "crossfade");
        assert_eq!(config.mix_duration_secs, 3.0);
        assert!((config.volume - 1.0).abs() < f64::EPSILON);
        assert_eq!(config.log_filter_names, "");
        assert_eq!(config.log_filter_regex, "");
    }

    #[test]
    fn test_config_save_load_roundtrip() {
        let config = AppConfig {
            mix_pattern: "fade".to_string(),
            mix_duration_secs: 5.0,
            volume: 0.75,
            muted: true,
            plugin_state: Vec::new(),
            log_filter_names: "player-status, audio-log".to_string(),
            log_filter_regex: "state=Stopped".to_string(),
        };
        let path = std::env::temp_dir().join("test_app_config.toml");
        config.save(&path).unwrap();
        let loaded = AppConfig::load(&path).unwrap();
        assert_eq!(loaded.mix_pattern, "fade");
        assert!((loaded.mix_duration_secs - 5.0).abs() < f64::EPSILON);
        assert!((loaded.volume - 0.75).abs() < f64::EPSILON);
        assert!(loaded.muted);
        assert_eq!(loaded.log_filter_names, "player-status, audio-log");
        assert_eq!(loaded.log_filter_regex, "state=Stopped");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_mix_config_conversion() {
        let app_config = AppConfig {
            mix_pattern: "hardfade".to_string(),
            mix_duration_secs: 2.0,
            volume: 1.0,
            muted: false,
            plugin_state: Vec::new(),
            log_filter_names: String::new(),
            log_filter_regex: String::new(),
        };
        let mix_config = app_config.to_mix_config();
        assert_eq!(mix_config.pattern, MixPattern::HardFade);
        assert!((mix_config.duration_secs - 2.0).abs() < f64::EPSILON);

        let back = AppConfig::from_mix_config(&mix_config);
        assert_eq!(back.mix_pattern, "hardfade");
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let path = std::env::temp_dir().join("nonexistent_config.toml");
        let config = AppConfig::load(&path).unwrap();
        assert_eq!(config.mix_pattern, "crossfade");
    }
}
