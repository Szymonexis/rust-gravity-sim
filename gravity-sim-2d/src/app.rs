use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{
    camera::Camera,
    config::AppConfig,
    gpu::{context::GpuContext, renderer::Renderer},
    input::CameraController,
    scene,
};

struct Gpu {
    ctx: GpuContext,
    renderer: Renderer,
}

pub struct App<'a> {
    config: &'a AppConfig,
    gpu: Option<Gpu>,
    camera: Camera,
    controller: CameraController,
}

impl<'a> App<'a> {
    pub fn new(config: &'a AppConfig) -> Self {
        Self {
            config,
            gpu: None,
            camera: Camera::new(config),
            controller: CameraController::new(config),
        }
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }

        let window_config = &self.config.window;
        let attributes = Window::default_attributes()
            .with_title("Gravity Sim 2D")
            .with_inner_size(LogicalSize::new(
                window_config.width as f64,
                window_config.height as f64,
            ))
            .with_resizable(window_config.resizable);
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("Failed to create window"),
        );

        let ctx = pollster::block_on(GpuContext::new(window, event_loop.owned_display_handle()));
        let renderer = Renderer::new(&ctx, &scene::initial());

        ctx.window.request_redraw();
        self.gpu = Some(Gpu { ctx, renderer });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(gpu) = &mut self.gpu else {
            return;
        };

        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit(),

            WindowEvent::Resized(new_size) => gpu.ctx.resize(new_size),

            WindowEvent::RedrawRequested => {
                if let Some(frame) = gpu.ctx.acquire_frame() {
                    gpu.renderer.render(&gpu.ctx, frame, &self.camera);
                }
                gpu.ctx.window.request_redraw();
            }

            WindowEvent::CursorMoved { position, .. } => {
                let resolution = gpu.ctx.resolution();
                self.controller
                    .on_cursor_moved(position, resolution, &mut self.camera);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.controller.on_mouse_button(button, state);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.controller.on_scroll(delta, &mut self.camera);
            }

            WindowEvent::Occluded(false) => gpu.ctx.window.request_redraw(),

            _ => {}
        }
    }
}
