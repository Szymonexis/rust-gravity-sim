use serde::Deserialize;
use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, leaf, warn_unknown_keys};

#[derive(Debug, Clone, Copy)]
pub struct GenerationConfig {
    pub amount: usize,
    pub area: Area,
    pub position: PositionStrategy,
    pub velocity: VelocityStrategy,
    pub mass: MassStrategy,
    pub overlap: OverlapStrategy,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            amount: 100,
            area: Area::Circle { radius: 1000.0 },
            position: PositionStrategy::Uniform,
            velocity: VelocityStrategy::Random { max_speed: 10.0 },
            mass: MassStrategy::Random {
                min: 1.0,
                max: 2000.0,
            },
            overlap: OverlapStrategy::default(),
        }
    }
}

impl FromJsonObject for GenerationConfig {
    fn from_json_object(obj: &Map<String, Value>, path: &str) -> Self {
        warn_unknown_keys(
            obj,
            &["amount", "area", "position", "velocity", "mass", "overlap"],
            path,
        );

        let defaults = Self::default();
        Self {
            amount: leaf(obj, "amount", path, defaults.amount),
            area: leaf(obj, "area", path, defaults.area),
            position: leaf(obj, "position", path, defaults.position),
            velocity: leaf(obj, "velocity", path, defaults.velocity),
            mass: leaf(obj, "mass", path, defaults.mass),
            overlap: leaf(obj, "overlap", path, defaults.overlap),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum Area {
    Circle { radius: f32 },
    Ellipse { semi_x: f32, semi_y: f32 },
}

impl Default for Area {
    fn default() -> Self {
        Area::Circle { radius: 50.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum PositionStrategy {
    Uniform,
    Gaussian { standard_deviation: f32 },
    Ring { inner_fraction: f32 },
    Sunflower,
    Clusters { count: usize, spread: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum VelocityStrategy {
    Stationary,
    CommonVector { velocity: [f32; 2] },
    Random { max_speed: f32 },
    Orbital { angular_speed: f32 },
    Radial { speed: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum MassStrategy {
    Constant { value: f32 },
    Random { min: f32, max: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum OverlapStrategy {
    Allow,
    Separate { iterations: u32, padding: f32 },
}

impl Default for OverlapStrategy {
    fn default() -> Self {
        OverlapStrategy::Separate {
            iterations: 32,
            padding: 0.0,
        }
    }
}
