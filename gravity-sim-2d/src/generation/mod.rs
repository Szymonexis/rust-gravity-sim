mod area;
mod grid;
mod overlap;
mod sampling;

use crate::config::{GenerationConfig, OverlapStrategy, ParticlesConfig};
use crate::math::Vec2;
use crate::simulation::Particle;

pub fn build(config: &ParticlesConfig) -> Vec<Particle> {
    match config {
        ParticlesConfig::Set(particles) => particles.clone(),
        ParticlesConfig::Generation(generation) => generate(*generation),
    }
}

pub fn generate(config: GenerationConfig) -> Vec<Particle> {
    let mut rng = rand::rng();

    let mut particles: Vec<Particle> =
        sampling::positions(config.position, config.amount, &mut rng)
            .into_iter()
            .map(|unit| {
                Particle::new(
                    config.area.from_unit_disc(unit),
                    sampling::mass(config.mass, &mut rng),
                    Vec2::ZERO,
                )
            })
            .collect();

    if let OverlapStrategy::Separate {
        iterations,
        padding,
    } = config.overlap
    {
        overlap::separate(&mut particles, config.area, iterations, padding);
    }

    for particle in &mut particles {
        let velocity = sampling::velocity(config.velocity, *particle.position(), &mut rng);
        particle.set_velocity(velocity);
    }

    particles
}
