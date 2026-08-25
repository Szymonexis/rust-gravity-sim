use std::mem;

use crate::math::Vec2;
use crate::simulation::Particle;

const MIN_CELL_SIZE: f32 = 1e-6;
const UNCLAIMED: u32 = u32::MAX;

#[derive(Debug, Default)]
pub struct Collider {
    grid: Grid,
    parents: Vec<u32>,
    clusters: Vec<Cluster>,
    slots: Vec<u32>,
    merged: Vec<Particle>,
}

impl Collider {
    pub fn resolve(&mut self, particles: &mut Vec<Particle>) {
        if particles.len() < 2 {
            return;
        }

        let Self {
            grid,
            parents,
            clusters,
            slots,
            merged,
        } = self;

        grid.rebuild(particles);

        parents.clear();
        parents.extend(0..particles.len() as u32);

        let mut merging = false;
        for (i, particle) in particles.iter().enumerate() {
            for j in grid.neighbours(*particle.position()) {
                if j <= i {
                    continue;
                }

                let other = &particles[j];
                let contact = particle.radius() + other.radius();
                let offset = *other.position() - *particle.position();
                if offset.length_squared() >= contact * contact {
                    continue;
                }

                merging |= union(parents, i as u32, j as u32);
            }
        }

        if !merging {
            return;
        }

        clusters.clear();
        slots.clear();
        slots.resize(particles.len(), UNCLAIMED);

        for (i, particle) in particles.iter().enumerate() {
            let root = find(parents, i as u32) as usize;
            let slot = match slots[root] {
                UNCLAIMED => {
                    slots[root] = clusters.len() as u32;
                    clusters.push(Cluster::seeded_by(particle));
                    clusters.len() - 1
                }
                claimed => claimed as usize,
            };

            clusters[slot].absorb(particle);
        }

        merged.clear();
        merged.extend(clusters.iter().map(Cluster::particle));
        mem::swap(particles, merged);
    }
}

#[derive(Debug, Clone, Copy)]
struct Cluster {
    seed: Particle,
    members: u32,
    mass: f32,
    offset: Vec2,
    momentum: Vec2,
}

impl Cluster {
    fn seeded_by(particle: &Particle) -> Self {
        Self {
            seed: *particle,
            members: 0,
            mass: 0.0,
            offset: Vec2::ZERO,
            momentum: Vec2::ZERO,
        }
    }

    fn absorb(&mut self, particle: &Particle) {
        let mass = *particle.mass();

        self.members += 1;
        self.mass += mass;
        self.offset += (*particle.position() - *self.seed.position()) * mass;
        self.momentum += *particle.velocity() * mass;
    }

    fn particle(&self) -> Particle {
        if self.members < 2 || self.mass <= 0.0 {
            return self.seed;
        }

        Particle::new(
            *self.seed.position() + self.offset / self.mass,
            self.mass,
            self.momentum / self.mass,
        )
    }
}

fn find(parents: &mut [u32], mut index: u32) -> u32 {
    while parents[index as usize] != index {
        let grandparent = parents[parents[index as usize] as usize];
        parents[index as usize] = grandparent;
        index = grandparent;
    }

    index
}

fn union(parents: &mut [u32], a: u32, b: u32) -> bool {
    let (a, b) = (find(parents, a), find(parents, b));
    if a == b {
        return false;
    }

    parents[a.max(b) as usize] = a.min(b);
    true
}

#[derive(Debug, Default)]
struct Grid {
    origin: Vec2,
    cell_size: f32,
    columns: usize,
    rows: usize,
    starts: Vec<u32>,
    cursors: Vec<u32>,
    items: Vec<u32>,
}

impl Grid {
    fn rebuild(&mut self, particles: &[Particle]) {
        self.fit(particles);

        let cells = self.columns * self.rows;

        self.starts.clear();
        self.starts.resize(cells + 1, 0);

        for particle in particles {
            let cell = self.cell_of(*particle.position());
            self.starts[cell + 1] += 1;
        }

        for cell in 1..=cells {
            self.starts[cell] += self.starts[cell - 1];
        }

        self.cursors.clear();
        self.cursors.extend_from_slice(&self.starts[..cells]);

        self.items.clear();
        self.items.resize(particles.len(), 0);

        for (i, particle) in particles.iter().enumerate() {
            let cell = self.cell_of(*particle.position());
            let slot = self.cursors[cell] as usize;
            self.cursors[cell] += 1;
            self.items[slot] = i as u32;
        }
    }

    fn fit(&mut self, particles: &[Particle]) {
        let mut min = Vec2::new(f32::INFINITY, f32::INFINITY);
        let mut max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut widest = 0.0f32;

        for particle in particles {
            let position = *particle.position();
            min = Vec2::new(min.x.min(position.x), min.y.min(position.y));
            max = Vec2::new(max.x.max(position.x), max.y.max(position.y));
            widest = widest.max(particle.radius());
        }

        let width = max.x - min.x;
        let height = max.y - min.y;
        let count = particles.len() as f32;

        self.origin = min;
        self.cell_size = (2.0 * widest)
            .max((width * height / count).sqrt())
            .max(width / count)
            .max(height / count)
            .max(MIN_CELL_SIZE);
        self.columns = span(width, self.cell_size, particles.len());
        self.rows = span(height, self.cell_size, particles.len());
    }

    fn neighbours(&self, position: Vec2) -> impl Iterator<Item = usize> {
        let (column, row) = self.column_row(position);

        (-1..=1isize)
            .flat_map(|dy| (-1..=1isize).map(move |dx| (dx, dy)))
            .filter_map(move |(dx, dy)| {
                let x = column as isize + dx;
                let y = row as isize + dy;
                let inside =
                    x >= 0 && y >= 0 && (x as usize) < self.columns && (y as usize) < self.rows;
                inside.then(|| y as usize * self.columns + x as usize)
            })
            .flat_map(move |cell| {
                let start = self.starts[cell] as usize;
                let end = self.starts[cell + 1] as usize;
                self.items[start..end].iter().map(|&index| index as usize)
            })
    }

    fn cell_of(&self, position: Vec2) -> usize {
        let (column, row) = self.column_row(position);
        row * self.columns + column
    }

    fn column_row(&self, position: Vec2) -> (usize, usize) {
        let x = ((position.x - self.origin.x) / self.cell_size).floor();
        let y = ((position.y - self.origin.y) / self.cell_size).floor();

        (
            (x.max(0.0) as usize).min(self.columns - 1),
            (y.max(0.0) as usize).min(self.rows - 1),
        )
    }
}

fn span(extent: f32, cell_size: f32, limit: usize) -> usize {
    let cells = (extent / cell_size).ceil();
    if !cells.is_finite() || cells <= 0.0 {
        return 1;
    }

    (cells as usize).min(limit) + 1
}
