use std::path::PathBuf;

use color_eyre::eyre::{Result, eyre};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

pub(crate) const LOCATION_FILENAME: &str = "location.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Location {
    pub(crate) city: String,
    pub(crate) country: String,
    pub(crate) country_code: String,
    pub(crate) region: String,
    pub(crate) latitude: f64,
    pub(crate) longitude: f64,
    pub(crate) timestamp: u64,
}

#[derive(Debug, Deserialize)]
struct IpWhoIs {
    country: String,
    country_code: String,
    region: String,
    city: String,
    latitude: f64,
    longitude: f64,
}

fn path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "keyrate", "keyrate")
        .ok_or_else(|| eyre!("could not determine config directory"))?;
    Ok(dirs.config_dir().join(LOCATION_FILENAME))
}

fn save(location: &Location) -> Result<()> {
    let path = path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(location)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub(crate) fn remove() -> Result<()> {
    let _ = std::fs::remove_file(path()?);
    Ok(())
}

fn fetch() -> Result<Location> {
    let resp: IpWhoIs = reqwest::blocking::get("https://ipwho.is/")?.json()?;
    Ok(Location {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        city: resp.city,
        country: resp.country,
        country_code: resp.country_code,
        region: resp.region,
        latitude: resp.latitude,
        longitude: resp.longitude,
    })
}

pub(crate) fn spawn_fetch(tx: std::sync::mpsc::Sender<Location>) {
    std::thread::spawn(move || {
        if let Ok(location) = fetch() {
            let _ = save(&location);
            let _ = tx.send(location);
        }
    });
}
