#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    position: [f32; 2],
    mass: f32,
    velocity: [f32; 2],
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
    // getters
    pub fn position(&self) -> &[f32; 2] {
        &self.position
    }

    pub fn mass(&self) -> &f32 {
        &self.mass
    }

    pub fn velocity(&self) -> &[f32; 2] {
        &self.velocity
    }

    // setters
    pub fn set_position(&mut self, position: [f32; 2]) {
        self.position = position;
    }

    /// Clamped here and in [`Particle::new`], the only two ways mass is ever
    /// written, so `radius` and the mass ramp never see a negative.
    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass.clamp(0.0, f32::INFINITY);
    }

    pub fn set_velocity(&mut self, velocity: [f32; 2]) {
        self.velocity = velocity;
    }

    // methods
    pub fn new(position: [f32; 2], mass: f32, velocity: [f32; 2]) -> Self {
        Self {
            position,
            mass: mass.clamp(0.0, f32::INFINITY),
            velocity,
        }
    }

    /// The renderer and the overlap pass both size particles through here, so
    /// they can't drift apart.
    pub fn radius(&self) -> f32 {
        self.mass.max(0.0).sqrt() * 0.5
    }
}
