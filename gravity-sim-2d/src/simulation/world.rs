use crate::simulation::Particle;

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
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn tick_count(&self) -> i64 {
        self.ticks
    }
}
