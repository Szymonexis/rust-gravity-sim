use std::time::Duration;

use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, leaf, warn_unknown_keys};

#[derive(Debug, Clone, Copy)]
pub struct SimulationConfig {
    pub tick_rate: f32,
    pub max_catch_up_ticks: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            tick_rate: 60.0,
            max_catch_up_ticks: 5,
        }
    }
}

impl SimulationConfig {
    pub fn tick_period(&self) -> Duration {
        Duration::from_secs_f32(1.0 / self.tick_rate)
    }
}

impl FromJsonObject for SimulationConfig {
    fn from_json_object(obj: &Map<String, Value>, path: &str) -> Self {
        warn_unknown_keys(obj, &["tickRate", "maxCatchUpTicks"], path);

        let defaults = Self::default();

        let tick_rate = leaf(obj, "tickRate", path, defaults.tick_rate);
        let tick_rate = if tick_rate.is_finite() && tick_rate > 0.0 {
            tick_rate
        } else {
            eprintln!("config: `tickRate` has to be a positive number; using default");
            defaults.tick_rate
        };

        Self {
            tick_rate,
            max_catch_up_ticks: leaf(obj, "maxCatchUpTicks", path, defaults.max_catch_up_ticks)
                .max(1),
        }
    }
}
