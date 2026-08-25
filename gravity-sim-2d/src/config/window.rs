use schemars::JsonSchema;
use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, leaf, warn_unknown_keys};

/// Size and behaviour of the OS window.
#[derive(Debug, JsonSchema)]
#[schemars(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct WindowConfig {
    /// Window height in logical pixels.
    pub height: f32,
    /// Window width in logical pixels.
    pub width: f32,
    /// Whether the window can be resized by dragging its edges.
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
