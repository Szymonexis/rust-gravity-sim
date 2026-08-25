use crate::config::AppConfig;
use crate::math::Vec2;

pub struct Camera {
    pub pan: Vec2,
    pub zoom: f32,
    min_zoom: f32,
    max_zoom: f32,
    zoom_step: f32,
}

impl Camera {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: config.zoom.initial,
            min_zoom: config.zoom.min,
            max_zoom: config.zoom.max,
            zoom_step: config.zoom.step,
        }
    }

    pub fn pan_by(&mut self, delta: Vec2) {
        self.pan += delta;
    }

    pub fn zoom_by(&mut self, scroll_lines: f32, anchor: Vec2) {
        let new_zoom =
            (self.zoom * self.zoom_step.powf(scroll_lines)).clamp(self.min_zoom, self.max_zoom);
        let k = new_zoom / self.zoom;
        self.pan = anchor - (anchor - self.pan) * k;
        self.zoom = new_zoom;
    }
}
