use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta},
};

use crate::{config::AppConfig, view::Camera};

const SCROLL_PIXELS_PER_LINE: f32 = 60.0;

pub struct CameraController {
    cursor: Option<[f32; 2]>,
    left_down: bool,
    pan_enabled: bool,
    zoom_enabled: bool,
}

impl CameraController {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            cursor: None,
            left_down: false,
            pan_enabled: config.pan.enable,
            zoom_enabled: config.zoom.enable,
        }
    }

    pub fn on_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Left {
            self.left_down = state == ElementState::Pressed;
        }
    }

    pub fn on_cursor_moved(
        &mut self,
        position: PhysicalPosition<f64>,
        resolution: [f32; 2],
        camera: &mut Camera,
    ) {
        let cursor = centered(position, resolution);
        if let Some(prev) = self.cursor {
            if self.left_down && self.pan_enabled {
                camera.pan_by([cursor[0] - prev[0], cursor[1] - prev[1]]);
            }
        }
        self.cursor = Some(cursor);
    }

    pub fn on_scroll(&mut self, delta: MouseScrollDelta, camera: &mut Camera) {
        if !self.zoom_enabled {
            return;
        }
        let lines = match delta {
            MouseScrollDelta::LineDelta(_, y) => y,
            MouseScrollDelta::PixelDelta(p) => p.y as f32 / SCROLL_PIXELS_PER_LINE,
        };
        let anchor = self.cursor.unwrap_or([0.0, 0.0]);
        camera.zoom_by(lines, anchor);
    }
}

fn centered(position: PhysicalPosition<f64>, resolution: [f32; 2]) -> [f32; 2] {
    [
        position.x as f32 - resolution[0] / 2.0,
        resolution[1] / 2.0 - position.y as f32,
    ]
}
