mod app;
mod colors;
mod generation;
mod pan;
mod partial;
mod particles;
mod schema;
mod simulation;
mod store;
mod ui;
mod window;
mod zoom;

pub use app::AppConfig;
pub use colors::ColorsConfig;
pub use generation::{
    Area, GenerationConfig, MassStrategy, OverlapStrategy, PositionStrategy, VelocityStrategy,
};
pub use pan::PanConfig;
pub use particles::ParticlesConfig;
pub use schema::schema;
pub use simulation::{CollisionStrategy, SimulationConfig, SimulationMethod};
pub use ui::UiConfig;
pub use window::WindowConfig;
pub use zoom::ZoomConfig;

use serde_json::Value;

use crate::config::partial::FromJsonObject;

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
