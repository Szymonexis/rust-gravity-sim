mod area;
mod grid;
mod overlap;
mod sampling;

use crate::config::{GenerationConfig, OverlapStrategy};
use crate::simulation::Particle;

pub fn generate(config: GenerationConfig) -> Vec<Particle> {
    let mut rng = rand::rng();

    let mut particles: Vec<Particle> =
        sampling::positions(config.position, config.amount, &mut rng)
            .into_iter()
            .map(|unit| {
                Particle::new(
                    config.area.from_unit_disc(unit),
                    sampling::mass(config.mass, &mut rng),
                    [0.0, 0.0],
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

    // Velocities last: Orbital and Radial are derived from the position, so
    // they have to see where separation actually left each particle.
    for particle in &mut particles {
        let velocity = sampling::velocity(config.velocity, *particle.position(), &mut rng);
        particle.set_velocity(velocity);
    }

    particles
}
