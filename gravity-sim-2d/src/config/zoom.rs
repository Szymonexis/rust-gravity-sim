use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, leaf, warn_unknown_keys};

#[derive(Debug)]
pub struct ZoomConfig {
    pub enable: bool,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub initial: f32,
}

impl Default for ZoomConfig {
    fn default() -> Self {
        Self {
            enable: true,
            min: 0.1,
            max: 10_000.0,
            step: 1.1,
            initial: 10.0,
        }
    }
}

impl FromJsonObject for ZoomConfig {
    fn from_json_object(obj: &Map<String, Value>, path: &str) -> Self {
        warn_unknown_keys(obj, &["enable", "min", "max", "step", "initial"], path);

        let defaults = Self::default();
        Self {
            enable: leaf(obj, "enable", path, defaults.enable),
            min: leaf(obj, "min", path, defaults.min),
            max: leaf(obj, "max", path, defaults.max),
            step: leaf(obj, "step", path, defaults.step),
            initial: leaf(obj, "initial", path, defaults.initial),
        }
    }
}
