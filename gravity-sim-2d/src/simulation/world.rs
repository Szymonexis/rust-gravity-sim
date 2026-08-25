use crate::config::{CollisionStrategy, SimulationConfig, SimulationMethod};
use crate::math::Vec2;
use crate::simulation::Particle;
use crate::simulation::collision::Collider;

#[derive(Debug)]
pub struct World {
    particles: Vec<Particle>,
    collider: Collider,
    elapsed: f32,
    ticks: i64,
    gravitational_constant: f32,
    collisions: CollisionStrategy,
}

impl World {
    pub fn new(particles: Vec<Particle>, config: SimulationConfig) -> Self {
        Self {
            particles,
            collider: Collider::default(),
            elapsed: 0.0,
            ticks: 0,
            gravitational_constant: config.gravitational_constant,
            collisions: config.collisions,
        }
    }

    pub fn on_tick(&mut self, delta: f32, method: SimulationMethod) {
        if delta == 0.0 {
            return;
        }

        self.elapsed += delta;
        self.ticks += if delta < 0.0 { -1 } else { 1 };

        match method {
            SimulationMethod::Naive => {
                self.naive(delta);
            }

            SimulationMethod::BarnesHut => {
                self.barnes_hut(delta);
            }

            SimulationMethod::Multipole => {
                self.multipole(delta);
            }
        }

        if self.collisions == CollisionStrategy::Merge {
            self.collider.resolve(&mut self.particles);
        }
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn tick_count(&self) -> i64 {
        self.ticks
    }

    fn naive(&mut self, delta: f32) {
        let accelerations: Vec<Vec2> = self
            .particles
            .iter()
            .map(|tested_particle| {
                self.particles
                    .iter()
                    .filter_map(|target_particle| {
                        self.g_accel_vector(*tested_particle, *target_particle)
                    })
                    .fold(Vec2::ZERO, |sum, accel| sum + accel)
            })
            .collect();

        for (particle, acceleration) in self.particles.iter_mut().zip(accelerations) {
            let new_velocity = *particle.velocity() + acceleration * delta;
            particle.set_velocity(new_velocity);
            particle.set_position(*particle.position() + new_velocity * delta);
        }
    }

    fn barnes_hut(&mut self, delta: f32) {
        todo!()
    }

    fn multipole(&mut self, delta: f32) {
        todo!()
    }

    fn g_accel_vector(&self, tested_particle: Particle, target_particle: Particle) -> Option<Vec2> {
        let offset = *target_particle.position() - *tested_particle.position();
        let distance = offset.length();

        if distance == 0.0 {
            return None;
        }

        Some(
            offset
                * (self.gravitational_constant * target_particle.mass()
                    / (distance * distance * distance)),
        )
    }
}
