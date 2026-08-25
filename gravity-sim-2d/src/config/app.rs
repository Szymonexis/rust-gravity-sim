use schemars::JsonSchema;
use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, section, warn_unknown_keys};
use crate::config::{
    ColorsConfig, PanConfig, ParticlesConfig, SimulationConfig, UiConfig, WindowConfig, ZoomConfig,
};

/// Every key is optional. Anything missing or invalid falls back to its own
/// default, so one bad entry never costs you the rest of the file.
#[derive(Default, Debug, JsonSchema)]
#[schemars(title = "Gravity Sim 2D config", deny_unknown_fields)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub zoom: ZoomConfig,
    pub pan: PanConfig,
    pub simulation: SimulationConfig,
    pub particles: ParticlesConfig,
    pub colors: ColorsConfig,
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
                "particles",
                "colors",
                "ui",
            ],
            path,
        );

        Self {
            window: section(obj, "window", path),
            zoom: section(obj, "zoom", path),
            pan: section(obj, "pan", path),
            simulation: section(obj, "simulation", path),
            particles: section(obj, "particles", path),
            colors: section(obj, "colors", path),
            ui: section(obj, "ui", path),
        }
    }
}
