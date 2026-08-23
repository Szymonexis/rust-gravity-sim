use crate::gpu::context::GpuContext;
use crate::gpu::scene::Shape;
use crate::view::Camera;

/// Mirrored by `Globals` in shader.wgsl. Nothing checks the two agree - if they
/// drift apart the shader silently reads the wrong offsets.
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
    globals_buffer: wgpu::Buffer,
    shapes_buffer: wgpu::Buffer,
    scene_layout: wgpu::BindGroupLayout,
    scene_bind_group: wgpu::BindGroup,
    shape_count: u32,
    shape_capacity: usize,
}

impl Renderer {
    pub fn new(ctx: &GpuContext, shapes: &[Shape]) -> Self {
        let globals_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shape_capacity = shapes.len().max(1);
        let shapes_buffer = create_shapes_buffer(&ctx.device, shape_capacity);
        if !shapes.is_empty() {
            ctx.queue
                .write_buffer(&shapes_buffer, 0, bytemuck::cast_slice(shapes));
        }

        let scene_layout = ctx
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("scene"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
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
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let scene_bind_group =
            create_scene_bind_group(&ctx.device, &scene_layout, &globals_buffer, &shapes_buffer);

        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&scene_layout)],
                immediate_size: 0,
            });

        let pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("shapes"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex_shader_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment_shader_main"),
                    compilation_options: Default::default(),
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
            scene_layout,
            scene_bind_group,
            shape_count: shapes.len() as u32,
            shape_capacity,
        }
    }

    pub fn update_shapes(&mut self, ctx: &GpuContext, shapes: &[Shape]) {
        if shapes.len() > self.shape_capacity {
            self.shape_capacity = shapes.len().next_power_of_two();
            self.shapes_buffer = create_shapes_buffer(&ctx.device, self.shape_capacity);
            self.scene_bind_group = create_scene_bind_group(
                &ctx.device,
                &self.scene_layout,
                &self.globals_buffer,
                &self.shapes_buffer,
            );
        }

        self.shape_count = shapes.len() as u32;
        if !shapes.is_empty() {
            ctx.queue
                .write_buffer(&self.shapes_buffer, 0, bytemuck::cast_slice(shapes));
        }
    }

    /// Clears the target and fills it, so this has to be the first pass of the
    /// frame. Overlays draw into the same encoder afterwards.
    pub fn draw(
        &self,
        ctx: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        camera: &Camera,
    ) {
        let globals = Globals {
            resolution: ctx.resolution(),
            pan: camera.pan,
            zoom: camera.zoom,
            _pad: 0.0,
        };
        ctx.queue
            .write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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
                rpass.draw(0..6, 0..self.shape_count);
            }
        }
    }
}

fn create_shapes_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("shapes"),
        size: (capacity.max(1) * size_of::<Shape>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_scene_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    globals: &wgpu::Buffer,
    shapes: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("scene"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: shapes.as_entire_binding(),
            },
        ],
    })
}
