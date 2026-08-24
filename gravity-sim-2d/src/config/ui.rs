use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, leaf, warn_unknown_keys};

#[derive(Debug, Clone)]
pub struct UiConfig {
    /// Absolute path to a `.ttf`/`.otf` file. `None` - or anything that fails
    /// to load - falls through to the JetBrains Mono that ships with the app.
    pub font_path: Option<String>,
    pub font_size: f32,
    pub show_stats: bool,
    pub show_manual: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            font_path: None,
            font_size: 14.0,
            show_stats: true,
            show_manual: true,
        }
    }
}

impl FromJsonObject for UiConfig {
    fn from_json_object(obj: &Map<String, Value>, path: &str) -> Self {
        warn_unknown_keys(
            obj,
            &["fontPath", "fontSize", "showStats", "showManual"],
            path,
        );

        let defaults = Self::default();

        let font_size = leaf(obj, "fontSize", path, defaults.font_size);
        let font_size = if font_size.is_finite() && font_size >= 4.0 {
            font_size
        } else {
            eprintln!("config: `fontSize` has to be at least 4; using default");
            defaults.font_size
        };

        Self {
            font_path: leaf(obj, "fontPath", path, defaults.font_path),
            font_size,
            show_stats: leaf(obj, "showStats", path, defaults.show_stats),
            show_manual: leaf(obj, "showManual", path, defaults.show_manual),
        }
    }
}
