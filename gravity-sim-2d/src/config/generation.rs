use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, leaf, warn_unknown_keys};
use crate::math::Vec2;

/// How a generated cloud is built. Positions, masses and velocities are all
/// sampled from the strategies below.
#[derive(Debug, Clone, Copy, JsonSchema)]
#[schemars(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct GenerationConfig {
    /// Number of particles to generate.
    pub amount: usize,
    /// Region the particles are spawned into. Positions are sampled on a unit
    /// disc, then scaled by the semi-axes of this area.
    pub area: Area,
    /// How positions are distributed across the unit disc before scaling.
    pub position: PositionStrategy,
    /// Initial velocity given to each particle.
    pub velocity: VelocityStrategy,
    /// How the mass of each particle is chosen. Mass is treated as area, so the
    /// drawn radius is sqrt(mass) / 2, and it drives render colour through the
    /// mass-colour ramp.
    pub mass: MassStrategy,
    /// What to do about particles that were sampled on top of each other.
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub enum Area {
    /// A disc of the given radius.
    Circle {
        /// Radius in world units.
        radius: f32,
    },
    /// An ellipse with independent horizontal and vertical semi-axes.
    Ellipse {
        /// Horizontal semi-axis in world units.
        semi_x: f32,
        /// Vertical semi-axis in world units.
        semi_y: f32,
    },
}

impl Default for Area {
    fn default() -> Self {
        Area::Circle { radius: 50.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub enum PositionStrategy {
    /// Even coverage per unit of area across the whole disc.
    Uniform,
    /// Normally distributed around the centre, resampled until inside the disc.
    Gaussian {
        /// Spread as a fraction of the disc radius. Raised to 1e-4 if smaller.
        spread: f32,
    },
    /// Uniform within an annulus, leaving a hole in the middle.
    Ring {
        /// Inner hole radius as a fraction of the outer radius. Clamped to
        /// 0.0 - 0.999.
        inner_fraction: f32,
    },
    /// Deterministic golden-angle spiral. Evenly spaced with no clumping.
    Sunflower,
    /// Several gaussian blobs at random centres, clamped back inside the disc.
    Clusters {
        /// How many clusters. Treated as at least 1.
        count: usize,
        /// Standard deviation of each cluster, as a fraction of the disc
        /// radius.
        spread: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub enum VelocityStrategy {
    /// Everything starts at rest.
    Stationary,
    /// Every particle starts with the same velocity.
    CommonVector {
        /// Velocity in world units.
        velocity: Vec2,
    },
    /// Random direction, with speed drawn from 0 up to max_speed.
    Random {
        /// Upper bound on the initial speed.
        max_speed: f32,
    },
    /// Perpendicular to the centre, giving the cloud a net rotation.
    Orbital {
        /// Radians per unit time. Negative spins the other way.
        angular_speed: f32,
    },
    /// Straight out from the centre, for an explosion. Particles exactly at the
    /// centre get a random direction instead.
    Radial {
        /// Outward speed. Negative collapses inward instead.
        speed: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub enum MassStrategy {
    /// Every particle gets the same mass.
    Constant {
        /// Mass for every particle. Negative values are clamped to 0.
        value: f32,
    },
    /// Mass drawn uniformly from a range.
    Random {
        /// Inclusive lower bound.
        min: f32,
        /// Exclusive upper bound. Must be above min.
        max: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub enum OverlapStrategy {
    /// Particles land wherever the position strategy put them, intersections
    /// included.
    Allow,
    /// Overlapping particles are pushed apart afterwards. Positions are nudged
    /// rather than resampled, so the shape the position strategy produced
    /// survives. Heavier particles give way less than lighter ones.
    Separate {
        /// Most relaxation passes to run. Stops early once nothing intersects,
        /// so raising this only costs anything on crowded setups.
        iterations: u32,
        /// Extra gap in world units left between two touching particles.
        padding: f32,
    },
}

impl Default for OverlapStrategy {
    fn default() -> Self {
        OverlapStrategy::Separate {
            iterations: 32,
            padding: 0.0,
        }
    }
}
