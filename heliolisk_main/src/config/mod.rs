pub mod keymaps;
pub mod theme;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
pub use keymaps::{KeyChord, KeyRouter, KeyTrieNode, MatchResult};
pub use theme::{Theme, ThemePreset};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelioliskConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_leader")]
    pub leader: String,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u16,
    #[serde(default)]
    pub keymaps: HashMap<String, String>,
    #[serde(default = "default_enabled_lsps")]
    pub enabled_lsps: Vec<String>,
}

fn default_enabled_lsps() -> Vec<String> {
    vec!["rust".to_string()]
}

fn default_theme() -> String {
    "tokyo-night".to_string()
}

fn default_leader() -> String {
    " ".to_string()
}

fn default_sidebar_width() -> u16 {
    30
}

impl Default for HelioliskConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            leader: default_leader(),
            sidebar_width: default_sidebar_width(),
            keymaps: HashMap::new(),
            enabled_lsps: default_enabled_lsps(),
        }
    }
}

impl HelioliskConfig {
    pub fn config_path() -> PathBuf {
        let local_path = Path::new("heliolisk.toml");
        if local_path.exists() {
            return local_path.to_path_buf();
        }
        if let Ok(home) = std::env::var("HOME") {
            let config_dir = PathBuf::from(home).join(".config/heliolisk");
            let _ = std::fs::create_dir_all(&config_dir);
            return config_dir.join("config.toml");
        }
        PathBuf::from("heliolisk.toml")
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load() -> Self {
        // Look in current working directory first, then user home config
        let local_path = Path::new("heliolisk.toml");
        if local_path.exists() {
            if let Ok(content) = std::fs::read_to_string(local_path) {
                if let Ok(cfg) = toml::from_str::<HelioliskConfig>(&content) {
                    return cfg;
                }
            }
        }

        if let Ok(home) = std::env::var("HOME") {
            let user_path = PathBuf::from(home).join(".config/heliolisk/config.toml");
            if user_path.exists() {
                if let Ok(content) = std::fs::read_to_string(user_path) {
                    if let Ok(cfg) = toml::from_str::<HelioliskConfig>(&content) {
                        return cfg;
                    }
                }
            }
        }

        Self::default()
    }
}
