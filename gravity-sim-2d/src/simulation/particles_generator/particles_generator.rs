use std::f32::consts::{PI, TAU};

use rand::{Rng, RngExt};

use crate::simulation::{
    particle::Particle,
    particles_generator::{
        area::Area, position_strategy::PositionStrategy, velocity_strategy::VelocityStrategy,
    },
};

pub const PARTICLE_RADIUS: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
pub struct GeneratorConfig {
    pub amount: usize,
    pub area: Area,
    pub position: PositionStrategy,
    pub velocity: VelocityStrategy,
    pub mass: f32,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            amount: 100,
            area: Area::default(),
            position: PositionStrategy::Uniform,
            velocity: VelocityStrategy::Random { max_speed: 10.0 },
            mass: 1.0,
        }
    }
}

pub fn generate_initial_particles(config: GeneratorConfig) -> Vec<Particle> {
    let mut rng = rand::rng();

    sample_positions(config.position, config.amount, &mut rng)
        .into_iter()
        .map(|unit| {
            let position = config.area.from_unit_disc(unit);
            let velocity = sample_velocity(config.velocity, position, &mut rng);

            Particle::new(Particle {
                position,
                mass: config.mass,
                velocity,
            })
        })
        .collect()
}

fn sample_positions(
    strategy: PositionStrategy,
    amount: usize,
    rng: &mut impl Rng,
) -> Vec<[f32; 2]> {
    match strategy {
        PositionStrategy::Uniform => (0..amount).map(|_| uniform_unit_disc(rng)).collect(),

        PositionStrategy::Gaussian { standard_deviation } => (0..amount)
            .map(|_| gaussian_unit_disc(standard_deviation, rng))
            .collect(),

        PositionStrategy::Ring { inner_fraction } => {
            let inner = inner_fraction.clamp(0.0, 0.999);
            (0..amount)
                .map(|_| {
                    let u: f32 = rng.random();
                    let r = (inner * inner + u * (1.0 - inner * inner)).sqrt();
                    let theta = rng.random_range(0.0..TAU);
                    [r * theta.cos(), r * theta.sin()]
                })
                .collect()
        }

        PositionStrategy::Sunflower => {
            let golden_angle = PI * (3.0 - 5.0_f32.sqrt());

            (0..amount)
                .map(|i| {
                    let r = ((i as f32 + 0.5) / amount as f32).sqrt();
                    let theta = i as f32 * golden_angle;
                    [r * theta.cos(), r * theta.sin()]
                })
                .collect()
        }

        PositionStrategy::Clusters { count, spread } => {
            let count = count.max(1);
            let centers: Vec<[f32; 2]> = (0..count).map(|_| uniform_unit_disc(rng)).collect();

            (0..amount)
                .map(|i| {
                    let [center_0, center_1] = centers[i % count];
                    let [offset_0, offset_1] =
                        [standard_normal(rng) * spread, standard_normal(rng) * spread];

                    clamp_to_unit_disc([center_0 + offset_0, center_1 + offset_1])
                })
                .collect()
        }
    }
}

#[inline]
fn uniform_unit_disc(rng: &mut impl Rng) -> [f32; 2] {
    let theta = rng.random_range(0.0..TAU);
    let r = rng.random::<f32>().sqrt(); // sqrt -> uniform per unit of area
    [r * theta.cos(), r * theta.sin()]
}

fn gaussian_unit_disc(std_dev: f32, rng: &mut impl Rng) -> [f32; 2] {
    let sigma = std_dev.max(1e-4);

    for _ in 0..64 {
        let p = [standard_normal(rng) * sigma, standard_normal(rng) * sigma];
        if p[0] * p[0] + p[1] * p[1] <= 1.0 {
            return p;
        }
    }

    uniform_unit_disc(rng)
}

fn standard_normal(rng: &mut impl Rng) -> f32 {
    let u1 = rng.random::<f32>().max(f32::MIN_POSITIVE);
    let u2 = rng.random::<f32>();
    (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos()
}

#[inline]
fn clamp_to_unit_disc(p: [f32; 2]) -> [f32; 2] {
    let len_sq = p[0] * p[0] + p[1] * p[1];
    if len_sq <= 1.0 {
        p
    } else {
        let len = len_sq.sqrt();
        [p[0] / len, p[1] / len]
    }
}

fn sample_velocity(strategy: VelocityStrategy, position: [f32; 2], rng: &mut impl Rng) -> [f32; 2] {
    match strategy {
        VelocityStrategy::Stationary => [0.0, 0.0],

        VelocityStrategy::CommonVector { velocity } => velocity,

        VelocityStrategy::Random { max_speed } => {
            let theta = rng.random_range(0.0..TAU);
            let speed = rng.random_range(0.0..=max_speed);
            [speed * theta.cos(), speed * theta.sin()]
        }

        VelocityStrategy::Orbital { angular_speed } => {
            [-angular_speed * position[1], angular_speed * position[0]]
        }

        VelocityStrategy::Radial { speed } => {
            let len = (position[0] * position[0] + position[1] * position[1]).sqrt();
            if len < f32::EPSILON {
                // dead centre has no radial direction - pick one at random
                let theta = rng.random_range(0.0..TAU);
                [speed * theta.cos(), speed * theta.sin()]
            } else {
                [speed * position[0] / len, speed * position[1] / len]
            }
        }
    }
}
