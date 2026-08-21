//! Drawing: the render pipeline, the camera uniform, and the per-frame pass.
//!
//! This is the file that will grow with the gravity simulation — particle
//! storage buffers, a compute pass for the physics, instanced rendering for
//! the bodies. The [`GpuContext`](super::context::GpuContext) it draws
//! through should barely need to change for any of that.
//!
//! Rendering references:
//! - pipelines:  <https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/>
//! - uniforms:   <https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/>
//! - WebGPU mental model: <https://webgpufundamentals.org/>

use crate::camera::Camera;
use crate::scene::Shape;

use super::context::GpuContext;

/// CPU-side mirror of `struct Globals` in `shader.wgsl` — field order, types
/// and total size (24 bytes) must match the WGSL struct exactly, since the
/// bytes are copied verbatim into the uniform buffer.
///
/// - `#[repr(C)]` pins the field order/layout (Rust otherwise reorders).
/// - `Pod`/`Zeroable` (bytemuck) certify "plain old data", making the
///   byte-cast in [`Renderer::render`] safe: <https://docs.rs/bytemuck/>
/// - `_pad` matches WGSL rounding the struct size up to a multiple of its
///   alignment: <https://www.w3.org/TR/WGSL/#memory-layouts>
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    resolution: [f32; 2],
    pan: [f32; 2],
    zoom: f32,
    _pad: f32,
}

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    /// GPU home of [`Globals`]; rewritten every frame in [`Renderer::render`].
    globals_buffer: wgpu::Buffer,
    /// GPU home of the scene's [`Shape`]s, written once in [`Renderer::new`].
    /// Kept alive because `scene_bind_group` refers to it.
    #[allow(dead_code)]
    shapes_buffer: wgpu::Buffer,
    /// Binds `globals_buffer` and `shapes_buffer` to `@group(0)` in the shader.
    scene_bind_group: wgpu::BindGroup,
    /// How many shapes are in `shapes_buffer` — the draw call's instance count.
    shape_count: u32,
}

impl Renderer {
    /// Build the (one) render pipeline, the uniform plumbing, and the GPU copy
    /// of `shapes`. Everything here is created once and reused every frame.
    ///
    /// The scene is uploaded now rather than per frame because it is static:
    /// once the physics exists, a compute pass will update these bytes on the
    /// GPU and the CPU will stop touching them entirely.
    pub fn new(ctx: &GpuContext, shapes: &[Shape]) -> Self {
        // Uniform buffer. `COPY_DST` lets `queue.write_buffer` update it.
        // Not initialized here — `render` writes it before the first draw.
        let globals_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Storage buffer for the scene. `STORAGE` is what makes it bindable as
        // `var<storage>`; the shader reads its length from the binding, so this
        // is the one place the scene's size is decided. wgpu rejects zero-sized
        // buffers, so an empty scene still reserves one slot — the draw in
        // `render` is skipped in that case.
        let shapes_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shapes"),
            size: (shapes.len().max(1) * size_of::<Shape>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        if !shapes.is_empty() {
            ctx.queue
                .write_buffer(&shapes_buffer, 0, bytemuck::cast_slice(shapes));
        }

        // Bind group *layout*: the shader-visible interface (types/stages),
        // part of the pipeline's signature. The bind group below supplies the
        // actual buffers. Separating the two lets many buffers share one
        // pipeline — useful later for per-pass data.
        // https://docs.rs/wgpu/30.0.0/wgpu/struct.BindGroupLayoutDescriptor.html
        let scene_layout = ctx
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("scene"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        // The camera is only needed to position vertices.
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        // Vertex stage only: the vertex shader reads each
                        // shape once and forwards kind/color to the fragment
                        // stage, so fragments never touch the buffer.
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            // `read_only` matches `var<storage, read>` in the
                            // shader; a writable binding would need `read: false`
                            // and a compute pass to do the writing.
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let scene_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene"),
            layout: &scene_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    // `as_entire_binding` is what tells the shader's
                    // runtime-sized `array<Shape>` how long it is.
                    resource: shapes_buffer.as_entire_binding(),
                },
            ],
        });

        // `include_wgsl!` embeds the file at compile time; the WGSL is parsed
        // and validated at pipeline creation (errors panic with a nice
        // message pointing into the shader source).
        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&scene_layout)],
                // "Immediates" (push constants) — small data passed without a
                // buffer. Unused.
                immediate_size: 0,
            });

        // The render pipeline: the full fixed-function + shader state for a
        // draw. Defaults give us triangle lists, no culling, no depth buffer,
        // no multisampling — exactly right for flat 2D.
        // https://docs.rs/wgpu/30.0.0/wgpu/struct.RenderPipelineDescriptor.html
        let pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("shapes"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    // No vertex buffers: the quad's corners are generated in
                    // the shader from `vertex_index`, and per-shape data comes
                    // from the storage buffer rather than a vertex attribute.
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    // Output format must match what the surface presents.
                    targets: &[Some(ctx.config.format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        Self {
            pipeline,
            globals_buffer,
            shapes_buffer,
            scene_bind_group,
            shape_count: shapes.len() as u32,
        }
    }

    /// Draw one frame into `frame` and present it.
    ///
    /// Flow: upload camera state → record a render pass (clear to black,
    /// draw the shapes) → submit to the GPU queue → present.
    pub fn render(&self, ctx: &GpuContext, frame: wgpu::SurfaceTexture, camera: &Camera) {
        // Refresh the uniform with this frame's camera + window size. Doing
        // it unconditionally every frame is idiomatic for real-time apps
        // (cheap: 24 bytes) and is how per-tick simulation data will flow too.
        let globals = Globals {
            resolution: ctx.resolution(),
            pan: camera.pan,
            zoom: camera.zoom,
            _pad: 0.0,
        };
        ctx.queue
            .write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));

        // A texture *view* is how a texture is attached to a render pass.
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // GPU work is recorded into a command encoder, then submitted as one
        // command buffer — nothing executes until `queue.submit`.
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            // The render pass: attach the surface texture, clear it to black
            // (our background color), keep the results (`Store`).
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if self.shape_count > 0 {
                rpass.set_pipeline(&self.pipeline);
                rpass.set_bind_group(0, &self.scene_bind_group, &[]);
                // 6 vertices (two triangles) per shape, one instance per
                // shape. However many shapes the scene has, this stays a
                // single draw call — that is the point of instancing.
                rpass.draw(0..6, 0..self.shape_count);
            }
        } // rpass dropped here: the pass must end before the encoder finishes.

        ctx.queue.submit(Some(encoder.finish()));

        // Lets winit time compositor handoff for smoother presentation.
        // https://docs.rs/winit/0.30/winit/window/struct.Window.html#method.pre_present_notify
        ctx.window.pre_present_notify();
        ctx.queue.present(frame);
    }
}
