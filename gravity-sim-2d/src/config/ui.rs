use schemars::JsonSchema;
use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, leaf, warn_unknown_keys};

/// The on-screen overlay. Drawn as its own pass on top of the scene, in
/// physical pixels, so none of it moves when you pan or zoom.
#[derive(Debug, Clone, JsonSchema)]
#[schemars(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct UiConfig {
    /// Absolute path to a .ttf or .otf file. Forward slashes work on Windows
    /// too, and save you escaping every backslash. Null - or a path that won't
    /// load - falls back to JetBrains Mono, which ships with the app.
    pub font_path: Option<String>,
    /// Text height in logical pixels. Multiplied by the display scale factor
    /// before the glyphs are rasterised, so it stays the same physical size on
    /// a hi-dpi screen.
    #[schemars(range(min = 4.0))]
    pub font_size: f32,
    /// Whether the top-left panel - particle count, tick, fps, tps, zoom, pan -
    /// is drawn.
    pub show_stats: bool,
    /// Whether the controls list - drag, wheel, space, arrows, esc - is drawn
    /// under the playback state in the top-right panel. The playback state
    /// itself is always shown.
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
