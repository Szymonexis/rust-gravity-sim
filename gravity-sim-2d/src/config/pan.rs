use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, leaf, warn_unknown_keys};

#[derive(Debug)]
pub struct PanConfig {
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
