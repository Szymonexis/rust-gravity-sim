use std::f32::consts::PI;

use crate::config::Area;
use crate::math::Vec2;

impl Area {
    #[inline]
    pub fn semi_axes(self) -> (f32, f32) {
        match self {
            Area::Circle { radius } => (radius, radius),
            Area::Ellipse { semi_x, semi_y } => (semi_x, semi_y),
        }
    }

    #[inline]
    pub fn from_unit_disc(self, p: Vec2) -> Vec2 {
        let (a, b) = self.semi_axes();
        Vec2::new(p.x * a, p.y * b)
    }

    pub fn surface(self) -> f32 {
        let (a, b) = self.semi_axes();
        PI * a * b
    }

    pub fn clamp_inside(self, p: Vec2) -> Vec2 {
        let (a, b) = self.semi_axes();
        if a <= 0.0 || b <= 0.0 {
            return Vec2::ZERO;
        }

        let norm = (p.x / a) * (p.x / a) + (p.y / b) * (p.y / b);
        if norm <= 1.0 {
            return p;
        }

        p / norm.sqrt()
    }
}
