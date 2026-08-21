//! What to draw: the CPU-side description of the scene.
//!
//! A [`Shape`] is the unit of drawing. The renderer uploads the whole list
//! into a GPU storage buffer and issues *one* draw call with one instance per
//! shape; the shader reads an entry per instance and turns it into a quad
//! (see `src/gpu/shader.wgsl`). Nothing about the scene — how many shapes,
//! where they are, what they look like — is baked into the shader, so adding
//! a shape here is all it takes to see it on screen.
//!
//! This buffer is also the seam the physics will grow into: a compute pass
//! will write body positions straight into it, and the drawing side won't
//! have to change.

/// One drawable shape, in the exact byte layout the GPU expects.
///
/// GPU-side counterpart: `struct Shape` in `src/gpu/shader.wgsl` — field
/// order, types and offsets must match exactly, since these bytes are copied
/// verbatim into the storage buffer.
///
/// - `#[repr(C)]` pins the field order/layout (Rust otherwise reorders).
/// - `Pod`/`Zeroable` (bytemuck) certify "plain old data", making the
///   byte-cast in the renderer safe: <https://docs.rs/bytemuck/>
/// - The field offsets (0, 8, 12, 16) and the 32-byte total fall out of
///   WGSL's layout rules: `color` is a `vec4<f32>`, whose 16-byte alignment
///   forces it to start at offset 16. That happens to leave no padding here,
///   which is what lets `Pod` be derived at all.
///   <https://www.w3.org/TR/WGSL/#memory-layouts>
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Shape {
    /// Center of the shape, in world units.
    pub center: [f32; 2],
    /// Side length (square) or diameter (circle), in world units.
    pub size: f32,
    /// Which silhouette to draw: one of the `KIND_*` constants. It stays a
    /// raw `u32` because that is what crosses to the GPU — prefer the
    /// [`Shape::square`] / [`Shape::circle`] constructors over setting it.
    pub kind: u32,
    /// Linear RGBA, each channel 0..=1.
    pub color: [f32; 4],
}

impl Shape {
    /// Mirrors `KIND_SQUARE` in `shader.wgsl`.
    pub const KIND_SQUARE: u32 = 0;
    /// Mirrors `KIND_CIRCLE` in `shader.wgsl`.
    pub const KIND_CIRCLE: u32 = 1;

    /// An axis-aligned square of side `size`, centered on `center`.
    pub fn square(center: [f32; 2], size: f32, color: [f32; 4]) -> Self {
        Self {
            center,
            size,
            kind: Self::KIND_SQUARE,
            color,
        }
    }

    /// A filled circle of diameter `size`, centered on `center`.
    pub fn circle(center: [f32; 2], size: f32, color: [f32; 4]) -> Self {
        Self {
            center,
            size,
            kind: Self::KIND_CIRCLE,
            color,
        }
    }
}

/// The shader's `Shape` is 32 bytes and the GPU reads the array at that
/// stride, so a field added here without a matching change in the WGSL
/// would garble every shape. Fail the build instead.
const _: () = assert!(size_of::<Shape>() == 32);

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];

/// The scene the program starts with: a white square on the world origin and
/// a yellow circle 1.5 world units to its right.
pub fn initial() -> Vec<Shape> {
    vec![
        Shape::square([0.0, 0.0], 1.0, WHITE),
        Shape::circle([1.5, 0.0], 1.0, YELLOW),
    ]
}
