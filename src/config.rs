use std::path::PathBuf;

use color_eyre::eyre::{Result, eyre};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

pub(crate) const CONFIG_FILENAME: &str = "keyrate.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) name: String,
}

fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "keyrate", "keyrate")
        .ok_or_else(|| eyre!("could not determine config directory"))?;
    Ok(dirs.config_dir().join(CONFIG_FILENAME))
}

pub(crate) fn load() -> Option<Config> {
    let path = config_path().ok()?;
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub(crate) fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub(crate) fn remove() -> Result<()> {
    let path = config_path()?;
    std::fs::remove_file(path).ok();
    Ok(())
}
