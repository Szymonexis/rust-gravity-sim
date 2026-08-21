//! Window lifecycle and event routing — the glue between winit and the rest
//! of the program.
//!
//! winit 0.30 drives applications through the [`ApplicationHandler`] trait
//! instead of a callback closure: the OS event loop calls into our [`App`]
//! (<https://docs.rs/winit/0.30/winit/application/trait.ApplicationHandler.html>).
//! This module contains no interesting logic of its own; every event is
//! delegated to the module that owns that concern:
//!
//! - GPU/window plumbing  → [`crate::gpu::context::GpuContext`]
//! - drawing              → [`crate::gpu::renderer::Renderer`]
//! - mouse controls       → [`crate::input::CameraController`]
//! - camera state         → [`crate::camera::Camera`]

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
    gpu::{context::GpuContext, renderer::Renderer},
    input::CameraController,
};

/// Initial window client size in *logical* pixels (winit multiplies by the
/// monitor's scale factor, so this stays sensible on HiDPI displays).
const INITIAL_WINDOW_SIZE: LogicalSize<f64> = LogicalSize::new(1280.0, 720.0);

/// Everything that exists only once a window exists. Bundled so `App` has a
/// single `Option` to check instead of several.
struct Gpu {
    ctx: GpuContext,
    renderer: Renderer,
}

#[derive(Default)]
pub struct App {
    /// `None` until the first `resumed` call — winit only allows window
    /// creation from inside the event loop, not before it starts.
    gpu: Option<Gpu>,
    camera: Camera,
    controller: CameraController,
}

impl ApplicationHandler for App {
    /// Called when the app gains (or regains) control of its windows; on
    /// desktop effectively once at startup. The winit docs require creating
    /// windows here rather than in `main`:
    /// <https://docs.rs/winit/0.30/winit/application/trait.ApplicationHandler.html#tymethod.resumed>
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return; // Already initialized (e.g. resumed after a suspend).
        }

        let attributes = Window::default_attributes()
            .with_title("Gravity Sim 2D")
            .with_inner_size(INITIAL_WINDOW_SIZE);
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("Failed to create window"),
        );

        // GPU setup is async (see GpuContext::new); on native it's fine to
        // just block the thread for it — there's nothing else to do yet.
        let ctx = pollster::block_on(GpuContext::new(
            window,
            event_loop.owned_display_handle(),
        ));
        let renderer = Renderer::new(&ctx);

        // Kick off the redraw loop (each render requests the next redraw).
        ctx.window.request_redraw();
        self.gpu = Some(Gpu { ctx, renderer });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(gpu) = &mut self.gpu else {
            return; // Events can arrive before initialization; ignore them.
        };

        match event {
            // Close on window X button or the Escape key.
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
                // `acquire_frame` returning None means the surface needed
                // recovery; skipping the frame is fine, the request_redraw
                // below retries immediately.
                if let Some(frame) = gpu.ctx.acquire_frame() {
                    gpu.renderer.render(&gpu.ctx, frame, &self.camera);
                }
                // Continuous rendering: always ask for the next frame. The
                // Fifo present mode throttles this loop to the monitor's
                // refresh rate — the natural tick for a simulation.
                gpu.ctx.window.request_redraw();
            }

            // Mouse controls (see src/input.rs for the scheme).
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

            // Redraw when the window becomes visible again after having been
            // fully covered (Fifo presents can stall while occluded).
            WindowEvent::Occluded(false) => gpu.ctx.window.request_redraw(),

            _ => {}
        }
    }
}
