use std::f32::consts::TAU;

use crate::simulation::Particle;

const OSCILLATION_MIN_MASS: f32 = 1.0;
const OSCILLATION_MAX_MASS: f32 = 1_000.0;
const OSCILLATION_PERIOD: f32 = 4.0;

#[derive(Debug)]
pub struct World {
    particles: Vec<Particle>,
    elapsed: f32,
    ticks: i64,
}

impl World {
    pub fn new(particles: Vec<Particle>) -> Self {
        Self {
            particles,
            elapsed: 0.0,
            ticks: 0,
        }
    }

    /// A negative `delta` runs the world backwards, so the tick counter is
    /// signed and can walk back down through states it has already been in.
    pub fn tick(&mut self, delta: f32) {
        if delta == 0.0 {
            return;
        }

        self.elapsed += delta;
        self.ticks += if delta < 0.0 { -1 } else { 1 };

        let mass = oscillating_mass(self.elapsed);
        for particle in &mut self.particles {
            particle.set_mass(mass);
        }
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn tick_count(&self) -> i64 {
        self.ticks
    }
}

fn oscillating_mass(elapsed: f32) -> f32 {
    let blend = 0.5 - 0.5 * (TAU * elapsed / OSCILLATION_PERIOD).cos();
    OSCILLATION_MIN_MASS + (OSCILLATION_MAX_MASS - OSCILLATION_MIN_MASS) * blend
}
