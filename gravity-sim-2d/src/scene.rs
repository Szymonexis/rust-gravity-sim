#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Shape {
    pub center: [f32; 2],
    pub size: f32,
    pub kind: u32,
    pub color: [f32; 4],
}

impl Shape {
    pub const KIND_CIRCLE: u32 = 0;

    pub fn circle(center: [f32; 2], size: f32, color: [f32; 4]) -> Self {
        Self {
            center,
            size,
            kind: Self::KIND_CIRCLE,
            color,
        }
    }
}

const _: () = assert!(size_of::<Shape>() == 32);

pub fn initial() -> Vec<Shape> {
    vec![Shape::circle([0.0, 0.0], 1.0, WHITE)]
}
