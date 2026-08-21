//! Everything that talks to the GPU through [wgpu](https://docs.rs/wgpu/30.0.0/wgpu/),
//! split along the natural boundary:
//!
//! - [`context`] — the *plumbing*: instance, surface (swapchain), adapter,
//!   device and queue. Created once at startup, reconfigured on resize,
//!   recovered when the OS invalidates the surface. Rarely changes as the
//!   project grows.
//! - [`renderer`] — the *drawing*: shader, render pipeline, uniform buffer,
//!   and the per-frame render pass. This is where a gravity simulation will
//!   grow (particle buffers, a compute pass, instanced drawing, ...).
//!
//! Background reading on the overall wgpu architecture:
//! <https://sotrh.github.io/learn-wgpu/> and the official examples at
//! <https://github.com/gfx-rs/wgpu/tree/v30.0.0/examples>.

pub mod context;
pub mod renderer;
