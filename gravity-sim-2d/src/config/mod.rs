mod generation;
mod pan;
mod partial;
mod simulation;
mod ui;
mod window;
mod zoom;

pub use generation::{
    Area, GenerationConfig, MassStrategy, OverlapStrategy, PositionStrategy, VelocityStrategy,
};
pub use pan::PanConfig;
pub use simulation::SimulationConfig;
pub use ui::UiConfig;
pub use window::WindowConfig;
pub use zoom::ZoomConfig;

use std::{fs, path::PathBuf};

use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, section, warn_unknown_keys};

#[derive(Default, Debug)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub zoom: ZoomConfig,
    pub pan: PanConfig,
    pub simulation: SimulationConfig,
    pub generation: GenerationConfig,
    pub ui: UiConfig,
}

impl FromJsonObject for AppConfig {
    fn from_json_object(obj: &Map<String, Value>, path: &str) -> Self {
        warn_unknown_keys(
            obj,
            &[
                "$schema",
                "window",
                "zoom",
                "pan",
                "simulation",
                "generation",
                "ui",
            ],
            path,
        );

        Self {
            window: section(obj, "window", path),
            zoom: section(obj, "zoom", path),
            pan: section(obj, "pan", path),
            simulation: section(obj, "simulation", path),
            generation: section(obj, "generation", path),
            ui: section(obj, "ui", path),
        }
    }
}

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
        Ok(Value::Object(root)) => AppConfig::from_json_object(&root, ""),

        Ok(_) => {
            eprintln!(
                "config at {} isn't a json object; using defaults",
                path.display()
            );
            AppConfig::default()
        }

        Err(err) => {
            eprintln!(
                "invalid json in config at {}: {err}; using defaults",
                path.display()
            );
            AppConfig::default()
        }
    }
}
