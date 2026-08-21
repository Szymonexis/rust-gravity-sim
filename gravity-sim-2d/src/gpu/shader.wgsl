// The one shader of the project: draws the scene (currently a white square
// and a yellow circle) through the 2D camera described by the `Globals`
// uniform.

// Language reference (WGSL): https://www.w3.org/TR/WGSL/
// A gentler introduction:    https://google.github.io/tour-of-wgsl/

// Coordinate spaces used here, from innermost to outermost:

//   world space   — the simulation's own units. Each shape is 1x1 world
//                   units; the square is centered on the origin. Nothing here
//                   ever references the window, so the scene has a size of
//                   its own.
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

// Everything the vertex stage hands to the fragment stage. `@builtin(position)`
// is mandatory (it is what the rasterizer consumes); the `@location(...)`
// fields are "inter-stage variables": written once per vertex, then
// interpolated across the triangle so every fragment gets its own value.
// https://webgpufundamentals.org/webgpu/lessons/webgpu-inter-stage-variables.html
struct VsOut {
    @builtin(position) clip: vec4<f32>,
    // Where this fragment sits inside its own quad, [-1, 1] on both axes —
    // (0, 0) at the quad's center. Interpolation is what makes this work:
    // only the 6 corner values are written here, and the rasterizer blends
    // them so each pixel in between knows its spot.
    @location(0) local: vec2<f32>,
    // Which instance this fragment belongs to (0 = square, 1 = circle).
    // Integers can't be blended across a triangle, so interpolation must be
    // explicitly switched off with `flat`: every fragment gets the value as-is.
    @location(1) @interpolate(flat) shape: u32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VsOut {
    // The quad as two triangles, generated from the vertex index instead of
    // a vertex buffer — a common trick for fixed geometry. The renderer issues
    // `draw(0..6, 0..2)`: `vertex_index` counts 0..5, and the whole thing runs
    // twice with `instance_index` 0 then 1 — same geometry, different placement.
    // https://webgpufundamentals.org/webgpu/lessons/webgpu-fundamentals.html
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    // Per-instance placement: the square sits on the origin, the circle's
    // quad 1.5 world units to its right.
    var centers = array<vec2<f32>, 2>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.5, 0.0),
    );

    let corner = corners[vertex_index];

    // Corners span [-1, 1]; halving gives a quad of side 1 world unit.
    let world = corner * 0.5 + centers[instance_index];

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

    var out: VsOut;
    out.clip = vec4<f32>(clip, 0.0, 1.0);
    out.local = corner;
    out.shape = instance_index;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // The circle is the same quad with its corners shaved off: any fragment
    // farther than 1 (in local units) from the quad's center is thrown away.
    // `discard` skips writing the pixel entirely, so the background shows
    // through. This fragment-shader test — not extra vertices — is the
    // standard way to draw filled circles in 2D.
    if in.shape == 1u && length(in.local) > 1.0 {
        discard;
    }

    // The render pass clears to black first (renderer.rs).
    if in.shape == 1u {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0); // yellow circle
    }
    return vec4<f32>(1.0, 1.0, 1.0, 1.0); // white square
}
