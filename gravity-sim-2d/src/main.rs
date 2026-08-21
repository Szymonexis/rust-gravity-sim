//! Entry point. All the substance lives in the modules:
//!
//! ```text
//! main.rs        — this file: logging + starting the event loop
//! app.rs         — window lifecycle, event routing (winit ApplicationHandler)
//! camera.rs      — pan/zoom camera model and its math
//! input.rs       — mouse state; maps raw events onto camera operations
//! gpu/
//!   context.rs   — wgpu setup: instance, surface, device, queue
//!   renderer.rs  — render pipeline, camera uniform, per-frame drawing
//!   shader.wgsl  — the WGSL shader (world → screen → clip transform)
//! ```
//!
//! Controls: scroll wheel = zoom (at cursor), left-drag = pan, Escape = quit.

mod app;
mod camera;
mod gpu;
mod input;

use winit::event_loop::EventLoop;

use crate::app::App;

fn main() {
    // wgpu reports most problems (validation errors, device loss, misbehaving
    // Vulkan layers, ...) through the `log` crate rather than by panicking —
    // without a logger installed those messages vanish. Control verbosity
    // with e.g. `RUST_LOG=warn cargo run`: https://docs.rs/env_logger
    env_logger::init();

    // The event loop is the OS-facing heart of a windowed program: it blocks
    // waiting for events and dispatches them to our `App` (see src/app.rs).
    // https://docs.rs/winit/0.30/winit/event_loop/struct.EventLoop.html
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
