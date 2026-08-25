use schemars::JsonSchema;
use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, leaf, warn_unknown_keys};

/// Click-and-drag panning.
#[derive(Debug, JsonSchema)]
#[schemars(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct PanConfig {
    /// Whether holding the left mouse button and dragging pans the camera.
    pub enable: bool,
}

impl Default for PanConfig {
    fn default() -> Self {
        Self { enable: true }
    }
}

impl FromJsonObject for PanConfig {
    fn from_json_object(obj: &Map<String, Value>, path: &str) -> Self {
        warn_unknown_keys(obj, &["enable"], path);

        let defaults = Self::default();
        Self {
            enable: leaf(obj, "enable", path, defaults.enable),
        }
    }
}
