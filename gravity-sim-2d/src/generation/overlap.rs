use std::f32::consts::PI;

use crate::config::Area;
use crate::generation::grid::SpatialGrid;
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
    let mut pushes = vec![[0.0f32; 2]; particles.len()];
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

            // Averaged over the contacts rather than summed: deep inside a
            // clump a particle is pushed from every side at once, and obeying
            // all of those at full strength launches it clear across the cloud.
            let scale = RELAXATION / *contacts as f32;
            let position = particle.position();
            let moved = [
                position[0] + push[0] * scale,
                position[1] + push[1] * scale,
            ];
            particle.set_position(area.clamp_inside(moved));
        }
    }
}

fn accumulate_pushes(
    particles: &[Particle],
    grid: &SpatialGrid,
    padding: f32,
    pushes: &mut [[f32; 2]],
    contacts: &mut [u32],
) -> f32 {
    pushes.fill([0.0, 0.0]);
    contacts.fill(0);
    let mut deepest = 0.0f32;

    for (i, particle) in particles.iter().enumerate() {
        for j in grid.neighbours(*particle.position()) {
            if j <= i {
                continue;
            }
            let other = &particles[j];

            let wanted = particle.radius() + other.radius() + padding;
            let delta = [
                other.position()[0] - particle.position()[0],
                other.position()[1] - particle.position()[1],
            ];
            let distance_sq = delta[0] * delta[0] + delta[1] * delta[1];
            if distance_sq >= wanted * wanted {
                continue;
            }

            contacts[i] += 1;
            contacts[j] += 1;

            let (axis, gap) = if distance_sq > COINCIDENT_EPSILON {
                let distance = distance_sq.sqrt();
                (
                    [delta[0] / distance, delta[1] / distance],
                    wanted - distance,
                )
            } else {
                let theta = i as f32 * GOLDEN_ANGLE;
                ([theta.cos(), theta.sin()], wanted)
            };
            deepest = deepest.max(gap);

            let total_mass = particle.mass() + other.mass();
            let (share, other_share) = if total_mass > 0.0 {
                (other.mass() / total_mass, particle.mass() / total_mass)
            } else {
                (0.5, 0.5)
            };

            pushes[i][0] -= axis[0] * gap * share;
            pushes[i][1] -= axis[1] * gap * share;
            pushes[j][0] += axis[0] * gap * other_share;
            pushes[j][1] += axis[1] * gap * other_share;
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
