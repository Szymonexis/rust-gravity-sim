use schemars::JsonSchema;
use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, leaf, warn_unknown_keys};

/// Scroll-wheel zoom.
#[derive(Debug, JsonSchema)]
#[schemars(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ZoomConfig {
    /// Whether the scroll wheel zooms at all. When false the other keys here
    /// are inert.
    pub enable: bool,
    /// Lower clamp on the zoom factor.
    pub min: f32,
    /// Upper clamp on the zoom factor.
    pub max: f32,
    /// Multiplier applied per scroll line: new zoom = zoom * step^lines. At or
    /// below 1 this inverts or freezes zooming.
    pub step: f32,
    /// Zoom factor the camera starts at.
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
