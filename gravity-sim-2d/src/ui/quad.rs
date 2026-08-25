use crate::color::ColorRGBA;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Quad {
    pub rect: [f32; 4],
    pub uv: [f32; 4],
    pub color: ColorRGBA,
    pub kind: u32,
    pub _pad: [u32; 3],
}

impl Quad {
    pub const KIND_RECT: u32 = 0;
    pub const KIND_GLYPH: u32 = 1;

    pub fn rect(rect: [f32; 4], color: ColorRGBA) -> Self {
        Self {
            rect,
            uv: [0.0; 4],
            color,
            kind: Self::KIND_RECT,
            _pad: [0; 3],
        }
    }

    pub fn glyph(rect: [f32; 4], uv: [f32; 4], color: ColorRGBA) -> Self {
        Self {
            rect,
            uv,
            color,
            kind: Self::KIND_GLYPH,
            _pad: [0; 3],
        }
    }
}

const _: () = assert!(size_of::<Quad>() == 64);
