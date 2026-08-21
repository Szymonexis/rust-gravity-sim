// The one shader of the project: draws the scene through the 2D camera
// described by the `Globals` uniform.
//
// It has no idea what the scene contains. Every shape comes from the `shapes`
// storage buffer, filled by the CPU (see src/scene.rs); the shader only knows
// how to turn one buffer entry into one quad, and how to color the pixels of
// that quad.

// Language reference (WGSL): https://www.w3.org/TR/WGSL/
// A gentler introduction:    https://google.github.io/tour-of-wgsl/

// Coordinate spaces used here, from innermost to outermost:

//   local space   — one shape's own quad: [-1, 1] on both axes, (0, 0) at the
//                   center, whatever the shape's real size. The circle test
//                   lives here, which is why it is a plain "distance < 1".
//   world space   — the simulation's own units. A shape's `center` and `size`
//                   place its quad here. Nothing in world space references
//                   the window, so the scene has a size of its own.
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

// Silhouettes a shape can have. CPU-side counterpart: the `Shape::KIND_*`
// constants in src/scene.rs.
const KIND_SQUARE: u32 = 0u;
const KIND_CIRCLE: u32 = 1u;

// One drawable shape. CPU-side counterpart: `Shape` in src/scene.rs — field
// order, types and offsets must match exactly. Offsets are in bytes:
struct Shape {
    center: vec2<f32>,  //  0: world units
    size: f32,          //  8: side length (square) or diameter (circle)
    kind: u32,          // 12: one of the KIND_* constants above
    color: vec4<f32>,   // 16: linear RGBA (16-byte aligned, hence offset 16)
};                      // 32 bytes total

// Storage buffer: like a uniform in that the CPU fills it, but bigger, and
// *runtime-sized* — `array<Shape>` has no length in the source; it is however
// long the buffer bound to it happens to be. That is what lets the scene grow
// without touching the shader. Storage layout is also kinder than uniform
// layout: array elements keep their natural 32-byte stride instead of being
// padded up. https://www.w3.org/TR/WGSL/#address-spaces
@group(0) @binding(1)
var<storage, read> shapes: array<Shape>;

// Everything the vertex stage hands to the fragment stage. `@builtin(position)`
// is mandatory (it is what the rasterizer consumes); the `@location(...)`
// fields are "inter-stage variables": written once per vertex, then handed to
// every fragment the triangle covers.
// https://webgpufundamentals.org/webgpu/lessons/webgpu-inter-stage-variables.html
struct VsOut {
    @builtin(position) clip: vec4<f32>,
    // Position within this shape's own quad, in local space. Written only at
    // the 6 corners; the rasterizer *interpolates* it, so each fragment in
    // between learns where it sits. That interpolation is the whole trick
    // behind the circle test below.
    @location(0) local: vec2<f32>,
    // The shape's kind and color. Both are constant across the whole quad, so
    // interpolating them would be wasted work — `flat` says "hand every
    // fragment the value as written". For `kind` it is not even optional:
    // integers cannot be interpolated at all.
    @location(1) @interpolate(flat) kind: u32,
    @location(2) @interpolate(flat) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VsOut {
    // The quad as two triangles, generated from the vertex index instead of a
    // vertex buffer — a common trick for fixed geometry. These 6 corners are
    // the one thing still hardcoded here, and deliberately so: they are the
    // same for every shape and never change, so uploading them would cost
    // bandwidth for nothing.
    // https://webgpufundamentals.org/webgpu/lessons/webgpu-fundamentals.html
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    // The renderer issues `draw(0..6, 0..shape_count)`: `vertex_index` counts
    // the 6 corners, and the whole thing reruns per instance with
    // `instance_index` selecting which shape this quad belongs to.
    let shape = shapes[instance_index];
    let corner = corners[vertex_index];

    // Local -> world. Corners span [-1, 1], i.e. 2 units across, so halving
    // makes `size` mean the shape's actual width in world units.
    let world = corner * shape.size * 0.5 + shape.center;

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
    out.kind = shape.kind;
    out.color = shape.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // A circle is the same quad with its corners shaved off: any fragment
    // farther than 1 (in local space) from the center is thrown away.
    // `discard` skips writing the pixel entirely, so the background shows
    // through. Testing per fragment — rather than approximating the outline
    // with many vertices — is the standard way to draw circles in 2D, and it
    // stays perfectly round at any zoom.
    if in.kind == KIND_CIRCLE && length(in.local) > 1.0 {
        discard;
    }

    // Whatever the CPU asked for. The render pass clears to black first
    // (renderer.rs), so a discarded fragment reads as background.
    return in.color;
}
