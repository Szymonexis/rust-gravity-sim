use schemars::JsonSchema;
use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, join, leaf, warn_unknown_keys};

/// Mass thresholds for the colour a particle is drawn in. A particle lighter
/// than the lowest threshold takes that stop's colour, one heavier than the
/// highest takes that one, and anything in between is blended across the two it
/// falls between. The three are sorted by mass, so the order you write them in
/// doesn't matter.
#[derive(Debug, Clone, Copy, JsonSchema)]
#[schemars(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ColorsConfig {
    /// Mass drawn white.
    #[schemars(extend("exclusiveMinimum" = 0.0))]
    pub white: f32,
    /// Mass drawn yellow.
    #[schemars(extend("exclusiveMinimum" = 0.0))]
    pub yellow: f32,
    /// Mass drawn red.
    #[schemars(extend("exclusiveMinimum" = 0.0))]
    pub red: f32,
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            white: 10.0,
            yellow: 200.0,
            red: 1000.0,
        }
    }
}

impl FromJsonObject for ColorsConfig {
    fn from_json_object(obj: &Map<String, Value>, path: &str) -> Self {
        warn_unknown_keys(obj, &["white", "yellow", "red"], path);

        let defaults = Self::default();
        Self {
            white: positive(obj, "white", path, defaults.white),
            yellow: positive(obj, "yellow", path, defaults.yellow),
            red: positive(obj, "red", path, defaults.red),
        }
    }
}

fn positive(obj: &Map<String, Value>, key: &str, path: &str, fallback: f32) -> f32 {
    let value = leaf(obj, key, path, fallback);
    if value.is_finite() && value > 0.0 {
        return value;
    }

    eprintln!(
        "config: `{}` has to be a positive number; using default",
        join(path, key)
    );
    fallback
}
