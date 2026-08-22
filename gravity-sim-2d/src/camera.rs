use crate::config::AppConfig;

pub struct Camera {
    pub pan: [f32; 2],
    pub zoom: f32,
    min_zoom: f32,
    max_zoom: f32,
    zoom_step: f32,
}

impl Camera {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            pan: [0.0, 0.0],
            zoom: config.zoom.initial,
            min_zoom: config.zoom.min,
            max_zoom: config.zoom.max,
            zoom_step: config.zoom.step,
        }
    }

    pub fn pan_by(&mut self, delta: [f32; 2]) {
        self.pan[0] += delta[0];
        self.pan[1] += delta[1];
    }

    pub fn zoom_by(&mut self, scroll_lines: f32, anchor: [f32; 2]) {
        let new_zoom =
            (self.zoom * self.zoom_step.powf(scroll_lines)).clamp(self.min_zoom, self.max_zoom);
        let k = new_zoom / self.zoom;
        self.pan[0] = anchor[0] - (anchor[0] - self.pan[0]) * k;
        self.pan[1] = anchor[1] - (anchor[1] - self.pan[1]) * k;
        self.zoom = new_zoom;
    }
}
