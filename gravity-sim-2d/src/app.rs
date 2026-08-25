use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{
    config::AppConfig,
    generation,
    gpu::{context::GpuContext, renderer::Renderer, scene::Scene},
    simulation::{STEP_TICKS, SimulationHandle, World},
    ui::{FontFace, Status, Ui},
    view::{Camera, CameraController},
};

const TITLE: &str = "Gravity Sim 2D";
const RATE_SAMPLE: Duration = Duration::from_secs(1);

struct Gpu {
    ctx: GpuContext,
    renderer: Renderer,
    ui: Ui,
}

struct RateMeter {
    started: Instant,
    frames: u32,
    ticks: i64,
    fps: f32,
    tps: f32,
}

impl RateMeter {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            frames: 0,
            ticks: 0,
            fps: 0.0,
            tps: 0.0,
        }
    }

    fn sample(&mut self, ticks: i64) -> bool {
        self.frames += 1;

        let elapsed = self.started.elapsed();
        if elapsed < RATE_SAMPLE {
            return false;
        }

        let seconds = elapsed.as_secs_f32();
        self.fps = self.frames as f32 / seconds;
        self.tps = (ticks - self.ticks) as f32 / seconds;

        self.started = Instant::now();
        self.frames = 0;
        self.ticks = ticks;

        true
    }
}

pub struct App<'a> {
    config: &'a AppConfig,
    gpu: Option<Gpu>,
    camera: Camera,
    controller: CameraController,
    scene: Scene,
    simulation: SimulationHandle,
    rates: RateMeter,
    font: Option<FontFace>,
    config_file: Option<&'a str>,
}

impl<'a> App<'a> {
    pub fn new(config: &'a AppConfig, config_file: Option<&'a str>) -> Self {
        let particles = generation::build(&config.particles);
        let scene = Scene::init(&particles, &config.colors);

        Self {
            simulation: SimulationHandle::spawn(
                config.simulation,
                World::new(particles, config.simulation),
            ),
            config,
            gpu: None,
            camera: Camera::new(config),
            controller: CameraController::new(config),
            scene,
            rates: RateMeter::new(),
            font: Some(FontFace::load(config.ui.font_path.as_deref())),
            config_file,
        }
    }

    fn on_key(&mut self, code: KeyCode, repeat: bool, event_loop: &ActiveEventLoop) {
        match code {
            KeyCode::Escape => event_loop.exit(),

            KeyCode::Space if !repeat => self.simulation.toggle_pause(),

            KeyCode::ArrowRight => {
                if self.simulation.is_paused() {
                    self.simulation.step(STEP_TICKS);
                } else {
                    self.simulation.faster();
                }
            }
            KeyCode::ArrowLeft => {
                if self.simulation.is_paused() {
                    self.simulation.step(-STEP_TICKS);
                } else {
                    self.simulation.slower();
                }
            }

            _ => {}
        }
    }

    fn redraw(&mut self) {
        let Some(gpu) = &mut self.gpu else {
            return;
        };

        if let Some(particles) = self.simulation.try_recv() {
            self.scene.sync(particles);
            gpu.renderer.update_shapes(&gpu.ctx, &self.scene.shapes);
        }

        if self.rates.sample(self.simulation.tick()) {
            let (fps, tps) = (self.rates.fps, self.rates.tps);
            gpu.ctx
                .window
                .set_title(&format!("{TITLE} - {fps:.0} fps / {tps:.0} tps"));
        }

        gpu.ui.prepare(
            &gpu.ctx,
            &Status {
                particles: self.scene.shapes.len(),
                tick: self.simulation.tick(),
                fps: self.rates.fps,
                tps: self.rates.tps,
                zoom: self.camera.zoom,
                pan: self.camera.pan,
                paused: self.simulation.is_paused(),
                speed: self.simulation.speed(),
            },
        );

        if let Some(frame) = gpu.ctx.acquire_frame() {
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = gpu
                .ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            gpu.renderer
                .draw(&gpu.ctx, &mut encoder, &view, &self.camera);
            gpu.ui.draw(&mut encoder, &view);

            gpu.ctx.queue.submit(Some(encoder.finish()));

            gpu.ctx.window.pre_present_notify();
            gpu.ctx.queue.present(frame);
        }

        gpu.ctx.window.request_redraw();
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }

        let window_config = &self.config.window;
        let attributes = Window::default_attributes()
            .with_title(TITLE)
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

        let scale = window.scale_factor() as f32;
        let ctx = pollster::block_on(GpuContext::new(window, event_loop.owned_display_handle()));
        let renderer = Renderer::new(&ctx, &self.scene.shapes);
        let face = self
            .font
            .take()
            .unwrap_or_else(|| FontFace::load(self.config.ui.font_path.as_deref()));
        let ui = Ui::new(&ctx, &self.config.ui, face, scale, self.config_file);

        ctx.window.request_redraw();
        self.gpu = Some(Gpu { ctx, renderer, ui });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if self.gpu.is_none() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: ElementState::Pressed,
                        repeat,
                        ..
                    },
                ..
            } => self.on_key(code, repeat, event_loop),

            WindowEvent::RedrawRequested => self.redraw(),

            _ => self.on_window_event(event),
        }
    }
}

impl App<'_> {
    fn on_window_event(&mut self, event: WindowEvent) {
        let Some(gpu) = &mut self.gpu else {
            return;
        };

        match event {
            WindowEvent::Resized(new_size) => gpu.ctx.resize(new_size),

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                gpu.ui.set_scale(&gpu.ctx, scale_factor as f32);
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
