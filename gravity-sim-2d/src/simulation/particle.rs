use optfield::optfield;

#[optfield(pub PartialParticle, attrs, merge_fn, from)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    pub position: [f32; 2],
    pub mass: f32,
    pub velocity: [f32; 2],
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            mass: 1.0,
            velocity: [0.0, 0.0],
        }
    }
}

impl Particle {
    pub fn new(particle: Particle) -> Self {
        Self {
            position: particle.position,
            mass: particle.mass.clamp(0.0, f32::INFINITY),
            velocity: particle.velocity,
        }
    }

    pub fn update(&self, changes: PartialParticle) -> Self {
        let mut updated = *self;
        updated.merge_opt(changes);
        updated
    }
}
