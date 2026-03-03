use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Serializable payload — separate from Settings so `path` is never written to disk.
#[derive(Serialize, Deserialize, Default)]
struct SettingsData {
    #[serde(default)]
    ignored_game_keys: HashSet<String>,
}

pub struct Settings {
    path: PathBuf,
    ignored_game_keys: HashSet<String>,
}

impl Settings {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, SettingsError> {
        let path = path.into();
        let data: SettingsData = if path.exists() {
            let raw = std::fs::read_to_string(&path)?;
            serde_json::from_str(&raw)?
        } else {
            SettingsData::default()
        };
        Ok(Self {
            path,
            ignored_game_keys: data.ignored_game_keys,
        })
    }

    pub fn ignored_game_keys(&self) -> &HashSet<String> {
        &self.ignored_game_keys
    }

    pub fn add_ignored(&mut self, key: String) -> Result<(), SettingsError> {
        self.ignored_game_keys.insert(key);
        self.persist()
    }

    pub fn remove_ignored(&mut self, key: &str) -> Result<(), SettingsError> {
        self.ignored_game_keys.remove(key);
        self.persist()
    }

    fn persist(&self) -> Result<(), SettingsError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = SettingsData {
            ignored_game_keys: self.ignored_game_keys.clone(),
        };
        std::fs::write(&self.path, serde_json::to_string_pretty(&data)?)?;
        Ok(())
    }
}
