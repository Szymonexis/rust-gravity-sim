mod blend;
mod ramp;

pub use blend::mix;
pub use ramp::ColorRamp;

pub type ColorRGBA = [f32; 4];

pub const WHITE: ColorRGBA = [1.0, 1.0, 1.0, 1.0];
pub const RED: ColorRGBA = [1.0, 0.0, 0.0, 1.0];
pub const YELLOW: ColorRGBA = [1.0, 1.0, 0.0, 1.0];
