use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::config::GenerationConfig;
use crate::config::partial::{FromJsonObject, join, section, warn_unknown_keys};
use crate::simulation::Particle;

/// The particles the world starts with. Either write them out yourself under
/// `Set`, or describe a cloud under `Generation` and let the app sample one.
/// Applied once at startup.
#[derive(Debug, Clone, JsonSchema)]
pub enum ParticlesConfig {
    /// An explicit list of particles, taken exactly as written.
    Set(Vec<Particle>),
    /// A recipe the generator follows to sample a cloud at startup.
    Generation(GenerationConfig),
}

impl Default for ParticlesConfig {
    fn default() -> Self {
        Self::Generation(GenerationConfig::default())
    }
}

impl FromJsonObject for ParticlesConfig {
    fn from_json_object(obj: &Map<String, Value>, path: &str) -> Self {
        warn_unknown_keys(obj, &["Set", "Generation"], path);

        match (obj.get("Set"), obj.contains_key("Generation")) {
            (Some(set), true) => {
                eprintln!("config: `{path}` has both `Set` and `Generation`; using `Set`");
                Self::set(set, path)
            }
            (Some(set), false) => Self::set(set, path),
            (None, true) => Self::Generation(section(obj, "Generation", path)),
            (None, false) => Self::default(),
        }
    }
}

impl ParticlesConfig {
    fn set(value: &Value, path: &str) -> Self {
        match Vec::<Particle>::deserialize(value) {
            Ok(particles) => Self::Set(particles),
            Err(err) => {
                eprintln!(
                    "config: `{}` is invalid ({err}); using defaults",
                    join(path, "Set")
                );
                Self::default()
            }
        }
    }
}
