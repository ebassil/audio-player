use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShortcutAction {
    PlayPause,
    NextTrack,
    PreviousTrack,
    Delete,
    DeletePlus,
    VolumeUp,
    VolumeDown,
    Mute,
    SeekForward,
    SeekBackward,
}

impl ShortcutAction {
    pub fn all() -> Vec<(ShortcutAction, &'static str)> {
        vec![
            (ShortcutAction::PlayPause, "Play/Pause"),
            (ShortcutAction::NextTrack, "Next Track"),
            (ShortcutAction::PreviousTrack, "Previous Track"),
            (ShortcutAction::Delete, "Delete from Playlist"),
            (ShortcutAction::DeletePlus, "Delete from Playlist + Disk"),
            (ShortcutAction::VolumeUp, "Volume Up"),
            (ShortcutAction::VolumeDown, "Volume Down"),
            (ShortcutAction::Mute, "Mute"),
            (ShortcutAction::SeekForward, "Seek Forward"),
            (ShortcutAction::SeekBackward, "Seek Backward"),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutBinding {
    pub action: ShortcutAction,
    pub key_combo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub shortcuts: Vec<ShortcutBinding>,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self::default_shortcuts()
    }
}

impl ShortcutConfig {
    pub fn default_shortcuts() -> Self {
        Self {
            shortcuts: vec![
                ShortcutBinding {
                    action: ShortcutAction::PlayPause,
                    key_combo: "Ctrl+Alt+Shift+P".to_string(),
                },
                ShortcutBinding {
                    action: ShortcutAction::NextTrack,
                    key_combo: "Ctrl+Alt+Shift+N".to_string(),
                },
                ShortcutBinding {
                    action: ShortcutAction::PreviousTrack,
                    key_combo: "Ctrl+Alt+Shift+B".to_string(),
                },
                ShortcutBinding {
                    action: ShortcutAction::Delete,
                    key_combo: "Ctrl+Alt+Shift+D".to_string(),
                },
                ShortcutBinding {
                    action: ShortcutAction::DeletePlus,
                    key_combo: "Ctrl+Alt+Shift+X".to_string(),
                },
                ShortcutBinding {
                    action: ShortcutAction::VolumeUp,
                    key_combo: "Ctrl+Alt+Shift+Up".to_string(),
                },
                ShortcutBinding {
                    action: ShortcutAction::VolumeDown,
                    key_combo: "Ctrl+Alt+Shift+Down".to_string(),
                },
                ShortcutBinding {
                    action: ShortcutAction::Mute,
                    key_combo: "Ctrl+Alt+Shift+M".to_string(),
                },
                ShortcutBinding {
                    action: ShortcutAction::SeekForward,
                    key_combo: "Ctrl+Alt+Shift+Right".to_string(),
                },
                ShortcutBinding {
                    action: ShortcutAction::SeekBackward,
                    key_combo: "Ctrl+Alt+Shift+Left".to_string(),
                },
            ],
        }
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default_shortcuts());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read shortcuts config: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("Invalid shortcuts config: {}", e))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize shortcuts: {}", e))?;
        std::fs::write(path, content).map_err(|e| format!("Failed to write shortcuts: {}", e))
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for binding in &self.shortcuts {
            if !seen.insert(binding.key_combo.clone()) {
                errors.push(format!(
                    "Duplicate shortcut key: {}",
                    binding.key_combo
                ));
            }
        }
        errors
    }
}

pub struct ShortcutEngine {
    config: ShortcutConfig,
    bindings: HashMap<String, ShortcutAction>,
}

impl ShortcutEngine {
    pub fn new(config: ShortcutConfig) -> Self {
        let bindings = config
            .shortcuts
            .iter()
            .map(|b| (b.key_combo.clone(), b.action.clone()))
            .collect();
        Self { config, bindings }
    }

    pub fn config(&self) -> &ShortcutConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut ShortcutConfig {
        &mut self.config
    }

    pub fn get_binding(&self, action: &ShortcutAction) -> Option<&String> {
        self.config
            .shortcuts
            .iter()
            .find(|b| b.action == *action)
            .map(|b| &b.key_combo)
    }

    pub fn set_binding(&mut self, action: ShortcutAction, key_combo: String) {
        if let Some(binding) = self
            .config
            .shortcuts
            .iter_mut()
            .find(|b| b.action == action)
        {
            binding.key_combo = key_combo;
        }
    }

    pub fn check_conflicts(&self, action: &ShortcutAction, key_combo: &str) -> Vec<String> {
        self.config
            .shortcuts
            .iter()
            .filter(|b| b.action != *action && b.key_combo == key_combo)
            .map(|b| format!("Conflicts with {:?}", b.action))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_shortcuts_count() {
        let config = ShortcutConfig::default_shortcuts();
        assert_eq!(config.shortcuts.len(), 10);
    }

    #[test]
    fn test_default_shortcuts_no_duplicates() {
        let config = ShortcutConfig::default_shortcuts();
        let errors = config.validate();
        assert!(errors.is_empty(), "Duplicate shortcuts: {:?}", errors);
    }

    #[test]
    fn test_engine_creates_bindings() {
        let config = ShortcutConfig::default_shortcuts();
        let engine = ShortcutEngine::new(config);
        assert_eq!(
            engine.get_binding(&ShortcutAction::PlayPause),
            Some(&"Ctrl+Alt+Shift+P".to_string())
        );
    }

    #[test]
    fn test_set_binding_updates() {
        let config = ShortcutConfig::default_shortcuts();
        let mut engine = ShortcutEngine::new(config);
        engine.set_binding(ShortcutAction::PlayPause, "Ctrl+Shift+Space".to_string());
        assert_eq!(
            engine.get_binding(&ShortcutAction::PlayPause),
            Some(&"Ctrl+Shift+Space".to_string())
        );
    }

    #[test]
    fn test_conflict_detection() {
        let config = ShortcutConfig::default_shortcuts();
        let engine = ShortcutEngine::new(config);
        let conflicts = engine.check_conflicts(&ShortcutAction::PlayPause, "Ctrl+Alt+Shift+N");
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn test_no_conflict_with_self() {
        let config = ShortcutConfig::default_shortcuts();
        let engine = ShortcutEngine::new(config);
        let conflicts = engine.check_conflicts(&ShortcutAction::PlayPause, "Ctrl+Alt+Shift+P");
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_validate_duplicates() {
        let mut config = ShortcutConfig::default_shortcuts();
        config.shortcuts.push(ShortcutBinding {
            action: ShortcutAction::PlayPause,
            key_combo: "Ctrl+Alt+Shift+N".to_string(),
        });
        let errors = config.validate();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_shortcut_save_load_roundtrip() {
        let config = ShortcutConfig::default_shortcuts();
        let path = std::env::temp_dir().join("test_shortcuts.toml");
        config.save(&path).unwrap();
        let loaded = ShortcutConfig::load(&path).unwrap();
        assert_eq!(config.shortcuts.len(), loaded.shortcuts.len());
        std::fs::remove_file(path).ok();
    }
}
