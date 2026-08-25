use std::time::Duration;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::config::partial::{FromJsonObject, leaf, warn_unknown_keys};

/// How a tick works out the forces acting on each particle.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum SimulationMethod {
    /// Every particle against every other one. Exact, and the cost per tick
    /// grows with the square of the particle count.
    Naive,
    /// A quadtree of the world, so a distant clump of particles is pulled on as
    /// if it were one mass at its centre. Approximate, and the cost per tick
    /// grows close to linearly with the particle count.
    BarnesHut,
    /// The same tree, but a clump is summarised by a series expansion rather
    /// than a single mass, so the approximation holds far closer in for the same
    /// amount of work. Costliest to build, cheapest per particle.
    Multipole,
}

/// What becomes of particles that end a tick intersecting each other.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum CollisionStrategy {
    /// Particles pass straight through one another. Only gravity ever moves
    /// them.
    Ignore,
    /// Every touching group - a chain of overlaps counts as one group, however
    /// long - collapses into a single particle at its centre of mass, carrying
    /// the mass and the momentum of everything that went into it. Mass is area,
    /// so the survivor is drawn exactly as large as the discs it swallowed put
    /// together. Merging is one-way: reversing the speed dial replays the
    /// motion, not the merges.
    Merge,
}

/// Pacing of the simulation thread. It runs on its own clock, so none of this
/// is tied to the display refresh rate.
#[derive(Debug, Clone, Copy, JsonSchema)]
#[schemars(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct SimulationConfig {
    /// Ticks per second. Also fixes the dt every tick integrates with, so
    /// changing it changes the physics resolution, not just the pace.
    #[schemars(extend("exclusiveMinimum" = 0.0))]
    pub tick_rate: f32,
    /// Most ticks run back to back when the thread has fallen behind. Past this
    /// the backlog is dropped and the simulation runs slower than real time
    /// instead of spiralling.
    #[schemars(range(min = 1))]
    pub max_catch_up_ticks: u32,
    /// Gravitational constant the force between two particles is scaled by.
    /// Deliberately not the real 6.674e-11 - the world runs in arbitrary units,
    /// so this is the dial for how hard the cloud pulls on itself.
    pub gravitational_constant: f32,
    /// Which algorithm a tick uses. They all model the same physics and differ
    /// only in what they cost and how exact they are.
    pub method: SimulationMethod,
    /// What a tick does about particles that have run into each other.
    pub collisions: CollisionStrategy,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            tick_rate: 60.0,
            max_catch_up_ticks: 5,
            gravitational_constant: 4.0,
            method: SimulationMethod::Naive,
            collisions: CollisionStrategy::Merge,
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
        warn_unknown_keys(
            obj,
            &[
                "tickRate",
                "maxCatchUpTicks",
                "gravitationalConstant",
                "method",
                "collisions",
            ],
            path,
        );

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
            gravitational_constant: leaf(
                obj,
                "gravitationalConstant",
                path,
                defaults.gravitational_constant,
            ),
            method: leaf(obj, "method", path, defaults.method),
            collisions: leaf(obj, "collisions", path, defaults.collisions),
        }
    }
}
