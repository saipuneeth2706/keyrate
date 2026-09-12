use std::path::PathBuf;

use color_eyre::eyre::{Result, eyre};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::app::TestMode;

const SCORES_TIME_FILENAME: &str = "scores_time.json";
const SCORES_WORDS_FILENAME: &str = "scores_words.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ScoreRecord {
    pub(crate) wpm: f64,
    pub(crate) accuracy: f64,
    pub(crate) errors: usize,
    pub(crate) consistency: f64,
    pub(crate) time_secs: f64,
    pub(crate) option: u64,
    pub(crate) timestamp: u64,
}

fn scores_dir() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "keyrate", "keyrate")
        .ok_or_else(|| eyre!("could not determine config directory"))?;
    Ok(dirs.config_dir().to_path_buf())
}

fn scores_path(mode: TestMode) -> Result<PathBuf> {
    let filename = match mode {
        TestMode::Time => SCORES_TIME_FILENAME,
        TestMode::Words => SCORES_WORDS_FILENAME,
    };
    Ok(scores_dir()?.join(filename))
}

pub(crate) fn load(mode: TestMode) -> Vec<ScoreRecord> {
    scores_path(mode)
        .ok()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub(crate) fn append(mode: TestMode, record: ScoreRecord) -> Result<()> {
    let path = scores_path(mode)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut records = load(mode);
    records.push(record);
    let content = serde_json::to_string_pretty(&records)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub(crate) fn remove_all() -> Result<()> {
    let dir = scores_dir()?;
    let _ = std::fs::remove_file(dir.join(SCORES_TIME_FILENAME));
    let _ = std::fs::remove_file(dir.join(SCORES_WORDS_FILENAME));
    Ok(())
}
