use std::f32::consts::{PI, TAU};

use rand::{Rng, RngExt};

use crate::config::{MassStrategy, PositionStrategy, VelocityStrategy};
use crate::math::Vec2;

pub fn positions(strategy: PositionStrategy, amount: usize, rng: &mut impl Rng) -> Vec<Vec2> {
    match strategy {
        PositionStrategy::Uniform => (0..amount).map(|_| uniform_unit_disc(rng)).collect(),

        PositionStrategy::Gaussian { spread } => (0..amount)
            .map(|_| gaussian_unit_disc(spread, rng))
            .collect(),

        PositionStrategy::Ring { inner_fraction } => {
            let inner = inner_fraction.clamp(0.0, 0.999);
            (0..amount)
                .map(|_| {
                    let u: f32 = rng.random();
                    let r = (inner * inner + u * (1.0 - inner * inner)).sqrt();
                    let theta = rng.random_range(0.0..TAU);
                    Vec2::new(r * theta.cos(), r * theta.sin())
                })
                .collect()
        }

        PositionStrategy::Sunflower => {
            let golden_angle = PI * (3.0 - 5.0_f32.sqrt());

            (0..amount)
                .map(|i| {
                    let r = ((i as f32 + 0.5) / amount as f32).sqrt();
                    let theta = i as f32 * golden_angle;
                    Vec2::new(r * theta.cos(), r * theta.sin())
                })
                .collect()
        }

        PositionStrategy::Clusters { count, spread } => {
            let count = count.max(1);
            let centers: Vec<Vec2> = (0..count).map(|_| uniform_unit_disc(rng)).collect();

            (0..amount)
                .map(|i| {
                    let center = centers[i % count];
                    let offset =
                        Vec2::new(standard_normal(rng) * spread, standard_normal(rng) * spread);

                    clamp_to_unit_disc(center + offset)
                })
                .collect()
        }
    }
}

pub fn velocity(strategy: VelocityStrategy, position: Vec2, rng: &mut impl Rng) -> Vec2 {
    match strategy {
        VelocityStrategy::Stationary => Vec2::ZERO,

        VelocityStrategy::CommonVector { velocity } => velocity,

        VelocityStrategy::Random { max_speed } => {
            let theta = rng.random_range(0.0..TAU);
            let speed = rng.random_range(0.0..=max_speed);
            Vec2::new(speed * theta.cos(), speed * theta.sin())
        }

        VelocityStrategy::Orbital { angular_speed } => {
            Vec2::new(-angular_speed * position.y, angular_speed * position.x)
        }

        VelocityStrategy::Radial { speed } => {
            let len = position.length();
            if len < f32::EPSILON {
                let theta = rng.random_range(0.0..TAU);
                Vec2::new(speed * theta.cos(), speed * theta.sin())
            } else {
                position * (speed / len)
            }
        }
    }
}

pub fn mass(strategy: MassStrategy, rng: &mut impl Rng) -> f32 {
    match strategy {
        MassStrategy::Constant { value } => value,
        MassStrategy::Random { min, max } if max > min => rng.random_range(min..max),
        MassStrategy::Random { min, .. } => min,
    }
}

fn uniform_unit_disc(rng: &mut impl Rng) -> Vec2 {
    let theta = rng.random_range(0.0..TAU);
    let r = rng.random::<f32>().sqrt();
    Vec2::new(r * theta.cos(), r * theta.sin())
}

fn gaussian_unit_disc(std_dev: f32, rng: &mut impl Rng) -> Vec2 {
    let sigma = std_dev.max(1e-4);

    for _ in 0..64 {
        let p = Vec2::new(standard_normal(rng) * sigma, standard_normal(rng) * sigma);
        if p.length_squared() <= 1.0 {
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

fn clamp_to_unit_disc(p: Vec2) -> Vec2 {
    let len_sq = p.length_squared();
    if len_sq <= 1.0 { p } else { p / len_sq.sqrt() }
}
