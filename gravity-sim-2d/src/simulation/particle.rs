use schemars::JsonSchema;
use serde::Deserialize;

use crate::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, JsonSchema)]
#[serde(from = "Fields")]
pub struct Particle {
    position: Vec2,
    mass: f32,
    velocity: Vec2,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            mass: 1.0,
            velocity: Vec2::ZERO,
        }
    }
}

impl Particle {
    pub fn position(&self) -> &Vec2 {
        &self.position
    }

    pub fn mass(&self) -> &f32 {
        &self.mass
    }

    pub fn velocity(&self) -> &Vec2 {
        &self.velocity
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass.clamp(0.0, f32::INFINITY);
    }

    pub fn set_velocity(&mut self, velocity: Vec2) {
        self.velocity = velocity;
    }

    pub fn new(position: Vec2, mass: f32, velocity: Vec2) -> Self {
        Self {
            position,
            mass: mass.clamp(0.0, f32::INFINITY),
            velocity,
        }
    }

    pub fn radius(&self) -> f32 {
        self.mass.max(0.0).sqrt() * 0.5
    }
}

/// One particle, placed by hand. Every field is optional and falls back to its
/// default, so a particle at rest is just a position and a mass.
#[derive(Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "Particle")]
struct Fields {
    /// Position in world units.
    position: Vec2,
    /// Mass in world units. Negative values are clamped to 0. Mass is treated
    /// as area, so the drawn radius is sqrt(mass) / 2, and it drives render
    /// colour through the mass-colour ramp.
    mass: f32,
    /// Velocity in world units per unit time.
    velocity: Vec2,
}

impl Default for Fields {
    fn default() -> Self {
        let Particle {
            position,
            mass,
            velocity,
        } = Particle::default();

        Self {
            position,
            mass,
            velocity,
        }
    }
}

impl From<Fields> for Particle {
    fn from(fields: Fields) -> Self {
        Self::new(fields.position, fields.mass, fields.velocity)
    }
}
