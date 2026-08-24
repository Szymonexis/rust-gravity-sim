mod font;
mod overlay;
mod panel;
mod quad;
mod renderer;

pub use font::FontFace;
pub use overlay::Status;

use crate::config::UiConfig;
use crate::gpu::context::GpuContext;
use crate::ui::font::Atlas;
use crate::ui::panel::Anchor;
use crate::ui::quad::Quad;
use crate::ui::renderer::UiRenderer;

/// The overlay: a font rasterised into one atlas, panels laid out into quads,
/// and a single instanced draw call on top of the scene.
pub struct Ui {
    face: FontFace,
    atlas: Atlas,
    renderer: UiRenderer,
    /// Rebuilt from scratch every frame. The buffer is kept so that rebuild
    /// costs nothing but a memcpy.
    quads: Vec<Quad>,
    font_size: f32,
    scale: f32,
    show_stats: bool,
    show_manual: bool,
    /// The settings file the manual points the user at.
    config_file: Option<String>,
}

impl Ui {
    pub fn new(
        ctx: &GpuContext,
        config: &UiConfig,
        face: FontFace,
        scale: f32,
        config_file: Option<&str>,
    ) -> Self {
        println!("Using font: {}", face.source);

        let atlas = face.rasterize(config.font_size * scale);
        let renderer = UiRenderer::new(ctx, &atlas);

        Self {
            face,
            atlas,
            renderer,
            quads: Vec::new(),
            font_size: config.font_size,
            scale,
            show_stats: config.show_stats,
            show_manual: config.show_manual,
            config_file: config_file.map(str::to_owned),
        }
    }

    /// The atlas is rasterised in physical pixels, so a window dragged onto a
    /// display with a different scale factor needs it built again.
    pub fn set_scale(&mut self, ctx: &GpuContext, scale: f32) {
        if scale == self.scale {
            return;
        }

        self.scale = scale;
        self.atlas = self.face.rasterize(self.font_size * scale);
        self.renderer.set_atlas(ctx, &self.atlas);
    }

    pub fn prepare(&mut self, ctx: &GpuContext, status: &Status) {
        let resolution = ctx.resolution();

        self.quads.clear();

        if self.show_stats {
            overlay::stats(status).layout(
                &mut self.quads,
                &self.atlas,
                Anchor::TopLeft,
                resolution,
                self.scale,
            );
        }

        overlay::playback(status, self.show_manual, self.config_file.as_deref()).layout(
            &mut self.quads,
            &self.atlas,
            Anchor::TopRight,
            resolution,
            self.scale,
        );

        self.renderer.prepare(ctx, &self.quads);
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        self.renderer.draw(encoder, view);
    }
}
