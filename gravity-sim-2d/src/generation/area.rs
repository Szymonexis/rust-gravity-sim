use std::f32::consts::PI;

use crate::config::Area;

impl Area {
    #[inline]
    pub fn semi_axes(self) -> (f32, f32) {
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

    pub fn clamp_inside(self, p: [f32; 2]) -> [f32; 2] {
        let (a, b) = self.semi_axes();
        if a <= 0.0 || b <= 0.0 {
            return [0.0, 0.0];
        }

        let [x, y] = p;
        let norm = (x / a) * (x / a) + (y / b) * (y / b);
        if norm <= 1.0 {
            return p;
        }

        let scale = norm.sqrt();
        [x / scale, y / scale]
    }
}
