use std::f32::consts::PI;

use crate::config::Area;
use crate::generation::grid::SpatialGrid;
use crate::math::Vec2;
use crate::simulation::Particle;

const CROWDED_FILL_FRACTION: f32 = 0.65;
const RELAXATION: f32 = 1.4;
const GOLDEN_ANGLE: f32 = 2.399_963_2;
const COINCIDENT_EPSILON: f32 = 1e-12;
const CONTACT_TOLERANCE: f32 = 1e-3;

pub fn separate(particles: &mut [Particle], area: Area, iterations: u32, padding: f32) {
    if particles.len() < 2 || iterations == 0 {
        return;
    }
    let padding = if padding.is_finite() {
        padding.max(0.0)
    } else {
        0.0
    };

    warn_if_crowded(particles, area, padding);

    let widest_radius = particles
        .iter()
        .map(Particle::radius)
        .fold(0.0f32, f32::max);
    let tolerance = CONTACT_TOLERANCE * widest_radius;

    let mut grid = SpatialGrid::new(particles, area, widest_radius, padding);
    let mut pushes = vec![Vec2::ZERO; particles.len()];
    let mut contacts = vec![0u32; particles.len()];

    for _ in 0..iterations {
        grid.rebuild(particles);

        if accumulate_pushes(particles, &grid, padding, &mut pushes, &mut contacts) <= tolerance {
            break;
        }

        for ((particle, push), contacts) in particles.iter_mut().zip(&pushes).zip(&contacts) {
            if *contacts == 0 {
                continue;
            }

            let scale = RELAXATION / *contacts as f32;
            let moved = *particle.position() + *push * scale;
            particle.set_position(area.clamp_inside(moved));
        }
    }
}

fn accumulate_pushes(
    particles: &[Particle],
    grid: &SpatialGrid,
    padding: f32,
    pushes: &mut [Vec2],
    contacts: &mut [u32],
) -> f32 {
    pushes.fill(Vec2::ZERO);
    contacts.fill(0);
    let mut deepest = 0.0f32;

    for (i, particle) in particles.iter().enumerate() {
        for j in grid.neighbours(*particle.position()) {
            if j <= i {
                continue;
            }
            let other = &particles[j];

            let wanted = particle.radius() + other.radius() + padding;
            let delta = *other.position() - *particle.position();
            let distance_sq = delta.length_squared();
            if distance_sq >= wanted * wanted {
                continue;
            }

            contacts[i] += 1;
            contacts[j] += 1;

            let (axis, gap) = if distance_sq > COINCIDENT_EPSILON {
                let distance = distance_sq.sqrt();
                (delta / distance, wanted - distance)
            } else {
                let theta = i as f32 * GOLDEN_ANGLE;
                (Vec2::new(theta.cos(), theta.sin()), wanted)
            };
            deepest = deepest.max(gap);

            let total_mass = particle.mass() + other.mass();
            let (share, other_share) = if total_mass > 0.0 {
                (other.mass() / total_mass, particle.mass() / total_mass)
            } else {
                (0.5, 0.5)
            };

            pushes[i] -= axis * gap * share;
            pushes[j] += axis * gap * other_share;
        }
    }

    deepest
}

fn warn_if_crowded(particles: &[Particle], area: Area, padding: f32) {
    let surface = area.surface();
    if surface <= 0.0 {
        return;
    }

    let occupied: f32 = particles
        .iter()
        .map(|particle| {
            let radius = particle.radius() + padding * 0.5;
            PI * radius * radius
        })
        .sum();

    let fill = occupied / surface;
    if fill > CROWDED_FILL_FRACTION {
        eprintln!(
            "generation: {} particles cover {:.0}% of the spawn area, over the ~{:.0}% \
             discs can be packed into - some overlap will be left behind. Grow `area`, \
             drop `amount`, or lighten `mass`.",
            particles.len(),
            fill * 100.0,
            CROWDED_FILL_FRACTION * 100.0
        );
    }
}
