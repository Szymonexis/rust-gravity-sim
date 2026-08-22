use std::{fs, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WindowConfig {
    pub height: f32,
    pub width: f32,
    pub resizable: bool,
}

#[derive(Debug, Deserialize)]
pub struct ZoomConfig {
    pub enable: bool,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub initial: f32,
}

#[derive(Debug, Deserialize)]
pub struct PanConfig {
    pub enable: bool,
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub window: WindowConfig,
    pub zoom: ZoomConfig,
    pub pan: PanConfig,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            height: 720.0,
            width: 1280.0,
            resizable: true,
        }
    }
}

impl Default for ZoomConfig {
    fn default() -> Self {
        Self {
            enable: true,
            min: 0.1,
            max: 10_000.0,
            step: 1.1,
            initial: 50.0
        }
    }
}

impl Default for PanConfig {
    fn default() -> Self {
        Self { enable: true }
    }
}

/// Config lives next to the Cargo.toml so using this as attach point.
/// Works only in `cargo run` cases.
fn config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("app-config.json")
}

pub fn load() -> AppConfig {
    let path = config_path();

    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!(
                "couldn't read config at {}: {err}; using defaults",
                path.display()
            );
            return AppConfig::default();
        }
    };

    match serde_json::from_str(&contents) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "invalid config at {}: {err}; using defaults",
                path.display()
            );
            AppConfig::default()
        }
    }
}
