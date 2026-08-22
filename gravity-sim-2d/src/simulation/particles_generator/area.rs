use std::f32::consts::PI;

use crate::simulation::particles_generator::particles_generator::PARTICLE_RADIUS;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Area {
    Circle { radius: f32 },
    Ellipse { semi_x: f32, semi_y: f32 },
}

impl Default for Area {
    fn default() -> Self {
        Area::Circle { radius: 50.0 }
    }
}

impl Area {
    #[inline]
    fn semi_axes(self) -> (f32, f32) {
        match self {
            Area::Circle { radius } => (radius, radius),
            Area::Ellipse { semi_x, semi_y } => (semi_x, semi_y),
        }
    }

    #[inline]
    pub fn from_unit_disc(self, p: [f32; 2]) -> [f32; 2] {
        let (a, b) = self.semi_axes();
        let [p0, p1] = p;
        [p0 * a, p1 * b]
    }

    pub fn surface(self) -> f32 {
        let (a, b) = self.semi_axes();
        PI * a * b
    }

    pub fn fill_fraction(self, amount: usize) -> f32 {
        amount as f32 * PI * PARTICLE_RADIUS * PARTICLE_RADIUS / self.surface()
    }
}
