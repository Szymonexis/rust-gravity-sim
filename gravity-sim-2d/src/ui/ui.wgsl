// The overlay layer. It shares nothing with shader.wgsl on purpose: this one
// works in physical pixels with the origin in the top-left corner, because
// that is how you describe a corner-anchored panel, whereas the scene works in
// world units around a centred origin.

struct Globals {
    resolution: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

const KIND_RECT: u32 = 0u;
const KIND_GLYPH: u32 = 1u;

struct Quad {
    // x, y, width, height - pixels, top-left origin, y growing downwards.
    rect: vec4<f32>,
    // u0, v0, u1, v1 into the glyph atlas. Ignored by KIND_RECT.
    uv: vec4<f32>,
    color: vec4<f32>,
    kind: u32,
    // Three scalars, not the vec3<u32> they look like: a vec3 is 16-byte
    // aligned in WGSL, which would push it to offset 64 and stretch this
    // struct to 80 bytes while Rust still packs it into 64. Every quad after
    // the first would then be read from a slightly wrong offset.
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(1)
var<storage, read> quads: array<Quad>;

@group(0) @binding(2)
var atlas: texture_2d<f32>;

@group(0) @binding(3)
var atlas_sampler: sampler;

struct VertexShaderOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) kind: u32,
    @location(2) @interpolate(flat) color: vec4<f32>,
};

@vertex
fn vertex_shader_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexShaderOut {
    // Unit corners rather than the scene's -1..1 ones: a quad here is anchored
    // by its top-left corner and grown by its size, not by a half-extent.
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );

    let quad = quads[instance_index];
    let corner = corners[vertex_index];
    let pixels = quad.rect.xy + corner * quad.rect.zw;

    // Pixels (y down) to clip space (y up).
    let clip = vec2<f32>(
        pixels.x / globals.resolution.x * 2.0 - 1.0,
        1.0 - pixels.y / globals.resolution.y * 2.0,
    );

    var out: VertexShaderOut;
    out.clip = vec4<f32>(clip, 0.0, 1.0);
    out.uv = mix(quad.uv.xy, quad.uv.zw, corner);
    out.kind = quad.kind;
    out.color = quad.color;
    return out;
}

@fragment
fn fragment_shader_main(in: VertexShaderOut) -> @location(0) vec4<f32> {
    // The atlas stores coverage, not colour: one channel saying how much of
    // this pixel the glyph covered. The colour comes from the quad, so the
    // same atlas draws text in any colour.
    //
    // Sampled unconditionally and picked afterwards, rather than sampled
    // inside an `if`: textureSample needs uniform control flow, because it
    // works out its mip level by comparing coordinates with the neighbouring
    // fragments - and neighbours that took the other branch have nothing to
    // compare against. A solid rect samples one wasted texel and discards it.
    let coverage = textureSample(atlas, atlas_sampler, in.uv).r;
    let alpha = select(1.0, coverage, in.kind == KIND_GLYPH);

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
