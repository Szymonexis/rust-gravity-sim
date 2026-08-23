use crate::color::{ColorRGBA, ColorRamp, RED, WHITE, YELLOW};
use crate::simulation::Particle;

const MASS_RAMP: ColorRamp = ColorRamp::new(&[(10.0, WHITE), (200.0, YELLOW), (1000.0, RED)]);

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Shape {
    pub center: [f32; 2],
    pub size: f32,
    pub kind: u32,
    pub color: ColorRGBA,
}

impl Shape {
    pub const KIND_CIRCLE: u32 = 0;

    pub fn circle(center: [f32; 2], size: f32, color: ColorRGBA) -> Self {
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
}

impl Scene {
    pub fn init(particles: &[Particle]) -> Self {
        let mut scene = Self {
            shapes: Vec::with_capacity(particles.len()),
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
                MASS_RAMP.sample(*particle.mass()),
            )
        }));
    }
}
