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
    /// Binds `globals_buffer` to `@group(0) @binding(0)` in the shader.
    globals_bind_group: wgpu::BindGroup,
}

impl Renderer {
    /// Build the (one) render pipeline and the uniform plumbing.
    /// Everything here is created once and reused every frame.
    pub fn new(ctx: &GpuContext) -> Self {
        // Uniform buffer. `COPY_DST` lets `queue.write_buffer` update it.
        // Not initialized here — `render` writes it before the first draw.
        let globals_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group *layout*: the shader-visible interface (types/stages),
        // part of the pipeline's signature. The bind group below supplies the
        // actual buffer. Separating the two lets many buffers share one
        // pipeline — useful later for per-pass data.
        // https://docs.rs/wgpu/30.0.0/wgpu/struct.BindGroupLayoutDescriptor.html
        let globals_layout = ctx
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("globals"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    // The camera is only needed to position vertices.
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let globals_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
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
                bind_group_layouts: &[Some(&globals_layout)],
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
                label: Some("square"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    // No vertex buffers: positions are generated in the
                    // shader from `vertex_index`.
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
            globals_bind_group,
        }
    }

    /// Draw one frame into `frame` and present it.
    ///
    /// Flow: upload camera state → record a render pass (clear to black,
    /// draw the square) → submit to the GPU queue → present.
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
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.globals_bind_group, &[]);
            // 6 vertices (two triangles), 1 instance — the square.
            rpass.draw(0..6, 0..1);
        } // rpass dropped here: the pass must end before the encoder finishes.

        ctx.queue.submit(Some(encoder.finish()));

        // Lets winit time compositor handoff for smoother presentation.
        // https://docs.rs/winit/0.30/winit/window/struct.Window.html#method.pre_present_notify
        ctx.window.pre_present_notify();
        ctx.queue.present(frame);
    }
}
