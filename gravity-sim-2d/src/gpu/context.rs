//! GPU plumbing: everything needed before any triangle can be drawn.
//!
//! One [`GpuContext`] is created at startup and lives for the whole program.
//! It owns the wgpu object hierarchy, created in this order:
//!
//! ```text
//! Instance ── entry point to wgpu itself
//!   └─ Surface ── the window's presentable area ("swapchain" in Vulkan terms)
//!   └─ Adapter ── a physical GPU (we ask for the high-performance one)
//!        └─ Device ── logical connection to that GPU; creates all resources
//!        └─ Queue  ── where command buffers are submitted for execution
//! ```
//!
//! References:
//! - wgpu API docs:            <https://docs.rs/wgpu/30.0.0/wgpu/>
//! - the tutorial this mirrors: <https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/>
//! - upstream example this was adapted from:
//!   <https://github.com/gfx-rs/wgpu/blob/v30.0.0/examples/features/src/hello_triangle/mod.rs>

use std::sync::Arc;

use winit::{dpi::PhysicalSize, event_loop::OwnedDisplayHandle, window::Window};

pub struct GpuContext {
    /// Kept alive because [`GpuContext::acquire_frame`] needs it to recreate
    /// the surface if the OS invalidates it (`CurrentSurfaceTexture::Lost`).
    pub instance: wgpu::Instance,
    /// The winit window, shared via `Arc` because the surface borrows it for
    /// its whole lifetime (hence `Surface<'static>`).
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    /// Current surface configuration; `width`/`height` track the window's
    /// client area in physical pixels and are the authoritative "resolution".
    pub config: wgpu::SurfaceConfiguration,
}

impl GpuContext {
    /// Set up the full wgpu stack for `window`.
    ///
    /// `async` because adapter/device requests may genuinely take time (driver
    /// initialization); on native we simply block on it with `pollster`.
    pub async fn new(window: Arc<Window>, display_handle: OwnedDisplayHandle) -> Self {
        let mut size = window.inner_size();
        // A minimized window reports 0x0, which is an invalid surface size.
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        // `new_with_display_handle_from_env`: backend selection (Vulkan /
        // DX12 / Metal / GL) honors the WGPU_BACKEND env var, and knowing the
        // display improves adapter enumeration on some platforms.
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_with_display_handle_from_env(Box::new(display_handle)),
        );

        let surface = instance.create_surface(window.clone()).unwrap();

        // The adapter is a physical GPU. HighPerformance selects the discrete
        // card on multi-GPU systems (e.g. laptop iGPU + dGPU).
        // https://docs.rs/wgpu/30.0.0/wgpu/struct.RequestAdapterOptions.html
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                // Only consider adapters that can actually present to our
                // window.
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find a suitable GPU adapter");

        let info = adapter.get_info();
        println!("Using GPU: {} ({:?})", info.name, info.backend);

        // The device is our logical handle for creating resources; the queue
        // executes command buffers. Requesting no optional features and
        // near-default limits keeps us portable across GPUs.
        // https://docs.rs/wgpu/30.0.0/wgpu/struct.DeviceDescriptor.html
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // Default limits, but never above what the adapter supports.
                required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create device");

        // Default config = the surface's preferred format and Fifo present
        // mode. Fifo is classic vsync: presents wait for vertical blank, so
        // our redraw loop naturally runs at monitor refresh rate.
        // https://docs.rs/wgpu/30.0.0/wgpu/enum.PresentMode.html
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        Self {
            instance,
            window,
            surface,
            device,
            queue,
            config,
        }
    }

    /// Window client size in physical pixels, as `[width, height]` floats —
    /// the form the shader uniform and the input math want.
    pub fn resolution(&self) -> [f32; 2] {
        [self.config.width as f32, self.config.height as f32]
    }

    /// Adapt to a new window size: reconfigure the surface (its texture must
    /// match the window exactly or presentation fails).
    ///
    /// Deliberately *nothing else* happens here — the camera is untouched, so
    /// the scene keeps its position and on-screen size and a resize merely
    /// reveals more or less of the world.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.config);
        self.window.request_redraw();
    }

    /// Get the next surface texture to draw into, or `None` if no frame can
    /// be produced right now (the caller should just try again next redraw).
    ///
    /// The surface is the one wgpu object the OS can yank out from under us —
    /// on resizes, display changes, driver resets, lock screens... Each
    /// failure mode has a standard recovery, handled here so the rest of the
    /// code only ever sees "a frame" or "no frame":
    /// <https://docs.rs/wgpu/30.0.0/wgpu/enum.CurrentSurfaceTexture.html>
    pub fn acquire_frame(&mut self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => Some(frame),

            // Transient states — nothing to fix, a later frame will succeed.
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => None,

            // The surface works but no longer matches the window optimally
            // (e.g. mid-resize). Reconfiguring with current settings fixes it.
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                drop(frame);
                self.surface.configure(&self.device, &self.config);
                None
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                None
            }

            // The surface itself is gone; rebuild it from the window.
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self.instance.create_surface(self.window.clone()).unwrap();
                self.surface.configure(&self.device, &self.config);
                None
            }

            // Only reachable when an error scope is pushed around acquisition;
            // we don't use error scopes, so wgpu panics before we get here.
            wgpu::CurrentSurfaceTexture::Validation => {
                unreachable!("no error scope registered, validation errors panic")
            }
        }
    }
}
