// The one shader of the project: draws the scene (currently a single white
// square) through the 2D camera described by the `Globals` uniform.
//
// Language reference (WGSL): https://www.w3.org/TR/WGSL/
// A gentler introduction:    https://google.github.io/tour-of-wgsl/
//
// Coordinate spaces used here, from innermost to outermost:
//
//   world space   — the simulation's own units. The square is 1x1 world units,
//                   centered on the origin. Nothing here ever references the
//                   window, so the scene has a size of its own.
//   screen space  — physical pixels, origin at the *center* of the window,
//                   +y pointing up. The camera (pan/zoom) maps world space
//                   into this space.
//   clip space    — what the GPU wants from a vertex shader: x and y in
//                   [-1, 1] across the whole viewport, +y up.
//                   https://www.w3.org/TR/webgpu/#coordinate-systems

// CPU-side counterpart: `Globals` in src/gpu/renderer.rs — field order, types
// and padding must match exactly. WGSL struct layout rules:
// https://www.w3.org/TR/WGSL/#memory-layouts
struct Globals {
    // Window client area size in physical pixels, updated on every resize.
    resolution: vec2<f32>,
    // Where the world origin sits on screen, in pixels from the window
    // center, +y up. Dragging with the left mouse button changes this.
    pan: vec2<f32>,
    // Pixels per world unit. Mouse wheel changes this.
    zoom: f32,
    // Explicit padding: WGSL rounds this struct's size up to a multiple of
    // its 8-byte alignment (24 bytes total), so the CPU side must send 24
    // bytes too. Handy slot for a future field (e.g. elapsed time).
    _pad: f32,
};

// Uniform buffer: small constants shared by every vertex/fragment in a draw.
// Bound by src/gpu/renderer.rs at bind group 0, binding 0.
// https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/
@group(0) @binding(0)
var<uniform> globals: Globals;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // The square as two triangles, generated from the vertex index instead of
    // a vertex buffer — a common trick for fixed geometry. The renderer issues
    // `draw(0..6, 0..1)`, and `vertex_index` counts 0..5.
    // https://webgpufundamentals.org/webgpu/lessons/webgpu-fundamentals.html
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );
    // Corners span [-1, 1]; halving gives a square of side 1 world unit.
    let world = corners[vertex_index] * 0.5;

    // World -> screen pixels. Zoom scales, pan translates. Because everything
    // up to here is in pixels — not window-relative fractions — the scene has
    // the same on-screen size whatever the window dimensions: resizing only
    // reveals more or less of the world, it never stretches it.
    let pixels = world * globals.zoom + globals.pan;

    // Screen pixels -> clip space: divide by half the resolution per axis.
    // (pixels * 2 / resolution) maps ±resolution/2 to ±1. Both spaces have
    // +y up, so no flip is needed here; the CPU side flips mouse coordinates
    // instead (see src/input.rs).
    let clip = pixels * 2.0 / globals.resolution;
    return vec4<f32>(clip, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // Solid white; the render pass clears to black first (renderer.rs).
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
