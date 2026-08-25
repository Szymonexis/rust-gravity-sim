use crate::color::{ColorRGBA, ColorRamp, RED, WHITE, YELLOW};
use crate::config::ColorsConfig;
use crate::math::Vec2;
use crate::simulation::Particle;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Shape {
    pub center: Vec2,
    pub size: f32,
    pub kind: u32,
    pub color: ColorRGBA,
}

impl Shape {
    pub const KIND_CIRCLE: u32 = 0;

    pub fn circle(center: Vec2, size: f32, color: ColorRGBA) -> Self {
        Self {
            center,
            size,
            kind: Self::KIND_CIRCLE,
            color,
        }
    }
}

const _: () = assert!(size_of::<Shape>() == 32);

pub struct Scene {
    pub shapes: Vec<Shape>,
    mass_ramp: ColorRamp,
}

impl Scene {
    pub fn init(particles: &[Particle], colors: &ColorsConfig) -> Self {
        let mut scene = Self {
            shapes: Vec::with_capacity(particles.len()),
            mass_ramp: ColorRamp::new(vec![
                (colors.white, WHITE),
                (colors.yellow, YELLOW),
                (colors.red, RED),
            ]),
        };
        scene.sync(particles);
        scene
    }

    pub fn sync(&mut self, particles: &[Particle]) {
        self.shapes.clear();
        self.shapes.extend(particles.iter().map(|particle| {
            Shape::circle(
                *particle.position(),
                particle.radius() * 2.0,
                self.mass_ramp.sample(*particle.mass()),
            )
        }));
    }
}
