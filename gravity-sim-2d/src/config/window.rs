use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, leaf, warn_unknown_keys};

#[derive(Debug)]
pub struct WindowConfig {
    pub height: f32,
    pub width: f32,
    pub resizable: bool,
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

impl FromJsonObject for WindowConfig {
    fn from_json_object(obj: &Map<String, Value>, path: &str) -> Self {
        warn_unknown_keys(obj, &["height", "width", "resizable"], path);

        let defaults = Self::default();
        Self {
            height: leaf(obj, "height", path, defaults.height),
            width: leaf(obj, "width", path, defaults.width),
            resizable: leaf(obj, "resizable", path, defaults.resizable),
        }
    }
}
