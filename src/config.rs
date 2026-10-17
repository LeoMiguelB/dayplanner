use std::path::{Path, PathBuf};
use std::fs;
use directories_next::ProjectDirs;
use serde::{Deserialize, Serialize};
use crate::model::Granularity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub db_path: Option<String>,
    pub default_granularity: Option<Granularity>,
    pub start_hour: Option<u32>,
    pub end_hour: Option<u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            db_path: None,
            default_granularity: Some(Granularity::FifteenMinutes),
            start_hour: Some(8),
            end_hour: Some(20),
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn config_dir() -> Option<PathBuf> {
        ProjectDirs::from("", "", "dayplanner").map(|dirs| dirs.config_dir().to_path_buf())
    }

    pub fn config_file_path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("config.toml"))
    }

    pub fn load_config() -> (AppConfig, bool) {
        if let Some(config_path) = Self::config_file_path() {
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                        return (config, true);
                    }
                }
            }
        }
        (AppConfig::default(), false)
    }

    pub fn save_config(config: &AppConfig) -> Result<(), std::io::Error> {
        if let Some(config_dir) = Self::config_dir() {
            fs::create_dir_all(&config_dir)?;
            let config_path = config_dir.join("config.toml");
            let content = toml::to_string_pretty(config)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            fs::write(config_path, content)?;
        }
        Ok(())
    }

    pub fn vault_preset_path() -> PathBuf {
        if let Some(home) = directories_next::BaseDirs::new().map(|d| d.home_dir().to_path_buf()) {
            home.join("documents/leeosvault/misc/dayplanner.db")
        } else {
            PathBuf::from("/home/lmb/documents/leeosvault/misc/dayplanner.db")
        }
    }

    pub fn xdg_data_default_path() -> PathBuf {
        if let Some(dirs) = ProjectDirs::from("", "", "dayplanner") {
            dirs.data_dir().join("planner.db")
        } else {
            PathBuf::from("dayplanner.db")
        }
    }

    pub fn temp_fallback_path() -> PathBuf {
        std::env::temp_dir().join("dayplanner/dayplanner_temp.db")
    }

    pub fn ensure_parent_dirs(path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}
