struct Globals {
    resolution: vec2<f32>,
    pan: vec2<f32>,
    zoom: f32,
    _pad: f32,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

const KIND_CIRCLE: u32 = 0u;

struct Shape {
    center: vec2<f32>,
    size: f32,
    kind: u32,
    color: vec4<f32>,
};

@group(0) @binding(1)
var<storage, read> shapes: array<Shape>;

struct VertexShaderOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) kind: u32,
    @location(2) @interpolate(flat) color: vec4<f32>,
};

@vertex
fn vertex_shader_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexShaderOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    let shape = shapes[instance_index];
    let corner = corners[vertex_index];
    let world = corner * shape.size * 0.5 + shape.center;
    let pixels = world * globals.zoom + globals.pan;
    let clip = pixels * 2.0 / globals.resolution;

    var out: VertexShaderOut;
    out.clip = vec4<f32>(clip, 0.0, 1.0);
    out.local = corner;
    out.kind = shape.kind;
    out.color = shape.color;
    return out;
}

@fragment
fn fragment_shader_main(in: VertexShaderOut) -> @location(0) vec4<f32> {
    if in.kind == KIND_CIRCLE && length(in.local) > 1.0 {
        discard;
    }

    return in.color;
}
