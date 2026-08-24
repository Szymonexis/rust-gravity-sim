mod generation;
mod pan;
mod partial;
mod simulation;
mod store;
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

/// Reads the settings, seeding the user's home directory with a copy of the
/// shipped defaults the first time the app runs. The second half of the pair is
/// the `~`-shortened path for the overlay to point at - `None` when no file
/// could be reached and the compiled-in defaults stood in for it.
pub fn load() -> (AppConfig, Option<String>) {
    let file = store::open();
    let (contents, origin) = match &file {
        Some(file) => (file.contents.as_str(), file.path.display().to_string()),
        None => (store::template(), "the built-in defaults".to_owned()),
    };

    println!("Using config: {origin}");

    let config = match serde_json::from_str(contents) {
        Ok(Value::Object(root)) => AppConfig::from_json_object(&root, ""),

        Ok(_) => {
            eprintln!("config: {origin} isn't a json object; using defaults");
            AppConfig::default()
        }

        Err(err) => {
            eprintln!("config: invalid json in {origin} ({err}); using defaults");
            AppConfig::default()
        }
    };

    (config, file.map(|file| file.display))
}
